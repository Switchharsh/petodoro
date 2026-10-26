//! Mochi, an orange tabby cat.

use super::{blush, brows, eyes, mouth, Mood, Pose};
use crate::gfx::{circle, ellipse, rgb, shade, Canvas, P};

pub fn draw(cv: &mut Canvas, pose: &Pose) {
    let fur = rgb(0xf7b46c);
    let cream = rgb(0xfff1de);
    let stripe = rgb(0xdb823c);
    let line = shade(fur, 0.6);
    let f = pose.frame as usize;

    // Tail behind the body, swishing.
    let s = match pose.mood {
        Mood::Idle | Mood::Focus => [0.0, 3.0][f % 2],
        Mood::Relax => [-3.0, 0.0, 3.0, 0.0][f % 4],
        Mood::Cheer | Mood::Love | Mood::Held => [-5.0, 0.0, 5.0, 0.0][f % 4],
        Mood::Sleep => 0.0,
    };
    let tail = if pose.mood == Mood::Sleep {
        // Curled around the paws.
        P::new().m(70.0, 88.0).c(84.0, 98.0, 60.0, 101.0, 40.0, 97.0).build()
    } else {
        P::new().m(70.0, 88.0).c(88.0, 92.0, 94.0 + s * 0.3, 70.0, 86.0 + s, 56.0).build()
    };
    cv.stroke(&tail, line, 9.4);
    cv.stroke(&tail, fur, 7.2);
    for k in 0..2 {
        let t = 0.45 + k as f32 * 0.22;
        let (x, y) =
            if pose.mood == Mood::Sleep { (70.0 - t * 35.0, 95.0) } else { (82.0 + t * 8.0 + s * t, 82.0 - t * 30.0) };
        cv.fill(&circle(x, y, 2.2), stripe);
    }

    // Body.
    cv.fs(&ellipse(50.0, 80.0, 27.0, 17.0), fur);
    cv.fill(&ellipse(50.0, 84.0, 14.0, 10.0), cream);
    for (x0, x1) in [(26.0, 30.0), (74.0, 70.0)] {
        cv.stroke(&P::new().m(x0, 76.0).l(x1, 78.0).build(), stripe, 2.0);
        cv.stroke(&P::new().m(x0 + 0.5, 82.0).l(x1, 83.0).build(), stripe, 2.0);
    }

    // Ears, then head.
    for m in [false, true] {
        let ear = P::mirrored(50.0, m).m(24.0, 46.0).q(19.0, 24.0, 27.0, 18.0).q(37.0, 22.0, 45.0, 33.0).z().build();
        cv.fs(&ear, fur);
        let inner = P::mirrored(50.0, m).m(27.0, 39.0).q(25.0, 27.0, 28.5, 24.0).q(34.5, 26.5, 39.0, 32.0).z().build();
        cv.fill(&inner, rgb(0xffb3c1));
    }
    cv.fs(&ellipse(50.0, 53.0, 28.0, 22.5), fur);
    cv.fill(&ellipse(50.0, 61.5, 14.0, 8.5), cream);
    for (x0, x1) in [(50.0, 50.0), (44.0, 45.0), (56.0, 55.0)] {
        cv.stroke(&P::new().m(x0, 31.5).l(x1, 37.5).build(), stripe, 2.0);
    }
    for (x0, x1) in [(22.5, 27.0), (77.5, 73.0)] {
        cv.stroke(&P::new().m(x0, 50.0).l(x1, 51.5).build(), stripe, 1.8);
    }

    blush(cv, 32.0, 68.0, 58.5, pose);
    eyes(cv, 39.5, 60.5, 51.5, 4.8, 6.0, rgb(0x69b34c), pose);
    brows(cv, 39.5, 60.5, 42.5, fur, pose);
    let nose = P::new().m(48.4, 57.6).l(51.6, 57.6).l(50.0, 59.4).z().build();
    cv.fill(&nose, rgb(0xff8fa3));
    mouth(cv, 50.0, 59.6, pose, true);
    for m in [false, true] {
        let w1 = P::mirrored(50.0, m).m(31.0, 57.5).l(21.0, 55.5).build();
        let w2 = P::mirrored(50.0, m).m(31.0, 60.5).l(21.5, 62.0).build();
        cv.stroke(&w1, shade(fur, 0.5), 0.7);
        cv.stroke(&w2, shade(fur, 0.5), 0.7);
    }

    // Front paws.
    let (py, spread) = match pose.mood {
        Mood::Held => (88.0, 2.0),
        Mood::Cheer => (if f % 2 == 0 { 70.0 } else { 74.0 }, 10.0),
        _ => (94.5, 0.0),
    };
    for x in [41.0 - spread, 59.0 + spread] {
        cv.fs(&ellipse(x, py, 5.2, 3.4), cream);
        cv.stroke(&P::new().m(x - 1.5, py + 0.6).l(x - 1.5, py + 2.2).build(), shade(cream, 0.7), 0.6);
        cv.stroke(&P::new().m(x + 1.5, py + 0.6).l(x + 1.5, py + 2.2).build(), shade(cream, 0.7), 0.6);
    }
}
