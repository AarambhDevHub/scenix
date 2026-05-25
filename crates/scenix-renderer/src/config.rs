use scenix_core::ValidationError;

/// Render target backing used by a renderer instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderTargetMode {
    /// Frames are presented to a platform surface.
    Surface,
    /// Frames are rendered to an offscreen texture for tests, tools, or captures.
    Headless,
}

/// Renderer initialization and resize configuration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RendererConfig {
    /// Render target width in pixels.
    pub width: u32,
    /// Render target height in pixels.
    pub height: u32,
    /// MSAA sample count. v0.6 supports `1` and `4`.
    pub sample_count: u32,
    /// Whether presentation should synchronize to display refresh.
    pub vsync: bool,
    /// Whether the renderer should use an HDR intermediate color target.
    pub hdr: bool,
    /// Surface present mode.
    pub present_mode: wgpu::PresentMode,
    /// Backends requested from wgpu.
    pub backends: wgpu::Backends,
    /// Clear color used before drawing.
    pub clear_color: wgpu::Color,
}

impl RendererConfig {
    /// Creates a renderer configuration for the supplied size.
    #[inline]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Self::default()
        }
    }

    /// Validates render-target dimensions and MSAA count.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.width == 0 || self.height == 0 {
            return Err(ValidationError::OutOfRange);
        }
        if !matches!(self.sample_count, 1 | 4) {
            return Err(ValidationError::OutOfRange);
        }
        if self.backends.is_empty() {
            return Err(ValidationError::InvalidState);
        }
        Ok(())
    }

    /// Returns the preferred color format for this configuration.
    #[inline]
    pub const fn preferred_color_format(&self) -> wgpu::TextureFormat {
        if self.hdr {
            wgpu::TextureFormat::Rgba16Float
        } else {
            wgpu::TextureFormat::Bgra8UnormSrgb
        }
    }

    /// Returns a copy with the dimensions changed.
    #[inline]
    pub fn resized(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Returns a copy configured for vsync.
    #[inline]
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self.present_mode = if vsync {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::Immediate
        };
        self
    }

    /// Returns a copy configured for HDR intermediates.
    #[inline]
    pub const fn hdr(mut self, hdr: bool) -> Self {
        self.hdr = hdr;
        self
    }
}

impl Default for RendererConfig {
    #[inline]
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            sample_count: 1,
            vsync: true,
            hdr: false,
            present_mode: wgpu::PresentMode::Fifo,
            backends: wgpu::Backends::all(),
            clear_color: wgpu::Color {
                r: 0.02,
                g: 0.025,
                b: 0.035,
                a: 1.0,
            },
        }
    }
}
