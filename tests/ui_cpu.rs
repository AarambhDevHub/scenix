//! Distributed `scenix-ui` v1.10.0 CPU validation suite.
//!
//! Run from the repository root:
//!
//! ```sh
//! cargo test -p scenix-ui --test ui_cpu
//! ```

use scenix_math::Vec2;
use scenix_ui::{
    Anchor, BudgetPolicy, BudgetWarning, Button, ColorSwatch, DOCK_MAX_FRACTION, DockKind,
    DockedLayout, Edges, FOCUS_CHAIN_MAX, FocusChain, FrameSample, MAX_OVERLAY_RECTS,
    MAX_OVERLAY_TEXTS, OverlayFrame, PANEL_MAX_THICKNESS, PanelEvent, PanelId, PanelSpec,
    PanelStack, Rect, RendererStats, STATS_HISTORY_FRAMES, Slider, SmallTextBuffer,
    TEXT_BUFFER_CAP, Toggle, UI_VERSION, UiScale, WidgetEvent, WidgetId, compose_panel_overlay,
    panel_placement, safe_rect, widget_hit,
};

#[test]
fn ui_version_matches_workspace_release() {
    assert_eq!(UI_VERSION, "1.10.0");
}

#[test]
fn anchors_cover_the_nine_way_grid() {
    let viewport = Vec2::new(800.0, 600.0);
    let inset = Edges::uniform(16.0);
    let toast = Vec2::new(120.0, 40.0);

    // Corners hug their edges inside the inset region.
    let top_left = Anchor::TopLeft.place(viewport, inset, toast, Vec2::ZERO);
    assert_eq!(top_left.min, Vec2::new(16.0, 16.0));

    let bottom_right = Anchor::BottomRight.place(viewport, inset, toast, Vec2::ZERO);
    assert_eq!(bottom_right.max(), Vec2::new(784.0, 584.0));

    // Centers are exact midpoints of the inset region.
    let center = Anchor::Center.place(viewport, inset, toast, Vec2::ZERO);
    assert_eq!(center.min, Vec2::new(340.0, 280.0));

    // An oversized element still clamps inside the safe region.
    let huge = Vec2::new(2_000.0, 900.0);
    let clamped = Anchor::TopRight.place(viewport, inset, huge, Vec2::ZERO);
    assert!(clamped.min.x >= 16.0 && clamped.max().x <= 784.0);
    assert!(clamped.min.y >= 16.0 && clamped.max().y <= 584.0);
}

#[test]
fn safe_rect_and_scale_text() {
    let viewport = Vec2::new(1280.0, 720.0);
    let insets = Edges::new(0.0, 240.0, 0.0, 64.0);
    let safe = safe_rect(viewport, insets);
    assert_eq!(safe.min, Vec2::new(0.0, 0.0));
    assert_eq!(safe.size, Vec2::new(1040.0, 656.0));

    // Negative insets are clamped, never inverted.
    let flipped = safe_rect(viewport, Edges::new(-8.0, 5000.0, -1.0, 0.0));
    assert_eq!(flipped.size, Vec2::new(0.0, 720.0));

    let dpr = UiScale::from_dpr(2.0);
    assert_eq!(dpr.px(12.0), 24.0);
    assert_eq!(
        UiScale::from_dpr(0.0).factor,
        1.0,
        "zero DPR falls back to one"
    );
    assert_eq!(UiScale::from_dpr(f32::NAN).factor, 1.0);
}

