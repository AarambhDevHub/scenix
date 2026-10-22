# `scenix-editor`

## Role

Offline CPU editor tooling for scenix (v1.10.0): an owning
`EditorSession` over a `SceneGraph` with exact inverse-command
undo/redo, gizmo pick and drag math, scene-tree row models, the
selected-node inspector surface, material field editing with clamps, asset
browser rows over the loader, timeline previews driven by animato's own
loop solver, and a versioned project metadata exchange (v1).

## Dependency Weight

Std-only tooling crate on `scenix-core`, `scenix-math`, `scenix-scene`, and
`scenix-ui`; optional `scenix-loader` (`assets`), `scenix-material`
(`material`), `scenix-animato` (`timeline`), and `serde` (JSON project
exchange). Because it is a tooling crate it links `std` and `serde_json` —
the engine crates it drives stay `no_std`-capable.

## Install

```toml
[dependencies]
scenix-editor = "1"
```

Or through the facade (the `editor` feature also implies `ui` because
sessions own a panel stack):

```toml
[dependencies]
scenix = { version = "1", features = ["editor"] }
```

## Key Public API

`EditorSession`, `EditCommand`, `EditHistory`, `HistoryEntry`,
`ActiveGizmo`, `GizmoMode`, `GizmoAxis`, `GizmoSettings`, `TransformDelta`,
`apply_axis_delta`, `axis_pick`, `snap_translation`, `standard_panels`,
`PANEL_SCENE_TREE` … `PANEL_MATERIAL`, `ScenePanelRow`, `scene_rows`,
`kind_short_name`, `InspectorEdit`, `PbrField`, `PhysicalField`,
`TextureSlot`, `MaterialEditorLog`, `apply_pbr_field`,
`apply_physical_field`, `assign_texture`, `TimelinePreview`,
`TimelineFrame`, `AssetBrowserModel`, `AssetBrowserRow`,
`ProjectMetadata`, `SceneRoute`, `ProjectError`, `PROJECT_SCHEMA_VERSION`,
`HISTORY_DEFAULT_CAP`, `COALESCE_WINDOW_TICKS`, `EDITOR_VERSION`

## Common Use

```rust
use scenix_editor::{EditCommand, EditorSession, GizmoAxis, GizmoMode, InspectorEdit};
use scenix_scene::{SceneNode, SelectionMode};

let mut session = EditorSession::new("Editor");
let root = session.graph_mut().add(SceneNode::group("Hero"));
session.select(root, SelectionMode::Replace)?;
session
    .apply_inspector_edit(root, InspectorEdit::SetPosition((2.0, 0.0, 1.0).into()))?;
session.begin_drag(root, GizmoMode::Translate, GizmoAxis::X)?;
session.drag_gizmo(0.4)?;
session.end_drag();
session.undo()?; // restores the pre-drag transform exactly
```

## Notes

- Every mutation goes through an `EditCommand` whose inverse is computed
  from the pre-apply graph, so undo and redo are exact, not approximate.
- Continuous gizmo updates coalesce under one key + tick window
  (`COALESCE_WINDOW_TICKS`) so a whole drag is one undo step that restores
  the pre-drag pose.
- Continuous gizmo updates coalesce under one key + tick window
  (`COALESCE_WINDOW_TICKS`) so a whole drag is one undo step that restores
  the pre-drag pose.
- Material edits are optimistic: materials are host-owned resources, edits
  apply immediately and log to `MaterialEditorLog` for host-side undo.
- Hosts doing out-of-band graph edits (through `graph_mut`) own their undo
  obligation; the session history only guarantees what it recorded.
