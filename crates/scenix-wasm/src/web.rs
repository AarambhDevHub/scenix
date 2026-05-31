use std::collections::BTreeMap;

use js_sys::{Float32Array, Reflect, Uint16Array};
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
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader,
    WebGlUniformLocation, window,
};

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

struct LabRuntime {
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

struct WebGlMesh {
    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    line_index_buffer: WebGlBuffer,
    index_count: i32,
    line_index_count: i32,
}

#[derive(Clone, Copy, Debug)]
struct WebGlMaterial {
    color: Color,
    unlit: bool,
    wireframe: bool,
}

struct WebGlProgramState {
    program: WebGlProgram,
    position_attrib: u32,
    normal_attrib: u32,
    color_attrib: u32,
    view_projection_uniform: WebGlUniformLocation,
    model_uniform: WebGlUniformLocation,
    material_uniform: WebGlUniformLocation,
    light_direction_uniform: WebGlUniformLocation,
    unlit_uniform: WebGlUniformLocation,
    bloom_uniform: WebGlUniformLocation,
    ssao_uniform: WebGlUniformLocation,
}

/// Preferred browser rendering backend.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserBackendPreference {
    /// Select WebGPU when the browser is known to support it safely, otherwise WebGL.
    Auto,
    /// Force the existing WebGPU/wgpu renderer.
    WebGpu,
    /// Force the WebGL compatibility renderer.
    WebGl,
}

/// Active browser rendering backend.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserBackendKind {
    /// The existing WebGPU/wgpu renderer is active.
    WebGpu,
    /// The WebGL compatibility renderer is active.
    WebGl,
    /// The caller is using an application-level Canvas2D fallback.
    CanvasFallback,
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
    lab: LabRuntime,
}

impl LabRuntime {
    fn new(width: u32, height: u32) -> Self {
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

        Self {
            scene,
            camera,
            orbit,
            pointer: PointerState::new(),
            keyboard: KeyboardState::new(),
            geometries,
            raycaster: Raycaster::with_layers(OBJECT_LAYER),
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
        }
    }

    fn tick(&mut self, timestamp_ms: f64) {
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
    }
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

        Ok(Self { renderer, lab })
    }

    /// Renders one frame. `timestamp_ms` should come from `requestAnimationFrame`.
    pub fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue> {
        self.lab.tick(timestamp_ms);
        self.renderer
            .render(&self.lab.scene, &self.lab.camera)
            .map(|_| ())
            .map_err(js_error)
    }

    /// Resizes the canvas and renderer. Zero dimensions are clamped to one pixel.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        let (width, height) = clamp_canvas_size(width, height);
        self.lab.resize(width, height);
        self.renderer.resize(width, height).map_err(js_error)
    }

    /// Updates pointer position.
    pub fn on_pointer_move(&mut self, x: f32, y: f32) {
        self.lab.on_pointer_move(x, y);
    }

    /// Updates pointer position and pressed state.
    pub fn on_pointer_down(&mut self, button: i16, x: f32, y: f32) {
        self.lab.on_pointer_down(button, x, y);
    }

    /// Updates pointer position, pressed state, and selected object.
    pub fn on_pointer_up(&mut self, button: i16, x: f32, y: f32) {
        let width = self.renderer.config().width.max(1) as f32;
        let height = self.renderer.config().height.max(1) as f32;
        self.lab.on_pointer_up(button, x, y, width, height);
    }

    /// Dolly the generated orbit camera in or out.
    pub fn on_wheel(&mut self, delta_y: f32) {
        self.lab.on_wheel(delta_y);
    }

    /// Marks a DOM key as pressed when it maps to scenix input.
    pub fn on_key_down(&mut self, code: &str) {
        self.lab.on_key_down(code);
    }

    /// Marks a DOM key as released when it maps to scenix input.
    pub fn on_key_up(&mut self, code: &str) {
        self.lab.on_key_up(code);
    }

    /// Enables or pauses animation.
    pub fn set_paused(&mut self, paused: bool) {
        self.lab.set_paused(paused);
    }

    /// Returns whether animation is paused.
    pub fn paused(&self) -> bool {
        self.lab.paused()
    }

    /// Shows or hides helper geometry.
    pub fn set_helpers_visible(&mut self, visible: bool) {
        self.lab.set_helpers_visible(visible);
    }

    /// Returns whether helper geometry is visible.
    pub fn helpers_visible(&self) -> bool {
        self.lab.helpers_visible()
    }

    /// Enables or disables wireframe preview materials on selectable objects.
    pub fn set_wireframe_enabled(&mut self, enabled: bool) {
        self.lab.set_wireframe_enabled(enabled);
    }

    /// Returns whether wireframe preview is enabled.
    pub fn wireframe_enabled(&self) -> bool {
        self.lab.wireframe_enabled()
    }

    /// Stores the Bloom UI toggle. The current browser wrapper reports it in feature flags.
    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        self.lab.set_bloom_enabled(enabled);
    }

    /// Returns whether the Bloom UI toggle is enabled.
    pub fn bloom_enabled(&self) -> bool {
        self.lab.bloom_enabled()
    }

    /// Stores the SSAO UI toggle. The current browser wrapper reports it in feature flags.
    pub fn set_ssao_enabled(&mut self, enabled: bool) {
        self.lab.set_ssao_enabled(enabled);
    }

    /// Returns whether the SSAO UI toggle is enabled.
    pub fn ssao_enabled(&self) -> bool {
        self.lab.ssao_enabled()
    }

    /// Restores the default orbit camera.
    pub fn reset_camera(&mut self) {
        self.lab.reset_camera();
    }

    /// Returns the generated scene name.
    pub fn scene_name(&self) -> String {
        String::from("Scenix Engine Lab")
    }

    /// Returns the most recent frames-per-second estimate.
    pub fn fps(&self) -> f32 {
        self.lab.fps()
    }

    /// Returns the selected scene node name.
    pub fn selected_node_name(&self) -> String {
        self.lab.selected_node_name()
    }

    /// Returns the raw selected node ID, or zero when nothing is selected.
    pub fn selected_node_id(&self) -> u64 {
        self.lab.selected_node_id()
    }

    /// Returns the current raycast hit distance.
    pub fn raycast_distance(&self) -> f32 {
        self.lab.raycast_distance()
    }

    /// Returns the active selected material label.
    pub fn active_material(&self) -> String {
        self.lab.active_material()
    }

    /// Returns active browser demo feature flags as a compact string.
    pub fn active_feature_flags(&self) -> String {
        format!(
            "backend=webgpu, helpers={}, wireframe={}, bloom={}, ssao={}, raycaster=true, animato=true",
            self.lab.helpers_visible(),
            self.lab.wireframe_enabled(),
            self.lab.bloom_enabled(),
            self.lab.ssao_enabled()
        )
    }
}

