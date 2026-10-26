//! Boo, a little ghost that floats instead of standing.

use super::{blush, brows, eyes, mouth, Mood, Pose};
use crate::gfx::{ellipse, rgb, rgba, Canvas, P};
use tiny_skia::Transform;

pub fn draw(cv: &mut Canvas, pose: &Pose) {
    let f = pose.frame as usize;
    let bob = match pose.mood {
        Mood::Held | Mood::Sleep => 0.0,
        _ => [0.0, -1.5][f % 2],
    };
    cv.with(Transform::from_translate(0.0, bob - 6.0), |cv| {
        // Wavy hem that ripples between frames.
        let w = if f % 2 == 0 { 2.5 } else { -2.5 };
        let body = P::new()
            .m(22.0, 88.0)
            .c(16.0, 60.0, 22.0, 28.0, 50.0, 26.0)
            .c(78.0, 28.0, 84.0, 60.0, 78.0, 88.0)
            .q(73.0, 94.0 + w, 68.0, 88.0)
            .q(63.5, 82.0 - w, 59.0, 89.0)
            .q(54.5, 95.0 + w, 50.0, 89.0)
            .q(45.5, 83.0 - w, 41.0, 89.0)
            .q(36.5, 95.0 + w, 32.0, 88.0)
            .q(27.0, 82.0 - w, 22.0, 88.0)
            .z()
            .build();
        cv.fill_vgrad(&body, 26.0, rgba(0xffffff, 0.96), 92.0, rgba(0xd6d0f5, 0.9));
        cv.stroke(&body, rgb(0x9a90d6), 1.2);

        // Little arms.
        let arm_y = match pose.mood {
            Mood::Cheer | Mood::Held => 54.0,
            Mood::Love | Mood::Eat => 60.0,
            _ => 66.0,
        };
        for m in [false, true] {
            let a = P::mirrored(50.0, m)
                .m(23.0, 62.0)
                .q(13.0, arm_y - 2.0, 15.0, arm_y + 3.0)
                .q(19.0, arm_y + 6.0, 24.0, 70.0)
                .z()
                .build();
            cv.fill(&a, rgba(0xfbfaff, 0.96));
            cv.stroke(&a, rgb(0x9a90d6), 1.0);
        }

        cv.with(Transform::from_rotate_at(-25.0, 34.0, 38.0), |cv| {
            cv.fill(&ellipse(34.0, 38.0, 3.0, 6.0), rgba(0xffffff, 0.9))
        });
        blush(cv, 33.0, 67.0, 55.5, pose);
        eyes(cv, 40.0, 60.0, 48.0, 4.4, 5.8, rgb(0x4a3f7a), pose);
        brows(cv, 40.0, 60.0, 39.5, rgb(0x9a90d6), pose);
        mouth(cv, 50.0, 57.0, pose, false);
    });
}
