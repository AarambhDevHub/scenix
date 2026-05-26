use scenix_camera::PerspectiveCamera;
use scenix_core::{Color, LightId, MaterialId, MeshId, ScenixError};
use scenix_input::{KeyboardState, PointerState};
use scenix_light::DirectionalLight;
use scenix_material::PbrMaterial;
use scenix_math::{Quat, Vec3};
use scenix_mesh::box_geometry;
use scenix_renderer::{Renderer, RendererConfig, wgpu};
use scenix_scene::{SceneGraph, SceneNode};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::{clamp_canvas_size, key_code_from_dom, pointer_button_from_dom};

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
    pointer: PointerState,
    keyboard: KeyboardState,
    cube_node: scenix_core::NodeId,
    last_timestamp_ms: Option<f64>,
}

#[wasm_bindgen]
impl WebRenderer {
    /// Creates a renderer for `canvas` and registers a small generated cube scene.
    pub async fn new(canvas: HtmlCanvasElement) -> Result<WebRenderer, JsValue> {
        crate::set_panic_hook();
        let (width, height) = canvas_size(&canvas);
        canvas.set_width(width);
        canvas.set_height(height);

        let config = RendererConfig::new(width, height).vsync(true);
        let mut renderer = Renderer::new(wgpu::SurfaceTarget::Canvas(canvas), config)
            .await
            .map_err(js_error)?;
        let (scene, camera, cube_node) = generated_scene(&mut renderer, width, height)?;

        Ok(Self {
            renderer,
            scene,
            camera,
            pointer: PointerState::new(),
            keyboard: KeyboardState::new(),
            cube_node,
            last_timestamp_ms: None,
        })
    }

    /// Renders one frame. `timestamp_ms` should come from `requestAnimationFrame`.
    pub fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue> {
        let dt = self
            .last_timestamp_ms
            .map_or(0.0, |last| ((timestamp_ms - last) * 0.001).max(0.0) as f32);
        self.last_timestamp_ms = Some(timestamp_ms);

        let yaw_speed = if self.keyboard.is_pressed(scenix_input::KeyCode::ShiftLeft)
            || self.keyboard.is_pressed(scenix_input::KeyCode::ShiftRight)
        {
            1.6
        } else {
            0.8
        };
        if let Some(node) = self.scene.get_mut(self.cube_node) {
            let euler = node.transform.rotation.to_euler_xyz();
            node.transform.rotation =
                Quat::from_euler_xyz(euler.x + dt * 0.35, euler.y + dt * yaw_speed, 0.0);
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
        self.pointer.set_position(scenix_math::Vec2::new(x, y));
    }

    /// Updates pointer position and pressed state.
    pub fn on_pointer_down(&mut self, button: i16, x: f32, y: f32) {
        self.pointer.set_position(scenix_math::Vec2::new(x, y));
        if let Some(button) = pointer_button_from_dom(button) {
            self.pointer.on_button_down(button);
        }
    }

    /// Updates pointer position and pressed state.
    pub fn on_pointer_up(&mut self, button: i16, x: f32, y: f32) {
        self.pointer.set_position(scenix_math::Vec2::new(x, y));
        if let Some(button) = pointer_button_from_dom(button) {
            self.pointer.on_button_up(button);
        }
    }

    /// Dolly the generated camera in or out.
    pub fn on_wheel(&mut self, delta_y: f32) {
        let direction = (self.camera.position - self.camera.target).normalize();
        let distance = self.camera.position.distance(self.camera.target);
        let next = (distance + delta_y.signum() * 0.2).clamp(1.5, 10.0);
        self.camera.position = self.camera.target + direction * next;
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
}

fn generated_scene(
    renderer: &mut Renderer,
    width: u32,
    height: u32,
) -> Result<(SceneGraph, PerspectiveCamera, scenix_core::NodeId), JsValue> {
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);
    let light_id = LightId::new(1);

    let geometry = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);
    let material = PbrMaterial::new()
        .named("wasm cube")
        .albedo(Color::rgb(0.18, 0.54, 0.95))
        .metallic_roughness(0.0, 0.45);
    let light = DirectionalLight::new(Vec3::new(-0.4, -0.8, -0.3), Color::WHITE, 3.0);

    renderer
        .register_mesh(mesh_id, &geometry)
        .map_err(js_error)?;
    renderer
        .register_pbr_material(material_id, &material)
        .map_err(js_error)?;
    renderer
        .register_directional_light(light_id, light)
        .map_err(js_error)?;

    let mut scene = SceneGraph::new();
    let cube_node = scene.add(SceneNode::mesh("cube", mesh_id, material_id));
    scene.add(SceneNode::light("sun", light_id));
    scene.update_world_transforms();

    let aspect = width as f32 / height.max(1) as f32;
    let camera = PerspectiveCamera::new(60.0, aspect, 0.1, 100.0)
        .position(Vec3::new(2.5, 1.6, 3.0))
        .target(Vec3::ZERO)
        .up(Vec3::Y);

    Ok((scene, camera, cube_node))
}

fn js_error(error: ScenixError) -> JsValue {
    JsValue::from_str(&error.to_string())
}