enum BrowserBackend {
    WebGpu(Box<WebRenderer>),
    WebGl(Box<WebGlRenderer>),
}

/// Browser renderer that selects WebGPU when safe and WebGL otherwise.
#[wasm_bindgen]
pub struct BrowserRenderer {
    backend: BrowserBackend,
}

#[wasm_bindgen]
impl BrowserRenderer {
    /// Creates a browser renderer with automatic backend selection.
    pub async fn new(canvas: HtmlCanvasElement) -> Result<BrowserRenderer, JsValue> {
        Self::new_with_preference(canvas, BrowserBackendPreference::Auto).await
    }

    /// Creates a browser renderer with an explicit backend preference.
    pub async fn new_with_preference(
        canvas: HtmlCanvasElement,
        preference: BrowserBackendPreference,
    ) -> Result<BrowserRenderer, JsValue> {
        match preference {
            BrowserBackendPreference::WebGpu => {
                WebRenderer::new(canvas).await.map(|renderer| Self {
                    backend: BrowserBackend::WebGpu(Box::new(renderer)),
                })
            }
            BrowserBackendPreference::WebGl => {
                WebGlRenderer::new(canvas).await.map(|renderer| Self {
                    backend: BrowserBackend::WebGl(Box::new(renderer)),
                })
            }
            BrowserBackendPreference::Auto => {
                if should_try_webgpu() {
                    let webgl_canvas = canvas.clone();
                    match WebRenderer::new(canvas).await {
                        Ok(renderer) => Ok(Self {
                            backend: BrowserBackend::WebGpu(Box::new(renderer)),
                        }),
                        Err(_) => WebGlRenderer::new(webgl_canvas).await.map(|renderer| Self {
                            backend: BrowserBackend::WebGl(Box::new(renderer)),
                        }),
                    }
                } else {
                    WebGlRenderer::new(canvas).await.map(|renderer| Self {
                        backend: BrowserBackend::WebGl(Box::new(renderer)),
                    })
                }
            }
        }
    }

