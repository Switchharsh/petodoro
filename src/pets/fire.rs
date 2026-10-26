//! Hinote, a tiny fire spirit with a short fuse.

use super::{blush, brows_c, eyes, mouth, Mood, Pose};
use crate::gfx::{circle, rgb, rgba, Canvas, P};

pub fn draw(cv: &mut Canvas, pose: &Pose) {
    let f = pose.frame as usize;
    // Flicker: tips alternate every frame (every mood cycle is even).
    let k = if f % 2 == 0 { 1.0 } else { -1.0 };
    let tall = match pose.mood {
        Mood::Love | Mood::Cheer => 6.0, // flares up
        Mood::Sleep => -8.0,             // embers
        _ => 0.0,
    };
    let body = P::new()
        .m(24.0, 92.0)
        .c(12.0, 80.0, 14.0, 60.0, 26.0, 48.0)
        .q(28.0, 38.0, 31.0 + k, 26.0 - tall * 0.6)
        .q(38.0, 38.0, 42.0, 40.0)
        .q(44.0, 22.0, 50.0 - k, 8.0 - tall)
        .q(56.0, 22.0, 58.0, 40.0)
        .q(62.0, 38.0, 69.0 - k, 24.0 - tall * 0.6)
        .q(72.0, 38.0, 74.0, 48.0)
        .c(86.0, 60.0, 88.0, 80.0, 76.0, 92.0)
        .q(50.0, 99.0, 24.0, 92.0)
        .z()
        .build();
    cv.fill_vgrad(&body, 10.0, rgb(0xff4b2e), 92.0, rgb(0xffc93c));
    cv.stroke(&body, rgb(0xb8301a), 1.3);
    let core = P::new()
        .m(32.0, 90.0)
        .c(24.0, 78.0, 30.0, 62.0, 40.0, 56.0)
        .q(46.0, 44.0 - k, 50.0, 40.0)
        .q(54.0, 44.0 + k, 60.0, 56.0)
        .c(70.0, 62.0, 76.0, 78.0, 68.0, 90.0)
        .q(50.0, 95.0, 32.0, 90.0)
        .z()
        .build();
    cv.fill_vgrad(&core, 40.0, rgba(0xffe46b, 0.9), 92.0, rgba(0xfff6c8, 0.95));

    // Embers drifting up.
    for i in 0..3usize {
        let q = ((f + i) % 2) as f32 * 0.5 + i as f32 * 0.15;
        let x = [30.0, 70.0, 56.0][i] + k * 1.5;
        cv.fill(&circle(x, 22.0 - q * 14.0, 1.4), rgba(0xff8a3a, 0.9 - q * 0.6));
    }

    blush(cv, 34.0, 66.0, 76.0, pose);
    eyes(cv, 40.0, 60.0, 67.0, 4.4, 5.6, rgb(0x5a1a08), pose);
    brows_c(cv, 40.0, 60.0, 58.0, rgb(0x7a200c), pose);
    mouth(cv, 50.0, 75.0, pose, false);
}
