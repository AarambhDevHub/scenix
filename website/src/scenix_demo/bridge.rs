use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent, PointerEvent, WheelEvent, window,
};

thread_local! {
    static RENDERER: RefCell<Option<scenix::WebRenderer>> = const { RefCell::new(None) };
    static STATUS: RefCell<String> = RefCell::new(String::from("Starting WebGPU demo"));
    static ANIMATION: RefCell<Option<Closure<dyn FnMut(f64)>>> = const { RefCell::new(None) };
    static FALLBACK: RefCell<FallbackState> = RefCell::new(FallbackState::new());
}

#[derive(Clone, Debug)]
pub struct DemoSnapshot {
    pub status: String,
    pub fps: f32,
    pub selected_name: String,
    pub selected_id: u64,
    pub distance: f32,
    pub material: String,
    pub flags: String,
}

#[derive(Clone, Debug)]
struct FallbackState {
    playing: bool,
    helpers: bool,
    wireframe: bool,
    bloom: bool,
    ssao: bool,
    angle: f64,
    last_timestamp: Option<f64>,
    fps: f32,
    selected: FallbackSelection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FallbackSelection {
    Rover,
    LightRig,
    FloorGrid,
}

impl FallbackState {
    const fn new() -> Self {
        Self {
            playing: true,
            helpers: true,
            wireframe: false,
            bloom: false,
            ssao: false,
            angle: 0.0,
            last_timestamp: None,
            fps: 0.0,
            selected: FallbackSelection::Rover,
        }
    }

    fn tick(&mut self, timestamp: f64) {
        let dt = self
            .last_timestamp
            .map_or(1.0 / 60.0, |last| ((timestamp - last) * 0.001).clamp(0.0, 0.08));
        self.last_timestamp = Some(timestamp);
        self.fps = if dt > 0.0 { (1.0 / dt) as f32 } else { 0.0 };
        if self.playing {
            self.angle = (self.angle + dt * 0.78) % core::f64::consts::TAU;
        }
    }

    fn selected_name(&self) -> &'static str {
        match self.selected {
            FallbackSelection::Rover => "Scenix Rover",
            FallbackSelection::LightRig => "Point Light Rig",
            FallbackSelection::FloorGrid => "Helper Floor Grid",
        }
    }

    fn selected_id(&self) -> u64 {
        match self.selected {
            FallbackSelection::Rover => 101,
            FallbackSelection::LightRig => 202,
            FallbackSelection::FloorGrid => 303,
        }
    }

    fn selected_distance(&self) -> f32 {
        match self.selected {
            FallbackSelection::Rover => 3.2,
            FallbackSelection::LightRig => 4.8,
            FallbackSelection::FloorGrid => 5.6,
        }
    }

    fn material(&self) -> &'static str {
        match self.selected {
            FallbackSelection::Rover => {
                if self.wireframe {
                    "Wireframe Debug"
                } else {
                    "PBR Teal Alloy"
                }
            }
            FallbackSelection::LightRig => "Warm Point Light",
            FallbackSelection::FloorGrid => "Helper Lines",
        }
    }

    fn flags(&self) -> String {
        format!(
            "fallback-canvas, helpers={}, wireframe={}, bloom={}, ssao={}, animated={}",
            self.helpers, self.wireframe, self.bloom, self.ssao, self.playing
        )
    }
}

