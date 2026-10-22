//! Offline CPU editor tooling for scenix.
//!
//! This crate glues the engine crates into the shape an editor needs: an
//! [`EditorSession`] that owns a [`scenix_scene::SceneGraph`] plus an undo
//! history of concrete [`EditCommand`] trees, gizmo drag math, panel models
//! for the scene tree and inspector, material field editing, asset browsing
//! over `scenix-loader`, timeline previews over `scenix-animato`, and a small
//! versioned project metadata exchange.
//!
//! The crate renders nothing and open no windows. It is the deterministic
//! model layer behind host faces — the egui facade example, HTML tools, and
//! native windows — so every host agrees on pixel solves, selection rules,
//! and undo granularity for the same input stream. Heavy integrations
//! (`scenix-loader`, `scenix-animato`, `scenix-material`) stay behind
//! features so bare-metal or web tool builds stay lean.
//!
//! - `shell`: the session state machine, standard panel stack, gizmo drags.
//! - `history`: concrete edit commands with inverse computation.
//! - `gizmo`: axis picking and snapped delta application.
//! - `panels`: scene tree rows with depth, selection, and lock state.
//! - `inspector`: selected-node rows plus edit routing (feature `inspector`).
//! - `material_editor`: PBR/physical field writes with clamps (feature
//!   `material`).
//! - `assets`: browser rows over loader statuses (feature `assets`).
//! - `timeline`: playback preview over the animato loop solver (feature
//!   `timeline`).
//! - `project`: versioned project metadata exchange.

mod gizmo;
mod history;
mod panels;
mod project;
mod shell;

#[cfg(feature = "assets")]
mod assets;
#[cfg(feature = "inspector")]
mod inspector;
#[cfg(feature = "material")]
mod material_editor;
#[cfg(feature = "timeline")]
mod timeline;

/// Crate version reported by editor sessions and project metadata.
pub const EDITOR_VERSION: &str = "1.10.0";

pub use gizmo::{
    GIZMO_HALF_LENGTH, GIZMO_PICK_THRESHOLD, GizmoAxis, GizmoMode, GizmoSettings,
    ROTATE_SNAP_RADIANS, SCALE_SNAP_STEP, TRANSLATE_SNAP_STEP, TransformDelta, apply_axis_delta,
    axis_pick, snap_translation,
};
pub use history::{
    COALESCE_WINDOW_TICKS, EditCommand, EditHistory, HISTORY_DEFAULT_CAP, HistoryEntry,
};
pub use panels::{SCENE_PANEL_ROW_LIMIT, ScenePanelRow, kind_short_name, scene_rows};
pub use project::{PROJECT_SCHEMA_VERSION, ProjectError, ProjectMetadata, SceneRoute};
pub use shell::{
    ActiveGizmo, EditorSession, GIZMO_COALESCE_KEY, PANEL_ASSETS, PANEL_INSPECTOR, PANEL_MATERIAL,
    PANEL_SCENE_TREE, PANEL_TIMELINE, standard_panels,
};

#[cfg(feature = "assets")]
pub use assets::{
    AssetBrowserModel, AssetBrowserRow, AssetStatusClass, BROWSER_LABEL_CAP, BROWSER_ROW_LIMIT,
};
#[cfg(feature = "inspector")]
pub use inspector::{INSPECTOR_ROW_LIMIT, InspectorEdit};
#[cfg(feature = "material")]
pub use material_editor::{
    MATERIAL_IOR_MAX, MATERIAL_IOR_MIN, MATERIAL_LOG_DEFAULT_CAP, MATERIAL_UNIT_MAX,
    MaterialChange, MaterialEditorLog, PbrField, PhysicalField, TextureSlot, apply_pbr_field,
    apply_physical_field, assign_texture,
};
#[cfg(feature = "timeline")]
pub use timeline::{
    TIMELINE_RATE_MAX, TIMELINE_RATE_MIN, TIMELINE_SNAP_FPS, TimelineFrame, TimelinePreview,
};
