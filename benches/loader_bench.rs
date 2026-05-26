use std::hint::black_box;
use std::time::Instant;

extern crate image as image_crate;

use image_crate::codecs::png::PngEncoder;
use image_crate::{ColorType, ImageEncoder};
use scenix_loader::{GltfLoader, image};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    println!("{name}: {iterations} iterations in {:?}", start.elapsed());
}

fn main() {
    let png = tiny_png();
    bench("image_decode_png_bytes", 100, || {
        black_box(image::load_bytes(black_box(&png)).unwrap());
    });

    let glb = tiny_glb();
    let loader = GltfLoader::new();
    bench("gltf_decode_glb_bytes", 100, || {
        black_box(loader.load_bytes(black_box(&glb), None).unwrap());
    });
}

fn tiny_png() -> Vec<u8> {
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);
    ImageEncoder::write_image(
        encoder,
        &[255, 255, 255, 255],
        1,
        1,
        ColorType::Rgba8.into(),
    )
    .unwrap();
    bytes
}

fn tiny_glb() -> Vec<u8> {
    let mut bin = Vec::new();
    for value in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
        bin.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u32, 1, 2] {
        bin.extend_from_slice(&index.to_le_bytes());
    }
    let mut json = br#"{
  "asset": {"version": "2.0"},
  "scene": 0,
  "scenes": [{"nodes": [0]}],
  "nodes": [{"mesh": 0}],
  "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 1}]}],
  "buffers": [{"byteLength": 48}],
  "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 36}, {"buffer": 0, "byteOffset": 36, "byteLength": 12}],
  "accessors": [{"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0, 0, 0], "max": [1, 1, 0]}, {"bufferView": 1, "componentType": 5125, "count": 3, "type": "SCALAR"}]
}"#
    .to_vec();
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let total_len = 12 + 8 + json.len() + 8 + bin.len();
    let mut glb = Vec::with_capacity(total_len);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2_u32.to_le_bytes());
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());
    glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&json);
    glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(&bin);
    glb
}
