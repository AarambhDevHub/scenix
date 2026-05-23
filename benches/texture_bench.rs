use std::hint::black_box;
use std::time::Instant;

use scenix_texture::{TextureAtlas, mipmap};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn main() {
    let pixels = vec![128_u8; 256 * 256 * 4];
    bench("mipmap_generate_rgba8_256", 200, || {
        black_box(mipmap::generate(black_box(&pixels), 256, 256).unwrap());
    });

    bench("texture_atlas_pack_512_tiles", 500, || {
        let mut atlas = TextureAtlas::with_padding(1024, 1024, 1);
        for index in 0..512 {
            let name = format!("tile-{index}");
            atlas.insert(name, 16, 16).unwrap();
        }
        black_box(atlas);
    });
}
