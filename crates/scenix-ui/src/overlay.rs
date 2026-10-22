use alloc::string::String;
use alloc::vec::Vec;

use scenix_math::Vec2;

use crate::layout::{Rect, UiScale};
use crate::panel::{DockKind, PanelStack, panel_placement};
use crate::stats::RendererStats;

/// Bounded rectangles one [`OverlayFrame`] may retain.
pub const MAX_OVERLAY_RECTS: usize = 256;

/// Bounded text commands one [`OverlayFrame`] may retain.
pub const MAX_OVERLAY_TEXTS: usize = 64;

/// Standard fade for panel backdrops, carried as an alpha byte hosts convert
/// into their own blend pipeline so CPU and GPU paths agree byte-for-byte.
pub const PANEL_BACKDROP_ALPHA: u8 = 200;

/// Accent alpha for the stats banner background.
pub const STATS_BACKDROP_ALPHA: u8 = 120;

/// One filled rectangle command.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UiRect {
    pub rect: Rect,
    /// sRGB8 + alpha fill color.
    pub fill: [u8; 4],
}

/// One positioned text command.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UiText {
    /// Bottom-left anchor of the glyph block.
    pub at: Vec2,
    pub text: String,
    /// sRGB8 + alpha glyph color.
    pub color: [u8; 4],
    /// Requested text size in pixels.
    pub size_px: f32,
}

/// Per-frame overlay draw list.
///
/// Capacity-bounded: pushes past the caps are dropped instead of growing the
/// allocation, guaranteeing a pathological host can never inflate a frame's
/// overlay beyond a fixed, predictable cost. All coordinates are physical
/// pixels unless the host scales [`UiScale`] beforehand.
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OverlayFrame {
    rects: Vec<UiRect>,
    texts: Vec<UiText>,
}

impl OverlayFrame {
    /// Empty frame.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a filled rectangle; the frame drops it past the cap.
    pub fn push_rect(&mut self, rect: Rect, fill: [u8; 4]) -> bool {
        if self.rects.len() >= MAX_OVERLAY_RECTS {
            return false;
        }
        self.rects.push(UiRect { rect, fill });
        true
    }

    /// Push a positioned string; the frame drops it past the cap.
    pub fn push_text(&mut self, at: Vec2, text: String, color: [u8; 4], size_px: f32) -> bool {
        if self.texts.len() >= MAX_OVERLAY_TEXTS {
            return false;
        }
        self.texts.push(UiText {
            at,
            text,
            color,
            size_px,
        });
        true
    }

    /// Push the backdrop of one visible panel from a pre-solved thickness.
    pub fn push_panel_backdrop(&mut self, panel: Rect) -> bool {
        self.push_rect(panel, [24, 24, 28, PANEL_BACKDROP_ALPHA])
    }

    /// Current rectangle commands in recording order.
    pub fn rects(&self) -> &[UiRect] {
        &self.rects
    }

    /// Current text commands in recording order.
    pub fn texts(&self) -> &[UiText] {
        &self.texts
    }

    /// Consume the frame into filtered lists for a single renderer draw.
    pub fn into_parts(self) -> (Vec<UiRect>, Vec<UiText>) {
        (self.rects, self.texts)
    }
}

/// Compose the standard editor overlay for one frame.
///
/// Deterministic op order: visible docked-panel backdrops (in registration
/// order from [`PanelStack`]), then the dock stripe outlines, then the stats
/// banner text when `stats` is supplied. This is the command list an egui,
/// winit, or HTML host replays; the pure solve keeps desktop, mobile, and
/// web pixels in lockstep.
pub fn compose_panel_overlay(
    viewport: Vec2,
    stack: &PanelStack,
    stats: Option<&RendererStats>,
) -> OverlayFrame {
    let mut frame = OverlayFrame::new();
    let layout = panel_placement(stack, viewport);

    for dock in [
        DockKind::Left,
        DockKind::Right,
        DockKind::Top,
        DockKind::Bottom,
    ] {
        if let Some(rect) = layout.edge_rect(viewport, dock) {
            frame.push_panel_backdrop(rect);
        }
    }

    if let Some(stats) = stats {
        let banner_lines = [stats.summary_line(), stats.summary_counts()];
        let scale = UiScale::ONE;
        let line_h = scale.px(14.0);
        let banner_align = Vec2::new(8.0, viewport.y - line_h * 2.0 - 8.0);
        let banner_rect = Rect::new(banner_align, Vec2::new(scale.px(268.0), line_h * 2.0 + 4.0));
        frame.push_rect(banner_rect, [16, 16, 20, STATS_BACKDROP_ALPHA]);
        for (row, text) in banner_lines.into_iter().enumerate() {
            frame.push_text(
                Vec2::new(
                    banner_align.x + 4.0,
                    banner_align.y + line_h * (row as f32 + 0.75),
                ),
                text,
                [232, 232, 240, 255],
                12.0,
            );
        }
    }

    frame
}
