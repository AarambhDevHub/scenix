use scenix_post::{
    BloomConfig, DofConfig, FogPostConfig, FxaaConfig, MotionBlurConfig, OutlineConfig, PostEffect,
    PostStack, SmaaConfig, SmaaQuality, SsaoConfig, TaaConfig, ToneMapper,
};

#[test]
fn effect_configs_clamp_to_supported_ranges() {
    let bloom = BloomConfig {
        threshold: -1.0,
        intensity: 99.0,
        radius: 99.0,
    }
    .normalized();
    assert_eq!(bloom.threshold, 0.0);
    assert_eq!(bloom.intensity, 16.0);
    assert_eq!(bloom.radius, 64.0);

    let ssao = SsaoConfig {
        radius: -1.0,
        intensity: 9.0,
        bias: 9.0,
    }
    .normalized();
    assert_eq!(ssao.radius, 0.0);
    assert_eq!(ssao.intensity, 4.0);
    assert_eq!(ssao.bias, 1.0);

    let fxaa = FxaaConfig {
        contrast_threshold: -1.0,
        relative_threshold: 2.0,
    }
    .normalized();
    assert_eq!(fxaa.contrast_threshold, 0.0);
    assert_eq!(fxaa.relative_threshold, 1.0);

    let taa = TaaConfig {
        feedback: 2.0,
        jitter: 9.0,
    }
    .normalized();
    assert_eq!(taa.feedback, 1.0);
    assert_eq!(taa.jitter, 2.0);

    let dof = DofConfig {
        focus_distance: 0.0,
        aperture: 99.0,
        max_blur_radius: 99.0,
    }
    .normalized();
    assert_eq!(dof.focus_distance, 0.001);
    assert_eq!(dof.aperture, 32.0);
    assert_eq!(dof.max_blur_radius, 64.0);

    let fog = FogPostConfig {
        color: [-1.0, 0.5, 2.0],
        density: 3.0,
    }
    .normalized();
    assert_eq!(fog.color, [0.0, 0.5, 1.0]);
    assert_eq!(fog.density, 1.0);

    let outline = OutlineConfig {
        color: [-1.0, 0.25, 2.0, 9.0],
        threshold: 2.0,
        thickness: 99.0,
    }
    .normalized();
    assert_eq!(outline.color, [0.0, 0.25, 1.0, 1.0]);
    assert_eq!(outline.threshold, 1.0);
    assert_eq!(outline.thickness, 16.0);

    let motion = MotionBlurConfig {
        strength: 2.0,
        sample_count: 99,
    }
    .normalized();
    assert_eq!(motion.strength, 1.0);
    assert_eq!(motion.sample_count, 32);
}

#[test]
fn post_stack_orders_removes_and_clears_effects() {
    let mut stack = PostStack::new()
        .with_ssao(SsaoConfig::default())
        .with_bloom(BloomConfig::default())
        .with_tonemap(ToneMapper::Aces)
        .with_fxaa(FxaaConfig::default())
        .with_taa(TaaConfig::default())
        .with_smaa(SmaaConfig {
            quality: SmaaQuality::Ultra,
        })
        .with_dof(DofConfig::default())
        .with_fog(FogPostConfig::default())
        .with_outline(OutlineConfig::default())
        .with_motion_blur(MotionBlurConfig::default());

    assert_eq!(stack.len(), 10);
    assert!(matches!(stack.effects()[0], PostEffect::Ssao(_)));
    assert_eq!(stack.effects()[2].kind_id(), 3);

    let removed = stack.remove(1).unwrap();
    assert!(matches!(removed, PostEffect::Bloom(_)));
    assert_eq!(stack.len(), 9);

    assert!(stack.remove(999).is_none());
    stack.clear();
    assert!(stack.is_empty());
}

#[cfg(feature = "serde")]
#[test]
fn post_effect_serde_round_trips() {
    let effect = PostEffect::Tonemap(ToneMapper::Exposure(1.25));
    let json = serde_json::to_string(&effect).unwrap();
    let decoded: PostEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, effect);
}
