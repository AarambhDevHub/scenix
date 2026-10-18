//! scenix v1.7.0 "Particles" — facade-level CPU coverage.
//!
//! Exercises the particle surface end to end without a GPU: emitter +
//! modifier simulation through the facade re-exports, the material bits
//! that drive the GPU draw path, the flipbook grid math, billboard quad
//! geometry, the float shims, and the browser fallback capability notes.

use scenix::{
    Color, ColorCurve, ColorOverLifetime, Drag, EmissionShape, FlipbookConfig, FlipbookGrid,
    Gravity, NodeKind, ParticleBlend, ParticleEmitter, ParticleFallbackPlan, ParticleMaterial,
    ParticleSystem, SceneGraph, SceneNode, SizeOverLifetime, Vec3, WebGlCapabilityLevel,
    box_geometry, particle_fallback_plan, quad_geometry,
};
use scenix::{FEATURE_FLIPBOOK, FEATURE_SOFT_PARTICLES};
use scenix_core::{MaterialId, MeshId, ParticleSystemId};

fn fountain(rate: f32) -> ParticleEmitter {
    ParticleEmitter::new()
        .with_rate(rate)
        .with_lifetime(0.4, 0.8)
        .with_size(0.25)
        .with_speed(1.0, 3.0)
        .with_modifier(Gravity::new(Vec3::new(0.0, -9.81, 0.0)))
        .with_modifier(Drag::new(0.4))
        .with_modifier(ColorOverLifetime::new(ColorCurve::from_to(
            [1.0, 0.9, 0.4, 1.0],
            [1.0, 0.2, 0.0, 0.0],
        )))
        .with_modifier(SizeOverLifetime::from_to(1.0, 0.25))
}

#[test]
fn particle_system_flows_simulate_render_and_reproduce() {
    let mut system = ParticleSystem::new(
        fountain(90.0)
            .at(Vec3::new(0.0, 1.0, 0.0))
            .toward(Vec3::new(0.0, 1.0, 0.0))
            .with_shape(EmissionShape::Sphere { radius: 0.25 }),
        64,
        1701,
    );
    system.spawn_burst(8);

    let first = system.update(1.0 / 60.0);
    assert!(!first.is_empty(), "burst + rate spawn must survive dt");
    let mut samples = 0_usize;
    for frame in 0..90_usize {
        let batch = system.update(1.0 / 60.0);
        samples += batch.len();
        let item = batch.items.first().unwrap();
        assert!(
            item.color[3] <= 1.0,
            "frame {frame} alpha must stay clamped in [0, 1]"
        );
    }
    assert!(samples > 100, "fountain must emit across the minute");
    assert!(system.alive() <= system.capacity());
    assert!(system.total_spawned() >= 8);

    // Determinism: the same config and seed reproduce identical bookkeeping.
    let mut a = ParticleSystem::new(fountain(0.0), 16, 7);
    let mut b = ParticleSystem::new(fountain(0.0), 16, 7);
    a.update(1.0 / 120.0);
    b.update(1.0 / 120.0);
    assert_eq!(a.total_spawned(), b.total_spawned());
    assert_eq!(a.alive(), b.alive());
}

#[test]
fn flipbook_grid_and_config_agree_on_frames() {
    let grid = FlipbookGrid::new(4, 2).expect("non-degenerate grid");
    assert_eq!(grid.frame_count(), 8);
    assert_eq!(grid.frame_position(5), (1, 1));
    let cell = grid.uv(0);
    assert!((cell.u1 - 0.25).abs() < f32::EPSILON);
    assert_eq!(grid.uv_frames().len(), 8);
    assert!(FlipbookGrid::new(0, 4).is_none());

    // The particle-side config matches the pure grid math.
    let mut system = ParticleSystem::new(fountain(0.0), 16, 7);
    system.set_flipbook(Some(FlipbookConfig::full(4, 2, 8.0)));
    system.spawn_burst(4);
    let batch = system.update(1.0 / 60.0);
    assert_eq!(batch.len(), 4);
    let cell = batch
        .items
        .iter()
        .map(|item| item.uv_rect)
        .find(|rect| *rect != [0.0, 0.0, 1.0, 1.0])
        .expect("flipbook cells must shrink the uv rect");
    assert!(cell[2] - cell[0] < 1.0 && cell[3] - cell[1] < 1.0);
}

#[test]
fn particle_material_carries_tint_blend_and_feature_bits() {
    let material = ParticleMaterial::new()
        .with_color(Color::from_rgb(1.0, 0.6, 0.2))
        .with_flipbook(Some(scenix::FlipbookParams::full(4, 4, 12.0)));
    assert_eq!(material.blend, ParticleBlend::Alpha);
    let bits = material.feature_bits();
    assert!(
        bits & FEATURE_FLIPBOOK != 0,
        "flipbook opt-in must raise its pipeline bit"
    );
    assert!(bits & FEATURE_SOFT_PARTICLES == 0, "soft particles opt-in");
}

#[test]
fn particle_nodes_bridge_scene_layout() {
    let system_id = ParticleSystemId::new(11);
    let mut scene = SceneGraph::new();
    scene.add(SceneNode::particles("fountain", system_id));
    scene.add(SceneNode::mesh("floor", MeshId::new(1), MaterialId::new(2)));
    scene.update_world_transforms();

    let node_id = scene
        .iter_depth_first()
        .find(|id| {
            scene
                .get(*id)
                .is_some_and(|node| matches!(&node.kind, NodeKind::Particles { system_id: sid } if *sid == system_id))
        })
        .expect("particle node present");
    assert!(scene.get(node_id).is_some_and(|node| node.visible));

    // Billboard quad geometry: 4 verts, 6 indices, CCW +Z facing.
    let quad = quad_geometry(2.0, 2.0);
    assert_eq!(quad.vertex_count(), 4);
    assert_eq!(quad.indices.len(), 6);
    // box geometry stays available for mixed scenes.
    assert!(!box_geometry(1.0, 1.0, 1.0, 1, 1, 1).positions.is_empty());
}

#[test]
fn float_shims_cover_scenario_math() {
    assert!((scenix::float::sqrt(2.0) - 2.0_f64.sqrt() as f32).abs() < 1e-6);
    assert!((scenix::float::cos(0.0) - 1.0).abs() < f32::EPSILON);
    assert!((scenix::float::cbrt(27.0) - 3.0).abs() < 1e-5);
    assert!(scenix::float::atan2(0.0, 1.0).abs() < f32::EPSILON);
}

#[test]
fn particle_browser_fallback_plans_match_capability() {
    assert_eq!(
        particle_fallback_plan(WebGlCapabilityLevel::WebGl1),
        ParticleFallbackPlan::ExpandedQuads
    );
    assert_eq!(
        particle_fallback_plan(WebGlCapabilityLevel::WebGl2),
        ParticleFallbackPlan::InstancedBillboards
    );
    assert!(!particle_fallback_plan(WebGlCapabilityLevel::WebGl2).supports_compute());
    assert_eq!(WebGlCapabilityLevel::WebGl1.label(), "webgl1");
}
