use std::collections::BTreeMap;

use scenix_animato::ScalarTrack;
use scenix_camera::{OrbitController, PerspectiveCamera};
use scenix_core::{Color, LightId, MaterialId, MeshId, NodeId, ScenixError};
use scenix_helpers::{AxesHelper, BoundingBoxHelper, GridHelper, LineGeometry};
use scenix_input::{KeyboardState, PointerState};
use scenix_light::{DirectionalLight, PointLight};
use scenix_material::{
    LambertMaterial, PbrMaterial, PhysicalMaterial, ToonMaterial, UnlitMaterial, WireframeMaterial,
};
use scenix_math::{Aabb, Quat, Transform, Vec2, Vec3};
use scenix_mesh::{Geometry, box_geometry, plane_geometry, sphere_geometry, torus_geometry};
use scenix_raycaster::Raycaster;
use scenix_renderer::{Renderer, RendererConfig, wgpu};
use scenix_scene::{NodeKind, SceneGraph, SceneNode};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::{clamp_canvas_size, key_code_from_dom, pointer_button_from_dom};

const OBJECT_LAYER: u32 = 1;
const HELPER_LAYER: u32 = 2;

#[derive(Clone, Debug)]
struct DemoObject {
    node_id: NodeId,
    material_id: MaterialId,
    name: &'static str,
    material_name: &'static str,
}

struct GeneratedLab {
    scene: SceneGraph,
    camera: PerspectiveCamera,
    orbit: OrbitController,
    geometries: BTreeMap<MeshId, Geometry>,
    objects: Vec<DemoObject>,
    helper_node: NodeId,
    animated_node: NodeId,
}

/// Returns a valid renderer size for a canvas.
pub fn canvas_size(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let css_width = canvas.client_width().max(0) as u32;
    let css_height = canvas.client_height().max(0) as u32;
    let width = if css_width == 0 {
        canvas.width()
    } else {
        css_width
    };
    let height = if css_height == 0 {
        canvas.height()
    } else {
        css_height
    };
    clamp_canvas_size(width, height)
}

/// Browser renderer wrapper with generated scene and DOM input state.
#[wasm_bindgen]
pub struct WebRenderer {
    renderer: Renderer,
    scene: SceneGraph,
    camera: PerspectiveCamera,
    orbit: OrbitController,
    pointer: PointerState,
    keyboard: KeyboardState,
    geometries: BTreeMap<MeshId, Geometry>,
    raycaster: Raycaster,
    objects: Vec<DemoObject>,
    helper_node: NodeId,
    animated_node: NodeId,
    pulse_track: ScalarTrack,
    pulse_forward: bool,
    last_timestamp_ms: Option<f64>,
    scroll_delta: f32,
    fps: f32,
    paused: bool,
    helpers_visible: bool,
    wireframe_enabled: bool,
    bloom_enabled: bool,
    ssao_enabled: bool,
    selected_node: Option<NodeId>,
    selected_name: String,
    selected_distance: f32,
    active_material: String,
}

#[wasm_bindgen]
impl WebRenderer {
    /// Creates a renderer for `canvas` and registers the generated Scenix Engine Lab scene.
    pub async fn new(canvas: HtmlCanvasElement) -> Result<WebRenderer, JsValue> {
        crate::set_panic_hook();
        let (width, height) = canvas_size(&canvas);
        canvas.set_width(width);
        canvas.set_height(height);

        let config = RendererConfig::new(width, height).vsync(true);
        let mut renderer = Renderer::new(wgpu::SurfaceTarget::Canvas(canvas), config)
            .await
            .map_err(js_error)?;
        let lab = generated_lab(&mut renderer, width, height)?;

        Ok(Self {
            renderer,
            scene: lab.scene,
            camera: lab.camera,
            orbit: lab.orbit,
            pointer: PointerState::new(),
            keyboard: KeyboardState::new(),
            geometries: lab.geometries,
            raycaster: Raycaster::with_layers(OBJECT_LAYER),
            objects: lab.objects,
            helper_node: lab.helper_node,
            animated_node: lab.animated_node,
            pulse_track: ScalarTrack::tween(0.0, 1.0, 1.8),
            pulse_forward: true,
            last_timestamp_ms: None,
            scroll_delta: 0.0,
            fps: 0.0,
            paused: false,
            helpers_visible: true,
            wireframe_enabled: false,
            bloom_enabled: false,
            ssao_enabled: false,
            selected_node: None,
            selected_name: String::from("None"),
            selected_distance: 0.0,
            active_material: String::from("None"),
        })
    }