pub fn start(canvas_id: &'static str) {
    set_status("Starting WebGPU demo");
    spawn_local(async move {
        let Some(document) = window().and_then(|window| window.document()) else {
            set_status("Browser document is unavailable");
            return;
        };
        let Some(element) = document.get_element_by_id(canvas_id) else {
            set_status("Demo canvas was not found");
            return;
        };
        let Ok(canvas) = element.dyn_into::<HtmlCanvasElement>() else {
            set_status("Demo element is not a canvas");
            return;
        };

        if should_use_canvas_fallback() {
            start_canvas_fallback(canvas);
            return;
        }

        attach_renderer_events(&canvas);
        match scenix::WebRenderer::new(canvas).await {
            Ok(renderer) => {
                RENDERER.with(|slot| *slot.borrow_mut() = Some(renderer));
                set_status("WebGPU demo running");
                start_animation_loop();
            }
            Err(error) => {
                set_status(&format!("WebGPU/WASM init failed: {}", js_value_text(&error)));
                if let Some(document) = window().and_then(|window| window.document())
                    && let Some(element) = document.get_element_by_id(canvas_id)
                    && let Ok(canvas) = element.dyn_into::<HtmlCanvasElement>()
                {
                    start_canvas_fallback(canvas);
                }
            }
        }
    });
}

pub fn start_snapshot_loop(mut update: impl FnMut() + 'static) {
    let closure = Closure::wrap(Box::new(move || update()) as Box<dyn FnMut()>);
    if let Some(window) = window() {
        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            160,
        );
    }
    closure.forget();
}

pub fn snapshot() -> DemoSnapshot {
    RENDERER.with(|slot| {
        if let Some(renderer) = slot.borrow().as_ref() {
            DemoSnapshot {
                status: STATUS.with(|status| status.borrow().clone()),
                fps: renderer.fps(),
                selected_name: renderer.selected_node_name(),
                selected_id: renderer.selected_node_id(),
                distance: renderer.raycast_distance(),
                material: renderer.active_material(),
                flags: renderer.active_feature_flags(),
            }
        } else {
            FALLBACK.with(|fallback| {
                let fallback = fallback.borrow();
                DemoSnapshot {
                    status: STATUS.with(|status| status.borrow().clone()),
                    fps: fallback.fps,
                    selected_name: String::from(fallback.selected_name()),
                    selected_id: fallback.selected_id(),
                    distance: fallback.selected_distance(),
                    material: String::from(fallback.material()),
                    flags: fallback.flags(),
                }
            })
        }
    })
}

pub fn set_playing(playing: bool) {
    FALLBACK.with(|fallback| fallback.borrow_mut().playing = playing);
    with_renderer(|renderer| renderer.set_paused(!playing));
}

pub fn set_helpers_visible(visible: bool) {
    FALLBACK.with(|fallback| fallback.borrow_mut().helpers = visible);
    with_renderer(|renderer| renderer.set_helpers_visible(visible));
}

pub fn set_wireframe_enabled(enabled: bool) {
    FALLBACK.with(|fallback| fallback.borrow_mut().wireframe = enabled);
    with_renderer(|renderer| renderer.set_wireframe_enabled(enabled));
}

pub fn set_bloom_enabled(enabled: bool) {
    FALLBACK.with(|fallback| fallback.borrow_mut().bloom = enabled);
    with_renderer(|renderer| renderer.set_bloom_enabled(enabled));
}

pub fn set_ssao_enabled(enabled: bool) {
    FALLBACK.with(|fallback| fallback.borrow_mut().ssao = enabled);
    with_renderer(|renderer| renderer.set_ssao_enabled(enabled));
}

pub fn reset_camera() {
    FALLBACK.with(|fallback| {
        let mut fallback = fallback.borrow_mut();
        fallback.angle = 0.0;
        fallback.selected = FallbackSelection::Rover;
    });
    with_renderer(scenix::WebRenderer::reset_camera);
}

fn start_animation_loop() {
    ANIMATION.with(|animation| {
        *animation.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
            with_renderer(|renderer| {
                if let Err(error) = renderer.tick(timestamp) {
                    set_status(&format!("Render failed: {}", js_value_text(&error)));
                }
            });
            request_next_frame();
        }) as Box<dyn FnMut(f64)>));
    });
    request_next_frame();
}

