//! The deterministic world step: broad phase, narrow phase, contact
//! resolution, sleeping, and spatial queries.
//!
//! # Solver design
//!
//! * Integration: semi-implicit Euler. `step(dt)` splits `dt` into up to
//!   [`MAX_SUBSTEPS`] fixed slices of at most 1/120 s, so recordings stay
//!   stable regardless of frame pacing.
//! * Broad phase: index-ordered O(n²) AABB sweeps with group filtering.
//!   Dense deterministic ordering is what makes double-tape replays work;
//!   a tree would be faster but order-sensitive.
//! * Narrow phase: analytic sphere/box/capsule/plane tests, normals always
//!   pointing from body `a` toward body `b`. Box-box uses
//!   axis-of-least-penetration separation for the two world-aligned boxes;
//!   capsule-box approximates the segment's closest approach to the box
//!   center, the industry-standard compromise for floor sliding.
//! * Contacts: restitution along the normal plus Baumgarte positional
//!   correction (`0.2` gain, `5 mm` slop), then clamped friction.

use alloc::vec::Vec;

use scenix_core::{ColliderId, PhysicsBodyId};
use scenix_math::float::sqrt;
use scenix_math::{Ray3, Transform, Vec3};

use crate::bodies::{MAX_SUBSTEPS, PhysicsBody};
use crate::colliders::{Collider, ColliderGroup, ColliderShape};

/// Hard body-slot cap keeping the flat loops cache-friendly.
pub const WORLD_MAX_BODIES: usize = 4_096;

/// Baumgarte positional correction gain.
pub const CONTACT_RESOLVE_PERCENT: f32 = 0.2;

/// Penetration slop below which positions are not corrected, meters.
pub const CONTACT_SLOP: f32 = 0.005;

/// Restitution velocities below this threshold collapse to zero to stop
/// jittering tables.
pub const REST_CONTACT_VELOCITY: f32 = 0.4;

/// Friction coefficient floor so tables never become iceless.
pub const MIN_FRICTION: f32 = 0.02;

/// Longest substep the solver takes, seconds.
pub const MAX_SUBSTEP_DT: f32 = 1.0 / 120.0;

/// One solved contact from the narrow phase.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Contact {
    /// First body (always the lower id).
    pub a: PhysicsBodyId,
    /// Second body (always the higher id).
    pub b: PhysicsBodyId,
    /// Contact normal pointing from `a` toward `b`.
    pub normal: Vec3,
    /// Penetration depth, meters, non-negative.
    pub penetration: f32,
}

/// Contact pair alias for engines tracking multi-frame stability.
pub type ContactEvent = (PhysicsBodyId, PhysicsBodyId);

/// Deterministic event ledger produced by each `step()` call.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WorldEvent {
    /// Pair began touching during this step.
    ContactStarted {
        /// First body.
        a: PhysicsBodyId,
        /// Second body.
        b: PhysicsBodyId,
    },
    /// Pair stopped touching during this step.
    ContactEnded {
        /// First body.
        a: PhysicsBodyId,
        /// Second body.
        b: PhysicsBodyId,
    },
    /// A sensor collider's volume was entered this step.
    SensorEntered {
        /// The sensor collider.
        sensor: ColliderId,
        /// Body entering the sensor.
        other: PhysicsBodyId,
    },
    /// Body crossed the sleep threshold this step.
    BodySlept {
        /// Body going to sleep.
        body: PhysicsBodyId,
    },
    /// Body was woken by impulse or contact this step.
    BodyWoke {
        /// Body waking up.
        body: PhysicsBodyId,
    },
}

/// One hit from a spatial query.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpatialQueryHit {
    /// Body struck.
    pub body: PhysicsBodyId,
    /// Collider struck.
    pub collider: ColliderId,
    /// Distance along the query ray or sphere travel, meters.
    pub distance: f32,
    /// Surface normal at the hit.
    pub normal: Vec3,
}

/// Deterministic physics world: the single source of truth for bodies.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicsWorld {
    /// Gravity vector, m/s².
    pub gravity: Vec3,
    bodies: Vec<PhysicsBody>,
    colliders: Vec<Collider>,
    contacts: Vec<Contact>,
    prior_contacts: Vec<u64>,
    events: Vec<WorldEvent>,
    next_body_id: u64,
    next_collider_id: u64,
}

impl PhysicsWorld {
    /// Creates a world with explicit gravity.
    #[must_use]
    pub fn new(gravity: Vec3) -> Self {
        Self {
            gravity,
            bodies: Vec::new(),
            colliders: Vec::new(),
            contacts: Vec::new(),
            prior_contacts: Vec::new(),
            events: Vec::new(),
            next_body_id: 1,
            next_collider_id: 1,
        }
    }

    /// Earth-gravity world.
    #[inline]
    #[must_use]
    pub fn earth() -> Self {
        Self::new(Vec3::new(0.0, -9.81, 0.0))
    }

    /// Zero-gravity world.
    #[inline]
    #[must_use]
    pub fn zero_g() -> Self {
        Self::new(Vec3::ZERO)
    }

    /// Adds a body, receiving its dense id; the world owns id allocation
    /// so callers cannot alias slots.
    #[must_use]
    pub fn add_body(&mut self, mut body: PhysicsBody) -> PhysicsBodyId {
        let id = PhysicsBodyId::new(self.next_body_id);
        self.next_body_id = self.next_body_id.saturating_add(1);
        body.id = id;
        if body.mass <= 0.0 {
            body.mass = 1.0;
        }
        self.bodies.push(body);
        id
    }

