/// GPU texture and view pair used by renderer-owned targets.
#[derive(Debug)]
pub struct TextureTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
}

/// Renderer-owned render target kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderTargetKind {
    /// Color target rendered as a 2D texture.
    Color2D,
    /// HDR color target rendered as a 2D texture.
    Hdr2D,
    /// Depth-only target.
    Depth,
    /// Cube target metadata. v1.2 renders individual captures through 2D views.
    Cube,
}

/// Descriptor for a renderer-owned render target registered by `TextureId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RenderTargetDescriptor {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Texture format.
    pub format: wgpu::TextureFormat,
    /// Target kind.
    pub kind: RenderTargetKind,
    /// MSAA sample count. v1.2 render-to-texture uses one sample.
    pub sample_count: u32,
}

impl RenderTargetDescriptor {
    /// Creates a color 2D render target descriptor.
    #[inline]
    pub const fn color(width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        Self {
            width,
            height,
            format,
            kind: RenderTargetKind::Color2D,
            sample_count: 1,
        }
    }

    /// Creates an HDR color 2D render target descriptor.
    #[inline]
    pub const fn hdr(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            format: wgpu::TextureFormat::Rgba16Float,
            kind: RenderTargetKind::Hdr2D,
            sample_count: 1,
        }
    }

    /// Creates a depth render target descriptor.
    #[inline]
    pub const fn depth(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            format: wgpu::TextureFormat::Depth32Float,
            kind: RenderTargetKind::Depth,
            sample_count: 1,
        }
    }
}

impl TextureTarget {
    /// Allocates a renderable texture target.
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            format,
            width,
            height,
        }
    }

    /// Returns the texture view.
    #[inline]
    pub const fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns the backing texture.
    #[inline]
    pub const fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Returns the target format.
    #[inline]
    pub const fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Returns the width in pixels.
    #[inline]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height in pixels.
    #[inline]
    pub const fn height(&self) -> u32 {
        self.height
    }
}

/// Deferred renderer G-buffer textures.
#[derive(Debug)]
pub struct GBuffer {
    albedo: TextureTarget,
    normal: TextureTarget,
    material: TextureTarget,
    depth: TextureTarget,
    width: u32,
    height: u32,
}

impl GBuffer {
    /// Allocates G-buffer attachments.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let color_usage =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let depth_usage =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        Self {
            albedo: TextureTarget::new(
                device,
                "scenix.gbuffer.albedo",
                width,
                height,
                wgpu::TextureFormat::Rgba8Unorm,
                color_usage,
            ),
            normal: TextureTarget::new(
                device,
                "scenix.gbuffer.normal",
                width,
                height,
                wgpu::TextureFormat::Rgba16Float,
                color_usage,
            ),
            material: TextureTarget::new(
                device,
                "scenix.gbuffer.material",
                width,
                height,
                wgpu::TextureFormat::Rgba8Unorm,
                color_usage,
            ),
            depth: TextureTarget::new(
                device,
                "scenix.gbuffer.depth",
                width,
                height,
                wgpu::TextureFormat::Depth32Float,
                depth_usage,
            ),
            width,
            height,
        }
    }

    /// Reallocates attachments when the target size changes.
    #[inline]
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.width != width || self.height != height {
            *self = Self::new(device, width, height);
        }
    }

    /// Returns the width in pixels.
    #[inline]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height in pixels.
    #[inline]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the albedo attachment.
    #[inline]
    pub const fn albedo(&self) -> &TextureTarget {
        &self.albedo
    }

    /// Returns the normal attachment.
    #[inline]
    pub const fn normal(&self) -> &TextureTarget {
        &self.normal
    }

    /// Returns the material attachment.
    #[inline]
    pub const fn material(&self) -> &TextureTarget {
        &self.material
    }

    /// Returns the depth attachment.
    #[inline]
    pub const fn depth(&self) -> &TextureTarget {
        &self.depth
    }
}
