use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

extern crate image as image_crate;

use scenix_core::{CameraId, LoadError, MaterialId, ScenixError, TextureId};
use scenix_loader::{AssetCache, GltfLoader, LoadedCamera, hdr, image, ktx2, obj, stl};
use scenix_scene::NodeKind;
use scenix_texture::TextureFormat;

#[test]
fn generated_gltf_loads_scene_mesh_material_texture_sampler_and_camera() {
    let dir = temp_dir("gltf");
    write_triangle_gltf(&dir);

    let asset = GltfLoader::new()
        .load_file(dir.join("triangle.gltf"))
        .unwrap();
    assert_eq!(asset.meshes.len(), 1);
    assert!(asset.materials.contains_key(&MaterialId::new(1)));
    assert_eq!(asset.textures[&TextureId::new(1)].width, 1);
    assert_eq!(asset.samplers[&TextureId::new(1)].anisotropy, 1);
    assert!(matches!(
        asset.cameras[&CameraId::new(1)],
        LoadedCamera::Perspective(_)
    ));

    let mesh_node = asset.scene.iter_depth_first().find(|id| {
        asset
            .scene
            .get(*id)
            .is_some_and(|node| matches!(node.kind, NodeKind::Mesh { .. }))
    });
    assert!(mesh_node.is_some());
    assert_eq!(asset.scene.roots().len(), 2);
}

#[test]
fn generated_glb_bytes_load() {
    let glb = make_triangle_glb();
    let asset = GltfLoader::new().load_bytes(&glb, None).unwrap();
    assert_eq!(asset.meshes.len(), 1);
    assert_eq!(asset.scene.roots().len(), 1);
}

#[test]
fn obj_loader_keeps_geometry_and_material_texture_metadata() {
    let dir = temp_dir("obj");
    fs::write(
        dir.join("mesh.obj"),
        "mtllib mesh.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nvn 0 0 1\nusemtl mat\nf 1/1/1 2/2/1 3/3/1\n",
    )
    .unwrap();
    fs::write(dir.join("mesh.mtl"), "newmtl mat\nmap_Kd diffuse.png\n").unwrap();

    let asset = obj::load_with_materials(dir.join("mesh.obj")).unwrap();
    assert_eq!(asset.geometries.len(), 1);
    assert_eq!(asset.geometries[0].triangle_count(), 1);
    assert_eq!(asset.materials[0].name, "mat");
    assert_eq!(asset.materials[0].texture_id, Some(TextureId::new(1)));
}

#[test]
fn stl_loader_reads_ascii_and_binary_triangles() {
    let ascii = b"solid tri
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 1 0 0
vertex 0 1 0
endloop
endfacet
endsolid tri
";
    let geometry = stl::load_bytes(ascii).unwrap();
    assert_eq!(geometry.triangle_count(), 1);

    let binary = make_binary_stl();
    let geometry = stl::load_bytes(&binary).unwrap();
    assert_eq!(geometry.triangle_count(), 1);
}

#[test]
fn image_loader_decodes_png_jpeg_and_webp_to_srgb_rgba8() {
    let dir = temp_dir("images");
    let png = dir.join("pixel.png");
    let jpg = dir.join("pixel.jpg");
    let webp = dir.join("pixel.webp");

    image_crate::save_buffer_with_format(
        &png,
        &[255, 0, 0, 255],
        1,
        1,
        image_crate::ColorType::Rgba8,
        image_crate::ImageFormat::Png,
    )
    .unwrap();
    image_crate::save_buffer_with_format(
        &jpg,
        &[0, 255, 0],
        1,
        1,
        image_crate::ColorType::Rgb8,
        image_crate::ImageFormat::Jpeg,
    )
    .unwrap();
    image_crate::save_buffer_with_format(
        &webp,
        &[0, 0, 255, 255],
        1,
        1,
        image_crate::ColorType::Rgba8,
        image_crate::ImageFormat::WebP,
    )
    .unwrap();

    for path in [png, jpg, webp] {
        let texture = image::load(path).unwrap();
        assert_eq!(texture.format, TextureFormat::Rgba8UnormSrgb);
        assert_eq!(texture.data.len(), 4);
    }
}

#[test]
fn ktx2_loader_maps_supported_metadata_and_sizes() {
    let bytes = make_ktx2_rgba8();
    let texture = ktx2::load_bytes(&bytes).unwrap();
    assert_eq!(texture.width, 1);
    assert_eq!(texture.height, 1);
    assert_eq!(texture.format, TextureFormat::Rgba8Unorm);
    assert_eq!(texture.data, vec![1, 2, 3, 4]);
}

#[test]
fn hdr_loader_outputs_cube_dimensions_and_face_byte_sizes() {
    let dir = temp_dir("hdr");
    let png = dir.join("source.png");
    image_crate::save_buffer_with_format(
        &png,
        &[32, 64, 128, 255],
        1,
        1,
        image_crate::ColorType::Rgba8,
        image_crate::ImageFormat::Png,
    )
    .unwrap();

    let cube = hdr::load_with_size(&png, 4).unwrap();
    assert_eq!(cube.size, 4);
    assert_eq!(cube.faces[0].len(), 4 * 4 * 4);
}