#[test]
fn panel_stack_bookkeeping_is_event_driven() {
    let mut stack = PanelStack::new();
    let id = PanelId(0);
    assert_eq!(stack.len(), 0);

    let title = String::from("Timeline");
    assert_eq!(
        stack.open(id, title.clone(), DockKind::Bottom),
        PanelEvent::Opened
    );
    assert_eq!(
        stack.open(id, title, DockKind::Bottom),
        PanelEvent::NoChange
    );

    assert!(stack.set_thickness(id, 9999.0));
    assert_eq!(
        stack.spec(id).expect("registered").thickness,
        PANEL_MAX_THICKNESS,
        "thickness clamps into the global band"
    );

    assert!(!stack.toggle(id), "toggling an open panel hides it");
    assert_eq!(stack.close(id), PanelEvent::NoChange, "already hidden");
    assert!(!stack.is_visible(id));
    assert!(stack.toggle(id), "toggling a hidden panel shows it");
    assert!(stack.is_visible(id));
}

#[test]
fn docked_layout_clamps_and_leaves_a_viewport() {
    let viewport = Vec2::new(1200.0, 800.0);
    let mut stack = PanelStack::new();
    stack.upsert(PanelSpec::docked(
        PanelId(0),
        String::from("Scene Tree"),
        DockKind::Left,
        320.0,
    ));
    stack.upsert(PanelSpec::docked(
        PanelId(1),
        String::from("Inspector"),
        DockKind::Left,
        360.0,
    ));

    // Budget for the left dock is 0.4 * 1200 = 480 px. The scene tree claims
    // 320 px; the remaining 160 px is below the minimum panel thickness, so
    // the inspector is handed nothing this frame.
    let layout = panel_placement(&stack, viewport);
    assert_eq!(layout.left, 320.0);
    let remaining = layout.viewport_remaining(viewport);
    assert_eq!(remaining.size, Vec2::new(880.0, 800.0));
    assert!(remaining.contains(Vec2::new(400.0, 400.0)));

    // Widening the viewport raises the budget, so the inspector docks at the
    // remaining thickness.
    let wide = panel_placement(&stack, Vec2::new(2000.0, 800.0));
    assert_eq!(wide.left, 680.0, "budget 800 px fits both panels in full");

    // A third panel on the same dock can never push past the fraction budget.
    stack.upsert(PanelSpec::docked(
        PanelId(2),
        String::from("Assets"),
        DockKind::Left,
        300.0,
    ));
    let crowded = panel_placement(&stack, viewport);
    assert!(crowded.left <= 1200.0 * DOCK_MAX_FRACTION + 1e-4);
}

#[test]
fn floating_panels_stay_out_of_the_dock_solver() {
    let viewport = Vec2::new(800.0, 600.0);
    let mut stack = PanelStack::new();
    stack.upsert(PanelSpec::floating(
        PanelId(3),
        String::from("Palette"),
        Vec2::new(400.0, 260.0),
        220.0,
    ));

    let layout = panel_placement(&stack, viewport);
    assert_eq!(layout, DockedLayout::default(), "floating docks solve zero");
    assert!(
        layout
            .viewport_remaining(viewport)
            .contains(Vec2::new(420.0, 300.0))
    );

    // The floating spec still carries its own usable rect for hit testing.
    let spec = stack.spec(PanelId(3)).expect("registered");
    let rect = spec.panel_rect(viewport, spec.thickness);
    assert!(rect.contains(Vec2::new(410.0, 265.0)));
}

fn warnings(stats: &RendererStats, policy: &BudgetPolicy) -> Vec<BudgetWarning> {
    let mut out = Vec::new();
    stats.budget_warnings(policy, &mut out);
    out
}

