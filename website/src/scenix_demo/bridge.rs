use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, KeyboardEvent, PointerEvent, WheelEvent, window};

thread_local! {
    static RENDERER: RefCell<Option<scenix::WebRenderer>> = const { RefCell::new(None) };
    static STATUS: RefCell<String> = RefCell::new(String::from("Starting WebGPU demo"));
    static ANIMATION: RefCell<Option<Closure<dyn FnMut(f64)>>> = const { RefCell::new(None) };
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
        attach_events(&canvas);
        match scenix::WebRenderer::new(canvas).await {
            Ok(renderer) => {
                RENDERER.with(|slot| *slot.borrow_mut() = Some(renderer));
                set_status("WebGPU demo running");
                start_animation_loop();
            }
            Err(error) => {
                set_status(&format!("WebGPU/WASM init failed: {}", js_value_text(&error)));
            }
        }
    });
}

pub fn start_snapshot_loop(mut update: impl FnMut() + 'static) {
    let closure = Closure::wrap(Box::new(move || update()) as Box<dyn FnMut()>);
    if let Some(window) = window() {
        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            250,
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
            DemoSnapshot {
                status: STATUS.with(|status| status.borrow().clone()),
                fps: 0.0,
                selected_name: String::from("None"),
                selected_id: 0,
                distance: 0.0,
                material: String::from("None"),
                flags: String::from("waiting"),
            }
        }
    })
}

pub fn set_playing(playing: bool) {
    with_renderer(|renderer| renderer.set_paused(!playing));
}

pub fn set_helpers_visible(visible: bool) {
    with_renderer(|renderer| renderer.set_helpers_visible(visible));
}

pub fn set_wireframe_enabled(enabled: bool) {
    with_renderer(|renderer| renderer.set_wireframe_enabled(enabled));
}

pub fn set_bloom_enabled(enabled: bool) {
    with_renderer(|renderer| renderer.set_bloom_enabled(enabled));
}

pub fn set_ssao_enabled(enabled: bool) {
    with_renderer(|renderer| renderer.set_ssao_enabled(enabled));
}

pub fn reset_camera() {
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

fn request_next_frame() {
    if let Some(window) = window() {
        ANIMATION.with(|animation| {
            if let Some(callback) = animation.borrow().as_ref() {
                let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
            }
        });
    }
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

fn attach_events(canvas: &HtmlCanvasElement) {
    let move_closure = Closure::wrap(Box::new(move |event: PointerEvent| {
        with_renderer(|renderer| renderer.on_pointer_move(event.offset_x() as f32, event.offset_y() as f32));
    }) as Box<dyn FnMut(_)>);
    let _ = canvas.add_event_listener_with_callback("pointermove", move_closure.as_ref().unchecked_ref());
    move_closure.forget();

    let down_closure = Closure::wrap(Box::new(move |event: PointerEvent| {
        with_renderer(|renderer| {
            renderer.on_pointer_down(event.button(), event.offset_x() as f32, event.offset_y() as f32)
        });
    }) as Box<dyn FnMut(_)>);
    let _ = canvas.add_event_listener_with_callback("pointerdown", down_closure.as_ref().unchecked_ref());
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

    if let Some(window) = window() {
        let key_down = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            with_renderer(|renderer| renderer.on_key_down(&event.code()));
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
