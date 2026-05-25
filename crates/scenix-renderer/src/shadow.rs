/// Shared depth texture array for shadow-casting lights.
#[derive(Debug)]
pub struct ShadowMapAtlas {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: u32,
    layers: u32,
}

impl ShadowMapAtlas {
    /// Allocates a square depth texture array.
    pub fn new(device: &wgpu::Device, size: u32, layers: u32) -> Self {
        let size = size.max(1);
        let layers = layers.max(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scenix.shadow.atlas"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("scenix.shadow.atlas.view"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        Self {
            texture,
            view,
            size,
            layers,
        }
    }

    /// Returns the texture view used by shaders.
    #[inline]
    pub const fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns the backing texture.
    #[inline]
    pub const fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Returns shadow map width and height.
    #[inline]
    pub const fn size(&self) -> u32 {
        self.size
    }

    /// Returns the number of texture-array layers.
    #[inline]
    pub const fn layers(&self) -> u32 {
        self.layers
    }
}
