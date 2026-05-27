use scenix_camera::PerspectiveCamera;
use scenix_core::{LightId, MaterialId, MeshId, TextureId, ValidationError};
use scenix_material::{
    AlphaMode, Material, NormalMaterial, PbrMaterial, PhysicalMaterial, PipelineKey, ToonMaterial,
    UnlitMaterial, WireframeMaterial,
};
use scenix_math::{Aabb, Mat4, Vec3};
use scenix_mesh::{Geometry, box_geometry};
use scenix_renderer::{
    DrawSubmission, GpuIndexFormat, GpuMaterial, GpuScene, MaterialUniform, RenderPassKind,
    RendererConfig, RendererMaterial, RendererPipelineKey, to_wgpu_address_mode, to_wgpu_compare,
    to_wgpu_filter_mode, to_wgpu_texture_format,
};
use scenix_scene::SceneGraph;
use scenix_texture::{AddressMode, CompareFunction, FilterMode, Sampler, Texture2D, TextureFormat};

fn draw(distance_to_camera: f32) -> DrawSubmission {
    DrawSubmission {
        mesh_id: MeshId::new(distance_to_camera as u64 + 1),
        material_id: MaterialId::new(1),
        world_matrix: Mat4::IDENTITY,
        world_aabb: Aabb::new(Vec3::ZERO, Vec3::ONE),
        distance_to_camera,
        transparent: false,
        render_order: 0,
    }
}

#[test]
fn renderer_config_validation_rejects_invalid_targets() {
    assert!(RendererConfig::new(64, 32).validate().is_ok());

    let mut zero_width = RendererConfig::new(0, 32);
    assert_eq!(zero_width.validate(), Err(ValidationError::OutOfRange));

    zero_width.width = 64;
    zero_width.sample_count = 2;
    assert_eq!(zero_width.validate(), Err(ValidationError::OutOfRange));

    zero_width.sample_count = 1;
    zero_width.backends = scenix_renderer::wgpu::Backends::empty();
    assert_eq!(zero_width.validate(), Err(ValidationError::InvalidState));
}

#[test]
fn geometry_packing_uses_interleaved_vertices_and_u16_indices_when_possible() {
    let geometry = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);
    let packed = GpuScene::pack_geometry(&geometry).unwrap();

    assert_eq!(packed.vertices.len(), geometry.positions.len());
    assert_eq!(packed.index_count as usize, geometry.indices.len());
    assert_eq!(packed.index_format, GpuIndexFormat::Uint16);
    assert_eq!(
        packed.index_bytes.len(),
        geometry.indices.len() * core::mem::size_of::<u16>()
    );
    assert_eq!(packed.aabb, geometry.aabb());
}

#[test]
fn geometry_packing_switches_to_u32_for_large_meshes() {
    let positions = (0..=u16::MAX as u32 + 1)
        .map(|index| Vec3::new(index as f32, 0.0, 0.0))
        .collect();
    let geometry = Geometry {
        positions,
        indices: vec![0, u16::MAX as u32, u16::MAX as u32 + 1],
        ..Geometry::new()
    };

    let packed = GpuScene::pack_geometry(&geometry).unwrap();
    assert_eq!(packed.index_format, GpuIndexFormat::Uint32);
    assert_eq!(packed.index_bytes.len(), 3 * core::mem::size_of::<u32>());
}

#[test]
fn resource_registry_validation_catches_bad_ids_and_textures() {
    let mut gpu_scene = GpuScene::new();

    assert_eq!(
        gpu_scene.register_material(
            MaterialId::default(),
            RendererMaterial::Unlit(UnlitMaterial::new())
        ),
        Err(ValidationError::InvalidId)
    );

    let bad_texture = Texture2D {
        width: 1,
        height: 1,
        format: TextureFormat::Rgba8Unorm,
        data: vec![255, 255, 255],
        mip_levels: 1,
        label: None,
    };
    assert_eq!(
        gpu_scene.register_texture2d(TextureId::new(1), &bad_texture, Sampler::new()),
        Err(ValidationError::OutOfRange)
    );

    let good_texture = Texture2D::new(
        1,
        1,
        TextureFormat::Rgba8UnormSrgb,
        vec![255, 255, 255, 255],
    )
    .unwrap();
    gpu_scene
        .register_texture2d(TextureId::new(2), &good_texture, Sampler::new())
        .unwrap();
    assert_eq!(gpu_scene.texture(TextureId::new(2)).unwrap().width, 1);

    assert_eq!(
        gpu_scene.register_light(
            LightId::default(),
            scenix_renderer::RendererLight::Ambient(Default::default())
        ),
        Err(ValidationError::InvalidId)
    );
}

