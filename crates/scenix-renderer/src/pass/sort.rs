use core::cmp::Ordering;

use crate::DrawSubmission;

/// Sorts opaque draws front-to-back for early depth rejection.
pub fn sort_opaque_front_to_back(draws: &mut [DrawSubmission]) {
    draws.sort_by(|a, b| {
        a.render_order
            .cmp(&b.render_order)
            .then_with(|| compare_f32(a.distance_to_camera, b.distance_to_camera))
    });
}

/// Sorts transparent draws back-to-front for alpha blending.
pub fn sort_transparent_back_to_front(draws: &mut [DrawSubmission]) {
    draws.sort_by(|a, b| {
        a.render_order
            .cmp(&b.render_order)
            .then_with(|| compare_f32(b.distance_to_camera, a.distance_to_camera))
    });
}

fn compare_f32(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}