    /// Attaches a collider to a body.
    #[must_use]
    pub fn attach_collider(&mut self, body_id: PhysicsBodyId, shape: ColliderShape) -> ColliderId {
        let id = ColliderId::new(self.next_collider_id);
        self.next_collider_id = self.next_collider_id.saturating_add(1);
        self.colliders
            .push(Collider::new(body_id, shape).with_id(id));
        id
    }

    /// Attaches a fully preconfigured collider (custom group, sensor flag,
    /// or surface overrides).
    ///
    /// The world always stamps a fresh id; any id carried by the incoming
    /// collider is replaced so duplicated identifiers can never occur.
    #[must_use]
    pub fn insert_collider(&mut self, mut collider: Collider) -> ColliderId {
        let id = ColliderId::new(self.next_collider_id);
        self.next_collider_id = self.next_collider_id.saturating_add(1);
        collider = collider.with_id(id);
        self.colliders.push(collider);
        id
    }

    /// Adds a body with one collider in one go.
    #[must_use]
    pub fn add_solid(&mut self, body: PhysicsBody, shape: ColliderShape) -> PhysicsBodyId {
        let id = self.add_body(body);
        let _collider = self.attach_collider(id, shape);
        id
    }

    /// Removes a body and its colliders; stale ids never resolve again.
    pub fn remove_body(&mut self, id: PhysicsBodyId) -> bool {
        let index = self.bodies.iter().position(|body| body.id == id);
        let Some(index) = index else {
            return false;
        };
        self.bodies.remove(index);
        self.colliders.retain(|collider| collider.body_id != id);
        self.contacts
            .retain(|contact| contact.a != id && contact.b != id);
        self.prior_contacts.retain(|&key| !touches(key, id));
        true
    }

    /// Body lookup by id.
    #[inline]
    #[must_use]
    pub fn body(&self, id: PhysicsBodyId) -> Option<&PhysicsBody> {
        self.bodies.iter().find(|body| body.id == id)
    }

    /// Mutable body lookup by id.
    #[inline]
    #[must_use]
    pub fn body_mut(&mut self, id: PhysicsBodyId) -> Option<&mut PhysicsBody> {
        self.bodies.iter_mut().find(|body| body.id == id)
    }

    /// Slot id at `index` (bodies are dense).
    #[inline]
    #[must_use]
    pub fn body_at(&self, index: usize) -> Option<&PhysicsBody> {
        self.bodies.get(index)
    }