    /// Renders one frame.
    pub fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue> {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.tick(timestamp_ms),
            BrowserBackend::WebGl(renderer) => renderer.tick(timestamp_ms),
        }
    }

    /// Resizes the active browser backend.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.resize(width, height),
            BrowserBackend::WebGl(renderer) => renderer.resize(width, height),
        }
    }

    /// Updates pointer position.
    pub fn on_pointer_move(&mut self, x: f32, y: f32) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.on_pointer_move(x, y),
            BrowserBackend::WebGl(renderer) => renderer.on_pointer_move(x, y),
        }
    }

    /// Updates pointer pressed state.
    pub fn on_pointer_down(&mut self, button: i16, x: f32, y: f32) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.on_pointer_down(button, x, y),
            BrowserBackend::WebGl(renderer) => renderer.on_pointer_down(button, x, y),
        }
    }

    /// Updates pointer release state and runs picking.
    pub fn on_pointer_up(&mut self, button: i16, x: f32, y: f32) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.on_pointer_up(button, x, y),
            BrowserBackend::WebGl(renderer) => renderer.on_pointer_up(button, x, y),
        }
    }

    /// Updates wheel dolly input.
    pub fn on_wheel(&mut self, delta_y: f32) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.on_wheel(delta_y),
            BrowserBackend::WebGl(renderer) => renderer.on_wheel(delta_y),
        }
    }

    /// Updates key pressed state.
    pub fn on_key_down(&mut self, code: &str) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.on_key_down(code),
            BrowserBackend::WebGl(renderer) => renderer.on_key_down(code),
        }
    }

    /// Updates key released state.
    pub fn on_key_up(&mut self, code: &str) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.on_key_up(code),
            BrowserBackend::WebGl(renderer) => renderer.on_key_up(code),
        }
    }

    /// Enables or pauses animation.
    pub fn set_paused(&mut self, paused: bool) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.set_paused(paused),
            BrowserBackend::WebGl(renderer) => renderer.set_paused(paused),
        }
    }

    /// Returns whether animation is paused.
    pub fn paused(&self) -> bool {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.paused(),
            BrowserBackend::WebGl(renderer) => renderer.paused(),
        }
    }

    /// Shows or hides helper geometry.
    pub fn set_helpers_visible(&mut self, visible: bool) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.set_helpers_visible(visible),
            BrowserBackend::WebGl(renderer) => renderer.set_helpers_visible(visible),
        }
    }

    /// Returns whether helper geometry is visible.
    pub fn helpers_visible(&self) -> bool {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.helpers_visible(),
            BrowserBackend::WebGl(renderer) => renderer.helpers_visible(),
        }
    }

    /// Enables or disables wireframe preview.
    pub fn set_wireframe_enabled(&mut self, enabled: bool) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.set_wireframe_enabled(enabled),
            BrowserBackend::WebGl(renderer) => renderer.set_wireframe_enabled(enabled),
        }
    }

    /// Returns whether wireframe preview is enabled.
    pub fn wireframe_enabled(&self) -> bool {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.wireframe_enabled(),
            BrowserBackend::WebGl(renderer) => renderer.wireframe_enabled(),
        }
    }

    /// Stores the Bloom UI toggle.
    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.set_bloom_enabled(enabled),
            BrowserBackend::WebGl(renderer) => renderer.set_bloom_enabled(enabled),
        }
    }

    /// Returns whether the Bloom UI toggle is enabled.
    pub fn bloom_enabled(&self) -> bool {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.bloom_enabled(),
            BrowserBackend::WebGl(renderer) => renderer.bloom_enabled(),
        }
    }

    /// Stores the SSAO UI toggle.
    pub fn set_ssao_enabled(&mut self, enabled: bool) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.set_ssao_enabled(enabled),
            BrowserBackend::WebGl(renderer) => renderer.set_ssao_enabled(enabled),
        }
    }

    /// Returns whether the SSAO UI toggle is enabled.
    pub fn ssao_enabled(&self) -> bool {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.ssao_enabled(),
            BrowserBackend::WebGl(renderer) => renderer.ssao_enabled(),
        }
    }

    /// Restores the default orbit camera.
    pub fn reset_camera(&mut self) {
        match &mut self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.reset_camera(),
            BrowserBackend::WebGl(renderer) => renderer.reset_camera(),
        }
    }

    /// Returns the active backend kind.
    pub fn backend_kind(&self) -> BrowserBackendKind {
        match &self.backend {
            BrowserBackend::WebGpu(_) => BrowserBackendKind::WebGpu,
            BrowserBackend::WebGl(_) => BrowserBackendKind::WebGl,
        }
    }

    /// Returns the active backend label.
    pub fn backend_label(&self) -> String {
        match self.backend_kind() {
            BrowserBackendKind::WebGpu => String::from("webgpu"),
            BrowserBackendKind::WebGl => String::from("webgl"),
            BrowserBackendKind::CanvasFallback => String::from("canvas-fallback"),
        }
    }

    /// Returns the generated scene name.
    pub fn scene_name(&self) -> String {
        String::from("Scenix Engine Lab")
    }

    /// Returns the most recent frames-per-second estimate.
    pub fn fps(&self) -> f32 {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.fps(),
            BrowserBackend::WebGl(renderer) => renderer.fps(),
        }
    }

    /// Returns the selected scene node name.
    pub fn selected_node_name(&self) -> String {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.selected_node_name(),
            BrowserBackend::WebGl(renderer) => renderer.selected_node_name(),
        }
    }

    /// Returns the selected node ID, or zero when nothing is selected.
    pub fn selected_node_id(&self) -> u64 {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.selected_node_id(),
            BrowserBackend::WebGl(renderer) => renderer.selected_node_id(),
        }
    }

    /// Returns the current raycast hit distance.
    pub fn raycast_distance(&self) -> f32 {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.raycast_distance(),
            BrowserBackend::WebGl(renderer) => renderer.raycast_distance(),
        }
    }

    /// Returns the active selected material label.
    pub fn active_material(&self) -> String {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.active_material(),
            BrowserBackend::WebGl(renderer) => renderer.active_material(),
        }
    }

    /// Returns active browser demo feature flags as a compact string.
    pub fn active_feature_flags(&self) -> String {
        match &self.backend {
            BrowserBackend::WebGpu(renderer) => renderer.active_feature_flags(),
            BrowserBackend::WebGl(renderer) => renderer.active_feature_flags(),
        }
    }
}

