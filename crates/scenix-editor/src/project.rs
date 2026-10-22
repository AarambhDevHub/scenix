use std::fmt;
use std::string::String;
use std::vec::Vec;

use crate::EDITOR_VERSION;
use crate::shell::EditorSession;

/// Current project metadata schema version.
///
/// Hosts embed this inside `.scenixproject` files; [`ProjectMetadata::upgrade_to_current`]
/// migrates older files forward deterministically as the schema grows.
pub const PROJECT_SCHEMA_VERSION: u32 = 1;

/// One scene reference inside a project.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SceneRoute {
    /// Display name of the scene.
    pub name: String,
    /// Asset path the scene loads from (or an editor-built scene name).
    pub asset: String,
}

impl SceneRoute {
    /// Route with both fields set.
    pub fn new(name: impl Into<String>, asset: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            asset: asset.into(),
        }
    }
}

/// Versioned project metadata exchanged between editor hosts.
///
/// Deliberately small: identity, authorship, schema version, and the scene
/// route list. Scene graph *content* serializes through `scenix-scene`; this
/// record is the wrapper hosts save alongside or inside their files.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectMetadata {
    /// Project title (non-empty after trim).
    pub title: String,
    /// Author or team name.
    pub author: String,
    /// Schema version this record validates against.
    pub schema_version: u32,
    /// Scene routes in save order.
    pub scenes: Vec<SceneRoute>,
    /// scenix editor version that produced the record.
    pub created_with: String,
}

impl ProjectMetadata {
    /// Fresh empty project with the current schema and editor versions.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            author: String::new(),
            schema_version: PROJECT_SCHEMA_VERSION,
            scenes: Vec::new(),
            created_with: String::from(EDITOR_VERSION),
        }
    }

    /// Metadata describing one open session (title from the session).
    pub fn from_session(session: &EditorSession) -> Self {
        Self {
            title: String::from(session.title()),
            author: String::new(),
            schema_version: PROJECT_SCHEMA_VERSION,
            scenes: Vec::new(),
            created_with: String::from(EDITOR_VERSION),
        }
    }

    /// Append a scene route.
    pub fn add_scene(&mut self, route: SceneRoute) {
        self.scenes.push(route);
    }

    /// Validate invariants: non-empty title, unique non-empty route names,
    /// non-empty route assets, and a supported schema version.
    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.title.trim().is_empty() {
            return Err(ProjectError::EmptyTitle);
        }
        if self.schema_version > PROJECT_SCHEMA_VERSION {
            return Err(ProjectError::UnsupportedSchema);
        }
        let mut seen: Vec<&str> = Vec::with_capacity(self.scenes.len());
        for route in &self.scenes {
            if route.name.trim().is_empty() || route.asset.trim().is_empty() {
                return Err(ProjectError::EmptyRoute);
            }
            if seen.contains(&route.name.as_str()) {
                return Err(ProjectError::DuplicateScene);
            }
            seen.push(route.name.as_str());
        }
        Ok(())
    }

    /// Upgrade the schema note to the current version when possible.
    ///
    /// Schema v1 is the baseline, so this only reports whether anything
    /// changed; future versions gain real migration steps. Downgrades and
    /// unknown future versions error.
    pub fn upgrade_to_current(&mut self) -> Result<bool, ProjectError> {
        match self.schema_version {
            PROJECT_SCHEMA_VERSION => Ok(false),
            version if version < PROJECT_SCHEMA_VERSION => {
                self.schema_version = PROJECT_SCHEMA_VERSION;
                Ok(true)
            }
            _ => Err(ProjectError::UnsupportedSchema),
        }
    }

    /// Serialize to pretty JSON (feature `serde`).
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> Result<String, ProjectError> {
        serde_json::to_string_pretty(self).map_err(|_| ProjectError::Serialization)
    }

    /// Parse from JSON, validating afterwards (feature `serde`).
    #[cfg(feature = "serde")]
    pub fn from_json(text: &str) -> Result<Self, ProjectError> {
        let metadata: Self = serde_json::from_str(text).map_err(|_| ProjectError::Serialization)?;
        metadata.validate()?;
        Ok(metadata)
    }
}

/// Validation and (de)serialization failures for project metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectError {
    /// Title was empty or whitespace.
    EmptyTitle,
    /// A scene route had an empty name or asset.
    EmptyRoute,
    /// Two scenes shared a name.
    DuplicateScene,
    /// The record's schema version is newer than this build.
    UnsupportedSchema,
    /// JSON encoding or decoding failed.
    Serialization,
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => write!(f, "project title is empty"),
            Self::EmptyRoute => write!(f, "scene route has an empty name or asset"),
            Self::DuplicateScene => write!(f, "duplicate scene name in project"),
            Self::UnsupportedSchema => write!(f, "project schema version is unsupported"),
            Self::Serialization => write!(f, "project JSON encoding failed"),
        }
    }
}

impl std::error::Error for ProjectError {}