fn start_canvas_fallback(canvas: HtmlCanvasElement) {
    set_status("Canvas demo running");
    FALLBACK.with(|fallback| *fallback.borrow_mut() = FallbackState::new());
    attach_fallback_events(&canvas);
    ANIMATION.with(|animation| {
        *animation.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
            FALLBACK.with(|fallback| fallback.borrow_mut().tick(timestamp));
            draw_fallback_scene(&canvas);
            request_next_frame();
        }) as Box<dyn FnMut(f64)>));
    });
    request_next_frame();
}

fn request_next_frame() {
    if let Some(window) = window() {
        ANIMATION.with(|animation| {
            if let Some(callback) = animation.borrow().as_ref() {
                let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
            }
        });
    }
}

fn should_use_canvas_fallback() -> bool {
    let Some(window) = window() else {
        return true;
    };
    let user_agent = window.navigator().user_agent().unwrap_or_default().to_lowercase();
    if user_agent.contains("firefox") {
        return true;
    }
    let navigator = JsValue::from(window.navigator());
    !js_sys::Reflect::has(&navigator, &JsValue::from_str("gpu")).unwrap_or(false)
}

fn draw_fallback_scene(canvas: &HtmlCanvasElement) {
    let Some(context) = fallback_context(canvas) else {
        return;
    };
    let ratio = window().map_or(1.0, |window| window.device_pixel_ratio().max(1.0));
    let width = canvas.client_width().max(1) as f64;
    let height = canvas.client_height().max(1) as f64;
    let pixel_width = (width * ratio).round() as u32;
    let pixel_height = (height * ratio).round() as u32;
    if canvas.width() != pixel_width || canvas.height() != pixel_height {
        canvas.set_width(pixel_width);
        canvas.set_height(pixel_height);
    }
    let _ = context.set_transform(ratio, 0.0, 0.0, ratio, 0.0, 0.0);

    FALLBACK.with(|fallback| {
        let fallback = fallback.borrow();
        context.set_fill_style_str("#07101b");
        context.fill_rect(0.0, 0.0, width, height);

        draw_canvas_environment(&context, width, height, &fallback);
        draw_rover(&context, width, height, &fallback);
        draw_scene_labels(&context, &fallback);
    });
}

fn fallback_context(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|context| context.dyn_into::<CanvasRenderingContext2d>().ok())
}

fn draw_canvas_environment(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    state: &FallbackState,
) {
    if state.bloom {
        context.set_global_alpha(0.18);
        context.set_fill_style_str("#38bdf8");
        context.begin_path();
        let _ = context.arc(width * 0.68, height * 0.22, height * 0.28, 0.0, core::f64::consts::TAU);
        context.fill();
        context.set_global_alpha(1.0);
    }

    context.set_fill_style_str("#0d1728");
    context.fill_rect(0.0, height * 0.62, width, height * 0.38);
    if state.helpers {
        draw_perspective_grid(context, width, height);
        draw_axes(context, height);
    }

    if state.ssao {
        context.set_fill_style_str("rgba(0, 0, 0, 0.34)");
        context.begin_path();
        let _ = context.ellipse(
            width * 0.5,
            height * 0.73,
            width * 0.22,
            height * 0.055,
            0.0,
            0.0,
            core::f64::consts::TAU,
        );
        context.fill();
    }

    context.set_fill_style_str("#facc15");
    context.begin_path();
    let _ = context.arc(width * 0.78, height * 0.22, 22.0, 0.0, core::f64::consts::TAU);
    context.fill();
}

fn draw_perspective_grid(context: &CanvasRenderingContext2d, width: f64, height: f64) {
    let horizon = height * 0.62;
    let bottom = height - 30.0;
    context.set_line_width(1.0);
    context.set_stroke_style_str("rgba(125, 211, 252, 0.16)");
    for i in 0..22 {
        let x = width * (i as f64 / 21.0);
        context.begin_path();
        context.move_to(width * 0.5, horizon);
        context.line_to(x, bottom);
        context.stroke();
    }
    for i in 0..11 {
        let t = i as f64 / 10.0;
        let y = horizon + (bottom - horizon) * t * t;
        context.begin_path();
        context.move_to(24.0, y);
        context.line_to(width - 24.0, y);
        context.stroke();
    }
}