/// WebGL compatibility renderer for browsers without usable WebGPU.
#[wasm_bindgen]
pub struct WebGlRenderer {
    canvas: HtmlCanvasElement,
    gl: WebGlRenderingContext,
    program: WebGlProgramState,
    lab: LabRuntime,
    meshes: BTreeMap<MeshId, WebGlMesh>,
    materials: BTreeMap<MaterialId, WebGlMaterial>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl WebGlRenderer {
    /// Creates a WebGL renderer for the generated Scenix Engine Lab scene.
    pub async fn new(canvas: HtmlCanvasElement) -> Result<WebGlRenderer, JsValue> {
        crate::set_panic_hook();
        let (width, height) = canvas_size(&canvas);
        let gl = webgl_context(&canvas)?;
        let program = create_webgl_program(&gl)?;
        let lab = LabRuntime::new(width, height);
        let mut renderer = Self {
            canvas,
            gl,
            program,
            lab,
            meshes: BTreeMap::new(),
            materials: BTreeMap::new(),
            width,
            height,
        };
        renderer.resize(width, height)?;
        renderer.register_lab_assets()?;
        Ok(renderer)
    }

    /// Renders one WebGL frame.
    pub fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue> {
        self.lab.tick(timestamp_ms);
        self.draw();
        Ok(())
    }

    /// Resizes the WebGL viewport. Zero dimensions are clamped to one pixel.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        let (width, height) = clamp_canvas_size(width, height);
        self.width = width;
        self.height = height;
        self.lab.resize(width, height);
        let ratio = window().map_or(1.0, |window| window.device_pixel_ratio().max(1.0));
        let pixel_width = (width as f64 * ratio).round().max(1.0) as u32;
        let pixel_height = (height as f64 * ratio).round().max(1.0) as u32;
        self.canvas.set_width(pixel_width);
        self.canvas.set_height(pixel_height);
        self.gl
            .viewport(0, 0, pixel_width as i32, pixel_height as i32);
        Ok(())
    }

    /// Updates pointer position.
    pub fn on_pointer_move(&mut self, x: f32, y: f32) {
        self.lab.on_pointer_move(x, y);
    }

    /// Updates pointer pressed state.
    pub fn on_pointer_down(&mut self, button: i16, x: f32, y: f32) {
        self.lab.on_pointer_down(button, x, y);
    }

    /// Updates pointer release state and runs picking.
    pub fn on_pointer_up(&mut self, button: i16, x: f32, y: f32) {
        self.lab.on_pointer_up(
            button,
            x,
            y,
            self.width.max(1) as f32,
            self.height.max(1) as f32,
        );
    }

    /// Updates wheel dolly input.
    pub fn on_wheel(&mut self, delta_y: f32) {
        self.lab.on_wheel(delta_y);
    }

    /// Updates key pressed state.
    pub fn on_key_down(&mut self, code: &str) {
        self.lab.on_key_down(code);
    }

    /// Updates key released state.
    pub fn on_key_up(&mut self, code: &str) {
        self.lab.on_key_up(code);
    }

    /// Enables or pauses animation.
    pub fn set_paused(&mut self, paused: bool) {
        self.lab.set_paused(paused);
    }

    /// Returns whether animation is paused.
    pub fn paused(&self) -> bool {
        self.lab.paused()
    }

    /// Shows or hides helper geometry.
    pub fn set_helpers_visible(&mut self, visible: bool) {
        self.lab.set_helpers_visible(visible);
    }

    /// Returns whether helper geometry is visible.
    pub fn helpers_visible(&self) -> bool {
        self.lab.helpers_visible()
    }

    /// Enables or disables wireframe preview.
    pub fn set_wireframe_enabled(&mut self, enabled: bool) {
        self.lab.set_wireframe_enabled(enabled);
    }

    /// Returns whether wireframe preview is enabled.
    pub fn wireframe_enabled(&self) -> bool {
        self.lab.wireframe_enabled()
    }

