# Scenix v1.10.0 — Editor And UI Tooling

Version 1.10 turns the v1.5 interaction backbone into full editing
infrastructure: two new optional tooling crates (`scenix-editor`,
`scenix-ui`) that own deterministic CPU rules for what an edit *is* and
what an overlay frame *contains*, so host faces (egui, HTML tools, native
windows) stay thin renderers over solved output. The engine crates are
untouched: selection, editor metadata, and inspector snapshots remain in
the scene/core crates; no `wgpu` code and no window is involved.

## Data Flow

```text
 EditorSession (owns graph + panels + history + gizmo + tick)
        |
        +--> EditCommand (Rename/SetVisibility/SetLayer/SetTransform/Reparent)
        |       |  inverse computed from the pre-apply graph
        |       v
        |   EditHistory (cap 128, redo cleared on new work)
        |       +--> undo() / redo() return the applied label
        |
        +--> gizmo flow                          scenix-ui layer
        |   begin_drag(mode, axis)               ------------------
        |       |                                PanelStack (upsert/open/close)
        |       v                                       |
        |   drag_gizmo(amount) ── re-solves             v
        |       |   absolutely from start,       panel_placement (first-fit
        |       |   coalesced key + 30 ticks      tab order, min-thickness
        |       v                                 floor, skip under floor)
        |   SetTransform (one undo step                 |
        |    restoring the pre-drag pose)               v
        |                                        DockedLayout (left/top/
        +--> scene_rows (depth-first, cap 256)    right/bottom bands)
        +--> inspect_selected (inspector rows)
        +--> apply_pbr_field / assign_texture           |
            (clamped writes + MaterialEditorLog)        v
                                                 compose_panel_overlay
                                                 → OverlayFrame (bounded:
                                                   256 rects / 64 texts)
                                                 RendererStats (120 samples,
                                                   BudgetPolicy warnings)
```

The editor never renders and the UI layer never computes engine state:
`scenix-ui` is `no_std`-capable on `scenix-math`, and `scenix-editor` is
std-only tooling that wraps the graph plus the UI panel stack. Continuous
pointer-plane camera work (orbit while dragging) stays host math — the
editor solves world-axis amounts, snapping, and axis picking
deterministically.

## New Public Surfaces

- `scenix-ui` (new crate): `Rect`/`Edges`/`Anchor`/`UiScale`/`safe_rect`;
  `PanelId`/`PanelSpec::docked`/`floating`/`PanelStack` (`upsert`,
  `open`, `close`)/`PanelEvent`/`DockKind`/`DockedLayout`/
  `panel_placement` with `DOCK_MAX_FRACTION`/
  `PANEL_MIN_THICKNESS`/`PANEL_MAX_THICKNESS`;
  `FrameSample::sanitized`/`BudgetPolicy::{THIRTY_HZ, SIXTY_HZ}`/
  `BudgetWarning`/`RendererStats` (`record_frame`, `len`,
  `budget_warnings`, `STATS_HISTORY_FRAMES` = 120);
  `Button::default` (edge-only `press`) /`Toggle::{OFF, ON}`/`Slider`/
  `SmallTextBuffer` (`TEXT_BUFFER_CAP` = 64)/`FocusChain` (`register`,
  `focused`, `focus_next`, `FOCUS_CHAIN_MAX` = 8)/`ColorSwatch`
  (`from_linear`, `from_rgba`, `hex`)/`WidgetId`/`WidgetEvent`/
  `widget_hit`; `UiRect`/`UiText`/`OverlayFrame` (`push_rect`,
  `push_text`, `rects`, `texts`)/`compose_panel_overlay` with
  `MAX_OVERLAY_RECTS` = 256, `MAX_OVERLAY_TEXTS` = 64,
  `PANEL_BACKDROP_ALPHA` = 200, `STATS_BACKDROP_ALPHA` = 120.