#[test]
fn texture_and_sampler_formats_map_to_wgpu() {
    assert_eq!(
        to_wgpu_texture_format(TextureFormat::Rgba8UnormSrgb),
        Some(scenix_renderer::wgpu::TextureFormat::Rgba8UnormSrgb)
    );
    assert_eq!(
        to_wgpu_texture_format(TextureFormat::Depth32Float),
        Some(scenix_renderer::wgpu::TextureFormat::Depth32Float)
    );
    assert_eq!(
        to_wgpu_filter_mode(FilterMode::Nearest),
        scenix_renderer::wgpu::FilterMode::Nearest
    );
    assert_eq!(
        to_wgpu_address_mode(AddressMode::MirrorRepeat),
        scenix_renderer::wgpu::AddressMode::MirrorRepeat
    );
    assert_eq!(
        to_wgpu_compare(Some(CompareFunction::LessEqual)),
        Some(scenix_renderer::wgpu::CompareFunction::LessEqual)
    );
}

#[test]
fn sort_helpers_order_draws_for_depth_and_blending() {
    let mut opaque = vec![draw(10.0), draw(2.0), draw(5.0)];
    scenix_renderer::sort_opaque_front_to_back(&mut opaque);
    assert_eq!(
        opaque
            .iter()
            .map(|draw| draw.distance_to_camera as u32)
            .collect::<Vec<_>>(),
        vec![2, 5, 10]
    );

    let mut transparent = vec![draw(10.0), draw(2.0), draw(5.0)];
    scenix_renderer::sort_transparent_back_to_front(&mut transparent);
    assert_eq!(
        transparent
            .iter()
            .map(|draw| draw.distance_to_camera as u32)
            .collect::<Vec<_>>(),
        vec![10, 5, 2]
    );
}

#[test]
fn material_uniform_bytes_are_stable_size() {
    let pbr = PbrMaterial::new()
        .metallic_roughness(0.5, 0.25)
        .alpha_mode(AlphaMode::Mask(0.4));

    assert_eq!(
        pbr.to_uniform_bytes().len(),
        core::mem::size_of::<MaterialUniform>()
    );
    assert_eq!(pbr.pipeline_key().shader, scenix_material::ShaderKind::Pbr);
}

#[test]
fn stable_renderer_material_variants_register_and_emit_uniforms() {
    let mut gpu_scene = GpuScene::new();
    let physical = PhysicalMaterial::new();
    let mut toon = ToonMaterial::new();
    toon.color = scenix_core::Color::from_hex(0xFFCC66);
    let wireframe = WireframeMaterial::new();
    let normal = NormalMaterial::new();

    gpu_scene
        .register_physical_material(MaterialId::new(1), &physical)
        .unwrap();
    gpu_scene
        .register_toon_material(MaterialId::new(2), &toon)
        .unwrap();
    gpu_scene
        .register_wireframe_material(MaterialId::new(3), &wireframe)
        .unwrap();
    gpu_scene
        .register_normal_material(MaterialId::new(4), &normal)
        .unwrap();

    assert_eq!(gpu_scene.material_count(), 4);
    assert_eq!(
        physical.to_uniform_bytes().len(),
        core::mem::size_of::<MaterialUniform>()
    );
    assert_eq!(
        toon.to_uniform_bytes().len(),
        core::mem::size_of::<MaterialUniform>()
    );
    assert_eq!(
        wireframe.to_uniform_bytes().len(),
        core::mem::size_of::<MaterialUniform>()
    );
    assert_eq!(
        normal.to_uniform_bytes().len(),
        core::mem::size_of::<MaterialUniform>()
    );
}

#[test]
fn culling_reports_missing_registry_entries() {
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);
    let mut scene = SceneGraph::new();
    scene.add(renderer_mesh_node(mesh_id, material_id));
    scene.update_world_transforms();
    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0).position(Vec3::new(0.0, 0.0, 5.0));

    let error =
        scenix_renderer::collect_visible_draws(&scene, &GpuScene::new(), &camera).unwrap_err();
    assert_eq!(error, ValidationError::InvalidId);
}

#[test]
fn renderer_pipeline_key_is_hashable_and_exact() {
    let key = RendererPipelineKey::new(
        PipelineKey::default(),
        RenderPassKind::Geometry,
        scenix_renderer::wgpu::TextureFormat::Bgra8UnormSrgb,
        1,
        true,
    );
    let same = RendererPipelineKey::new(
        PipelineKey::default(),
        RenderPassKind::Geometry,
        scenix_renderer::wgpu::TextureFormat::Bgra8UnormSrgb,
        1,
        true,
    );

    assert_eq!(key, same);
}

#[cfg(feature = "serde")]
#[test]
fn renderer_config_serde_round_trips() {
    let config = RendererConfig::new(320, 180).vsync(false).hdr(true);
    let json = serde_json::to_string(&config).unwrap();
    let decoded: RendererConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, config);
}

fn renderer_mesh_node(mesh_id: MeshId, material_id: MaterialId) -> scenix_scene::SceneNode {
    scenix_scene::SceneNode::mesh("missing", mesh_id, material_id)
}