#[test]
fn stats_window_rolls_and_reports_budgets() {
    let mut stats = RendererStats::new();
    for frame in 0..STATS_HISTORY_FRAMES * 2 {
        stats.record_frame(FrameSample::sanitized(
            if frame < 60 { 17.5 } else { 20.0 },
            1_100,
            3_100_000,
            2 * 1024 * 1024,
        ));
    }
    assert_eq!(stats.len(), STATS_HISTORY_FRAMES, "ring stays bounded");
    assert!(
        (stats.mean_frame_ms() - 20.0).abs() < 1e-4,
        "the 17.5 ms frames scroll out"
    );
    assert!((stats.max_frame_ms() - 20.0).abs() < 1e-4);
    assert!((stats.fps() - 50.0).abs() < 1e-3);

    let found = warnings(&stats, &BudgetPolicy::SIXTY_HZ);
    assert!(
        found.contains(&BudgetWarning::FrameTime),
        "20 ms mean breaks the 16.7 ms budget"
    );
    assert!(found.contains(&BudgetWarning::DrawCalls));
    assert!(found.contains(&BudgetWarning::Triangles));
    assert!(
        !found.contains(&BudgetWarning::ResourceUploads),
        "2 MiB is well under 64 MiB"
    );

    let fresh = RendererStats::new();
    assert_eq!(fresh.mean_frame_ms(), 0.0);
    assert_eq!(fresh.fps(), 0.0);
    assert_eq!(fresh.last(), None);
    assert!(fresh.frames().next().is_none());
    assert!(warnings(&fresh, &BudgetPolicy::SIXTY_HZ).is_empty());
}

#[test]
fn stats_summary_lines_are_human_readable() {
    let mut stats = RendererStats::new();
    stats.record_frame(FrameSample::sanitized(16.7, 640, 900_000, 96 * 1024));
    let line = stats.summary_line();
    let counts = stats.summary_counts();
    assert!(line.contains("fps 59.9"), "{line}");
    assert!(line.contains("mean 16.7 ms"), "{line}");
    assert!(counts.contains("draw 640"), "{counts}");
    assert!(counts.contains("tris 900000"), "{counts}");
    assert!(counts.contains("upload 96 KiB"), "{counts}");
}

#[test]
fn widgets_clamp_nudge_and_cycle_focus() {
    let mut slider = Slider::new(0.0, 10.0);
    slider.step = 0.5;
    assert_eq!(slider.snap(3.7), 3.5);
    assert_eq!(dragged_value(&slider.from_ratio(1.0)), Some(10.0));
    assert_eq!(
        dragged_value(&slider.nudge(3.0)),
        Some(10.0),
        "nudges clamp at max"
    );
    assert_eq!(
        dragged_value(&slider.nudge(-30.0)),
        Some(0.0),
        "nudges clamp at min"
    );
    assert_eq!(Slider::new(2.0, 2.0).snap(5.0), 2.0);

    let mut toggle = Toggle::OFF;
    assert_eq!(toggle.flip(), WidgetEvent::Toggled(true));
    assert!(toggle.on);

    let mut button = Button::default();
    assert_eq!(button.press(), Some(WidgetEvent::Pressed));
    assert_eq!(button.press(), None, "held press is not a new edge");
    button.release();
    assert_eq!(button.press(), Some(WidgetEvent::Pressed));

    let mut buffer = SmallTextBuffer::new();
    buffer.push_str("scene");
    buffer.pop();
    assert_eq!(buffer.text(), "scen");
    let long = "x".repeat(64);
    buffer.push_str(&long);
    assert_eq!(
        buffer.text().chars().count(),
        TEXT_BUFFER_CAP,
        "text clamps at the cap"
    );

    assert_eq!(
        ColorSwatch::from_linear(0.5, 2.0, -4.0, 1.0).rgba,
        [128, 255, 0, 255]
    );
    assert_eq!(ColorSwatch::from_rgba([255, 136, 0, 51]).hex(), "#FF880033");

    let mut chain = FocusChain::new();
    assert_eq!(chain.focused(), None, "empty chain never focuses");
    chain.register(0, WidgetId(1));
    chain.register(2, WidgetId(3));
    chain.register(5, WidgetId(6));
    assert_eq!(chain.focused(), Some(WidgetId(1)));
    assert_eq!(chain.focus_next(), Some(WidgetId(3)));
    assert_eq!(chain.focus_next(), Some(WidgetId(6)));
    assert_eq!(chain.focus_next(), Some(WidgetId(1)), "focus wraps forward");
    assert_eq!(chain.focus_prev(), Some(WidgetId(6)));
    let _ = FOCUS_CHAIN_MAX;
}

