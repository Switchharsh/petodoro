//! Puru, a jelly slime with a little sprout.

use super::{blush, brows, eyes, mouth, Mood, Pose};
use crate::gfx::{circle, ellipse, rgb, rgba, Canvas, P};
use tiny_skia::Transform;

pub fn draw(cv: &mut Canvas, pose: &Pose) {
    let f = pose.frame as usize;
    // Extra jelly wobble on top of the shared body motion.
    let (sx, sy) = match pose.mood {
        Mood::Idle => [(1.0, 1.0), (1.04, 0.96)][f % 2],
        Mood::Relax => [(1.0, 1.0), (1.03, 0.97), (1.0, 1.0), (0.97, 1.03)][f % 4],
        Mood::Held => [(0.92, 1.1), (0.95, 1.06)][f % 2],
        _ => (1.0, 1.0),
    };
    let wobble = Transform::from_translate(50.0, 96.0).pre_scale(sx, sy).pre_translate(-50.0, -96.0);
    cv.with(wobble, |cv| {
        let body = P::new()
            .m(14.0, 95.0)
            .c(6.0, 95.0, 7.0, 80.0, 14.0, 70.0)
            .c(21.0, 50.0, 34.0, 33.0, 50.0, 32.0)
            .c(66.0, 33.0, 79.0, 50.0, 86.0, 70.0)
            .c(93.0, 80.0, 94.0, 95.0, 86.0, 95.0)
            .z()
            .build();
        cv.fill_vgrad(&body, 32.0, rgba(0xb4f5df, 0.96), 95.0, rgba(0x45c7a1, 0.96));
        cv.stroke(&body, rgb(0x2b977a), 1.3);

        // Glossy highlights and bubbles.
        cv.with(Transform::from_rotate_at(28.0, 30.0, 55.0), |cv| {
            cv.fill(&ellipse(30.0, 55.0, 3.6, 8.5), rgba(0xffffff, 0.65))
        });
        cv.fill(&circle(36.0, 43.0, 2.0), rgba(0xffffff, 0.8));
        cv.fill(&circle(70.0, 84.0, 2.4), rgba(0xffffff, 0.35));
        cv.fill(&circle(76.0, 77.0, 1.3), rgba(0xffffff, 0.35));

        // Sprout.
        let stem = P::new().m(50.0, 33.0).q(50.0, 25.0, 52.0, 20.5).build();
        cv.stroke(&stem, rgb(0x4f9e3c), 1.8);
        let leaf_l = P::new().m(51.5, 22.0).q(42.0, 13.0, 37.0, 19.0).q(43.0, 25.0, 51.5, 22.0).z().build();
        let leaf_r = P::new().m(51.5, 22.0).q(61.0, 11.0, 65.0, 17.0).q(59.0, 24.0, 51.5, 22.0).z().build();
        cv.fs(&leaf_l, rgb(0x82d45f));
        cv.fs(&leaf_r, rgb(0x82d45f));

        blush(cv, 31.0, 69.0, 71.0, pose);
        eyes(cv, 39.5, 60.5, 63.0, 4.3, 5.6, rgb(0x1f4a5a), pose);
        brows(cv, 39.5, 60.5, 54.5, rgb(0x2b977a), pose);
        mouth(cv, 50.0, 71.0, pose, false);
    });
}
