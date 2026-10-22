use alloc::string::String;
use alloc::vec::Vec;
use scenix_math::Vec2;

use crate::layout::Rect;

/// Thinnest a visible side/edge panel may be, in pixels.
pub const PANEL_MIN_THICKNESS: f32 = 180.0;

/// Thickest a visible side/edge panel may be, in pixels.
pub const PANEL_MAX_THICKNESS: f32 = 480.0;

/// Greatest fraction of the viewport width/height a single dock may consume.
///
/// The solver never lets a docked column/row take more than this, leaving the
/// central viewport interactive even with every panel open.
pub const DOCK_MAX_FRACTION: f32 = 0.4;

/// Identifier of an editor panel.
///
/// Panel kinds are enumerations rather than strings so solve results are
/// deterministic and cheap to compare inside layout logs and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PanelId(pub u16);

impl PanelId {
    /// First panel id editors may allocate; scenix reserves none by default.
    pub const FIRST: Self = Self(0);
}

/// Panel dock position.
///
/// `Floating` carries its own pixel position; docked panels share an edge
/// solved by [`panel_placement`].
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DockKind {
    Left,
    Right,
    Bottom,
    Top,
    /// Panel floats over the viewport with its own pixel rect.
    Floating(Vec2),
}

impl DockKind {
    /// `true` for the four edge docks consumed by the space solver.
    pub fn is_docked(self) -> bool {
        !matches!(self, DockKind::Floating(_))
    }
}

/// One panel in an editor shell.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PanelSpec {
    pub id: PanelId,
    pub title: String,
    pub dock: DockKind,
    pub min_size: f32,
    pub thickness: f32,
    pub visible: bool,
}

impl PanelSpec {
    /// Panel docked on `dock` with the current thickness clamped into the
    /// global min/max band.
    pub fn docked(id: PanelId, title: String, dock: DockKind, thickness: f32) -> Self {
        Self {
            id,
            title,
            dock,
            min_size: PANEL_MIN_THICKNESS,
            thickness: thickness.clamp(PANEL_MIN_THICKNESS, PANEL_MAX_THICKNESS),
            visible: true,
        }
    }

    /// Floating panel pinned at `position` with `size` clamped into the band
    /// along both axes.
    pub fn floating(id: PanelId, title: String, position: Vec2, size: f32) -> Self {
        Self {
            id,
            title,
            dock: DockKind::Floating(position),
            min_size: PANEL_MIN_THICKNESS,
            thickness: size.clamp(PANEL_MIN_THICKNESS, PANEL_MAX_THICKNESS),
            visible: true,
        }
    }

    /// Result rect of a docked panel given its computed thickness and the
    /// viewport size. Pure — callers keep solve results in their own frame
    /// state so hits confirm with the same numbers layout used.
    pub fn panel_rect(&self, viewport: Vec2, thickness: f32) -> Rect {
        match self.dock {
            DockKind::Left => Rect::new(Vec2::ZERO, Vec2::new(thickness, viewport.y)),
            DockKind::Right => Rect::new(
                Vec2::new(viewport.x - thickness, 0.0),
                Vec2::new(thickness, viewport.y),
            ),
            DockKind::Top => Rect::new(Vec2::ZERO, Vec2::new(viewport.x, thickness)),
            DockKind::Bottom => Rect::new(
                Vec2::new(0.0, viewport.y - thickness),
                Vec2::new(viewport.x, thickness),
            ),
            DockKind::Floating(p) => Rect::new(p, Vec2::new(self.thickness, self.thickness)),
        }
    }
}

/// Geometry allocated to the four edge docks by the first solve pass.
///
/// Every field is the clamped total thickness of the visible panels docked on
/// that edge; `viewport_remaining` derives the interactive center window.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct DockedLayout {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl DockedLayout {
    /// Center viewport rect not occupied by docked panels.
    pub fn viewport_remaining(&self, viewport: Vec2) -> Rect {
        let min = Vec2::new(self.left, self.top);
        let size = Vec2::new(
            (viewport.x - self.left - self.right).max(0.0),
            (viewport.y - self.top - self.bottom).max(0.0),
        );
        Rect::new(min, size)
    }

    /// Panel rect for a docked edge given the solved dock thickness.
    pub fn edge_rect(&self, viewport: Vec2, dock: DockKind) -> Option<Rect> {
        match dock {
            DockKind::Left if self.left > 0.0 => {
                Some(Rect::new(Vec2::ZERO, Vec2::new(self.left, viewport.y)))
            }
            DockKind::Right if self.right > 0.0 => Some(Rect::new(
                Vec2::new(viewport.x - self.right, 0.0),
                Vec2::new(self.right, viewport.y),
            )),
            DockKind::Top if self.top > 0.0 => {
                Some(Rect::new(Vec2::ZERO, Vec2::new(viewport.x, self.top)))
            }
            DockKind::Bottom if self.bottom > 0.0 => Some(Rect::new(
                Vec2::new(0.0, viewport.y - self.bottom),
                Vec2::new(viewport.x, self.bottom),
            )),
            _ => None,
        }
    }
}

/// Result of a visibility/dock mutation, so shells can log deterministic
/// panel state transitions without diffing specs themselves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelEvent {
    Opened,
    Closed,
    NoChange,
}

