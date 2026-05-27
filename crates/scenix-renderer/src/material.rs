use scenix_core::Color;
use scenix_material::{
    LambertMaterial, Material, NormalMaterial, PbrMaterial, PhysicalMaterial, ToonMaterial,
    UnlitMaterial, WireframeMaterial,
};
use scenix_math::Vec3;

use crate::TextureStore;

/// GPU-ready material uniform shared by built-in v0.6 material paths.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    /// Base color or diffuse color in linear RGBA.
    pub base_color: [f32; 4],
    /// Emissive color and alpha cutoff.
    pub emissive_cutoff: [f32; 4],
    /// Metallic, roughness, shader kind, and flags.
    pub params: [f32; 4],
}

impl MaterialUniform {
    /// Creates a material uniform from common factors.
    #[inline]
    pub fn new(
        base_color: Color,
        emissive: Vec3,
        metallic: f32,
        roughness: f32,
        alpha_cutoff: Option<f32>,
        shader_kind: f32,
        feature_bits: u64,
    ) -> Self {
        Self {
            base_color: base_color.to_array(),
            emissive_cutoff: [
                emissive.x,
                emissive.y,
                emissive.z,
                alpha_cutoff.unwrap_or(-1.0),
            ],
            params: [
                metallic.clamp(0.0, 1.0),
                roughness.clamp(0.0, 1.0),
                shader_kind,
                feature_bits as f32,
            ],
        }
    }
}

/// Bridges CPU-side material descriptions into renderer-owned GPU resources.
pub trait GpuMaterial: Material {
    /// Returns the bind-group layout used by this material family.
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
    where
        Self: Sized,
    {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scenix.material.empty_layout"),
            entries: &[],
        })
    }

    /// Serializes material state into uniform bytes.
    fn to_uniform_bytes(&self) -> Vec<u8>;

    /// Creates a bind group for this material.
    fn create_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        _textures: &TextureStore,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scenix.material.bind_group"),
            layout,
            entries: &[],
        })
    }
}

impl GpuMaterial for PbrMaterial {
    fn to_uniform_bytes(&self) -> Vec<u8> {
        let uniform = MaterialUniform::new(
            self.albedo,
            self.emissive,
            self.metallic,
            self.roughness,
            self.alpha_cutoff(),
            0.0,
            self.pipeline_key().feature_bits,
        );
        bytemuck::bytes_of(&uniform).to_vec()
    }
}

impl GpuMaterial for UnlitMaterial {
    fn to_uniform_bytes(&self) -> Vec<u8> {
        let uniform = MaterialUniform::new(
            self.color,
            Vec3::ZERO,
            0.0,
            1.0,
            self.alpha_cutoff(),
            1.0,
            self.pipeline_key().feature_bits,
        );
        bytemuck::bytes_of(&uniform).to_vec()
    }
}

impl GpuMaterial for LambertMaterial {
    fn to_uniform_bytes(&self) -> Vec<u8> {
        let uniform = MaterialUniform::new(
            self.color,
            self.emissive,
            0.0,
            1.0,
            self.alpha_cutoff(),
            2.0,
            self.pipeline_key().feature_bits,
        );
        bytemuck::bytes_of(&uniform).to_vec()
    }
}

impl GpuMaterial for PhysicalMaterial {
    fn to_uniform_bytes(&self) -> Vec<u8> {
        let uniform = MaterialUniform::new(
            self.base.albedo,
            self.base.emissive,
            self.base.metallic,
            self.base.roughness,
            self.alpha_cutoff(),
            3.0,
            self.pipeline_key().feature_bits,
        );
        bytemuck::bytes_of(&uniform).to_vec()
    }
}

impl GpuMaterial for ToonMaterial {
    fn to_uniform_bytes(&self) -> Vec<u8> {
        let uniform = MaterialUniform::new(
            self.color,
            Vec3::ZERO,
            0.0,
            1.0,
            self.alpha_cutoff(),
            4.0,
            self.pipeline_key().feature_bits,
        );
        bytemuck::bytes_of(&uniform).to_vec()
    }
}

impl GpuMaterial for WireframeMaterial {
    fn to_uniform_bytes(&self) -> Vec<u8> {
        let uniform = MaterialUniform::new(
            Color {
                a: self.opacity,
                ..self.color
            },
            Vec3::ZERO,
            0.0,
            1.0,
            self.alpha_cutoff(),
            5.0,
            self.pipeline_key().feature_bits,
        );
        bytemuck::bytes_of(&uniform).to_vec()
    }
}

impl GpuMaterial for NormalMaterial {
    fn to_uniform_bytes(&self) -> Vec<u8> {
        let uniform = MaterialUniform::new(
            Color::WHITE,
            Vec3::ZERO,
            0.0,
            1.0,
            self.alpha_cutoff(),
            6.0,
            self.pipeline_key().feature_bits,
        );
        bytemuck::bytes_of(&uniform).to_vec()
    }
}