#[test]
fn asset_cache_deduplicates_invalidates_and_reports_missing_files() {
    let dir = temp_dir("cache");
    let path = dir.join("asset.txt");
    fs::write(&path, "asset").unwrap();

    let mut cache = AssetCache::<String>::new();
    let first = cache
        .get_or_load(&path, |path| Ok(fs::read_to_string(path).unwrap()))
        .unwrap();
    let second = cache
        .get_or_load(&path, |_| Ok(String::from("different")))
        .unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert!(cache.contains(&path));
    assert!(cache.invalidate(&path));
    assert_eq!(cache.len(), 0);

    let missing = cache.get_or_load(dir.join("missing.txt"), |_| Ok(String::new()));
    assert_eq!(missing.unwrap_err(), ScenixError::Load(LoadError::NotFound));
}

#[cfg(feature = "serde")]
#[test]
fn loader_metadata_serde_round_trips() {
    let options = scenix_loader::LoaderOptions::default();
    let json = serde_json::to_string(&options).unwrap();
    let decoded: scenix_loader::LoaderOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, options);
}

fn write_triangle_gltf(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("triangle.bin"), triangle_bin()).unwrap();
    image_crate::save_buffer_with_format(
        dir.join("pixel.png"),
        &[255, 255, 255, 255],
        1,
        1,
        image_crate::ColorType::Rgba8,
        image_crate::ImageFormat::Png,
    )
    .unwrap();
    fs::write(dir.join("triangle.gltf"), triangle_gltf_json(true)).unwrap();
}

fn triangle_gltf_json(with_texture: bool) -> String {
    let texture_json = if with_texture {
        r#",
  "images": [{"uri": "pixel.png"}],
  "samplers": [{"magFilter": 9729, "minFilter": 9729, "wrapS": 33071, "wrapT": 33071}],
  "textures": [{"source": 0, "sampler": 0}],
  "materials": [{"name": "mat", "pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0.0, "roughnessFactor": 0.8}}]"#
    } else {
        r#",
  "materials": [{"name": "mat", "pbrMetallicRoughness": {"baseColorFactor": [1.0, 0.0, 0.0, 1.0]}}]"#
    };
    format!(
        r#"{{
  "asset": {{"version": "2.0"}},
  "scene": 0,
  "scenes": [{{"nodes": [0, 1]}}],
  "nodes": [
    {{"name": "tri", "mesh": 0, "translation": [1.0, 2.0, 3.0]}},
    {{"name": "camera", "camera": 0}}
  ],
  "cameras": [{{"type": "perspective", "perspective": {{"yfov": 1.0471976, "znear": 0.1, "zfar": 100.0, "aspectRatio": 1.0}}}}],
  "meshes": [{{"primitives": [{{"attributes": {{"POSITION": 0}}, "indices": 1, "material": 0}}]}}],
  "buffers": [{{"uri": "triangle.bin", "byteLength": 48}}],
  "bufferViews": [
    {{"buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962}},
    {{"buffer": 0, "byteOffset": 36, "byteLength": 12, "target": 34963}}
  ],
  "accessors": [
    {{"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0.0, 0.0, 0.0], "max": [1.0, 1.0, 0.0]}},
    {{"bufferView": 1, "componentType": 5125, "count": 3, "type": "SCALAR"}}
  ]{texture_json}
}}"#
    )
}

fn triangle_bin() -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u32, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes
}

fn make_triangle_glb() -> Vec<u8> {
    let mut json = triangle_gltf_json(false).replace("\"uri\": \"triangle.bin\", ", "");
    json = json.replace(
        "\"scenes\": [{\"nodes\": [0, 1]}]",
        "\"scenes\": [{\"nodes\": [0]}]",
    );
    json = json.replace(
        r#",
    {"name": "camera", "camera": 0}"#,
        "",
    );
    json = json.replace(
        r#",
  "cameras": [{"type": "perspective", "perspective": {"yfov": 1.0471976, "znear": 0.1, "zfar": 100.0, "aspectRatio": 1.0}}]"#,
        "",
    );
    let mut json_bytes = json.into_bytes();
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }
    let mut bin = triangle_bin();
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }

    let total_len = 12 + 8 + json_bytes.len() + 8 + bin.len();
    let mut glb = Vec::with_capacity(total_len);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2_u32.to_le_bytes());
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(&bin);
    glb
}

fn make_binary_stl() -> Vec<u8> {
    let mut bytes = vec![0_u8; 80];
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    for value in [
        0.0_f32, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes
}

fn make_ktx2_rgba8() -> Vec<u8> {
    let mut bytes = vec![0_u8; 104];
    bytes[0..12].copy_from_slice(&[
        0xAB, b'K', b'T', b'X', b' ', b'2', b'0', 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ]);
    write_u32(&mut bytes, 12, 37);
    write_u32(&mut bytes, 16, 1);
    write_u32(&mut bytes, 20, 1);
    write_u32(&mut bytes, 24, 1);
    write_u32(&mut bytes, 36, 1);
    write_u32(&mut bytes, 40, 1);
    write_u64(&mut bytes, 80, 104);
    write_u64(&mut bytes, 88, 4);
    write_u64(&mut bytes, 96, 4);
    bytes.extend_from_slice(&[1, 2, 3, 4]);
    bytes
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "scenix-loader-{name}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
