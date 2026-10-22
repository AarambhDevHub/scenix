//! Renderer-agnostic UI layer over the v1.10 tooling: docked panels solved
//! through the space solver, rolling renderer stats with budget policies,
//! tool widgets, and the bounded per-frame overlay composition hosts paint.
//! Nothing opens a window — the demo prints the solved frames.
//!
//! Run with: `cargo run -p scenix --features ui`

use scenix::{
    BudgetPolicy, DockKind, FocusChain, FrameSample, MAX_OVERLAY_RECTS, OverlayFrame, PanelId,
    PanelSpec, PanelStack, RendererStats, Slider, SmallTextBuffer, Toggle, UiRect, Vec2, WidgetId,
    compose_panel_overlay, panel_placement, widget_hit,
};

fn build_stats() -> RendererStats {
    let mut stats = RendererStats::new();
    // A realistic 60-ish Hz stretch that occasionally trips the policy.
    let budgets = [16.8_f32, 16.6, 16.9, 17.1, 31.9, 16.7, 16.7, 16.8];
    for frame_ms in budgets {
        stats.record_frame(FrameSample::sanitized(frame_ms, 820, 460_000, 48 * 1024));
    }
    stats
}

fn main() {
    let viewport = Vec2::new(1280.0, 800.0);

    // Dock the tools; the solver floors panels at their minimum thickness.
    let mut panels = PanelStack::new();
    panels.upsert(PanelSpec::docked(
        PanelId(0),
        String::from("Scene Tree"),
        DockKind::Left,
        300.0,
    ));
    panels.upsert(PanelSpec::docked(
        PanelId(1),
        String::from("Inspector"),
        DockKind::Right,
        340.0,
    ));
    let layout = panel_placement(&panels, viewport);
    println!("layout: left {:?} | right {:?}", layout.left, layout.right);

    // Stats roll over the frame history and warn against budgets.
    let stats = build_stats();
    let mut warnings = Vec::new();
    stats.budget_warnings(&BudgetPolicy::SIXTY_HZ, &mut warnings);
    println!(
        "stats: {} samples | {} budget warnings",
        stats.len(),
        warnings.len()
    );

    // Compose one bounded overlay frame for painting.
    let frame: OverlayFrame = compose_panel_overlay(viewport, &panels, Some(&stats));
    println!(
        "overlay: {} rects (cap {}) / {} texts",
        frame.rects().len(),
        MAX_OVERLAY_RECTS,
        frame.texts().len(),
    );

    // The first drawn rect shows what hosts receive: extent + fill.
    let first: &UiRect = &frame.rects()[0];
    println!("first rect: {:?} fill {:?}", RectProbe(first), first.fill);

    // Widgets behave the same headless: edges, focus, and text entry.
    let mut toggle = Toggle::ON;
    let event = toggle.flip();
    println!("toggle: {:?} on={}", event, toggle.on);

    let slider = Slider::new(0.0, 1.0);
    println!("slider snap(0.51): {}", slider.snap(0.51));

    let mut buffer = SmallTextBuffer::new();
    buffer.push_str("frame");
    buffer.push_str(" delta");
    println!("text buffer: {:?}", buffer.text());

    let mut chain = FocusChain::new();
    chain.register(0, WidgetId(1));
    chain.register(1, WidgetId(2));
    println!("focus: {:?} -> {:?}", chain.focused(), chain.focus_next());

    // Hit testing bounds widgets before composition does.
    println!(
        "hit at (16, 12): {}",
        widget_hit(Vec2::new(16.0, 12.0), Vec2::ZERO, Vec2::new(64.0, 32.0))
    );
}

/// Small probe type that keeps the debug print tidy without extra traits.
struct RectProbe<'a>(&'a UiRect);

impl std::fmt::Debug for RectProbe<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiRect")
            .field("min", &self.0.rect.min)
            .field("size", &self.0.rect.size)
            .finish()
    }
}
