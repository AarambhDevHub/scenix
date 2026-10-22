use core::ops::Add;

use scenix_math::Vec2;

/// Pixel distances on each side of a rectangle.
///
/// Used for safe-area insets, panel margins, and anchor offsets. Construction
/// clamps negatives to zero so layout solves never invert their regions.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Edges {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Edges {
    /// All sides zero — the neutral inset.
    pub const ZERO: Self = Self {
        left: 0.0,
        right: 0.0,
        top: 0.0,
        bottom: 0.0,
    };

    /// Uniform edge distance on all four sides.
    pub fn uniform(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    /// Independent distances with negatives clamped to zero.
    pub const fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left: if left < 0.0 { 0.0 } else { left },
            right: if right < 0.0 { 0.0 } else { right },
            top: if top < 0.0 { 0.0 } else { top },
            bottom: if bottom < 0.0 { 0.0 } else { bottom },
        }
    }

    /// Distance removed from the left-to-right axis.
    pub const fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Distance removed from the top-to-bottom axis.
    pub const fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    /// Per-side maximum — merge helper for combining safe-area sources.
    pub fn union(self, other: Edges) -> Edges {
        Self {
            left: self.left.max(other.left),
            right: self.right.max(other.right),
            top: self.top.max(other.top),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

impl Add for Edges {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            left: self.left + rhs.left,
            right: self.right + rhs.right,
            top: self.top + rhs.top,
            bottom: self.bottom + rhs.bottom,
        }
    }
}

/// Elementwise maximum — scenix-math `Vec2` has no elementwise helpers, so
/// this keeps every layout solve allocation free and branch cheap.
fn elementwise_max(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x.max(b.x), a.y.max(b.y))
}

/// Elementwise minimum.
fn elementwise_min(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x.min(b.x), a.y.min(b.y))
}

/// Pixel-aligned screen rectangle: top-left minimum plus extent.
///
/// Sizes are clamped non-negative on construction; every method stays
/// allocation free so overlays can be solved inside the frame budget.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect {
    pub min: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub const EMPTY: Self = Self {
        min: Vec2::ZERO,
        size: Vec2::ZERO,
    };

    pub fn new(min: Vec2, size: Vec2) -> Self {
        Self {
            min,
            size: Vec2::new(size.x.max(0.0), size.y.max(0.0)),
        }
    }

    /// Bottom-right corner (`min + size`).
    pub fn max(&self) -> Vec2 {
        self.min + self.size
    }

    /// Center point of the rectangle.
    pub fn center(&self) -> Vec2 {
        self.min + self.size * 0.5
    }

    /// Whether `point` lies inside the rectangle (left/top included,
    /// right/bottom excluded so adjacent rects do not overlap).
    pub fn contains(&self, point: Vec2) -> bool {
        let max = self.max();
        point.x >= self.min.x && point.x < max.x && point.y >= self.min.y && point.y < max.y
    }

    /// Largest rectangle fully covered by both rectangles.
    pub fn intersect(self, other: Rect) -> Rect {
        let min = elementwise_max(self.min, other.min);
        let max = elementwise_min(self.max(), other.max());
        let size = Vec2::new((max.x - min.x).max(0.0), (max.y - min.y).max(0.0));
        Self::new(min, size)
    }
}

/// Nine-way anchor placement for fixed-size elements inside a viewport.
///
/// The solve is pure and allocation free: [`Anchor::place`] positions the
/// element flush against the anchored edge or centers it, then clamps the
/// result back into the inset-bounded region so oversized elements never
/// bleed outside the safe area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Anchor {
    /// Rectangle for `size` anchored inside `viewport` (inset by `inset`),
    /// offset toward the anchored edge by `offset` pixels.
    pub fn place(self, viewport: Vec2, inset: Edges, size: Vec2, offset: Vec2) -> Rect {
        let inner_min = Vec2::new(inset.left, inset.top);
        let inner_size = Vec2::new(
            (viewport.x - inset.horizontal()).max(0.0),
            (viewport.y - inset.vertical()).max(0.0),
        );
        let inner_max = inner_min + inner_size;
        let size = Vec2::new(size.x.max(0.0), size.y.max(0.0));

        let min = match self {
            Anchor::TopLeft | Anchor::CenterLeft | Anchor::BottomLeft => {
                inner_min + Vec2::new(offset.x, 0.0)
            }
            Anchor::TopCenter | Anchor::Center | Anchor::BottomCenter => {
                inner_min + Vec2::new((inner_size.x - size.x) * 0.5 + offset.x, 0.0)
            }
            Anchor::TopRight | Anchor::CenterRight | Anchor::BottomRight => {
                inner_max - Vec2::new(size.x + offset.x, 0.0)
            }
        };
        let min = match self {
            Anchor::TopLeft | Anchor::TopCenter | Anchor::TopRight => {
                Vec2::new(min.x, inner_min.y + offset.y)
            }
            Anchor::CenterLeft | Anchor::Center | Anchor::CenterRight => Vec2::new(
                min.x,
                inner_min.y + (inner_size.y - size.y) * 0.5 + offset.y,
            ),
            Anchor::BottomLeft | Anchor::BottomCenter | Anchor::BottomRight => {
                Vec2::new(min.x, inner_max.y - size.y - offset.y)
            }
        };

        // Clamp back into the inset region so roundoff can never leak outside
        // the safe area, then clamp the size against the remaining extent.
        let min = elementwise_max(elementwise_min(min, inner_max), inner_min);
        let remain = inner_max - min;
        let size = elementwise_min(size, Vec2::new(remain.x.max(0.0), remain.y.max(0.0)));
        Rect::new(min, size)
    }
}

/// Largest rectangle inside `viewport` after removing `insets` on every side.
pub fn safe_rect(viewport: Vec2, insets: Edges) -> Rect {
    let min = Vec2::new(insets.left, insets.top);
    let size = Vec2::new(
        (viewport.x - insets.horizontal()).max(0.0),
        (viewport.y - insets.vertical()).max(0.0),
    );
    Rect::new(min, size)
}

/// UI density scale between logical points and physical pixels.
///
/// A factor of `1.0` maps one logical point to one pixel; DPR-aware hosts use
/// the physical device-pixel ratio so windows stay equally readable across
/// desktop, mobile, and web canvases.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UiScale {
    pub factor: f32,
}

impl UiScale {
    /// One logical point per pixel.
    pub const ONE: Self = Self { factor: 1.0 };

    /// Scale from a device pixel ratio, clamped away from zero.
    pub fn from_dpr(dpr: f32) -> Self {
        Self {
            factor: if dpr.is_finite() && dpr > 0.0 {
                dpr
            } else {
                1.0
            },
        }
    }

    /// Physical pixels for a logical value.
    pub fn px(&self, logical: f32) -> f32 {
        logical * self.factor
    }

    /// Physical size for a logical size.
    pub fn size_px(&self, size: Vec2) -> Vec2 {
        size * self.factor
    }
}