fn draw_axes(context: &CanvasRenderingContext2d, height: f64) {
    let x = 58.0;
    let y = height - 62.0;
    draw_axis(context, x, y, x + 52.0, y, "#ef4444", "X");
    draw_axis(context, x, y, x, y - 52.0, "#22c55e", "Y");
    draw_axis(context, x, y, x + 38.0, y - 28.0, "#3b82f6", "Z");
}

fn draw_axis(context: &CanvasRenderingContext2d, x0: f64, y0: f64, x1: f64, y1: f64, color: &str, label: &str) {
    context.set_stroke_style_str(color);
    context.set_fill_style_str(color);
    context.set_line_width(2.0);
    context.begin_path();
    context.move_to(x0, y0);
    context.line_to(x1, y1);
    context.stroke();
    let _ = context.fill_text(label, x1 + 5.0, y1 + 4.0);
}

fn draw_rover(context: &CanvasRenderingContext2d, width: f64, height: f64, state: &FallbackState) {
    let cx = width * 0.5 + state.angle.sin() * 18.0;
    let cy = height * 0.57 + state.angle.cos() * 7.0;
    let tilt = state.angle.sin() * 0.08;

    context.save();
    let _ = context.translate(cx, cy);
    let _ = context.rotate(tilt);

    draw_wheel(context, -118.0, 74.0, 34.0, state);
    draw_wheel(context, 118.0, 74.0, 34.0, state);
    draw_wheel(context, -62.0, 82.0, 26.0, state);
    draw_wheel(context, 62.0, 82.0, 26.0, state);

    context.set_fill_style_str("#0f766e");
    context.begin_path();
    context.move_to(-146.0, 24.0);
    context.line_to(-96.0, -52.0);
    context.line_to(76.0, -62.0);
    context.line_to(148.0, 18.0);
    context.line_to(102.0, 74.0);
    context.line_to(-112.0, 72.0);
    context.close_path();
    context.fill();

    context.set_fill_style_str("#14b8a6");
    context.begin_path();
    context.move_to(-92.0, -48.0);
    context.line_to(34.0, -54.0);
    context.line_to(84.0, 4.0);
    context.line_to(-34.0, 18.0);
    context.close_path();
    context.fill();

    context.set_fill_style_str("#bae6fd");
    context.begin_path();
    context.move_to(38.0, -48.0);
    context.line_to(72.0, -8.0);
    context.line_to(22.0, -2.0);
    context.line_to(-8.0, -42.0);
    context.close_path();
    context.fill();

    draw_rover_arm(context, state);
    draw_material_panels(context, state);

    if state.wireframe {
        context.set_stroke_style_str("#e2e8f0");
        context.set_line_width(1.4);
        for (x0, y0, x1, y1) in [
            (-146.0, 24.0, -96.0, -52.0),
            (-96.0, -52.0, 76.0, -62.0),
            (76.0, -62.0, 148.0, 18.0),
            (148.0, 18.0, 102.0, 74.0),
            (102.0, 74.0, -112.0, 72.0),
            (-112.0, 72.0, -146.0, 24.0),
            (-70.0, -30.0, 40.0, 52.0),
            (34.0, -54.0, -36.0, 66.0),
        ] {
            context.begin_path();
            context.move_to(x0, y0);
            context.line_to(x1, y1);
            context.stroke();
        }
    }

    if state.helpers {
        context.set_stroke_style_str("#facc15");
        context.set_line_width(1.5);
        context.stroke_rect(-158.0, -74.0, 316.0, 176.0);
    }

    context.restore();
}

