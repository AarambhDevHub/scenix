use std::fs;

use scenix::{AssetManager, ScenixError, export};

fn main() -> Result<(), ScenixError> {
    let path = std::env::temp_dir().join("scenix-example-export-scene.stl");
    fs::write(&path, ascii_stl()).expect("stl fixture");

    let mut manager = AssetManager::new();
    let package = manager.load_file(&path)?;
    let scene_json = export::scene_json_string(&package);
    let stl = export::stl_ascii_string(&package);

    println!("json={} stl_bytes={}", scene_json, stl.len());
    Ok(())
}

fn ascii_stl() -> &'static [u8] {
    b"solid tri
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 1 0 0
vertex 0 1 0
endloop
endfacet
endsolid tri
"
}
