use scenix::{
    PerspectiveCamera, Sampler, Texture2D, TextureAtlas, TextureFormat, Vec2, Vec3, mipmap,
};

fn main() {
    let pixels = vec![
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
    ];
    let mip_chain = mipmap::generate(&pixels, 2, 2).unwrap();
    let texture = Texture2D::from_mips(2, 2, TextureFormat::Rgba8UnormSrgb, mip_chain)
        .unwrap()
        .labeled("checker");
    let sampler = Sampler::new().anisotropy(4);

    let mut atlas = TextureAtlas::new(256, 256);
    let _albedo_rect = atlas.insert("checker", 64, 64).unwrap();
    let uv = atlas.uv("checker").unwrap();

    let camera = PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 1.0, 5.0))
        .target(Vec3::ZERO);
    let ray = camera.screen_to_ray(Vec2::ZERO);

    assert_eq!(texture.mip_levels, 2);
    assert_eq!(sampler.anisotropy, 4);
    assert!(uv.u1 > uv.u0);
    assert!(ray.direction.z < 0.0);
}
