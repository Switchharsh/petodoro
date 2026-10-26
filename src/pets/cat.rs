//! Cats: Mochi the orange tabby and Kuro, a pure-black (AMOLED) cat with a neon outline.

use super::{blush, brows_c, eyes_ex, mouth_c, Mood, Pose, LASH};
use crate::gfx::{circle, ellipse, rgb, shade, with_alpha, Canvas, P};
use tiny_skia::{Color, Path};

pub struct Style {
    fur: u32,
    cream: u32,
    /// Tabby stripes (None = solid).
    stripe: Option<u32>,
    eyes: u32,
    /// Outline color; None = darker fur. Neon outlines also get a glow.
    neon: Option<u32>,
}

pub const MOCHI: Style = Style { fur: 0xf7b46c, cream: 0xfff1de, stripe: Some(0xdb823c), eyes: 0x69b34c, neon: None };
pub const KURO: Style = Style { fur: 0x000000, cream: 0x000000, stripe: None, eyes: 0xffd23f, neon: Some(0x6ff4ff) };

struct Ink {
    neon: Option<Color>,
}

impl Ink {
    /// Fill, then outline (with a soft glow for neon cats).
    fn body(&self, cv: &mut Canvas, path: &Path, fill: Color) {
        cv.fill(path, fill);
        match self.neon {
            Some(n) => {
                cv.stroke(path, with_alpha(n, 0.22), 3.6);
                cv.stroke(path, n, 1.0);
            }
            None => cv.stroke(path, shade(fill, 0.62), 1.1),
        }
    }
    fn line(&self, fallback: Color) -> Color {
        self.neon.unwrap_or(fallback)
    }
}

pub fn draw(cv: &mut Canvas, st: &Style, pose: &Pose) {
    let fur = rgb(st.fur);
    let cream = rgb(st.cream);
    let ink = Ink { neon: st.neon.map(rgb) };
    let line = ink.line(shade(fur, 0.6));
    let f = pose.frame as usize;

    // Tail behind the body, swishing.
    let s = match pose.mood {
        Mood::Idle | Mood::Focus | Mood::Walk => [0.0, 3.0][f % 2],
        Mood::Relax => [-3.0, 0.0, 3.0, 0.0][f % 4],
        Mood::Cheer | Mood::Love | Mood::Held | Mood::Dizzy => [-5.0, 0.0, 5.0, 0.0][f % 4],
        Mood::Sleep | Mood::Eat => 0.0,
    };
    let tail = if pose.mood == Mood::Sleep {
        // Curled around the paws.
        P::new().m(70.0, 88.0).c(84.0, 98.0, 60.0, 101.0, 40.0, 97.0).build()
    } else {
        P::new().m(70.0, 88.0).c(88.0, 92.0, 94.0 + s * 0.3, 70.0, 86.0 + s, 56.0).build()
    };
    if let Some(n) = ink.neon {
        cv.stroke(&tail, with_alpha(n, 0.22), 11.5);
        cv.stroke(&tail, n, 9.4);
    } else {
        cv.stroke(&tail, line, 9.4);
    }
    cv.stroke(&tail, fur, 7.2);
    if let Some(stripe) = st.stripe.map(rgb) {
        for k in 0..2 {
            let t = 0.45 + k as f32 * 0.22;
            let (x, y) = if pose.mood == Mood::Sleep {
                (70.0 - t * 35.0, 95.0)
            } else {
                (82.0 + t * 8.0 + s * t, 82.0 - t * 30.0)
            };
            cv.fill(&circle(x, y, 2.2), stripe);
        }
    }

    // Body.
    ink.body(cv, &ellipse(50.0, 80.0, 27.0, 17.0), fur);
    cv.fill(&ellipse(50.0, 84.0, 14.0, 10.0), cream);
    if let Some(stripe) = st.stripe.map(rgb) {
        for (x0, x1) in [(26.0, 30.0), (74.0, 70.0)] {
            cv.stroke(&P::new().m(x0, 76.0).l(x1, 78.0).build(), stripe, 2.0);
            cv.stroke(&P::new().m(x0 + 0.5, 82.0).l(x1, 83.0).build(), stripe, 2.0);
        }
    }

    // Ears, then head.
    for m in [false, true] {
        let ear = P::mirrored(50.0, m).m(24.0, 46.0).q(19.0, 24.0, 27.0, 18.0).q(37.0, 22.0, 45.0, 33.0).z().build();
        ink.body(cv, &ear, fur);
        let inner = P::mirrored(50.0, m).m(27.0, 39.0).q(25.0, 27.0, 28.5, 24.0).q(34.5, 26.5, 39.0, 32.0).z().build();
        cv.fill(&inner, if ink.neon.is_some() { rgb(0x8a2a5a) } else { rgb(0xffb3c1) });
    }
    ink.body(cv, &ellipse(50.0, 53.0, 28.0, 22.5), fur);
    cv.fill(&ellipse(50.0, 61.5, 14.0, 8.5), cream);
    if let Some(stripe) = st.stripe.map(rgb) {
        for (x0, x1) in [(50.0, 50.0), (44.0, 45.0), (56.0, 55.0)] {
            cv.stroke(&P::new().m(x0, 31.5).l(x1, 37.5).build(), stripe, 2.0);
        }
        for (x0, x1) in [(22.5, 27.0), (77.5, 73.0)] {
            cv.stroke(&P::new().m(x0, 50.0).l(x1, 51.5).build(), stripe, 1.8);
        }
    }

    blush(cv, 32.0, 68.0, 58.5, pose);
    let lash = ink.line(rgb(LASH));
    eyes_ex(cv, 39.5, 60.5, 51.5, 4.8, 6.0, rgb(st.eyes), lash, false, pose);
    brows_c(cv, 39.5, 60.5, 42.5, ink.line(shade(fur, 0.5)), pose);
    let nose = P::new().m(48.4, 57.6).l(51.6, 57.6).l(50.0, 59.4).z().build();
    cv.fill(&nose, rgb(0xff8fa3));
    let mline = ink.line(rgb(0x8a3a48));
    mouth_c(cv, 50.0, 59.6, pose, true, mline, rgb(0xb8404f));
    let whisker = ink.line(shade(fur, 0.5));
    for m in [false, true] {
        cv.stroke(&P::mirrored(50.0, m).m(31.0, 57.5).l(21.0, 55.5).build(), whisker, 0.7);
        cv.stroke(&P::mirrored(50.0, m).m(31.0, 60.5).l(21.5, 62.0).build(), whisker, 0.7);
    }

    // Front paws.
    let (py, spread) = match pose.mood {
        Mood::Held => (88.0, 2.0),
        Mood::Cheer => (if f % 2 == 0 { 70.0 } else { 74.0 }, 10.0),
        Mood::Eat => (68.0, -2.0),
        Mood::Walk => (if f % 2 == 0 { 93.0 } else { 95.5 }, 0.0),
        _ => (94.5, 0.0),
    };
    for x in [41.0 - spread, 59.0 + spread] {
        ink.body(cv, &ellipse(x, py, 5.2, 3.4), cream);
        let toe = ink.line(shade(cream, 0.7));
        cv.stroke(&P::new().m(x - 1.5, py + 0.6).l(x - 1.5, py + 2.2).build(), toe, 0.6);
        cv.stroke(&P::new().m(x + 1.5, py + 0.6).l(x + 1.5, py + 2.2).build(), toe, 0.6);
    }
}
