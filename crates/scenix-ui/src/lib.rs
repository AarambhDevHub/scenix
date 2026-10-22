#![cfg_attr(not(feature = "std"), no_std)]

//! Renderer-agnostic UI tooling for scenix.
//!
//! This crate is the CPU source of truth for editor-facing and debug-facing
//! interfaces: anchor/edge layout math, docked panel stacks, rolling renderer
//! statistics with budget policies, and the reusable tool widgets that editor
//! panels are built from. It draws nothing itself — hosts (winit desktops,
//! egui overlays, HTML/CSS on the web, WASM viewers) consume the emitted
//! rectangles and text commands, so every runtime renders the same numbers
//! from the same deterministic layout solve.
//!
//! Layout decisions are pure functions over `scenix-math` geometry, the ring
//! buffers are fixed capacity, and the widget state machines allocate only
//! for text buffers and overlay commands, keeping the crate usable on
//! embedded and browser targets.
//!
//! - `layout`: anchors, edges, safe areas, and density scaling.
//! - `panel`: docked side/top/bottom/floating panels and the space solver.
//! - `stats`: rolling frame/draw-call/triangle windows with budget warnings.
//! - `widget`: toggles, sliders, buttons, text buffers, and focus chains.
//! - `overlay`: bounded per-frame draw-command composition.

extern crate alloc;

mod layout;
mod overlay;
mod panel;
mod stats;
mod widget;

/// Crate version reported for UI tooling and wasm inspector overlays.
pub const UI_VERSION: &str = "1.10.0";

pub use layout::{Anchor, Edges, Rect, UiScale, safe_rect};
pub use overlay::{
    MAX_OVERLAY_RECTS, MAX_OVERLAY_TEXTS, OverlayFrame, PANEL_BACKDROP_ALPHA, STATS_BACKDROP_ALPHA,
    UiRect, UiText, compose_panel_overlay,
};
pub use panel::{
    DOCK_MAX_FRACTION, DockKind, DockedLayout, PANEL_MAX_THICKNESS, PANEL_MIN_THICKNESS,
    PanelEvent, PanelId, PanelSpec, PanelStack, panel_placement,
};
pub use stats::{BudgetPolicy, BudgetWarning, FrameSample, RendererStats, STATS_HISTORY_FRAMES};
pub use widget::{
    Button, ColorSwatch, FOCUS_CHAIN_MAX, FocusChain, SLIDER_DEFAULT_STEP, Slider, SmallTextBuffer,
    TEXT_BUFFER_CAP, Toggle, WidgetEvent, WidgetId, widget_hit,
};