fn draw_wheel(context: &CanvasRenderingContext2d, x: f64, y: f64, radius: f64, state: &FallbackState) {
    context.set_fill_style_str("#020617");
    context.begin_path();
    let _ = context.arc(x, y, radius, 0.0, core::f64::consts::TAU);
    context.fill();
    context.set_stroke_style_str("#64748b");
    context.set_line_width(4.0);
    context.stroke();

    context.set_stroke_style_str(if state.bloom { "#67e8f9" } else { "#334155" });
    context.set_line_width(2.0);
    for i in 0..6 {
        let a = state.angle * 4.0 + i as f64 * core::f64::consts::TAU / 6.0;
        context.begin_path();
        context.move_to(x, y);
        context.line_to(x + a.cos() * radius * 0.75, y + a.sin() * radius * 0.75);
        context.stroke();
    }
}

fn draw_rover_arm(context: &CanvasRenderingContext2d, state: &FallbackState) {
    let arm = state.angle.sin() * 16.0;
    context.set_stroke_style_str("#cbd5e1");
    context.set_line_width(8.0);
    context.begin_path();
    context.move_to(90.0, -36.0);
    context.line_to(142.0, -82.0 + arm);
    context.line_to(188.0, -62.0 + arm);
    context.stroke();

    context.set_fill_style_str(if state.bloom { "#67e8f9" } else { "#38bdf8" });
    context.begin_path();
    let _ = context.arc(196.0, -58.0 + arm, 14.0, 0.0, core::f64::consts::TAU);
    context.fill();
}

fn draw_material_panels(context: &CanvasRenderingContext2d, state: &FallbackState) {
    context.set_fill_style_str(if state.bloom { "#67e8f9" } else { "#0f172a" });
    context.fill_rect(-72.0, -18.0, 42.0, 18.0);
    context.set_fill_style_str("#facc15");
    context.fill_rect(-22.0, -20.0, 42.0, 18.0);
    context.set_fill_style_str("#c084fc");
    context.fill_rect(28.0, -22.0, 42.0, 18.0);
}

fn draw_scene_labels(context: &CanvasRenderingContext2d, state: &FallbackState) {
    context.set_font("700 16px Inter, sans-serif");
    context.set_fill_style_str("#e2e8f0");
    let _ = context.fill_text("Scenix Engine Lab", 24.0, 34.0);
    context.set_font("500 12px Inter, sans-serif");
    context.set_fill_style_str("#7dd3fc");
    let _ = context.fill_text("Interactive Canvas preview: controls and picking are active", 24.0, 55.0);

    context.set_font("600 12px Inter, sans-serif");
    context.set_fill_style_str("#cbd5e1");
    let _ = context.fill_text(state.selected_name(), 24.0, 78.0);
}

fn pick_fallback(width: f64, height: f64, x: f64, y: f64) {
    FALLBACK.with(|fallback| {
        let mut fallback = fallback.borrow_mut();
        fallback.selected = if y > height * 0.62 {
            FallbackSelection::FloorGrid
        } else if x > width * 0.68 && y < height * 0.36 {
            FallbackSelection::LightRig
        } else {
            FallbackSelection::Rover
        };
    });
}

#[allow(dead_code)]
fn start_single_frame_loop() {
    let callback = Closure::wrap(Box::new(move |timestamp: f64| {
        with_renderer(|renderer| {
            if let Err(error) = renderer.tick(timestamp) {
                set_status(&format!("Render failed: {}", js_value_text(&error)));
            }
        });
    }) as Box<dyn FnMut(f64)>);
    if let Some(window) = window() {
        let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    }
    callback.forget();
}