    /// Live body count.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.bodies.len()
    }

    /// True when the world has no bodies.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }

    /// Bodies in deterministic slot order.
    #[inline]
    #[must_use]
    pub fn bodies(&self) -> &[PhysicsBody] {
        &self.bodies
    }

    /// Colliders in deterministic slot order.
    #[inline]
    #[must_use]
    pub fn colliders(&self) -> &[Collider] {
        &self.colliders
    }

    /// Contacts from the last completed step.
    #[inline]
    #[must_use]
    pub fn contacts(&self) -> &[Contact] {
        &self.contacts
    }

    /// Empties and returns the event ledger from the last step.
    #[inline]
    pub fn take_events(&mut self) -> Vec<WorldEvent> {
        core::mem::take(&mut self.events)
    }

    /// Worldwide body inventory combined with colliders for engine views.
    #[must_use]
    pub fn body_pair_count(&self) -> usize {
        self.bodies.len() * (self.bodies.len().saturating_sub(1)) / 2
    }

    /// Whether two bodies are touching as of the last step.
    #[inline]
    #[must_use]
    pub fn are_touching(&self, a: PhysicsBodyId, b: PhysicsBodyId) -> bool {
        self.contacts
            .iter()
            .any(|contact| (contact.a == a && contact.b == b) || (contact.a == b && contact.b == a))
    }

    /// Deepest penetration between two bodies, if touching.
    #[must_use]
    pub fn contact_depth(&self, a: PhysicsBodyId, b: PhysicsBodyId) -> Option<f32> {
        self.contacts
            .iter()
            .filter(|contact| {
                (contact.a == a && contact.b == b) || (contact.a == b && contact.b == a)
            })
            .map(|contact| contact.penetration)
            .fold(None::<f32>, |best, depth| {
                Some(match best {
                    Some(best) if best >= depth => best,
                    _ => depth,
                })
            })
    }

    /// Runs one deterministic world step over `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.25);
        if dt <= 0.0 || self.bodies.is_empty() {
            self.contacts.clear();
            return;
        }
        let substeps =
            (scenix_math::float::ceil(dt / MAX_SUBSTEP_DT) as usize).clamp(1, MAX_SUBSTEPS);
        let sub_dt = dt / substeps as f32;

        for _ in 0..substeps {
            self.integrate(sub_dt);
            self.resolve();
        }
        self.poll_sleep();
        self.emit_contact_events();
    }

    /// Gravity setter for gameplay layers.
    #[inline]
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    /// Semi-implicit Euler velocity + transform integration.
    fn integrate(&mut self, dt: f32) {
        for body in self.bodies.iter_mut() {
            if body.sleep.asleep {
                continue;
            }
            body.integrate_velocity(self.gravity, dt);
            body.transform = body.advance_transform(dt);
        }
    }

    /// Broad phase + narrow phase + contact resolution for one substep.
    fn resolve(&mut self) {
        let mut pairs: Vec<(usize, usize)> = Vec::new();
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                if self.pair_passable(i, j) {
                    pairs.push((i, j));
                }
            }
        }

        self.contacts.clear();
        for (i, j) in pairs {
            let (a_shape, b_shape) = match (self.shape_of_index(i), self.shape_of_index(j)) {
                (Some(a_shape), Some(b_shape)) => (a_shape, b_shape),
                _ => continue,
            };
            let a_transform = self.bodies[i].transform;
            let b_transform = self.bodies[j].transform;
            let mut contact = Contact {
                a: self.bodies[i].id,
                b: self.bodies[j].id,
                normal: Vec3::ZERO,
                penetration: 0.0,
            };
            if narrow_phase(a_shape, &a_transform, b_shape, &b_transform, &mut contact) {
                self.contacts.push(contact);
            }
        }
        self.apply_contacts();
    }

    /// Whether a pair passes AABB overlap and group filtering.
    fn pair_passable(&self, i: usize, j: usize) -> bool {
        let (Some(a), Some(b)) = (self.bodies.get(i), self.bodies.get(j)) else {
            return false;
        };
        let (Some(shape_a), Some(shape_b)) = (self.shape_of_index(i), self.shape_of_index(j))
        else {
            return false;
        };
        if !shape_a
            .world_aabb(&a.transform)
            .intersects_aabb(shape_b.world_aabb(&b.transform))
        {
            return false;
        }
        if !self
            .collider_group_of(i)
            .interacts_with(&self.collider_group_of(j))
        {
            return false;
        }
        !(a.sleep.asleep && b.sleep.asleep)
    }

    /// Collider shape for a slot index.
    #[inline]
    #[must_use]
    fn shape_of_index(&self, index: usize) -> Option<ColliderShape> {
        let body = self.bodies.get(index)?;
        self.colliders
            .iter()
            .find(|c| c.body_id == body.id)
            .map(|c| c.shape)
    }

    /// Collider group for a slot index.
    #[inline]
    #[must_use]
    fn collider_group_of(&self, index: usize) -> ColliderGroup {
        self.bodies
            .get(index)
            .and_then(|body| {
                self.colliders
                    .iter()
                    .find(|c| c.body_id == body.id)
                    .map(|c| c.group)
            })
            .unwrap_or(ColliderGroup::all())
    }

    /// Resolves every collected contact with impulses + Baumgarte push.
    fn apply_contacts(&mut self) {
        for contact in self.contacts.clone().iter().copied() {
            self.apply_contact(contact);
        }
    }

    /// Applies the impulse and positional correction for one contact.
    fn apply_contact(&mut self, contact: Contact) {
        let a_index = self.bodies.iter().position(|b| b.id == contact.a);
        let b_index = self.bodies.iter().position(|b| b.id == contact.b);
        let (Some(a_index), Some(b_index)) = (a_index, b_index) else {
            return;
        };

        // Sensor colliders record hits but never resolve.
        let sensor = self
            .colliders
            .iter()
            .any(|c| (c.body_id == contact.a || c.body_id == contact.b) && c.sensor);
        if sensor {
            for collider in &self.colliders {
                if collider.sensor
                    && (collider.body_id == contact.a || collider.body_id == contact.b)
                {
                    let other = if collider.body_id == contact.a {
                        contact.b
                    } else {
                        contact.a
                    };
                    self.events.push(WorldEvent::SensorEntered {
                        sensor: collider.id,
                        other,
                    });
                }
            }
            return;
        }

        let mass_a = effective_mass(&self.bodies[a_index]);
        let mass_b = effective_mass(&self.bodies[b_index]);
        let restitution = contact_restitution(self, a_index, b_index);
        let friction = contact_friction(self, a_index, b_index);
        let normal = contact.normal;

        let rel_velocity =
            self.bodies[b_index].linear_velocity - self.bodies[a_index].linear_velocity;
        let along_normal = rel_velocity.dot(normal);
        let denom = inv_mass(mass_a) + inv_mass(mass_b);
        // Only meaningful impacts disturb sleep; the resting gravity bleed
        // stays below the threshold so settled stacks stay asleep.
        let impacts_sleep = -along_normal >= REST_CONTACT_VELOCITY;
        if denom > 1.0e-9 && along_normal < 0.0 {
            let restitution_effective = if -along_normal < REST_CONTACT_VELOCITY {
                0.0
            } else {
                restitution
            };
            let impulse_magnitude = (-(1.0 + restitution_effective) * along_normal) / denom;
            if impulse_magnitude.is_finite() && impulse_magnitude > 0.0 {
                let impulse = normal * impulse_magnitude;
                if mass_a > 0.0 {
                    let body = &mut self.bodies[a_index];
                    body.linear_velocity -= impulse * inv_mass(mass_a);
                    if impacts_sleep {
                        let woke = body.sleep.wake();
                        if woke {
                            self.events.push(WorldEvent::BodyWoke { body: body.id });
                        }
                    }
                }
                if mass_b > 0.0 {
                    let body = &mut self.bodies[b_index];
                    body.linear_velocity += impulse * inv_mass(mass_b);
                    if impacts_sleep {
                        let woke = body.sleep.wake();
                        if woke {
                            self.events.push(WorldEvent::BodyWoke { body: body.id });
                        }
                    }
                }

                // Coulomb friction over the tangent plane.
                let post = effective_relative_velocity(
                    &self.bodies[a_index],
                    &self.bodies[b_index],
                    normal,
                );
                let tangent = post * -1.0;
                let tangent_speed = tangent.length();
                if tangent_speed > 1.0e-6 && friction > MIN_FRICTION {
                    let direction = tangent * (1.0 / tangent_speed);
                    let max_friction = impulse_magnitude * friction;
                    let applied = tangent_speed.min(max_friction);
                    let friction_impulse = direction * applied;
                    if mass_a > 0.0 {
                        self.bodies[a_index].linear_velocity -= friction_impulse * inv_mass(mass_a);
                    }
                    if mass_b > 0.0 {
                        self.bodies[b_index].linear_velocity += friction_impulse * inv_mass(mass_b);
                    }
                }
            }
        }

        // Baumgarte positional correction, mass-weighted, one push per pair.
        let correction_depth = (contact.penetration - CONTACT_SLOP).max(0.0);
        if correction_depth > 0.0 && denom > 1.0e-9 {
            let correction = correction_depth * CONTACT_RESOLVE_PERCENT;
            if mass_a > 0.0 {
                let share = inv_mass(mass_a) / denom;
                let body = &mut self.bodies[a_index];
                body.transform.translation -= normal * (correction * share);
            }
            if mass_b > 0.0 {
                let share = inv_mass(mass_b) / denom;
                let body = &mut self.bodies[b_index];
                body.transform.translation += normal * (correction * share);
            }
        }
    }

    /// Sleep accounting once per completed frame.
    fn poll_sleep(&mut self) {
        for body in self.bodies.iter_mut() {
            if body.kind.is_dynamically_forced()
                && !body.sleep.asleep
                && body.sleep.tally(body.motion_speed())
            {
                body.linear_velocity = Vec3::ZERO;
                self.events.push(WorldEvent::BodySlept { body: body.id });
            }
        }
    }

    /// Converts contacts into Started/Ended events against last step.
    fn emit_contact_events(&mut self) {
        let current: Vec<u64> = self
            .contacts
            .iter()
            .map(|c| contact_key(c.a, c.b))
            .collect();
        for &key in &current {
            if !self.prior_contacts.contains(&key) {
                let (a, b) = unpair(key);
                self.events.push(WorldEvent::ContactStarted { a, b });
            }
        }
        for &key in &self.prior_contacts {
            if !current.contains(&key) {
                let (a, b) = unpair(key);
                self.events.push(WorldEvent::ContactEnded { a, b });
            }
        }
        self.prior_contacts = current;
    }

    // ── Spatial queries ──────────────────────────────────────────────

    /// Casts a ray against every collider, returning the first contact.
    #[must_use]
    pub fn cast_ray(&self, ray: Ray3, max_distance: f32) -> Option<SpatialQueryHit> {
        self.cast_ray_excluding(ray, max_distance, PhysicsBodyId::new(u64::MAX))
    }

    /// Like [`PhysicsWorld::cast_ray`] but skips every collider of `skip`,
    /// which lets controllers probe the ground beneath their own capsule
    /// without striking their own shell (a ray inside a capsule hits its
    /// exit face, corrupting ground probes).
    #[must_use]
    pub fn cast_ray_excluding(
        &self,
        ray: Ray3,
        max_distance: f32,
        skip: PhysicsBodyId,
    ) -> Option<SpatialQueryHit> {
        let span = max_distance.max(0.0);
        let mut best: Option<(f32, PhysicsBodyId, ColliderId, Vec3)> = None;
        for collider in &self.colliders {
            if collider.body_id == skip {
                continue;
            }
            let Some(body) = self.body(collider.body_id) else {
                continue;
            };
            let Some((distance, normal)) =
                ray_collider(ray, &collider.shape, &body.transform, span)
            else {
                continue;
            };
            if distance > span {
                continue;
            }
            if best
                .map(|(current, _, _, _)| distance < current)
                .unwrap_or(true)
            {
                best = Some((distance, collider.body_id, collider.id, normal));
            }
        }
        best.map(|(distance, body, collider, normal)| SpatialQueryHit {
            body,
            collider,
            distance,
            normal,
        })
    }

    /// Sphere overlap: every body whose shape intersects the sphere.
    #[must_use]
    pub fn overlap_sphere(&self, center: Vec3, radius: f32) -> Vec<PhysicsBodyId> {
        let mut hits = Vec::new();
        for collider in &self.colliders {
            let Some(body) = self.body(collider.body_id) else {
                continue;
            };
            let intersects = match collider.shape {
                ColliderShape::Sphere(shape_radius) => {
                    distance(body.transform.translation, center) <= shape_radius + radius
                }
                ColliderShape::Box(half) => box_overlaps_sphere(
                    body.transform.translation,
                    half.mul_elements(body.transform.scale),
                    center,
                    radius,
                ),
                ColliderShape::Capsule {
                    half_height,
                    radius: cap_radius,
                } => {
                    let closest =
                        closest_point_on_capsule_axis(&body.transform, half_height, center);
                    distance(closest, center) <= cap_radius + radius
                }
                ColliderShape::Plane => {
                    body.transform.translation.y - radius < 0.0
                        && center.y <= body.transform.translation.y + radius
                }
            };
            if intersects && !hits.contains(&collider.body_id) {
                hits.push(collider.body_id);
            }
        }
        hits
    }
}

