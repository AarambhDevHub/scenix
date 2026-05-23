#![cfg(feature = "serde")]

use scenix_texture::{Sampler, Texture2D, TextureAtlas, TextureFormat, VideoTexture, mipmap};

#[test]
fn texture_types_round_trip_with_serde() {
    let texture = Texture2D::new(2, 2, TextureFormat::Rgba8UnormSrgb, vec![7; 16])
        .unwrap()
        .labeled("albedo");
    let sampler = Sampler::new().anisotropy(8);
    let mut atlas = TextureAtlas::new(8, 8);
    atlas.insert("tile", 4, 4).unwrap();
    let video = VideoTexture::new(texture.clone());

    assert_eq!(
        serde_json::from_str::<Texture2D>(&serde_json::to_string(&texture).unwrap()).unwrap(),
        texture
    );
    assert_eq!(
        serde_json::from_str::<Sampler>(&serde_json::to_string(&sampler).unwrap()).unwrap(),
        sampler
    );
    assert_eq!(
        serde_json::from_str::<TextureAtlas>(&serde_json::to_string(&atlas).unwrap()).unwrap(),
        atlas
    );
    assert_eq!(
        serde_json::from_str::<VideoTexture>(&serde_json::to_string(&video).unwrap()).unwrap(),
        video
    );

    assert_eq!(mipmap::generate(&texture.data, 2, 2).unwrap().len(), 2);
}
