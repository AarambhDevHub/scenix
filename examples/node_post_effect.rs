use scenix::{PostNodeEffect, WgslOptions, generate_post_wgsl, slots};

fn main() {
    // Bundled post presets all compile against the shared pass contract:
    // group 0 binding 0 source texture, binding 1 linear sampler, binding 2
    // the [f32; 8] PostParams block.
    let vignette = PostNodeEffect::vignette();
    assert!(vignette.validate().is_empty());
    let wgsl = generate_post_wgsl(&vignette, &WgslOptions::default()).unwrap();
    let source = wgsl.source();
    assert!(source.contains("@group(0) @binding(0)"));
    assert!(source.contains("PostParams"));
    println!("--- vignette node effect (WGSL) ---\n{source}\n");

    // Every preset is valid; parameter slots are stable shared constants.
    for (name, effect) in [
        ("passthrough", PostNodeEffect::passthrough()),
        ("vignette", PostNodeEffect::vignette()),
        ("desaturate", PostNodeEffect::desaturate()),
        ("posterize", PostNodeEffect::posterize()),
        ("pixelate", PostNodeEffect::pixelate()),
        ("color grade", PostNodeEffect::color_grade()),
        ("exposure +1.0", PostNodeEffect::exposure(1.0)),
    ] {
        assert!(effect.validate().is_empty(), "{name} must be valid");
        let program = generate_post_wgsl(&effect, &WgslOptions::default()).unwrap();
        assert!(program.source().contains("void main()"));
        println!("{name:14} -> {} bytes of WGSL", program.source().len());
    }

    // Slots document where host-driven sliders land inside PostParams.
    println!(
        "slots: vignette strength {}, grade saturation {}, grade contrast {}",
        slots::VIGNETTE_STRENGTH,
        slots::GRADE_SATURATION,
        slots::GRADE_CONTRAST
    );
}