- `scenix-editor` (new crate): `EditorSession` (`graph`, `graph_mut`,
  `panels`, `panels_mut`, `select`, `clear_selection`, `do_command`,
  `do_transform`, `undo`, `redo`, `can_undo`/`can_redo`,
  `undo_len`, `next_frame`, `begin_drag`, `drag_gizmo`, `end_drag`,
  `pick_gizmo`, `layout`, `inspect_selected`, `inspect_node`,
  `scene_snapshot`, `apply_inspector_edit`, `project_metadata`);
  `EditCommand` (`RenameNode`, `SetVisibility`, `SetLayer`,
  `SetTransform`, `ReparentNode`)/`EditHistory` (cap 128)/
  `HistoryEntry` with `undo_len`; `GizmoMode`/`GizmoAxis`/
  `GizmoSettings::{FREE, SNAPPED}`/`TransformDelta` (`apply_to`,
  labels)/`apply_axis_delta`/`snap_translation`/
  `ROTATE_SNAP_RADIANS` (15°)/`SCALE_SNAP_STEP` (0.1)/
  `TRANSLATE_SNAP_STEP` (0.5)/`axis_pick` (`GIZMO_HALF_LENGTH` = 1.0,
  `GIZMO_PICK_THRESHOLD` = 0.08, `GIZMO_COALESCE_KEY` = 1)/
  `COALESCE_WINDOW_TICKS` = 30/`ActiveGizmo`;
  `ScenePanelRow`/`scene_rows` (depth-first, `SCENE_PANEL_ROW_LIMIT`
  = 256, empty-name collapsing)/`kind_short_name`; `InspectorEdit`
  (`Rename`, `SetVisible`, `SetLayer`, `SetPosition`, `SetRotation`,
  `SetScale`) with locked-node refusal;
  `PbrField`/`PhysicalField` clamped writes (`MATERIAL_UNIT_MAX`,
  `MATERIAL_IOR_MIN..=MATERIAL_IOR_MAX`)/`TextureSlot` descriptions/
  `assign_texture` (`Option<TextureId>` sets or clears)/
  `MaterialEditorLog` (`MATERIAL_LOG_DEFAULT_CAP` = 32) under the
  `material` feature; `AssetBrowserModel`/`AssetBrowserRow`/
  `AssetStatusClass` over loader statuses (`assets` feature,
  `BROWSER_ROW_LIMIT` = 256); `TimelinePreview`/`TimelineFrame` over
  animato's own `LoopMode::advance` (`timeline` feature,
  `TIMELINE_SNAP_FPS` = 60, rate `0.05..=8`); `ProjectMetadata`/
  `SceneRoute`/`ProjectError`/`PROJECT_SCHEMA_VERSION` (schema v1,
  validation + `upgrade_to_current` + JSON round-trip under `serde`).
- Facade: `ui`/`editor` features with curated re-exports; `editor`
  implies `ui`; `loader`/`animato`/`serde` forward the cross-crate
  editor surfaces; examples `editor_shell` and `ui_overlay`; facade
  test `scenix_v110`.

## Feature Flags

| Feature | Drives | Notes |
| --- | --- | --- |
| `ui` | `scenix-ui` | `no_std`-capable panels, stats, widgets, overlays. Compiles for `wasm32-unknown-unknown`. |
| `editor` | `scenix-editor` + `ui` | Owning sessions; asset rows need `loader`, timeline needs `animato`, project JSON needs `serde`. |

## Verification

```sh
cargo test -p scenix-ui -p scenix-editor --all-features
cargo test -p scenix --test scenix_v110 --features editor,ui
cargo test -p scenix --test scenix_v110 --features editor,ui,loader,animato,serde
cargo check -p scenix-ui --target wasm32-unknown-unknown --no-default-features
cargo run -p scenix --features editor --example editor_shell
cargo run -p scenix --features ui --example ui_overlay
```

## Notes

- Undo/redo is exact because every command computes its inverse from the
  pre-apply graph; hosts doing out-of-band edits through `graph_mut` own
  their undo obligation.
- Material edits are optimistic by design: materials are host-owned
  resources, so writes apply immediately and log to `MaterialEditorLog`
  instead of entering the command history.
- One drag is one undo step: gizmo updates coalesce by key + tick window
  while keeping the first inverse, which restores the pre-drag pose.