/// Ordered registry of editor panels with solve-ready visibility state.
///
/// Insert order is the tab order for stacked docks; the solver walks it once
/// per frame, so panel count is bounded by the host application rather than
/// any internal dynamic allocation beyond the `Vec` itself.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PanelStack {
    panels: Vec<PanelSpec>,
}

impl PanelStack {
    /// Empty stack.
    pub fn new() -> Self {
        Self {
            panels: alloc::vec::Vec::new(),
        }
    }

    /// Number of registered panels.
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    /// Whether no panel is registered.
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }

    /// Insert (or replace) a panel spec; first insertion wins a slot in the
    /// dock tab order.
    pub fn upsert(&mut self, spec: PanelSpec) -> PanelEvent {
        for slot in self.panels.iter_mut() {
            if slot.id == spec.id {
                *slot = spec;
                return PanelEvent::NoChange;
            }
        }
        self.panels.push(spec);
        PanelEvent::Opened
    }

    /// Show a panel, creating a default docked spec when absent.
    pub fn open(&mut self, id: PanelId, title: String, dock: DockKind) -> PanelEvent {
        if let Some(slot) = self.panels.iter_mut().find(|slot| slot.id == id) {
            if slot.visible {
                slot.dock = dock;
                return PanelEvent::NoChange;
            }
            slot.dock = dock;
            slot.visible = true;
            return PanelEvent::Opened;
        }
        self.panels
            .push(PanelSpec::docked(id, title, dock, PANEL_MIN_THICKNESS));
        PanelEvent::Opened
    }

    /// Hide a panel.
    pub fn close(&mut self, id: PanelId) -> PanelEvent {
        match self.panels.iter_mut().find(|slot| slot.id == id) {
            Some(slot) if slot.visible => {
                slot.visible = false;
                PanelEvent::Closed
            }
            _ => PanelEvent::NoChange,
        }
    }

    /// Toggle a panel's visibility, returning whether it ended up visible.
    pub fn toggle(&mut self, id: PanelId) -> bool {
        if let Some(slot) = self.panels.iter_mut().find(|slot| slot.id == id) {
            slot.visible = !slot.visible;
            return slot.visible;
        }
        false
    }

    /// Retarget a panel onto another dock.
    pub fn set_dock(&mut self, id: PanelId, dock: DockKind) -> bool {
        self.panels
            .iter_mut()
            .find(|slot| slot.id == id)
            .map(|slot| slot.dock = dock)
            .is_some()
    }

    /// Resize a panel thickness, clamped to the global band.
    pub fn set_thickness(&mut self, id: PanelId, thickness: f32) -> bool {
        self.panels
            .iter_mut()
            .find(|slot| slot.id == id)
            .map(|slot| slot.thickness = thickness.clamp(PANEL_MIN_THICKNESS, PANEL_MAX_THICKNESS))
            .is_some()
    }

    /// Whether the panel is registered and visible.
    pub fn is_visible(&self, id: PanelId) -> bool {
        self.panels.iter().any(|slot| slot.id == id && slot.visible)
    }

    /// Registration lookup: the matching `PanelSpec`.
    pub fn spec(&self, id: PanelId) -> Option<&PanelSpec> {
        self.panels.iter().find(|slot| slot.id == id)
    }

    /// Visible panels in registration (tab) order.
    pub fn visible_in_order(&self) -> impl Iterator<Item = &PanelSpec> {
        self.panels.iter().filter(|slot| slot.visible)
    }
}

/// Solve the dock thickness per edge from a panel stack.
///
/// Deterministic first-fit allocator: visible docked panels walk in
/// registration (tab) order and each claims `min(configured thickness,
/// remaining budget)` while the remaining budget is still at least
/// [`PANEL_MIN_THICKNESS`], so unusable slivers are never handed out and the
/// per-dock budget (`DOCK_MAX_FRACTION` of the matching viewport axis, but at
/// least one minimum panel) is never exceeded. Every visible panel therefore
/// receives its configured size under light docking load; hosts close or
/// float panels when a dock runs full. Pure, allocation free, and identical
/// for identical input stacks so two hosts agree on the same pixel layout
/// every frame.
pub fn panel_placement(stack: &PanelStack, viewport: Vec2) -> DockedLayout {
    let mut solved = DockedLayout::default();

    // Dock budget per edge: fraction of the matching axis, but at least one
    // minimum panel so the first panel always docks.
    let budget_x = (viewport.x * DOCK_MAX_FRACTION).max(PANEL_MIN_THICKNESS);
    let budget_y = (viewport.y * DOCK_MAX_FRACTION).max(PANEL_MIN_THICKNESS);

    for spec in stack.panels.iter().filter(|spec| spec.visible) {
        match spec.dock {
            DockKind::Left if budget_x - solved.left >= spec.min_size => {
                solved.left += spec.thickness.min(budget_x - solved.left);
            }
            DockKind::Right if budget_x - solved.right >= spec.min_size => {
                solved.right += spec.thickness.min(budget_x - solved.right);
            }
            DockKind::Top if budget_y - solved.top >= spec.min_size => {
                solved.top += spec.thickness.min(budget_y - solved.top);
            }
            DockKind::Bottom if budget_y - solved.bottom >= spec.min_size => {
                solved.bottom += spec.thickness.min(budget_y - solved.bottom);
            }
            _ => {}
        }
    }
    solved
}