impl Default for PhysicsWorld {
    #[inline]
    fn default() -> Self {
        Self::earth()
    }
}

// ── Mass agreement ───────────────────────────────────────────────────

/// Mass the solver credits to a body; statics contribute nothing.
#[inline]
#[must_use]
fn effective_mass(body: &PhysicsBody) -> f32 {
    if body.kind.can_be_pushed() {
        body.mass
    } else {
        0.0
    }
}

#[inline]
#[must_use]
fn inv_mass(mass: f32) -> f32 {
    if mass > 1.0e-9 { 1.0 / mass } else { 0.0 }
}

/// Restitution agreement: collider overrides beat body coefficients.
#[must_use]
fn contact_restitution(world: &PhysicsWorld, a_index: usize, b_index: usize) -> f32 {
    let body_a = &world.bodies[a_index];
    let body_b = &world.bodies[b_index];
    let pick = |index: usize| {
        world
            .colliders
            .iter()
            .find(|c| c.body_id == world.bodies[index].id)
            .map(|c| c.restitution)
            .unwrap_or(-1.0)
    };
    let mut resolved = body_a.restitution.max(body_b.restitution);
    let a_override = pick(a_index);
    let b_override = pick(b_index);
    if a_override >= 0.0 {
        resolved = resolved.max(a_override);
    }
    if b_override >= 0.0 {
        resolved = resolved.max(b_override);
    }
    resolved.clamp(0.0, 1.0)
}

