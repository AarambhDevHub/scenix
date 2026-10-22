# `scenix-ui`

## Role

Renderer-agnostic model layer for UI tooling (v1.10.0): anchors, edges, safe
areas, and density scaling; docked panel stacks solved through the space
allocator; rolling renderer-stat windows with budget policies; tool widgets
(toggles, sliders, buttons, text buffers, focus chains) with hit testing; and
a bounded per-frame overlay composition that stays inside a fixed draw
budget. It renders nothing itself — hosts draw the solved rects and text
commands with whatever surface they have (egui, HTML canvas, native).

## Dependency Weight

`no_std`-capable on `scenix-math` only; optional `serde` for all types.
Fixed-size history (120 samples) and overlay caps (256 rects / 64 texts) mean
no hidden allocation growth.

## Install

```toml
[dependencies]
scenix-ui = "1"
```

Or through the facade:

```toml
[dependencies]
scenix = { version = "1", features = ["ui"] }
```

## Key Public API

`Rect`, `Edges`, `Anchor`, `UiScale`, `safe_rect`, `PanelId`, `PanelSpec`,
`PanelStack`, `PanelEvent`, `DockKind`, `DockedLayout`, `panel_placement`,
`DOCK_MAX_FRACTION`, `PANEL_MIN_THICKNESS`, `PANEL_MAX_THICKNESS`,
`FrameSample`, `BudgetPolicy`, `BudgetWarning`, `RendererStats`,
`STATS_HISTORY_FRAMES`, `Button`, `Toggle`, `Slider`, `ColorSwatch`,
`SmallTextBuffer`, `FocusChain`, `WidgetId`, `WidgetEvent`, `widget_hit`,
`FOCUS_CHAIN_MAX`, `TEXT_BUFFER_CAP`, `SLIDER_DEFAULT_STEP`, `UiRect`,
`UiText`, `OverlayFrame`, `compose_panel_overlay`, `MAX_OVERLAY_RECTS`,
`MAX_OVERLAY_TEXTS`, `PANEL_BACKDROP_ALPHA`, `STATS_BACKDROP_ALPHA`,
`UI_VERSION`

## Common Use

```rust
use scenix_ui::{
    BudgetPolicy, compose_panel_overlay, panel_placement, PanelId, PanelSpec,
    PanelStack, RendererStats, DockKind,
};

let mut panels = PanelStack::new();
panels.upsert(PanelSpec::docked(
    PanelId(0),
    "Scene Tree".into(),
    DockKind::Left,
    300.0,
));

let layout = panel_placement(&panels, (1280.0, 800.0).into());
let mut stats = RendererStats::new();
stats.record_frame(FrameSample::sanitized(16.7, 820, 460_000, 48 * 1024));
let frame = compose_panel_overlay((1280.0, 800.0).into(), &panels, Some(&stats));
for rect in frame.rects() { /* host paints rect.fill with rect.rect */ }
```

## Notes

- Docking is first-fit in tab order with a hard
  [`PANEL_MIN_THICKNESS`] floor: panels never silently shrink away.
- `RendererStats` serializes (under `serde`) oldest-first so hosts replay
  identical windows.
- `compose_panel_overlay` is deterministic: the same panel stack, stats, and
  viewport always produce the same draw list, which makes overlay tests
  exact instead of approximate.
