//! Pochi, a shiba inu with endless energy.

use super::{blush, brows, eyes, mouth, Mood, Pose};
use crate::gfx::{circle, ellipse, rgb, shade, Canvas, P};

pub fn draw(cv: &mut Canvas, pose: &Pose) {
    let fur = rgb(0xeba35a);
    let cream = rgb(0xfff4e2);
    let f = pose.frame as usize;

    // Curly tail, wagging.
    let wag = match pose.mood {
        Mood::Sleep | Mood::Focus => 0.0,
        Mood::Idle | Mood::Walk | Mood::Relax => [-4.0, 4.0][f % 2],
        _ => [-7.0, 0.0, 7.0, 0.0][f % 4],
    };
    cv.with(tiny_skia::Transform::from_rotate_at(wag, 72.0, 80.0), |cv| {
        let ring = circle(80.0, 66.0, 8.0);
        cv.stroke(&ring, shade(fur, 0.62), 8.4);
        cv.stroke(&ring, fur, 6.4);
        cv.fill(&circle(80.0, 66.0, 3.2), cream);
    });

    // Body with cream chest.
    cv.fs(&ellipse(50.0, 80.0, 25.0, 17.0), fur);
    cv.fill(&ellipse(50.0, 82.0, 13.0, 12.0), cream);

    // Red collar with a gold tag.
    cv.stroke(&P::new().m(32.0, 69.0).q(50.0, 76.0, 68.0, 69.0).build(), rgb(0xd8384a), 3.2);
    cv.fs(&circle(50.0, 74.5, 2.4), rgb(0xffcf4a));

    // Ears: upright and rounded.
    for m in [false, true] {
        let ear = P::mirrored(50.0, m).m(27.0, 42.0).q(23.0, 22.0, 32.0, 17.0).q(42.0, 24.0, 44.0, 34.0).z().build();
        cv.fs(&ear, fur);
        let inner = P::mirrored(50.0, m).m(29.5, 37.0).q(28.0, 26.0, 32.5, 22.5).q(38.0, 27.0, 39.5, 33.0).z().build();
        cv.fill(&inner, cream);
    }

    // Head with the shiba "urajiro" markings.
    cv.fs(&ellipse(50.0, 52.0, 27.0, 22.0), fur);
    let mask = P::new()
        .m(26.0, 56.0)
        .q(30.0, 49.0, 38.0, 53.0)
        .q(44.0, 56.0, 50.0, 55.0)
        .q(56.0, 56.0, 62.0, 53.0)
        .q(70.0, 49.0, 74.0, 56.0)
        .q(70.0, 72.0, 50.0, 73.0)
        .q(30.0, 72.0, 26.0, 56.0)
        .z()
        .build();
    cv.fill(&mask, cream);
    cv.fill(&ellipse(38.5, 41.5, 2.6, 1.7), cream);
    cv.fill(&ellipse(61.5, 41.5, 2.6, 1.7), cream);

    blush(cv, 31.0, 69.0, 58.0, pose);
    eyes(cv, 39.5, 60.5, 49.5, 4.4, 5.6, rgb(0x5a3a22), pose);
    brows(cv, 39.5, 60.5, 41.5, fur, pose);
    cv.fs(&ellipse(50.0, 57.5, 3.2, 2.2), rgb(0x2a1e1a));
    cv.fill(&circle(49.0, 56.9, 0.8), rgb(0xffffff));
    mouth(cv, 50.0, 61.0, pose, true);

    // Front paws.
    let (py, spread) = match pose.mood {
        Mood::Held => (88.0, 2.0),
        Mood::Cheer => (if f % 2 == 0 { 72.0 } else { 76.0 }, 9.0),
        Mood::Eat => (70.0, -2.0),
        Mood::Walk => (if f % 2 == 0 { 93.0 } else { 95.5 }, 0.0),
        _ => (94.5, 0.0),
    };
    for x in [41.0 - spread, 59.0 + spread] {
        cv.fs(&ellipse(x, py, 5.0, 3.4), cream);
    }
}