/// Friction agreement; `max` keeps walls grippy under mixed overrides.
#[must_use]
fn contact_friction(world: &PhysicsWorld, a_index: usize, b_index: usize) -> f32 {
    let body_a = &world.bodies[a_index];
    let body_b = &world.bodies[b_index];
    let pick = |index: usize| {
        world
            .colliders
            .iter()
            .find(|c| c.body_id == world.bodies[index].id)
            .map(|c| c.friction)
            .unwrap_or(-1.0)
    };
    let mut resolved = body_a.friction.max(body_b.friction);
    let a_override = pick(a_index);
    let b_override = pick(b_index);
    if a_override >= 0.0 {
        resolved = resolved.max(a_override);
    }
    if b_override >= 0.0 {
        resolved = resolved.max(b_override);
    }
    resolved.max(MIN_FRICTION)
}

/// Post-impulse relative velocity along `normal`, for friction math.
#[inline]
#[must_use]
fn effective_relative_velocity(a: &PhysicsBody, b: &PhysicsBody, normal: Vec3) -> Vec3 {
    let full = b.linear_velocity - a.linear_velocity;
    let along = full.dot(normal);
    full - normal * along
}

/// Deterministic contact pairing key (lower id first).
#[inline]
#[must_use]
fn contact_key(a: PhysicsBodyId, b: PhysicsBodyId) -> u64 {
    let (a, b) = if a.get() <= b.get() {
        (a.get(), b.get())
    } else {
        (b.get(), a.get())
    };
    (a << 32) | b
}

/// Splits the pairing key back into both ids.
#[inline]
#[must_use]
fn unpair(key: u64) -> (PhysicsBodyId, PhysicsBodyId) {
    (
        PhysicsBodyId::new(key >> 32),
        PhysicsBodyId::new(key & 0xFFFF_FFFF),
    )
}

/// Whether a pairing key touches body `id`.
#[inline]
#[must_use]
fn touches(key: u64, id: PhysicsBodyId) -> bool {
    let (a, b) = unpair(key);
    a == id || b == id
}

// ── Geometry helpers ─────────────────────────────────────────────────

/// Euclidean distance between two points.
#[inline]
#[must_use]
fn distance(a: Vec3, b: Vec3) -> f32 {
    sqrt((b - a).length_squared())
}

/// Unit vector, or +Y for degenerate inputs (worlds are ground-first).
#[inline]
#[must_use]
fn normalize_or_unit(vector: Vec3) -> Vec3 {
    let scale = vector.length_squared();
    if scale <= 1.0e-12 {
        Vec3::Y
    } else {
        vector * (1.0 / sqrt(scale))
    }
}

/// Signed axis value: `-1` for negative, `+1` for positive, `0` for zero.
#[inline]
#[must_use]
fn signed(value: f32) -> f32 {
    if value < 0.0 {
        -1.0
    } else if value > 0.0 {
        1.0
    } else {
        0.0
    }
}

/// Capsule axis endpoints in world space.
#[must_use]
fn capsule_axis(transform: &Transform, half_height: f32) -> (Vec3, Vec3) {
    let axis = transform.rotation * Vec3::Y;
    let offset = axis * half_height;
    (
        transform.translation - offset,
        transform.translation + offset,
    )
}

/// Closest point on a capsule axis segment to `point`.
#[must_use]
fn closest_point_on_capsule_axis(transform: &Transform, half_height: f32, point: Vec3) -> Vec3 {
    let (a, b) = capsule_axis(transform, half_height);
    closest_point_on_segment(a, b, point)
}

/// Ericson closest-point-on-segment.
#[must_use]
fn closest_point_on_segment(a: Vec3, b: Vec3, point: Vec3) -> Vec3 {
    let ab = b - a;
    let length_sq = ab.length_squared();
    if length_sq <= 1.0e-12 {
        return a;
    }
    let t = ((point - a).dot(ab) / length_sq).clamp(0.0, 1.0);
    a + ab * t
}