    /// Stores the Bloom UI toggle.
    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        self.lab.set_bloom_enabled(enabled);
    }

    /// Returns whether the Bloom UI toggle is enabled.
    pub fn bloom_enabled(&self) -> bool {
        self.lab.bloom_enabled()
    }

    /// Stores the SSAO UI toggle.
    pub fn set_ssao_enabled(&mut self, enabled: bool) {
        self.lab.set_ssao_enabled(enabled);
    }

    /// Returns whether the SSAO UI toggle is enabled.
    pub fn ssao_enabled(&self) -> bool {
        self.lab.ssao_enabled()
    }

    /// Restores the default orbit camera.
    pub fn reset_camera(&mut self) {
        self.lab.reset_camera();
    }

    /// Returns the generated scene name.
    pub fn scene_name(&self) -> String {
        String::from("Scenix Engine Lab")
    }

    /// Returns the most recent frames-per-second estimate.
    pub fn fps(&self) -> f32 {
        self.lab.fps()
    }

    /// Returns the selected scene node name.
    pub fn selected_node_name(&self) -> String {
        self.lab.selected_node_name()
    }

    /// Returns the selected node ID, or zero when nothing is selected.
    pub fn selected_node_id(&self) -> u64 {
        self.lab.selected_node_id()
    }

    /// Returns the current raycast hit distance.
    pub fn raycast_distance(&self) -> f32 {
        self.lab.raycast_distance()
    }

    /// Returns the active selected material label.
    pub fn active_material(&self) -> String {
        self.lab.active_material()
    }

    /// Returns active WebGL feature flags as a compact string.
    pub fn active_feature_flags(&self) -> String {
        format!(
            "backend=webgl, helpers={}, wireframe={}, bloom={}, ssao={}, raycaster=true, animato=true",
            self.lab.helpers_visible(),
            self.lab.wireframe_enabled(),
            self.lab.bloom_enabled(),
            self.lab.ssao_enabled()
        )
    }
}

impl WebGlRenderer {
    fn register_lab_assets(&mut self) -> Result<(), JsValue> {
        let geometries: Vec<(MeshId, Geometry)> = self
            .lab
            .geometries
            .iter()
            .map(|(id, geometry)| (*id, geometry.clone()))
            .collect();
        for (mesh_id, geometry) in geometries {
            self.register_mesh(mesh_id, &geometry)?;
        }
        self.register_pbr_material(
            MaterialId::new(1),
            &PbrMaterial::new()
                .named("lab blue PBR")
                .albedo(Color::from_hex(0x4EA1FF))
                .metallic_roughness(0.18, 0.38),
        );
        let mut toon = ToonMaterial::new().steps(4).outline(0.025, Color::BLACK);
        toon.color = Color::from_hex(0xFFCC66);
        self.register_toon_material(MaterialId::new(2), &toon);
        self.register_physical_material(
            MaterialId::new(3),
            &PhysicalMaterial::new()
                .base(
                    PbrMaterial::new()
                        .albedo(Color::from_hex(0xD970FF))
                        .metallic_roughness(0.55, 0.25),
                )
                .clearcoat(0.65, 0.16),
        );
        self.register_lambert_material(
            MaterialId::new(4),
            &LambertMaterial::new().color(Color::from_hex(0x2D3446)),
        );
        self.register_unlit_material(
            MaterialId::new(5),
            &UnlitMaterial::new().color(Color::from_hex(0xA7F3D0)),
        );
        self.register_wireframe_material(
            MaterialId::new(6),
            &WireframeMaterial {
                color: Color::from_hex(0xE8F0FF),
                opacity: 0.85,
                line_width: 1.0,
                double_sided: true,
            },
        );
        Ok(())
    }

    fn register_mesh(&mut self, mesh_id: MeshId, geometry: &Geometry) -> Result<(), JsValue> {
        if geometry.positions.len() > u16::MAX as usize {
            return Err(JsValue::from_str(
                "WebGL fallback supports up to 65535 vertices per mesh",
            ));
        }
        geometry
            .validate()
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        let vertex_count = geometry.positions.len();
        let mut vertices = Vec::with_capacity(vertex_count * 10);
        for index in 0..vertex_count {
            let position = geometry.positions[index];
            let normal = geometry.normals.get(index).copied().unwrap_or(Vec3::Y);
            let color = geometry.colors.get(index).copied().unwrap_or(Color::WHITE);
            vertices.extend_from_slice(&[
                position.x, position.y, position.z, normal.x, normal.y, normal.z, color.r, color.g,
                color.b, color.a,
            ]);
        }

        let indices: Vec<u16> = if geometry.indices.is_empty() {
            (0..vertex_count as u16).collect()
        } else {
            geometry.indices.iter().map(|index| *index as u16).collect()
        };
        let mut line_indices = Vec::with_capacity(indices.len() * 2);
        for triangle in indices.chunks_exact(3) {
            line_indices.extend_from_slice(&[
                triangle[0],
                triangle[1],
                triangle[1],
                triangle[2],
                triangle[2],
                triangle[0],
            ]);
        }

        let vertex_buffer = self.create_array_buffer(&vertices)?;
        let index_buffer = self.create_element_buffer(&indices)?;
        let line_index_buffer = self.create_element_buffer(&line_indices)?;
        self.meshes.insert(
            mesh_id,
            WebGlMesh {
                vertex_buffer,
                index_buffer,
                line_index_buffer,
                index_count: indices.len() as i32,
                line_index_count: line_indices.len() as i32,
            },
        );
        Ok(())
    }

    fn register_pbr_material(&mut self, id: MaterialId, material: &PbrMaterial) {
        self.materials.insert(
            id,
            WebGlMaterial {
                color: material.albedo,
                unlit: false,
                wireframe: false,
            },
        );
    }

