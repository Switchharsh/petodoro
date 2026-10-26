//! Bolt, a little robot with an LED face. Dark metal, glowing cyan.

use super::{eyes_ex, mouth_c, Mood, Pose};
use crate::gfx::{circle, ellipse, rgb, rgba, rrect, shade, with_alpha, Canvas, P};

const LED: u32 = 0x4df0ff;

pub fn draw(cv: &mut Canvas, pose: &Pose) {
    let metal = rgb(0x4a5163);
    let dark = rgb(0x2c313d);
    let led = rgb(LED);
    let f = pose.frame as usize;

    // Wheels and body.
    for x in [38.0, 62.0] {
        cv.fs(&ellipse(x, 94.0, 7.0, 4.0), dark);
        cv.fill(&circle(x, 94.0, 1.6), rgb(0x8a93a8));
    }
    cv.fs(&rrect(32.0, 66.0, 36.0, 26.0, 9.0), metal);
    cv.fl(&rrect(40.0, 72.0, 20.0, 12.0, 3.0), rgb(0x10151c), shade(metal, 0.6));
    for (i, c) in [0xff5c8a, 0xffd84d, LED].iter().enumerate() {
        let on = (f + i) % 2 == 0 || pose.mood == Mood::Sleep;
        cv.fill(&circle(45.0 + i as f32 * 5.0, 78.0, 1.6), if on { rgb(*c) } else { rgba(*c, 0.25) });
    }

    // Arms: little pipes with round claws.
    let hy = match pose.mood {
        Mood::Cheer | Mood::Held => 56.0,
        Mood::Love | Mood::Eat => 64.0,
        Mood::Focus => 78.0,
        _ => 84.0,
    };
    for m in [false, true] {
        let arm = P::mirrored(50.0, m).m(33.0, 72.0).q(24.0, 74.0, 25.0, hy).build();
        cv.stroke(&arm, shade(metal, 0.6), 4.6);
        cv.stroke(&arm, rgb(0x6b7489), 3.0);
        let x = if m { 75.0 } else { 25.0 };
        cv.fs(&circle(x, hy, 3.2), rgb(0x6b7489));
    }

    // Antenna with a blinking light.
    cv.stroke(&P::new().m(50.0, 24.0).l(50.0, 12.0).build(), rgb(0x8a93a8), 1.6);
    let tip = if f % 2 == 0 || pose.mood == Mood::Cheer { rgb(0xff5c8a) } else { led };
    cv.fill(&circle(50.0, 10.0, 5.0), with_alpha(tip, 0.25));
    cv.fs(&circle(50.0, 10.0, 3.0), tip);

    // Head: a monitor with a dark screen.
    for x in [22.0, 78.0] {
        cv.fs(&circle(x, 45.0, 4.5), rgb(0x6b7489));
    }
    cv.fs(&rrect(23.0, 24.0, 54.0, 42.0, 13.0), metal);
    cv.stroke(&P::new().m(30.0, 27.5).q(50.0, 25.5, 70.0, 27.5).build(), rgba(0xffffff, 0.25), 1.4);
    let screen = rrect(29.0, 30.0, 42.0, 31.0, 9.0);
    cv.fill(&screen, rgb(0x070a0f));
    cv.stroke(&screen, with_alpha(led, 0.35), 1.0);

    // Pixel blush.
    if matches!(pose.mood, Mood::Love | Mood::Cheer | Mood::Eat | Mood::Relax) {
        for x in [32.5, 61.5] {
            for k in 0..3 {
                cv.fill(&rrect(x + k as f32 * 2.2, 50.0, 1.5, 1.5, 0.3), rgb(0xff6f9c));
            }
        }
    }
    eyes_ex(cv, 40.5, 59.5, 43.0, 3.6, 5.6, led, led, true, pose);
    mouth_c(cv, 50.0, 53.0, pose, false, led, with_alpha(led, 0.35));
}