#[test]
fn empty_focus_chain_is_inert() {
    let mut chain = FocusChain::new();
    assert_eq!(chain.focus_next(), None, "no widgets registered");
    chain.register(0, WidgetId(7));
    assert_eq!(chain.focus_next(), Some(WidgetId(7)));
    chain.clear();
    assert_eq!(chain.focus_next(), None);
}

#[test]
fn widget_hit_and_overlay_caps_stay_bounded() {
    let hit = widget_hit(Vec2::new(20.0, 30.0), Vec2::ZERO, Vec2::new(64.0, 48.0));
    assert!(hit);
    let edge = widget_hit(Vec2::new(64.0, 30.0), Vec2::ZERO, Vec2::new(64.0, 48.0));
    assert!(!edge, "right edge is exclusive");

    let mut frame = OverlayFrame::new();
    for index in 0..MAX_OVERLAY_RECTS + 8 {
        let rect = Rect::new(Vec2::new(index as f32, 0.0), Vec2::ONE);
        if !frame.push_rect(rect, [255, 0, 0, 255]) {
            break;
        }
    }
    assert_eq!(
        frame.rects().len(),
        MAX_OVERLAY_RECTS,
        "rects stop at the cap"
    );

    for index in 0..MAX_OVERLAY_TEXTS + 4 {
        if !frame.push_text(
            Vec2::new(index as f32, 0.0),
            std::format!("row {index}"),
            [255, 255, 255, 255],
            12.0,
        ) {
            break;
        }
    }
    assert_eq!(
        frame.texts().len(),
        MAX_OVERLAY_TEXTS,
        "texts stop at the cap"
    );

    let (rects, texts) = frame.into_parts();
    assert_eq!(rects.len(), MAX_OVERLAY_RECTS);
    assert_eq!(texts.len(), MAX_OVERLAY_TEXTS);
}

#[test]
fn compose_panel_overlay_is_deterministic() {
    let viewport = Vec2::new(1024.0, 768.0);
    let mut stack = PanelStack::new();
    stack.upsert(PanelSpec::docked(
        PanelId(0),
        String::from("Scene Tree"),
        DockKind::Left,
        260.0,
    ));

    let mut stats = RendererStats::new();
    stats.record_frame(FrameSample::sanitized(16.6, 80, 12_000, 2_048));

    let frame = compose_panel_overlay(viewport, &stack, Some(&stats));
    // One dock backdrop + one stats banner rect, then two stats text lines.
    assert_eq!(frame.rects().len(), 2);
    assert_eq!(frame.texts().len(), 2);
    assert!(
        frame.texts()[0].text.contains("fps"),
        "{}",
        frame.texts()[0].text
    );

    // Same inputs solve the same overlay; no host ever drifts.
    let again = compose_panel_overlay(viewport, &stack, Some(&stats));
    assert_eq!(frame, again);
}

#[cfg(feature = "serde")]
#[test]
fn serde_roundtrips_ui_state() {
    use serde_json as _;

    let mut stack = PanelStack::new();
    stack.upsert(PanelSpec::docked(
        PanelId(0),
        String::from("Assets"),
        DockKind::Right,
        300.0,
    ));
    let encoded = serde_json::to_string(&stack).expect("stack serializes");
    let decoded: PanelStack = serde_json::from_str(&encoded).expect("stack deserializes");
    assert_eq!(decoded.len(), 1);
    assert!(decoded.is_visible(PanelId(0)));
}

/// Pull the scalar out of a [`WidgetEvent::Dragged`] for assertions.
fn dragged_value(event: &WidgetEvent) -> Option<f32> {
    match event {
        WidgetEvent::Dragged(value) => Some(*value),
        _ => None,
    }
}
