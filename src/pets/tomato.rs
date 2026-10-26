//! Tomo, the Petodoro mascot: a little tomato (pomodoro means tomato!).

use super::{blush, brows, eyes, mouth, Mood, Pose};
use crate::gfx::{ellipse, rgb, rgba, Canvas, P};
use tiny_skia::Transform;

pub fn draw(cv: &mut Canvas, pose: &Pose) {
    let f = pose.frame as usize;
    let red = rgb(0xf0473c);
    let leaf = rgb(0x5cbf4a);

    // Stubby feet.
    for x in [40.0, 60.0] {
        cv.fs(&ellipse(x, 95.5, 6.0, 3.2), rgb(0xc83228));
    }

    // Little arms.
    let hy = match pose.mood {
        Mood::Cheer | Mood::Held => 52.0,
        Mood::Love | Mood::Eat => 68.0,
        Mood::Dizzy => 62.0,
        _ => 80.0,
    };
    for m in [false, true] {
        let a =
            P::mirrored(50.0, m).m(24.0, 72.0).q(14.0, hy - 2.0, 15.0, hy).q(17.0, hy + 4.0, 23.0, 78.0).z().build();
        cv.fs(&a, red);
    }

    // Body: a squat tomato with soft lobes.
    let body = P::new()
        .m(50.0, 44.0)
        .c(64.0, 38.0, 82.0, 46.0, 82.0, 66.0)
        .c(82.0, 86.0, 66.0, 96.0, 50.0, 96.0)
        .c(34.0, 96.0, 18.0, 86.0, 18.0, 66.0)
        .c(18.0, 46.0, 36.0, 38.0, 50.0, 44.0)
        .z()
        .build();
    cv.fill_vgrad(&body, 40.0, rgb(0xff7a62), 96.0, rgb(0xd9322a));
    cv.stroke(&body, rgb(0xa8241c), 1.3);
    for (x0, x1) in [(38.0, 34.0), (62.0, 66.0)] {
        cv.stroke(&P::new().m(x0, 47.0).q(x1, 70.0, x0 + (x1 - x0) * 0.2, 92.0).build(), rgba(0xa8241c, 0.25), 1.2);
    }
    cv.with(Transform::from_rotate_at(-30.0, 30.0, 56.0), |cv| {
        cv.fill(&ellipse(30.0, 56.0, 4.0, 7.5), rgba(0xffffff, 0.55))
    });

    // Leafy crown and stem, wiggling a little.
    let wig = [0.0, 1.5, 0.0, -1.5][f % 4];
    cv.with(Transform::from_rotate_at(wig, 50.0, 44.0), |cv| {
        let crown = P::new()
            .m(50.0, 46.0)
            .q(40.0, 48.0, 31.0, 50.0)
            .q(38.0, 43.0, 44.0, 42.0)
            .q(38.0, 36.0, 35.0, 31.0)
            .q(45.0, 34.0, 50.0, 39.0)
            .q(55.0, 34.0, 65.0, 31.0)
            .q(62.0, 36.0, 56.0, 42.0)
            .q(62.0, 43.0, 69.0, 50.0)
            .q(60.0, 48.0, 50.0, 46.0)
            .z()
            .build();
        cv.fs(&crown, leaf);
        cv.stroke(&P::new().m(50.0, 40.0).q(50.0, 33.0, 54.0, 28.0).build(), rgb(0x3f8f35), 2.4);
    });

    blush(cv, 33.0, 67.0, 76.0, pose);
    eyes(cv, 40.0, 60.0, 68.0, 4.6, 5.8, rgb(0x5a2418), pose);
    brows(cv, 40.0, 60.0, 59.5, red, pose);
    mouth(cv, 50.0, 77.5, pose, false);
}