    fn register_physical_material(&mut self, id: MaterialId, material: &PhysicalMaterial) {
        self.materials.insert(
            id,
            WebGlMaterial {
                color: material.base.albedo,
                unlit: false,
                wireframe: false,
            },
        );
    }

    fn register_unlit_material(&mut self, id: MaterialId, material: &UnlitMaterial) {
        self.materials.insert(
            id,
            WebGlMaterial {
                color: material.color,
                unlit: true,
                wireframe: false,
            },
        );
    }

    fn register_lambert_material(&mut self, id: MaterialId, material: &LambertMaterial) {
        self.materials.insert(
            id,
            WebGlMaterial {
                color: material.color,
                unlit: false,
                wireframe: false,
            },
        );
    }

    fn register_toon_material(&mut self, id: MaterialId, material: &ToonMaterial) {
        self.materials.insert(
            id,
            WebGlMaterial {
                color: material.color,
                unlit: false,
                wireframe: false,
            },
        );
    }

    fn register_wireframe_material(&mut self, id: MaterialId, material: &WireframeMaterial) {
        self.materials.insert(
            id,
            WebGlMaterial {
                color: Color::rgba(
                    material.color.r,
                    material.color.g,
                    material.color.b,
                    material.opacity.min(material.color.a),
                ),
                unlit: true,
                wireframe: true,
            },
        );
    }

    fn create_array_buffer(&self, values: &[f32]) -> Result<WebGlBuffer, JsValue> {
        let buffer = self
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("failed to create WebGL vertex buffer"))?;
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        let array = Float32Array::new_with_length(values.len() as u32);
        array.copy_from(values);
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        Ok(buffer)
    }

    fn create_element_buffer(&self, values: &[u16]) -> Result<WebGlBuffer, JsValue> {
        let buffer = self
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("failed to create WebGL index buffer"))?;
        self.gl
            .bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));
        let array = Uint16Array::new_with_length(values.len() as u32);
        array.copy_from(values);
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        Ok(buffer)
    }

    fn draw(&mut self) {
        self.gl.enable(WebGlRenderingContext::DEPTH_TEST);
        self.gl.depth_func(WebGlRenderingContext::LEQUAL);
        self.gl.disable(WebGlRenderingContext::CULL_FACE);
        let clear = if self.lab.ssao_enabled() { 0.025 } else { 0.04 };
        self.gl.clear_color(clear, clear * 1.6, clear * 2.5, 1.0);
        self.gl.clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );
        self.gl.use_program(Some(&self.program.program));
        self.gl.uniform3f(
            Some(&self.program.light_direction_uniform),
            -0.45,
            -0.85,
            -0.25,
        );
        self.gl.uniform1f(
            Some(&self.program.bloom_uniform),
            if self.lab.bloom_enabled() { 1.0 } else { 0.0 },
        );
        self.gl.uniform1f(
            Some(&self.program.ssao_uniform),
            if self.lab.ssao_enabled() { 1.0 } else { 0.0 },
        );

        let view_projection = self.lab.camera.view_projection().to_cols_array();
        self.gl.uniform_matrix4fv_with_f32_array(
            Some(&self.program.view_projection_uniform),
            false,
            &view_projection,
        );

        for node_id in self.lab.scene.iter_depth_first() {
            let Some(node) = self.lab.scene.get(node_id) else {
                continue;
            };
            if !node.visible {
                continue;
            }
            let (mesh_id, material_id) = match &node.kind {
                NodeKind::Mesh {
                    mesh_id,
                    material_id,
                } => (*mesh_id, *material_id),
                _ => continue,
            };
            let Some(mesh) = self.meshes.get(&mesh_id) else {
                continue;
            };
            let material = self
                .materials
                .get(&material_id)
                .copied()
                .unwrap_or(WebGlMaterial {
                    color: Color::WHITE,
                    unlit: false,
                    wireframe: false,
                });
            let model = self
                .lab
                .scene
                .world_matrix(node_id)
                .unwrap_or(scenix_math::Mat4::IDENTITY)
                .to_cols_array();
            let color = material.color.to_array();
            self.gl.uniform_matrix4fv_with_f32_array(
                Some(&self.program.model_uniform),
                false,
                &model,
            );
            self.gl
                .uniform4fv_with_f32_array(Some(&self.program.material_uniform), &color);
            self.gl.uniform1f(
                Some(&self.program.unlit_uniform),
                if material.unlit || material.wireframe {
                    1.0
                } else {
                    0.0
                },
            );
            self.bind_mesh(mesh);
            if material.wireframe {
                self.gl.bind_buffer(
                    WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                    Some(&mesh.line_index_buffer),
                );
                self.gl.draw_elements_with_i32(
                    WebGlRenderingContext::LINES,
                    mesh.line_index_count,
                    WebGlRenderingContext::UNSIGNED_SHORT,
                    0,
                );
            } else {
                self.gl.bind_buffer(
                    WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                    Some(&mesh.index_buffer),
                );
                self.gl.draw_elements_with_i32(
                    WebGlRenderingContext::TRIANGLES,
                    mesh.index_count,
                    WebGlRenderingContext::UNSIGNED_SHORT,
                    0,
                );
            }
        }
    }

    fn bind_mesh(&self, mesh: &WebGlMesh) {
        const STRIDE: i32 = 10 * 4;
        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&mesh.vertex_buffer),
        );
        self.gl
            .enable_vertex_attrib_array(self.program.position_attrib);
        self.gl.vertex_attrib_pointer_with_i32(
            self.program.position_attrib,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            STRIDE,
            0,
        );
        self.gl
            .enable_vertex_attrib_array(self.program.normal_attrib);
        self.gl.vertex_attrib_pointer_with_i32(
            self.program.normal_attrib,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            STRIDE,
            3 * 4,
        );
        self.gl
            .enable_vertex_attrib_array(self.program.color_attrib);
        self.gl.vertex_attrib_pointer_with_i32(
            self.program.color_attrib,
            4,
            WebGlRenderingContext::FLOAT,
            false,
            STRIDE,
            6 * 4,
        );
    }
}

