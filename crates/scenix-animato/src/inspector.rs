use scenix_core::{Inspectable, InspectorId, InspectorItem, InspectorSnapshot};

use crate::AnimationMixer;

impl Inspectable for AnimationMixer {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        let mut item = InspectorItem::new(InspectorId(1), "Animation Mixer", "animation")
            .field("clips", self.clip_count())
            .field("actions", self.action_count())
            .field("global_time_scale", self.global_time_scale());
        for index in 0..self.clip_count() {
            if let Some(clip) = self.clip(index) {
                item.children.push(
                    InspectorItem::new(InspectorId(index as u64 + 2), &clip.name, "animation_clip")
                        .field("duration", clip.duration)
                        .field("channels", clip.channels.len())
                        .field("markers", clip.markers.len()),
                );
            }
        }
        snapshot.push(item);
    }
}
