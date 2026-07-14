use scenix_math::Vec2;

/// Logical/physical viewport measurements used by input and camera controls.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ViewportMetrics {
    /// CSS/window size in logical pixels.
    pub logical_size: Vec2,
    /// Backing-buffer size in physical pixels.
    pub physical_size: [u32; 2],
    /// Physical pixels per logical pixel.
    pub scale_factor: f32,
}

impl ViewportMetrics {
    /// Creates sanitized viewport metrics from a logical size and scale factor.
    pub fn new(logical_size: Vec2, scale_factor: f32) -> Self {
        let logical_size = Vec2::new(logical_size.x.max(1.0), logical_size.y.max(1.0));
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        Self {
            logical_size,
            physical_size: [
                ((logical_size.x * scale_factor + 0.5) as u32).max(1),
                ((logical_size.y * scale_factor + 0.5) as u32).max(1),
            ],
            scale_factor,
        }
    }

    /// Creates metrics from a physical backing size and scale factor.
    pub fn from_physical(physical_size: [u32; 2], scale_factor: f32) -> Self {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        let physical_size = [physical_size[0].max(1), physical_size[1].max(1)];
        Self {
            logical_size: Vec2::new(
                physical_size[0] as f32 / scale_factor,
                physical_size[1] as f32 / scale_factor,
            ),
            physical_size,
            scale_factor,
        }
    }

    /// Converts a logical point to physical pixels.
    #[inline]
    pub fn logical_to_physical(self, point: Vec2) -> Vec2 {
        point * self.scale_factor
    }

    /// Converts a physical point to logical pixels.
    #[inline]
    pub fn physical_to_logical(self, point: Vec2) -> Vec2 {
        point / self.scale_factor
    }

    /// Converts a top-left-origin logical point to WebGPU NDC.
    #[inline]
    pub fn logical_to_ndc(self, point: Vec2) -> Vec2 {
        Vec2::new(
            point.x / self.logical_size.x * 2.0 - 1.0,
            1.0 - point.y / self.logical_size.y * 2.0,
        )
    }

    /// Width divided by height.
    #[inline]
    pub fn aspect(self) -> f32 {
        self.logical_size.x / self.logical_size.y
    }
}

impl Default for ViewportMetrics {
    fn default() -> Self {
        Self::new(Vec2::ONE, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions_preserve_logical_coordinates() {
        let viewport = ViewportMetrics::new(Vec2::new(400.0, 200.0), 2.0);
        assert_eq!(viewport.physical_size, [800, 400]);
        assert_eq!(viewport.logical_to_ndc(Vec2::new(200.0, 100.0)), Vec2::ZERO);
        let point = Vec2::new(12.0, 34.0);
        assert_eq!(
            viewport.physical_to_logical(viewport.logical_to_physical(point)),
            point
        );
    }
}