/// Ericson segment-segment closest points.
#[must_use]
fn closest_segment_segment(a1: Vec3, b1: Vec3, a2: Vec3, b2: Vec3) -> (Vec3, Vec3) {
    let d1 = b1 - a1;
    let d2 = b2 - a2;
    let r = a1 - a2;
    let a = d1.length_squared();
    let e = d2.length_squared();
    let f = d2.dot(r);
    if a <= 1.0e-12 && e <= 1.0e-12 {
        return (a1, a2);
    }
    if a <= 1.0e-12 {
        let s = (f / e).clamp(0.0, 1.0);
        return (a1, a2 + d2 * s);
    }
    let c = d1.dot(r);
    if e <= 1.0e-12 {
        let t = (-c / a).clamp(0.0, 1.0);
        return (a1 + d1 * t, a2);
    }
    let b = d1.dot(d2);
    let denom = a * e - b * b;
    let mut s = if denom.abs() > 1.0e-12 {
        ((b * f - c * e) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let t = (b * s + f) / e;
    let t_clamped;
    if t < 0.0 {
        s = (-c / a).clamp(0.0, 1.0);
        t_clamped = 0.0;
    } else if t > 1.0 {
        s = ((b - c) / a).clamp(0.0, 1.0);
        t_clamped = 1.0;
    } else {
        t_clamped = t;
    }
    (a1 + d1 * s, a2 + d2 * t_clamped)
}

/// Box-sphere overlap for queries: expanded slab test.
#[inline]
#[must_use]
fn box_overlaps_sphere(center: Vec3, half: Vec3, point: Vec3, radius: f32) -> bool {
    let delta = point - center;
    let abs = Vec3::new(delta.x.abs(), delta.y.abs(), delta.z.abs());
    abs.x <= half.x + radius && abs.y <= half.y + radius && abs.z <= half.z + radius
}

// ── Narrow phase ─────────────────────────────────────────────────────

/// Narrow phase dispatch; normals always point from `a` toward `b`.
fn narrow_phase(
    a_shape: ColliderShape,
    a_transform: &Transform,
    b_shape: ColliderShape,
    b_transform: &Transform,
    contact: &mut Contact,
) -> bool {
    match (a_shape, b_shape) {
        (ColliderShape::Sphere(ra), ColliderShape::Sphere(rb)) => sphere_sphere(
            a_transform.translation,
            ra,
            b_transform.translation,
            rb,
            contact,
        ),
        (ColliderShape::Sphere(r), ColliderShape::Box(half)) => sphere_box(
            a_transform.translation,
            r,
            b_transform.translation,
            half,
            Sign::Flipped,
            contact,
        ),
        (ColliderShape::Box(half), ColliderShape::Sphere(r)) => sphere_box(
            b_transform.translation,
            r,
            a_transform.translation,
            half,
            Sign::Direct,
            contact,
        ),
        (
            ColliderShape::Sphere(r),
            ColliderShape::Capsule {
                half_height,
                radius,
            },
        ) => sphere_capsule(
            a_transform.translation,
            r,
            b_transform,
            half_height,
            radius,
            Sign::Direct,
            contact,
        ),
        (
            ColliderShape::Capsule {
                half_height,
                radius,
            },
            ColliderShape::Sphere(r),
        ) => sphere_capsule(
            b_transform.translation,
            r,
            a_transform,
            half_height,
            radius,
            Sign::Flipped,
            contact,
        ),
        (
            ColliderShape::Capsule {
                half_height: ah,
                radius: ar,
            },
            ColliderShape::Capsule {
                half_height: bh,
                radius: br,
            },
        ) => capsule_capsule(a_transform, ah, ar, b_transform, bh, br, contact),
        (ColliderShape::Box(ha), ColliderShape::Box(hb)) => {
            box_box(a_transform, ha, b_transform, hb, contact)
        }
        (
            ColliderShape::Capsule {
                half_height: ah,
                radius: ar,
            },
            ColliderShape::Box(hb),
        ) => capsule_box(a_transform, ah, ar, b_transform, hb, Sign::Flipped, contact),
        (
            ColliderShape::Box(ha),
            ColliderShape::Capsule {
                half_height: bh,
                radius: br,
            },
        ) => capsule_box(b_transform, bh, br, a_transform, ha, Sign::Direct, contact),
        (ColliderShape::Plane, _) => shape_vs_plane(
            b_shape,
            b_transform,
            plane_ground(a_transform),
            Sign::Direct,
            contact,
        ),
        (_, ColliderShape::Plane) => shape_vs_plane(
            a_shape,
            a_transform,
            plane_ground(b_transform),
            Sign::Flipped,
            contact,
        ),
    }
}

/// Normal orientation control for flipped dispatches.
#[derive(Clone, Copy, PartialEq)]
enum Sign {
    Direct,
    Flipped,
}

impl Sign {
    #[inline]
    fn apply(self, normal: Vec3) -> Vec3 {
        match self {
            Self::Direct => normal,
            Self::Flipped => normal * -1.0,
        }
    }

    #[inline]
    const fn is_flipped(self) -> bool {
        matches!(self, Self::Flipped)
    }
}

/// Plane ground height; planes expose their body's `Y`.
#[inline]
#[must_use]
const fn plane_ground(transform: &Transform) -> f32 {
    transform.translation.y
}

/// Narrow placement of two spheres; normal points from first to second.
fn sphere_sphere(
    center_a: Vec3,
    radius_a: f32,
    center_b: Vec3,
    radius_b: f32,
    contact: &mut Contact,
) -> bool {
    let delta = center_b - center_a;
    let distance_sq = delta.length_squared();
    let sum = radius_a + radius_b;
    if distance_sq >= sum * sum {
        return false;
    }
    let distance = sqrt(distance_sq);
    if distance <= 1.0e-6 {
        contact.normal = Vec3::Y;
        contact.penetration = sum;
    } else {
        contact.normal = delta * (1.0 / distance);
        contact.penetration = sum - distance;
    }
    true
}

/// Sphere against a world-aligned box, from the box's own frame.
fn sphere_box(
    center: Vec3,
    radius: f32,
    box_center: Vec3,
    half: Vec3,
    sign: Sign,
    contact: &mut Contact,
) -> bool {
    let relative = center - box_center;
    let clamped = Vec3::new(
        relative.x.clamp(-half.x, half.x),
        relative.y.clamp(-half.y, half.y),
        relative.z.clamp(-half.z, half.z),
    );
    let to_center = relative - clamped;
    let distance_sq = to_center.length_squared();
    if distance_sq > radius * radius {
        return false;
    }
    if distance_sq > 1.0e-12 {
        let punch = sqrt(distance_sq);
        contact.normal = sign.apply(to_center * (1.0 / punch));
        contact.penetration = radius - punch;
    } else {
        // Center inside the box: exit along the least-penetration axis.
        let dx = half.x - relative.x.abs();
        let dy = half.y - relative.y.abs();
        let dz = half.z - relative.z.abs();
        let mut depth = dx;
        let mut axis = 0u8;
        if dy < depth {
            depth = dy;
            axis = 1;
        }
        if dz < depth {
            depth = dz;
            axis = 2;
        }
        let normal = match axis {
            0 => Vec3::X * signed(relative.x),
            1 => Vec3::Y * signed(relative.y),
            _ => Vec3::Z * signed(relative.z),
        };
        contact.normal = sign.apply(normal);
        contact.penetration = depth + radius;
    }
    true
}

/// Sphere vs capsule: reduce to sphere-sphere against the axis.
fn sphere_capsule(
    sphere_center: Vec3,
    radius: f32,
    capsule_transform: &Transform,
    half_height: f32,
    capsule_radius: f32,
    sign: Sign,
    contact: &mut Contact,
) -> bool {
    let mut probe = Contact {
        a: contact.a,
        b: contact.b,
        normal: Vec3::ZERO,
        penetration: 0.0,
    };
    let closest = closest_point_on_capsule_axis(capsule_transform, half_height, sphere_center);
    if !sphere_sphere(sphere_center, radius, closest, capsule_radius, &mut probe) {
        return false;
    }
    if sign.is_flipped() {
        // First iteration flipped normals: `a`-side normal reorients.
        contact.normal = probe.normal * -1.0;
        contact.penetration = probe.penetration;
    } else {
        contact.normal = probe.normal;
        contact.penetration = probe.penetration;
    }
    true
}

/// Capsule-vs-capsule via segment-segment closest points.
fn capsule_capsule(
    a_transform: &Transform,
    a_half: f32,
    a_radius: f32,
    b_transform: &Transform,
    b_half: f32,
    b_radius: f32,
    contact: &mut Contact,
) -> bool {
    let (a1, b1) = capsule_axis(a_transform, a_half);
    let (a2, b2) = capsule_axis(b_transform, b_half);
    let (closest_a, closest_b) = closest_segment_segment(a1, b1, a2, b2);
    sphere_sphere(closest_a, a_radius, closest_b, b_radius, contact)
}

/// World-aligned box pair via least-penetration axis.
fn box_box(
    a_transform: &Transform,
    half_a: Vec3,
    b_transform: &Transform,
    half_b: Vec3,
    contact: &mut Contact,
) -> bool {
    let delta = b_transform.translation - a_transform.translation;
    let dx = half_a.x + half_b.x - delta.x.abs();
    let dy = half_a.y + half_b.y - delta.y.abs();
    let dz = half_a.z + half_b.z - delta.z.abs();
    if dx <= 0.0 || dy <= 0.0 || dz <= 0.0 {
        return false;
    }
    if dx <= dy && dx <= dz {
        contact.normal = Vec3::X * signed(delta.x);
        contact.penetration = dx;
    } else if dy <= dz {
        contact.normal = Vec3::Y * signed(delta.y);
        contact.penetration = dy;
    } else {
        contact.normal = Vec3::Z * signed(delta.z);
        contact.penetration = dz;
    }
    true
}

/// Capsule vs world-aligned box: axis probe then sphere-box.
fn capsule_box(
    capsule_transform: &Transform,
    half_height: f32,
    radius: f32,
    box_transform: &Transform,
    half_box: Vec3,
    sign: Sign,
    contact: &mut Contact,
) -> bool {
    let probe =
        closest_point_on_capsule_axis(capsule_transform, half_height, box_transform.translation);
    let mut unit = Contact {
        a: contact.a,
        b: contact.b,
        normal: Vec3::ZERO,
        penetration: 0.0,
    };
    if !sphere_box(
        probe,
        radius,
        box_transform.translation,
        half_box,
        Sign::Direct,
        &mut unit,
    ) {
        return false;
    }
    let raw = if sign.is_flipped() {
        unit.normal * -1.0
    } else {
        unit.normal
    };
    contact.normal = raw;
    contact.penetration = unit.penetration;
    true
}

/// Any shape against an infinite plane, `+Y` convention.
fn shape_vs_plane(
    shape: ColliderShape,
    transform: &Transform,
    plane_y: f32,
    sign: Sign,
    contact: &mut Contact,
) -> bool {
    let (penetration, normal) = match shape {
        ColliderShape::Sphere(radius) => {
            let bottom = transform.translation.y - radius;
            if bottom >= plane_y {
                return false;
            }
            (plane_y - bottom, Vec3::Y)
        }
        ColliderShape::Box(half) => {
            let reach = half.y * transform.scale.y;
            let bottom = transform.translation.y - reach;
            if bottom >= plane_y {
                return false;
            }
            (plane_y - bottom, Vec3::Y)
        }
        ColliderShape::Capsule {
            half_height,
            radius,
        } => {
            let (a, b) = capsule_axis(transform, half_height);
            let lowest = a.y.min(b.y) - radius;
            if lowest >= plane_y {
                return false;
            }
            (plane_y - lowest, Vec3::Y)
        }
        ColliderShape::Plane => return false,
    };
    contact.normal = sign.apply(normal);
    contact.penetration = penetration;
    true
}

// ── Ray queries ──────────────────────────────────────────────────────

/// Ray vs one collider, honoring its world transform.
#[must_use]
fn ray_collider(
    ray: Ray3,
    shape: &ColliderShape,
    transform: &Transform,
    span: f32,
) -> Option<(f32, Vec3)> {
    match shape {
        ColliderShape::Sphere(radius) => {
            ray_sphere(ray, transform.translation, *radius, span).map(|distance| {
                let point = ray.origin + ray.direction * distance;
                let delta = point - transform.translation;
                let normal = normalize_or_unit(delta);
                (distance, normal)
            })
        }
        ColliderShape::Box(half) => ray_box(ray, transform.translation, *half, span),
        ColliderShape::Capsule {
            half_height,
            radius,
        } => {
            let (a, b) = capsule_axis(transform, *half_height);
            ray_capsule(ray, a, b, *radius, span)
        }
        ColliderShape::Plane => {
            let ground = transform.translation.y;
            if ray.direction.y.abs() <= 1.0e-8 {
                return None;
            }
            let rising = ray.direction.y > 0.0;
            if (ray.origin.y >= ground && rising) || (ray.origin.y <= ground && !rising) {
                return None;
            }
            let hit = (ground - ray.origin.y) / ray.direction.y;
            if hit >= 0.0 && hit <= span {
                Some((hit, Vec3::Y))
            } else {
                None
            }
        }
    }
}

/// Ray-sphere nearest root along the span.
#[must_use]
fn ray_sphere(ray: Ray3, center: Vec3, radius: f32, span: f32) -> Option<f32> {
    let offset = ray.origin - center;
    let b = offset.dot(ray.direction);
    let c = offset.length_squared() - radius * radius;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let root = sqrt(disc);
    let mut t = -b - root;
    if t < 0.0 {
        t = -b + root;
    }
    if t >= 0.0 && t <= span { Some(t) } else { None }
}

/// Ray-capsule: both cap spheres then the shaft slab.
#[must_use]
fn ray_capsule(ray: Ray3, a: Vec3, b: Vec3, radius: f32, span: f32) -> Option<(f32, Vec3)> {
    let cap_hit = |endpoint: Vec3| -> Option<(f32, Vec3)> {
        ray_sphere(ray, endpoint, radius, span).map(|distance| {
            let point = ray.origin + ray.direction * distance;
            let delta = point - endpoint;
            (distance, normalize_or_unit(delta))
        })
    };
    let first = cap_hit(a);
    let second = cap_hit(b);
    match (first, second) {
        (Some(hit), None) => Some(hit),
        (None, Some(hit)) => Some(hit),
        (Some(left), Some(right)) => Some(if left.0 <= right.0 { left } else { right }),
        (None, None) => {
            let axis = b - a;
            let height = axis.length();
            if height <= 1.0e-8 {
                return None;
            }
            let unit = axis * (1.0 / height);
            let dn = ray.direction.dot(unit);
            if dn.abs() <= 1.0e-8 {
                return None;
            }
            let on = (ray.origin - a).dot(unit);
            let mut t0 = -on / dn;
            let mut t1 = (height - on) / dn;
            if t0 > t1 {
                core::mem::swap(&mut t0, &mut t1);
            }
            let t = t0.max(0.0);
            if t0 <= span && t1 >= 0.0 && t <= t1 {
                let point = ray.origin + ray.direction * t;
                Some((
                    t,
                    normalize_or_unit(point - closest_point_on_segment(a, b, point)),
                ))
            } else {
                None
            }
        }
    }
}

/// Ray-AABB slab test with face normals.
#[must_use]
fn ray_box(ray: Ray3, center: Vec3, half: Vec3, span: f32) -> Option<(f32, Vec3)> {
    let min = center - half;
    let max = center + half;
    let mut t_min = 0.0f32;
    let mut t_max = span;
    let mut normal_axis = 0u8;
    let mut normal_sign = 1.0f32;
    for axis in 0..3u8 {
        let o = axis_value(ray.origin, axis);
        let d = axis_value(ray.direction, axis);
        let lo = axis_value(min, axis);
        let hi = axis_value(max, axis);
        if d.abs() <= 1.0e-9 {
            if o < lo || o > hi {
                return None;
            }
            continue;
        }
        let mut t0 = (lo - o) / d;
        let mut t1 = (hi - o) / d;
        if t0 > t1 {
            core::mem::swap(&mut t0, &mut t1);
        }
        if t0 > t_min {
            t_min = t0;
            normal_axis = axis;
            normal_sign = if d > 0.0 { -1.0 } else { 1.0 };
        }
        if t1 < t_max {
            t_max = t1;
        }
        if t_min > t_max {
            return None;
        }
    }
    if t_min < 0.0 || t_min > span {
        return None;
    }
    let normal = match normal_axis {
        0 => Vec3::X * normal_sign,
        1 => Vec3::Y * normal_sign,
        _ => Vec3::Z * normal_sign,
    };
    Some((t_min, normal))
}

#[inline]
#[must_use]
fn axis_value(vector: Vec3, axis: u8) -> f32 {
    match axis {
        0 => vector.x,
        1 => vector.y,
        _ => vector.z,
    }
}