impl LabRuntime {
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

    fn resize(&mut self, width: u32, height: u32) {
        self.camera.aspect = width as f32 / height.max(1) as f32;
    }

    fn on_pointer_move(&mut self, x: f32, y: f32) {
        self.pointer.set_position(Vec2::new(x, y));
    }

    fn on_pointer_down(&mut self, button: i16, x: f32, y: f32) {
        self.pointer.set_position(Vec2::new(x, y));
        if let Some(button) = pointer_button_from_dom(button) {
            self.pointer.on_button_down(button);
        }
    }

    fn on_pointer_up(&mut self, button: i16, x: f32, y: f32, width: f32, height: f32) {
        self.pointer.set_position(Vec2::new(x, y));
        if let Some(button) = pointer_button_from_dom(button) {
            self.pointer.on_button_up(button);
        }
        self.pick_at(x, y, width, height);
    }

    fn on_wheel(&mut self, delta_y: f32) {
        self.scroll_delta += delta_y.signum() * 0.12;
    }

    fn on_key_down(&mut self, code: &str) {
        if let Some(code) = key_code_from_dom(code) {
            self.keyboard.on_key_down(code);
        }
    }

    fn on_key_up(&mut self, code: &str) {
        if let Some(code) = key_code_from_dom(code) {
            self.keyboard.on_key_up(code);
        }
    }

    fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    fn paused(&self) -> bool {
        self.paused
    }

    fn set_helpers_visible(&mut self, visible: bool) {
        self.helpers_visible = visible;
        if let Some(node) = self.scene.get_mut(self.helper_node) {
            node.visible = visible;
        }
    }

    fn helpers_visible(&self) -> bool {
        self.helpers_visible
    }

