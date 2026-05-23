use scenix_core::ValidationError;
use scenix_texture::{
    AddressMode, FilterMode, Sampler, Texture2D, Texture3D, TextureAtlas, TextureCube,
    TextureFormat, VideoTexture, mipmap,
};

#[test]
fn texture_formats_compute_expected_byte_lengths() {
    assert_eq!(TextureFormat::Rgba8Unorm.expected_2d_len(4, 4).unwrap(), 64);
    assert_eq!(
        TextureFormat::Rgba16Float.expected_2d_len(2, 2).unwrap(),
        32
    );
    assert_eq!(
        TextureFormat::Bc7RgbaUnorm.expected_2d_len(5, 5).unwrap(),
        64
    );
    assert!(TextureFormat::Astc4x4RgbaUnorm.is_compressed());
}

#[test]
fn texture_containers_validate_raw_byte_sizes_and_mips() {
    let base = vec![255; 4 * 4 * 4];
    let texture = Texture2D::new(4, 4, TextureFormat::Rgba8Unorm, base.clone()).unwrap();
    assert_eq!(texture.base_level_len().unwrap(), 64);

    assert_eq!(
        Texture2D::new(4, 4, TextureFormat::Rgba8Unorm, vec![0; 63]).unwrap_err(),
        ValidationError::OutOfRange
    );

    let mips = vec![base, vec![128; 2 * 2 * 4], vec![64; 4]];
    let mip_texture = Texture2D::from_mips(4, 4, TextureFormat::Rgba8Unorm, mips).unwrap();
    assert_eq!(mip_texture.mip_levels, 3);
    assert_eq!(mip_texture.data.len(), 84);

    let face = vec![0; 2 * 2 * 4];
    let cube = TextureCube::new(
        2,
        TextureFormat::Rgba8Unorm,
        [
            face.clone(),
            face.clone(),
            face.clone(),
            face.clone(),
            face.clone(),
            face,
        ],
    )
    .unwrap();
    assert_eq!(cube.size, 2);

    let volume = Texture3D::new(2, 2, 2, TextureFormat::Rgba8Unorm, vec![0; 32]).unwrap();
    assert_eq!(volume.depth, 2);
}

#[test]
fn sampler_clamps_anisotropy_and_keeps_modes() {
    let sampler = Sampler::new()
        .filters(FilterMode::Nearest, FilterMode::Nearest, FilterMode::Linear)
        .address_modes(
            AddressMode::Repeat,
            AddressMode::MirrorRepeat,
            AddressMode::ClampToEdge,
        )
        .anisotropy(99);

    assert_eq!(sampler.mag_filter, FilterMode::Nearest);
    assert_eq!(sampler.address_u, AddressMode::Repeat);
    assert_eq!(sampler.anisotropy, 16);
    assert_eq!(Sampler::new().anisotropy(0).anisotropy, 1);
}

#[test]
fn texture_atlas_packs_rects_and_reports_uvs() {
    let mut atlas = TextureAtlas::new(4, 4);
    let a = atlas.insert("a", 2, 2).unwrap();
    let b = atlas.insert("b", 2, 2).unwrap();
    let c = atlas.insert("c", 4, 2).unwrap();

    assert_eq!((a.x, a.y), (0, 0));
    assert_eq!((b.x, b.y), (2, 0));
    assert_eq!((c.x, c.y), (0, 2));
    assert_eq!(atlas.rect("b"), Some(b));

    let uv = atlas.uv("c").unwrap();
    assert_eq!((uv.u0, uv.v0, uv.u1, uv.v1), (0.0, 0.5, 1.0, 1.0));
    assert_eq!(atlas.entries().len(), 3);
    assert_eq!(
        atlas.insert("a", 1, 1).unwrap_err(),
        ValidationError::InvalidState
    );
    assert_eq!(
        atlas.insert("overflow", 1, 1).unwrap_err(),
        ValidationError::OutOfRange
    );
}

#[test]
fn mipmap_generation_returns_full_averaged_chain() {
    let data = vec![0, 0, 0, 0, 100, 0, 0, 100, 0, 100, 0, 100, 100, 100, 0, 200];
    let levels = mipmap::generate(&data, 2, 2).unwrap();

    assert_eq!(levels.len(), 2);
    assert_eq!(levels[0], data);
    assert_eq!(levels[1], vec![50, 50, 0, 100]);
}

#[test]
fn video_texture_updates_frames_and_dirty_state() {
    let texture = Texture2D::new(2, 2, TextureFormat::Rgba8Unorm, vec![0; 16]).unwrap();
    let mut video = VideoTexture::new(texture);
    video.mark_clean();

    assert!(!video.dirty);
    video.update_frame(&[1; 16]).unwrap();
    assert_eq!(video.frame_index, 1);
    assert!(video.dirty);
    assert_eq!(
        video.update_frame(&[1; 15]).unwrap_err(),
        ValidationError::OutOfRange
    );
}
