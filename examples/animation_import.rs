use scenix::{
    AnimationClipId, AssetId, AssetPackage, LoadedAnimationChannel, LoadedAnimationClip,
    LoadedAnimationInterpolation, LoadedAnimationProperty,
};

fn main() {
    let mut package = AssetPackage::empty(AssetId::new(1), "animation-import");
    package.animations.insert(
        AnimationClipId::new(1),
        LoadedAnimationClip {
            id: AnimationClipId::new(1),
            name: String::from("move"),
            duration: 1.0,
            channels: vec![LoadedAnimationChannel {
                node_index: 0,
                property: LoadedAnimationProperty::Translation,
                interpolation: LoadedAnimationInterpolation::Linear,
                times: vec![0.0, 1.0],
                output_components: 3,
            }],
        },
    );

    println!("animations={}", package.animations.len());
}