fn attach_renderer_events(canvas: &HtmlCanvasElement) {
    let move_closure = Closure::wrap(Box::new(move |event: PointerEvent| {
        with_renderer(|renderer| {
            renderer.on_pointer_move(event.offset_x() as f32, event.offset_y() as f32)
        });
    }) as Box<dyn FnMut(_)>);
    let _ =
        canvas.add_event_listener_with_callback("pointermove", move_closure.as_ref().unchecked_ref());
    move_closure.forget();

    let down_closure = Closure::wrap(Box::new(move |event: PointerEvent| {
        with_renderer(|renderer| {
            renderer.on_pointer_down(event.button(), event.offset_x() as f32, event.offset_y() as f32)
        });
    }) as Box<dyn FnMut(_)>);
    let _ =
        canvas.add_event_listener_with_callback("pointerdown", down_closure.as_ref().unchecked_ref());
    down_closure.forget();

    let up_closure = Closure::wrap(Box::new(move |event: PointerEvent| {
        with_renderer(|renderer| {
            renderer.on_pointer_up(event.button(), event.offset_x() as f32, event.offset_y() as f32)
        });
    }) as Box<dyn FnMut(_)>);
    let _ = canvas.add_event_listener_with_callback("pointerup", up_closure.as_ref().unchecked_ref());
    up_closure.forget();

    let wheel_closure = Closure::wrap(Box::new(move |event: WheelEvent| {
        event.prevent_default();
        with_renderer(|renderer| renderer.on_wheel(event.delta_y() as f32));
    }) as Box<dyn FnMut(_)>);
    let _ = canvas.add_event_listener_with_callback("wheel", wheel_closure.as_ref().unchecked_ref());
    wheel_closure.forget();

    attach_keyboard_events();
}

fn attach_fallback_events(canvas: &HtmlCanvasElement) {
    let up_canvas = canvas.clone();
    let up_closure = Closure::wrap(Box::new(move |event: PointerEvent| {
        pick_fallback(
            up_canvas.client_width().max(1) as f64,
            up_canvas.client_height().max(1) as f64,
            event.offset_x() as f64,
            event.offset_y() as f64,
        );
    }) as Box<dyn FnMut(_)>);
    let _ = canvas.add_event_listener_with_callback("pointerup", up_closure.as_ref().unchecked_ref());
    up_closure.forget();

    let wheel_closure = Closure::wrap(Box::new(move |event: WheelEvent| {
        event.prevent_default();
        FALLBACK.with(|fallback| {
            let mut fallback = fallback.borrow_mut();
            fallback.angle += event.delta_y().signum() * 0.12;
        });
    }) as Box<dyn FnMut(_)>);
    let _ = canvas.add_event_listener_with_callback("wheel", wheel_closure.as_ref().unchecked_ref());
    wheel_closure.forget();

    attach_keyboard_events();
}

fn attach_keyboard_events() {
    if let Some(window) = window() {
        let key_down = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            with_renderer(|renderer| renderer.on_key_down(&event.code()));
            if event.code() == "Space" {
                FALLBACK.with(|fallback| {
                    let mut fallback = fallback.borrow_mut();
                    fallback.playing = !fallback.playing;
                });
            }
        }) as Box<dyn FnMut(_)>);
        let _ = window.add_event_listener_with_callback("keydown", key_down.as_ref().unchecked_ref());
        key_down.forget();

        let key_up = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            with_renderer(|renderer| renderer.on_key_up(&event.code()));
        }) as Box<dyn FnMut(_)>);
        let _ = window.add_event_listener_with_callback("keyup", key_up.as_ref().unchecked_ref());
        key_up.forget();
    }
}

fn with_renderer(mut f: impl FnMut(&mut scenix::WebRenderer)) {
    RENDERER.with(|slot| {
        if let Some(renderer) = slot.borrow_mut().as_mut() {
            f(renderer);
        }
    });
}

fn set_status(status: &str) {
    STATUS.with(|slot| *slot.borrow_mut() = String::from(status));
}

fn js_value_text(value: &wasm_bindgen::JsValue) -> String {
    value
        .as_string()
        .unwrap_or_else(|| String::from("unknown browser error"))
}