    /// Renders one frame. `timestamp_ms` should come from `requestAnimationFrame`.
    pub fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue> {
        let dt = self
            .last_timestamp_ms
            .map_or(0.0, |last| ((timestamp_ms - last) * 0.001).max(0.0) as f32);
        self.last_timestamp_ms = Some(timestamp_ms);
        if dt > 0.0 {
            self.fps = 1.0 / dt.max(1.0 / 240.0);
        }

        self.orbit
            .update_from_pointer(self.pointer, self.scroll_delta, dt);
        self.orbit.apply_to_perspective(&mut self.camera);
        self.scroll_delta = 0.0;
        self.pointer.clear_delta();

        if !self.paused {
            self.animate_lab(dt);
        }

        self.scene.update_world_transforms();
        self.renderer
            .render(&self.scene, &self.camera)
            .map(|_| ())
            .map_err(js_error)
    }

    /// Resizes the canvas and renderer. Zero dimensions are clamped to one pixel.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        let (width, height) = clamp_canvas_size(width, height);
        self.camera.aspect = width as f32 / height as f32;
        self.renderer.resize(width, height).map_err(js_error)
    }

    /// Updates pointer position.
    pub fn on_pointer_move(&mut self, x: f32, y: f32) {
        self.pointer.set_position(Vec2::new(x, y));
    }

    /// Updates pointer position and pressed state.
    pub fn on_pointer_down(&mut self, button: i16, x: f32, y: f32) {
        self.pointer.set_position(Vec2::new(x, y));
        if let Some(button) = pointer_button_from_dom(button) {
            self.pointer.on_button_down(button);
        }
    }

    /// Updates pointer position, pressed state, and selected object.
    pub fn on_pointer_up(&mut self, button: i16, x: f32, y: f32) {
        self.pointer.set_position(Vec2::new(x, y));
        if let Some(button) = pointer_button_from_dom(button) {
            self.pointer.on_button_up(button);
        }
        self.pick_at(x, y);
    }

    /// Dolly the generated orbit camera in or out.
    pub fn on_wheel(&mut self, delta_y: f32) {
        self.scroll_delta += delta_y.signum() * 0.12;
    }

    /// Marks a DOM key as pressed when it maps to scenix input.
    pub fn on_key_down(&mut self, code: &str) {
        if let Some(code) = key_code_from_dom(code) {
            self.keyboard.on_key_down(code);
        }
    }

    /// Marks a DOM key as released when it maps to scenix input.
    pub fn on_key_up(&mut self, code: &str) {
        if let Some(code) = key_code_from_dom(code) {
            self.keyboard.on_key_up(code);
        }
    }

    /// Enables or pauses animation.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Returns whether animation is paused.
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Shows or hides helper geometry.
    pub fn set_helpers_visible(&mut self, visible: bool) {
        self.helpers_visible = visible;
        if let Some(node) = self.scene.get_mut(self.helper_node) {
            node.visible = visible;
        }
    }

    /// Returns whether helper geometry is visible.
    pub fn helpers_visible(&self) -> bool {
        self.helpers_visible
    }

    /// Enables or disables wireframe preview materials on selectable objects.
    pub fn set_wireframe_enabled(&mut self, enabled: bool) {
        self.wireframe_enabled = enabled;
        let wireframe_id = MaterialId::new(6);
        for object in &self.objects {
            if let Some(node) = self.scene.get_mut(object.node_id)
                && let NodeKind::Mesh { material_id, .. } = &mut node.kind
            {
                *material_id = if enabled {
                    wireframe_id
                } else {
                    object.material_id
                };
            }
        }
    }

    /// Returns whether wireframe preview is enabled.
    pub fn wireframe_enabled(&self) -> bool {
        self.wireframe_enabled
    }

    /// Stores the Bloom UI toggle. The current browser wrapper reports it in feature flags.
    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        self.bloom_enabled = enabled;
    }

    /// Returns whether the Bloom UI toggle is enabled.
    pub fn bloom_enabled(&self) -> bool {
        self.bloom_enabled
    }

    /// Stores the SSAO UI toggle. The current browser wrapper reports it in feature flags.
    pub fn set_ssao_enabled(&mut self, enabled: bool) {
        self.ssao_enabled = enabled;
    }

    /// Returns whether the SSAO UI toggle is enabled.
    pub fn ssao_enabled(&self) -> bool {
        self.ssao_enabled
    }

    /// Restores the default orbit camera.
    pub fn reset_camera(&mut self) {
        self.orbit = default_orbit();
        self.orbit.apply_to_perspective(&mut self.camera);
    }

    /// Returns the generated scene name.
    pub fn scene_name(&self) -> String {
        String::from("Scenix Engine Lab")
    }

    /// Returns the most recent frames-per-second estimate.
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Returns the selected scene node name.
    pub fn selected_node_name(&self) -> String {
        self.selected_name.clone()
    }

    /// Returns the raw selected node ID, or zero when nothing is selected.
    pub fn selected_node_id(&self) -> u64 {
        self.selected_node.map_or(0, NodeId::get)
    }

    /// Returns the current raycast hit distance.
    pub fn raycast_distance(&self) -> f32 {
        self.selected_distance
    }

    /// Returns the active selected material label.
    pub fn active_material(&self) -> String {
        self.active_material.clone()
    }

    /// Returns active browser demo feature flags as a compact string.
    pub fn active_feature_flags(&self) -> String {
        format!(
            "helpers={}, wireframe={}, bloom={}, ssao={}, raycaster=true, animato=true",
            self.helpers_visible, self.wireframe_enabled, self.bloom_enabled, self.ssao_enabled
        )
    }
}

