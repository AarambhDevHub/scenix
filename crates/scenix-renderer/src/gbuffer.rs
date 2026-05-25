/// GPU texture and view pair used by renderer-owned targets.
#[derive(Debug)]
pub struct TextureTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
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
