# Scenix v1.4.0 — Animation Runtime

Scenix `1.4.0` adds a clip-based animation runtime on top of the existing
Animato tween/spring bridge, bringing scenix to parity with Three.js's
`AnimationClip` / `AnimationAction` / `AnimationMixer` model while keeping
scenix's typed-ID, deterministic, renderer-agnostic discipline.

## Highlights

- Workspace crates bumped to `1.4.0`; Animato bumped to `1.7.0` (the previous
  `1.6.0` release gate is now resolved).
- New runtime: `AnimationClip`, `AnimationAction`, `AnimationMixer`,
  `PropertyBinding`, `LoopMode`, `AnimationMarker` / `AnimationEvent`,
  crossfade, additive blending, deterministic per-tick sampling.
- New keyframe tracks (`KeyframeScalar` / `Vec3` / `Quat` / `Color` / `Bool`)
  with `Linear`, `Step`, and `CubicSpline` interpolation.
- New light (`LightAnimator`) and morph-weight (`MorphWeightAnimator`) targets.
- Skeletal animation: `scenix-mesh` `SkinningData` + `cpu_skin` / `apply_morph`
  CPU fallback; `scenix-renderer` `register_skin` / `update_bone_matrices` /
  `register_morph_targets` / `update_morph_weights` GPU hooks + `SKINNING_WGSL`.
- Retargeting helpers (`RetargetMap`) and animation / pose debug helpers
  (`AnimationPathHelper`, `PoseHelper`).
- Loader now decodes animation accessor output bytes into
  `LoadedAnimationChannel::output`.
- Facade `clip_from_loaded` bridges imported clips to the runtime.

## Install

```toml
[dependencies]
scenix = "1.4"
```

Animation runtime:

```toml
[dependencies]
scenix = { version = "1.4", features = ["animato"] }
```

Animation runtime with imported clips:

```toml
[dependencies]
scenix = { version = "1.4", features = ["animato", "loader"] }
```

## Code Example

```rust
use scenix::{
    AnimationClip, AnimationMixer, ClipChannel, ClipTrack, KeyframeInterpolation,
    KeyframeVec3, LoopMode, NodeProperty, PropertyBinding, SceneGraph, SceneNode, Vec3,
};

let mut scene = SceneGraph::new();
let node = scene.add(SceneNode::new("mover"));

let clip = AnimationClip::empty("move").with_channel(ClipChannel {
    binding: PropertyBinding::Node { node_id: node, property: NodeProperty::Translation },
    track: ClipTrack::Vec3(KeyframeVec3::new(
        vec![0.0, 1.0],
        vec![Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0)],
        KeyframeInterpolation::Linear,
    )),
});

let mut mixer = AnimationMixer::new();
let clip_index = mixer.add_clip(clip);
let action = mixer.add_action(clip_index);
mixer.action_mut(action).unwrap().set_loop_mode(LoopMode::REPEAT);
mixer.action_mut(action).unwrap().play(0.0);

// Each frame:
// mixer.tick(dt, &mut scene, &mut cameras, &mut materials,
//            &mut lights, &mut skeletons, &mut morphs)
```

## Migration Notes

- `ScenixAnimationDriver::tick` now takes `lights` and `morphs` stores in
  addition to `skeletons`. Pass empty stores if unused.
- `LoadedAnimationChannel` gained an `output: Vec<f32>` field; update struct
  literals accordingly.
- Animato `1.5.0` → `1.7.0` is a drop-in for the `std` / `tween` / `spring` /
  `serde` feature set; no scenix-side changes required.

## Known Limitations

- Cubic-spline interpolation is fully implemented for scalar tracks; vec3 /
  quat / color cubic channels fall back to linear sampling in v1.4.
- Additive blending accumulates weighted deltas into the same normal
  accumulator for v1.4; full base-clip-relative additive is planned.
- GPU skinning ships the registry, upload hooks, and `SKINNING_WGSL` snippet;
  full shader-pipeline wiring behind a `SKINNING` define is additive in a
  follow-up patch.

## Links

- Website and demo: `https://aarambhdevhub.github.io/scenix/`
- Documentation: `https://docs.rs/scenix`
- Crates: `https://crates.io/crates/scenix`