impl WebRenderer {
    fn animate_lab(&mut self, dt: f32) {
        if let Some(node) = self.scene.get_mut(self.animated_node) {
            let euler = node.transform.rotation.to_euler_xyz();
            node.transform.rotation =
                Quat::from_euler_xyz(euler.x + dt * 0.35, euler.y + dt * 0.9, euler.z);
        }

        if !self.pulse_track.update(dt) || self.pulse_track.is_complete() {
            self.pulse_forward = !self.pulse_forward;
            self.pulse_track = if self.pulse_forward {
                ScalarTrack::tween(0.0, 1.0, 1.8)
            } else {
                ScalarTrack::tween(1.0, 0.0, 1.8)
            };
        }
        let lift = self.pulse_track.value() * 0.18;
        if let Some(object) = self.objects.iter().find(|object| object.name == "Sphere")
            && let Some(node) = self.scene.get_mut(object.node_id)
        {
            node.transform.translation.y = 0.85 + lift;
        }
    }

    fn pick_at(&mut self, x: f32, y: f32) {
        let width = self.renderer.config().width.max(1) as f32;
        let height = self.renderer.config().height.max(1) as f32;
        let ndc = Vec2::new((x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0);
        self.scene.update_world_transforms();
        if self
            .raycaster
            .build_bvh(&self.scene, &self.geometries)
            .is_err()
        {
            self.clear_selection();
            return;
        }
        let ray = Raycaster::from_camera_ndc(&self.camera, ndc);
        let Some(hit) = self.raycaster.cast_ray(ray, &self.scene, &self.geometries) else {
            self.clear_selection();
            return;
        };
        self.selected_node = Some(hit.node_id);
        self.selected_distance = hit.distance;
        self.selected_name = self
            .scene
            .get(hit.node_id)
            .map_or_else(|| String::from("Unknown"), |node| node.name.clone());
        self.active_material = self
            .objects
            .iter()
            .find(|object| object.node_id == hit.node_id)
            .map_or_else(
                || String::from("Unknown"),
                |object| String::from(object.material_name),
            );
    }

    fn clear_selection(&mut self) {
        self.selected_node = None;
        self.selected_distance = 0.0;
        self.selected_name = String::from("None");
        self.active_material = String::from("None");
    }
}

fn generated_lab(
    renderer: &mut Renderer,
    width: u32,
    height: u32,
) -> Result<GeneratedLab, JsValue> {
    let cube_mesh = MeshId::new(1);
    let sphere_mesh = MeshId::new(2);
    let torus_mesh = MeshId::new(3);
    let floor_mesh = MeshId::new(4);
    let helper_mesh = MeshId::new(5);

    let pbr_id = MaterialId::new(1);
    let toon_id = MaterialId::new(2);
    let physical_id = MaterialId::new(3);
    let floor_id = MaterialId::new(4);
    let helper_id = MaterialId::new(5);
    let wireframe_id = MaterialId::new(6);

    let mut geometries = BTreeMap::new();
    geometries.insert(
        cube_mesh,
        with_color(
            box_geometry(0.9, 0.9, 0.9, 1, 1, 1),
            Color::from_hex(0x4EA1FF),
        ),
    );
    geometries.insert(
        sphere_mesh,
        with_color(sphere_geometry(0.52, 32, 16), Color::from_hex(0xFFCC66)),
    );
    geometries.insert(
        torus_mesh,
        with_color(
            torus_geometry(0.48, 0.14, 32, 12),
            Color::from_hex(0xD970FF),
        ),
    );
    geometries.insert(
        floor_mesh,
        with_color(plane_geometry(7.0, 7.0, 1, 1), Color::from_hex(0x2D3446)),
    );
    geometries.insert(helper_mesh, helper_geometry());

    for (mesh_id, geometry) in &geometries {
        renderer
            .register_mesh(*mesh_id, geometry)
            .map_err(js_error)?;
    }

    renderer
        .register_pbr_material(
            pbr_id,
            &PbrMaterial::new()
                .named("lab blue PBR")
                .albedo(Color::from_hex(0x4EA1FF))
                .metallic_roughness(0.18, 0.38),
        )
        .map_err(js_error)?;
    let mut toon = ToonMaterial::new().steps(4).outline(0.025, Color::BLACK);
    toon.color = Color::from_hex(0xFFCC66);
    renderer
        .register_toon_material(toon_id, &toon)
        .map_err(js_error)?;
    renderer
        .register_physical_material(
            physical_id,
            &PhysicalMaterial::new()
                .base(
                    PbrMaterial::new()
                        .albedo(Color::from_hex(0xD970FF))
                        .metallic_roughness(0.55, 0.25),
                )
                .clearcoat(0.65, 0.16),
        )
        .map_err(js_error)?;
    renderer
        .register_lambert_material(
            floor_id,
            &LambertMaterial::new().color(Color::from_hex(0x2D3446)),
        )
        .map_err(js_error)?;
    renderer
        .register_unlit_material(
            helper_id,
            &UnlitMaterial::new().color(Color::from_hex(0xA7F3D0)),
        )
        .map_err(js_error)?;
    renderer
        .register_wireframe_material(
            wireframe_id,
            &WireframeMaterial {
                color: Color::from_hex(0xE8F0FF),
                opacity: 0.85,
                line_width: 1.0,
                double_sided: true,
            },
        )
        .map_err(js_error)?;

    renderer
        .register_directional_light(
            LightId::new(1),
            DirectionalLight::new(Vec3::new(-0.45, -0.85, -0.25), Color::WHITE, 3.2),
        )
        .map_err(js_error)?;
    renderer
        .register_point_light(
            LightId::new(2),
            PointLight::new(Color::from_hex(0x66CCFF), 1.6, 5.0),
        )
        .map_err(js_error)?;

    let mut scene = SceneGraph::new();
    let cube = scene.add(
        SceneNode::mesh("Cube", cube_mesh, pbr_id)
            .transform(Transform::from_translation(Vec3::new(-1.25, 0.55, 0.0)))
            .layer(OBJECT_LAYER),
    );
    let sphere = scene.add(
        SceneNode::mesh("Sphere", sphere_mesh, toon_id)
            .transform(Transform::from_translation(Vec3::new(0.0, 0.85, -0.25)))
            .layer(OBJECT_LAYER),
    );
    let torus = scene.add(
        SceneNode::mesh("Torus", torus_mesh, physical_id)
            .transform(Transform::from_translation(Vec3::new(1.25, 0.75, 0.1)))
            .layer(OBJECT_LAYER),
    );
    scene.add(
        SceneNode::mesh("Floor", floor_mesh, floor_id)
            .transform(Transform::from_translation(Vec3::new(0.0, -0.03, 0.0)))
            .layer(OBJECT_LAYER),
    );
    let helper_node =
        scene.add(SceneNode::mesh("Helpers", helper_mesh, helper_id).layer(HELPER_LAYER));
    scene.add(SceneNode::light("Sun", LightId::new(1)));
    scene.add(
        SceneNode::light("Point Light", LightId::new(2))
            .transform(Transform::from_translation(Vec3::new(2.0, 2.1, 1.4))),
    );
    scene.update_world_transforms();

    let aspect = width as f32 / height.max(1) as f32;
    let mut camera = PerspectiveCamera::new(55.0, aspect, 0.1, 100.0);
    let orbit = default_orbit();
    orbit.apply_to_perspective(&mut camera);

    Ok(GeneratedLab {
        scene,
        camera,
        orbit,
        geometries,
        objects: vec![
            DemoObject {
                node_id: cube,
                material_id: pbr_id,
                name: "Cube",
                material_name: "PBR material",
            },
            DemoObject {
                node_id: sphere,
                material_id: toon_id,
                name: "Sphere",
                material_name: "Toon material",
            },
            DemoObject {
                node_id: torus,
                material_id: physical_id,
                name: "Torus",
                material_name: "Physical material",
            },
        ],
        helper_node,
        animated_node: cube,
    })
}

fn default_orbit() -> OrbitController {
    let mut orbit = OrbitController::new(Vec3::new(0.0, 0.65, 0.0), 5.2);
    orbit.theta = 0.58;
    orbit.phi = 1.15;
    orbit.min_distance = 2.4;
    orbit.max_distance = 9.0;
    orbit
}

fn with_color(mut geometry: Geometry, color: Color) -> Geometry {
    geometry.colors.clear();
    geometry.colors.resize(geometry.positions.len(), color);
    if geometry.normals.is_empty() {
        geometry.compute_normals();
    }
    geometry
}

fn helper_geometry() -> Geometry {
    let mut lines = LineGeometry::new();
    lines.merge(&GridHelper::new(7.0, 14).to_geometry());
    lines.merge(&AxesHelper::new(1.8).to_geometry());
    lines.merge(
        &BoundingBoxHelper::new(
            Aabb::new(Vec3::new(-1.7, 0.0, -0.7), Vec3::new(1.7, 1.6, 0.75)),
            Color::from_hex(0xE8F0FF),
        )
        .to_geometry(),
    );
    line_geometry_to_mesh(&lines, 0.01)
}

fn line_geometry_to_mesh(lines: &LineGeometry, radius: f32) -> Geometry {
    let mut geometry = Geometry::new();
    if lines.indices.is_empty() {
        for segment in lines.positions.chunks_exact(2) {
            append_segment_box(
                &mut geometry,
                segment[0],
                segment[1],
                radius,
                Color::from_hex(0xA7F3D0),
            );
        }
    } else {
        for pair in lines.indices.chunks_exact(2) {
            let a = pair[0] as usize;
            let b = pair[1] as usize;
            if a < lines.positions.len() && b < lines.positions.len() {
                let color = lines
                    .colors
                    .get(a)
                    .copied()
                    .unwrap_or(Color::from_hex(0xA7F3D0));
                append_segment_box(
                    &mut geometry,
                    lines.positions[a],
                    lines.positions[b],
                    radius,
                    color,
                );
            }
        }
    }
    geometry.compute_normals();
    geometry
}

fn append_segment_box(geometry: &mut Geometry, start: Vec3, end: Vec3, radius: f32, color: Color) {
    let axis = end - start;
    if axis.length_squared() <= 1.0e-8 {
        return;
    }
    let forward = axis.normalize();
    let reference = if forward.y.abs() < 0.9 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let right = forward.cross(reference).normalize() * radius;
    let up = right.cross(forward).normalize() * radius;
    let base = geometry.positions.len() as u32;
    let corners = [
        start - right - up,
        start + right - up,
        start + right + up,
        start - right + up,
        end - right - up,
        end + right - up,
        end + right + up,
        end - right + up,
    ];
    geometry.positions.extend_from_slice(&corners);
    geometry
        .colors
        .extend(core::iter::repeat_n(color, corners.len()));
    geometry.indices.extend_from_slice(&[
        base,
        base + 1,
        base + 5,
        base,
        base + 5,
        base + 4,
        base + 1,
        base + 2,
        base + 6,
        base + 1,
        base + 6,
        base + 5,
        base + 2,
        base + 3,
        base + 7,
        base + 2,
        base + 7,
        base + 6,
        base + 3,
        base,
        base + 4,
        base + 3,
        base + 4,
        base + 7,
    ]);
}

fn js_error(error: ScenixError) -> JsValue {
    JsValue::from_str(&error.to_string())
}