    fn set_wireframe_enabled(&mut self, enabled: bool) {
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

    fn wireframe_enabled(&self) -> bool {
        self.wireframe_enabled
    }

    fn set_bloom_enabled(&mut self, enabled: bool) {
        self.bloom_enabled = enabled;
    }

    fn bloom_enabled(&self) -> bool {
        self.bloom_enabled
    }

    fn set_ssao_enabled(&mut self, enabled: bool) {
        self.ssao_enabled = enabled;
    }

    fn ssao_enabled(&self) -> bool {
        self.ssao_enabled
    }

    fn reset_camera(&mut self) {
        self.orbit = default_orbit();
        self.orbit.apply_to_perspective(&mut self.camera);
    }

    fn fps(&self) -> f32 {
        self.fps
    }

    fn selected_node_name(&self) -> String {
        self.selected_name.clone()
    }

    fn selected_node_id(&self) -> u64 {
        self.selected_node.map_or(0, NodeId::get)
    }

    fn raycast_distance(&self) -> f32 {
        self.selected_distance
    }

    fn active_material(&self) -> String {
        self.active_material.clone()
    }

    fn pick_at(&mut self, x: f32, y: f32, width: f32, height: f32) {
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

fn generated_lab(renderer: &mut Renderer, width: u32, height: u32) -> Result<LabRuntime, JsValue> {
    let lab = LabRuntime::new(width, height);
    register_lab_assets_wgpu(renderer, &lab.geometries)?;
    Ok(lab)
}

fn register_lab_assets_wgpu(
    renderer: &mut Renderer,
    geometries: &BTreeMap<MeshId, Geometry>,
) -> Result<(), JsValue> {
    let pbr_id = MaterialId::new(1);
    let toon_id = MaterialId::new(2);
    let physical_id = MaterialId::new(3);
    let floor_id = MaterialId::new(4);
    let helper_id = MaterialId::new(5);
    let wireframe_id = MaterialId::new(6);

    for (mesh_id, geometry) in geometries {
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
    Ok(())
}

fn should_try_webgpu() -> bool {
    let Some(window) = window() else {
        return false;
    };
    let user_agent = window
        .navigator()
        .user_agent()
        .unwrap_or_default()
        .to_lowercase();
    if user_agent.contains("firefox") {
        return false;
    }
    let is_safari = user_agent.contains("safari")
        && !user_agent.contains("chrome")
        && !user_agent.contains("chromium")
        && !user_agent.contains("edg/");
    if is_safari {
        return false;
    }
    let navigator = JsValue::from(window.navigator());
    Reflect::has(&navigator, &JsValue::from_str("gpu")).unwrap_or(false)
}

fn webgl_context(canvas: &HtmlCanvasElement) -> Result<WebGlRenderingContext, JsValue> {
    canvas
        .get_context("webgl")?
        .or_else(|| canvas.get_context("experimental-webgl").ok().flatten())
        .ok_or_else(|| JsValue::from_str("WebGL is not available for this canvas"))?
        .dyn_into::<WebGlRenderingContext>()
        .map_err(|_| JsValue::from_str("canvas context is not a WebGLRenderingContext"))
}

fn create_webgl_program(gl: &WebGlRenderingContext) -> Result<WebGlProgramState, JsValue> {
    let vertex = compile_shader(
        gl,
        WebGlRenderingContext::VERTEX_SHADER,
        WEBGL_VERTEX_SHADER,
    )?;
    let fragment = compile_shader(
        gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        WEBGL_FRAGMENT_SHADER,
    )?;
    let program = link_program(gl, &vertex, &fragment)?;
    gl.use_program(Some(&program));
    let position_attrib = attrib_location(gl, &program, "a_position")?;
    let normal_attrib = attrib_location(gl, &program, "a_normal")?;
    let color_attrib = attrib_location(gl, &program, "a_color")?;
    Ok(WebGlProgramState {
        view_projection_uniform: uniform_location(gl, &program, "u_view_projection")?,
        model_uniform: uniform_location(gl, &program, "u_model")?,
        material_uniform: uniform_location(gl, &program, "u_material")?,
        light_direction_uniform: uniform_location(gl, &program, "u_light_direction")?,
        unlit_uniform: uniform_location(gl, &program, "u_unlit")?,
        bloom_uniform: uniform_location(gl, &program, "u_bloom")?,
        ssao_uniform: uniform_location(gl, &program, "u_ssao")?,
        program,
        position_attrib,
        normal_attrib,
        color_attrib,
    })
}

fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("failed to create WebGL shader"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("WebGL shader compilation failed")),
        ))
    }
}

fn link_program(
    gl: &WebGlRenderingContext,
    vertex: &WebGlShader,
    fragment: &WebGlShader,
) -> Result<WebGlProgram, JsValue> {
    let program = gl
        .create_program()
        .ok_or_else(|| JsValue::from_str("failed to create WebGL program"))?;
    gl.attach_shader(&program, vertex);
    gl.attach_shader(&program, fragment);
    gl.link_program(&program);
    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(JsValue::from_str(
            &gl.get_program_info_log(&program)
                .unwrap_or_else(|| String::from("WebGL program link failed")),
        ))
    }
}

fn attrib_location(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    name: &str,
) -> Result<u32, JsValue> {
    let location = gl.get_attrib_location(program, name);
    if location < 0 {
        Err(JsValue::from_str(&format!(
            "WebGL attribute `{name}` was not found"
        )))
    } else {
        Ok(location as u32)
    }
}

fn uniform_location(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    name: &str,
) -> Result<WebGlUniformLocation, JsValue> {
    gl.get_uniform_location(program, name)
        .ok_or_else(|| JsValue::from_str(&format!("WebGL uniform `{name}` was not found")))
}

const WEBGL_VERTEX_SHADER: &str = r#"
attribute vec3 a_position;
attribute vec3 a_normal;
attribute vec4 a_color;

uniform mat4 u_view_projection;
uniform mat4 u_model;

varying vec3 v_normal;
varying vec4 v_color;

void main() {
    vec4 world = u_model * vec4(a_position, 1.0);
    v_normal = normalize((u_model * vec4(a_normal, 0.0)).xyz);
    v_color = a_color;
    gl_Position = u_view_projection * world;
}
"#;

const WEBGL_FRAGMENT_SHADER: &str = r#"
precision mediump float;

uniform vec4 u_material;
uniform vec3 u_light_direction;
uniform float u_unlit;
uniform float u_bloom;
uniform float u_ssao;

varying vec3 v_normal;
varying vec4 v_color;

void main() {
    vec3 normal = normalize(v_normal);
    float ndl = max(dot(normal, normalize(-u_light_direction)), 0.0);
    float light = mix(0.42 + ndl * 0.72, 1.0, clamp(u_unlit, 0.0, 1.0));
    light += u_bloom * 0.14;
    light -= u_ssao * 0.08;
    vec4 color = v_color * u_material;
    gl_FragColor = vec4(color.rgb * clamp(light, 0.08, 1.35), color.a);
}
"#;

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
