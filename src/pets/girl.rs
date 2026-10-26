//! Chibi anime girls, built from a shared rig with per-character hair and outfit.

use super::{blush, brows, eyes, mouth, Mood, Pose};
use crate::gfx::{circle, ellipse, rgb, rgba, rrect, shade, Canvas, P};
use tiny_skia::Color;

#[derive(Clone, Copy, PartialEq)]
pub enum Hair {
    TwinTails,
    Hood,
    Ponytail,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Outfit {
    Sailor,
    Hoodie,
    Maid,
}

pub struct Style {
    pub hair: u32,
    pub eyes: u32,
    pub hair_kind: Hair,
    pub outfit: Outfit,
    /// Ribbons / bows / book cover.
    pub accent: u32,
    /// Hoodie color (Hood/Hoodie only).
    pub cloth: u32,
}

pub const SAKURA: Style = Style {
    hair: 0xf7a8c4,
    eyes: 0xa64fd6,
    hair_kind: Hair::TwinTails,
    outfit: Outfit::Sailor,
    accent: 0xe8475f,
    cloth: 0x2f3e74,
};
pub const YUKI: Style = Style {
    hair: 0xe4e6f7,
    eyes: 0x3aa0e8,
    hair_kind: Hair::Hood,
    outfit: Outfit::Hoodie,
    accent: 0x6c8cff,
    cloth: 0xa9cdfc,
};
pub const HIKARI: Style = Style {
    hair: 0xffd36e,
    eyes: 0x2fb36b,
    hair_kind: Hair::Ponytail,
    outfit: Outfit::Maid,
    accent: 0xe23d4f,
    cloth: 0x2b2b3a,
};

const SKIN: u32 = 0xffe7da;
const WHITE: u32 = 0xfbfbff;

fn sway(pose: &Pose) -> f32 {
    let f = pose.frame as usize;
    match pose.mood {
        Mood::Held => [3.0, 1.5, -1.5, -3.0, -1.5, 1.5][f % 6],
        Mood::Cheer => [0.0, -1.0, -2.5, -3.5, -3.5, -2.5, -1.0, 0.0][f % 8],
        Mood::Relax => [-1.5, 0.0, 1.5, 0.0][f % 4],
        Mood::Idle => [0.0, 0.8][f % 2],
        _ => 0.0,
    }
}

pub fn draw(cv: &mut Canvas, st: &Style, pose: &Pose) {
    let hair = rgb(st.hair);
    let hair_back = shade(hair, 0.84);
    let s = sway(pose);

    // 1. Everything behind the head.
    match st.hair_kind {
        Hair::TwinTails => {
            for m in [false, true] {
                let tail = P::mirrored(50.0, m)
                    .m(31.0, 22.0)
                    .c(13.0, 24.0, 6.0, 50.0, 11.0 + s, 72.0)
                    .q(13.0 + s, 85.0, 19.0 + s, 89.0)
                    .c(21.0 + s, 76.0, 24.0, 56.0, 36.0, 32.0)
                    .z()
                    .build();
                cv.fs(&tail, hair_back);
            }
            cv.fs(&ellipse(50.0, 40.0, 28.0, 26.0), hair_back);
        }
        Hair::Hood => {
            for m in [false, true] {
                let lock = P::mirrored(50.0, m)
                    .m(26.0, 36.0)
                    .q(14.0 + s, 62.0, 19.0 + s, 86.0)
                    .q(26.0 + s, 80.0, 30.0, 82.0)
                    .q(29.0, 66.0, 36.0, 52.0)
                    .z()
                    .build();
                cv.fs(&lock, hair_back);
            }
            let cloth = rgb(st.cloth);
            for m in [false, true] {
                let ear =
                    P::mirrored(50.0, m).m(24.0, 28.0).q(20.0, 12.0, 22.0, 4.0).q(32.0, 8.0, 41.0, 16.0).z().build();
                cv.fs(&ear, cloth);
                let inner =
                    P::mirrored(50.0, m).m(26.5, 23.0).q(24.5, 14.0, 25.5, 9.0).q(31.0, 11.0, 36.0, 16.0).z().build();
                cv.fill(&inner, rgb(0xffb7cb));
            }
            cv.fs(&ellipse(50.0, 40.0, 31.0, 30.0), cloth);
            cv.fill(&ellipse(50.0, 43.0, 26.0, 25.0), shade(cloth, 0.8));
        }
        Hair::Ponytail => {
            let tail = P::new()
                .m(60.0, 16.0)
                .c(82.0, 4.0, 98.0, 28.0, 89.0 + s, 56.0)
                .q(86.0 + s, 68.0, 79.0 + s, 76.0)
                .c(83.0 + s, 58.0, 80.0, 34.0, 64.0, 26.0)
                .z()
                .build();
            cv.fs(&tail, hair_back);
            cv.fs(&ellipse(50.0, 40.0, 28.0, 26.0), hair_back);
        }
    }

    // 2. Legs and shoes.
    let (sock, shoe) = match st.outfit {
        Outfit::Sailor => (rgb(WHITE), rgb(0x5a3426)),
        Outfit::Hoodie => (rgb(SKIN), rgb(0xf0f0f8)),
        Outfit::Maid => (rgb(WHITE), rgb(0x2a2030)),
    };
    for x in [42.0, 52.0] {
        cv.fs(&rrect(x, 82.0, 6.0, 13.0, 2.5), rgb(SKIN));
        cv.fs(&rrect(x, 88.0, 6.0, 7.0, 2.0), sock);
    }
    cv.fs(&ellipse(45.0, 96.2, 4.8, 2.6), shoe);
    cv.fs(&ellipse(55.0, 96.2, 4.8, 2.6), shoe);

    // 3. Clothes.
    let sleeve = match st.outfit {
        Outfit::Sailor => {
            let navy = rgb(st.cloth);
            let skirt = P::new()
                .m(39.0, 70.0)
                .l(61.0, 70.0)
                .q(66.0, 80.0, 70.0, 88.0)
                .q(50.0, 92.0, 30.0, 88.0)
                .q(34.0, 80.0, 39.0, 70.0)
                .z()
                .build();
            cv.fs(&skirt, navy);
            for (a, b) in [(44.0, 41.0), (50.0, 50.0), (56.0, 59.0)] {
                cv.stroke(&P::new().m(a, 72.0).l(b, 89.0).build(), shade(navy, 0.7), 0.8);
            }
            cv.fs(&rrect(38.0, 60.0, 24.0, 14.0, 5.0), rgb(WHITE));
            let collar =
                P::new().m(37.5, 61.0).q(50.0, 64.0, 62.5, 61.0).l(60.0, 68.5).q(50.0, 72.0, 40.0, 68.5).z().build();
            cv.fs(&collar, navy);
            cv.stroke(&P::new().m(39.5, 64.5).q(50.0, 69.5, 60.5, 64.5).build(), rgb(WHITE), 0.8);
            bow(cv, 50.0, 68.5, 3.6, rgb(st.accent));
            rgb(WHITE)
        }
        Outfit::Hoodie => {
            let cloth = rgb(st.cloth);
            let body = P::new()
                .m(36.0, 60.0)
                .q(50.0, 57.0, 64.0, 60.0)
                .q(70.0, 74.0, 68.0, 86.0)
                .q(50.0, 90.0, 32.0, 86.0)
                .q(30.0, 74.0, 36.0, 60.0)
                .z()
                .build();
            cv.fs(&body, cloth);
            cv.stroke(&rrect(41.0, 75.0, 18.0, 7.0, 3.0), shade(cloth, 0.7), 0.8);
            for (x0, x1) in [(47.0, 46.0), (53.0, 54.0)] {
                cv.stroke(&P::new().m(x0, 62.0).l(x1, 70.0).build(), rgb(WHITE), 0.9);
                cv.fill(&circle(x1, 70.5, 0.9), rgb(WHITE));
            }
            cloth
        }
        Outfit::Maid => {
            let black = rgb(st.cloth);
            let skirt = P::new()
                .m(39.0, 70.0)
                .l(61.0, 70.0)
                .q(67.0, 80.0, 71.0, 89.0)
                .q(50.0, 93.0, 29.0, 89.0)
                .q(33.0, 80.0, 39.0, 70.0)
                .z()
                .build();
            cv.fs(&skirt, black);
            // Frilly petticoat edge.
            for k in 0..8 {
                let x = 32.0 + k as f32 * 5.1;
                cv.fill(&circle(x, 89.4 + if (3..5).contains(&k) { 1.0 } else { 0.0 }, 2.0), rgb(WHITE));
            }
            cv.fs(&rrect(38.0, 60.0, 24.0, 14.0, 5.0), black);
            let apron = P::new()
                .m(43.0, 66.0)
                .l(57.0, 66.0)
                .q(60.0, 77.0, 61.0, 87.0)
                .q(50.0, 90.0, 39.0, 87.0)
                .q(40.0, 77.0, 43.0, 66.0)
                .z()
                .build();
            cv.fl(&apron, rgb(WHITE), rgb(0xc9c9d9));
            for m in [false, true] {
                let c = P::mirrored(50.0, m).m(50.0, 61.0).l(43.0, 61.5).q(44.0, 65.5, 49.0, 64.5).z().build();
                cv.fl(&c, rgb(WHITE), rgb(0xc9c9d9));
            }
            bow(cv, 50.0, 63.0, 2.6, rgb(st.accent));
            black
        }
    };

    // 4. Arms (and the book when focusing).
    arms(cv, pose, sleeve, rgb(st.accent));

    // 5. Head and face.
    let face = P::new()
        .m(26.0, 40.0)
        .c(26.0, 26.0, 38.0, 20.0, 50.0, 20.0)
        .c(62.0, 20.0, 74.0, 26.0, 74.0, 40.0)
        .c(74.0, 54.0, 64.0, 64.5, 50.0, 65.0)
        .c(36.0, 64.5, 26.0, 54.0, 26.0, 40.0)
        .z()
        .build();
    cv.fl(&face, rgb(SKIN), rgb(0xe3ad98));
    blush(cv, 33.5, 66.5, 54.5, pose);
    eyes(cv, 39.5, 60.5, 47.0, 5.2, 6.8, rgb(st.eyes), pose);
    mouth(cv, 50.0, 57.5, pose, false);

    // 6. Bangs and side locks in front of the face.
    for m in [false, true] {
        let lock = P::mirrored(50.0, m)
            .m(27.5, 33.0)
            .q(21.5, 50.0, 25.0, 67.0)
            .q(28.0, 60.0, 31.5, 50.0)
            .q(32.0, 42.0, 31.0, 34.0)
            .z()
            .build();
        cv.fs(&lock, hair);
    }
    let bangs = P::new()
        .m(24.0, 48.0)
        .c(20.0, 22.0, 34.0, 11.5, 50.0, 11.5)
        .c(66.0, 11.5, 80.0, 22.0, 76.0, 48.0)
        .q(73.0, 38.0, 70.5, 31.0)
        .q(69.5, 35.0, 67.0, 39.5)
        .q(65.0, 34.0, 61.5, 28.5)
        .q(60.0, 33.5, 57.0, 38.0)
        .q(55.0, 33.0, 51.5, 28.0)
        .q(49.5, 33.5, 46.5, 38.5)
        .q(44.5, 33.5, 41.5, 29.0)
        .q(39.5, 34.5, 36.0, 39.5)
        .q(34.5, 35.0, 32.0, 31.5)
        .q(28.5, 37.0, 24.0, 48.0)
        .z()
        .build();
    cv.fs(&bangs, hair);
    cv.stroke(&P::new().m(35.0, 19.0).q(50.0, 13.5, 65.0, 19.0).build(), rgba(0xffffff, 0.45), 1.8);
    brows(cv, 39.5, 60.5, 37.5, hair, pose);

    // 7. Accessories on top.
    match st.hair_kind {
        Hair::TwinTails => {
            bow(cv, 29.0, 24.0, 4.5, rgb(st.accent));
            bow(cv, 71.0, 24.0, 4.5, rgb(st.accent));
            let ahoge = P::new().m(49.0, 12.5).q(50.0, 3.0, 58.0, 5.0).q(53.0, 6.5, 52.0, 12.5).z().build();
            cv.fs(&ahoge, hair);
        }
        Hair::Hood => {
            let rim =
                P::new().m(20.5, 58.0).c(15.0, 24.0, 32.0, 9.5, 50.0, 9.5).c(68.0, 9.5, 85.0, 24.0, 79.5, 58.0).build();
            cv.stroke(&rim, shade(rgb(st.cloth), 0.75), 5.6);
            cv.stroke(&rim, rgb(st.cloth), 4.0);
            // Snowflake hair pin.
            let (x, y) = (66.0, 24.0);
            for a in [0.0f32, 60.0, 120.0] {
                let (dx, dy) = (a.to_radians().cos() * 3.2, a.to_radians().sin() * 3.2);
                cv.stroke(&P::new().m(x - dx, y - dy).l(x + dx, y + dy).build(), rgb(st.accent), 1.2);
            }
        }
        Hair::Ponytail => {
            // Maid headdress with a frilly edge, then the big bow.
            let band =
                P::new().m(31.0, 22.0).q(50.0, 9.0, 69.0, 22.0).l(66.5, 25.5).q(50.0, 14.5, 33.5, 25.5).z().build();
            cv.fl(&band, rgb(WHITE), rgb(0xc9c9d9));
            for k in 0..6 {
                let t = k as f32 / 5.0;
                let x = 34.0 + t * 32.0;
                let y = 20.5 - (1.0 - (2.0 * t - 1.0).powi(2)) * 6.3;
                cv.fl(&circle(x, y, 2.0), rgb(WHITE), rgb(0xc9c9d9));
            }
            bow(cv, 64.0, 16.0, 5.0, rgb(st.accent));
        }
    }
}

fn bow(cv: &mut Canvas, x: f32, y: f32, s: f32, c: Color) {
    for m in [false, true] {
        let wing = P::mirrored(x, m)
            .m(x, y)
            .q(x - s * 0.6, y - s * 1.1, x - s * 1.5, y - s * 0.8)
            .q(x - s * 1.7, y, x - s * 1.5, y + s * 0.8)
            .q(x - s * 0.6, y + s * 1.0, x, y)
            .z()
            .build();
        cv.fs(&wing, c);
    }
    cv.fs(&circle(x, y, s * 0.42), shade(c, 0.9));
}

fn arms(cv: &mut Canvas, pose: &Pose, sleeve: Color, accent: Color) {
    let skin = rgb(SKIN);
    let f = pose.frame as usize;
    // Hand positions for the left arm; the right one is mirrored.
    let (hx, hy) = match pose.mood {
        Mood::Cheer => (if f % 2 == 0 { 24.0 } else { 22.0 }, if f % 2 == 0 { 50.0 } else { 47.0 }),
        Mood::Love => (27.0, 56.0),
        Mood::Held => (30.0, 46.0),
        Mood::Focus => (44.0, 76.0),
        _ => (35.0, 78.0),
    };
    let edge = shade(sleeve, 0.62);
    for m in [false, true] {
        let arm = P::mirrored(50.0, m).m(40.0, 63.5).l(hx + (40.0 - hx) * 0.15, hy + (63.5 - hy) * 0.15).build();
        cv.stroke(&arm, edge, 7.2);
        cv.stroke(&arm, sleeve, 5.4);
    }
    if pose.mood == Mood::Focus {
        let book = rrect(40.0, 69.0, 20.0, 12.5, 1.5);
        cv.fs(&book, accent);
        cv.fl(&rrect(41.5, 70.0, 17.0, 10.5, 1.0), rgb(0xfffdf4), rgb(0xd8cfb8));
        cv.stroke(&P::new().m(50.0, 70.0).l(50.0, 80.5).build(), rgb(0xd8cfb8), 0.8);
        for k in 0..3 {
            let y = 72.5 + k as f32 * 2.4;
            cv.stroke(&P::new().m(43.0, y).l(48.0, y).build(), rgb(0xb7b0a0), 0.6);
            cv.stroke(&P::new().m(52.0, y).l(57.0, y).build(), rgb(0xb7b0a0), 0.6);
        }
    }
    for m in [false, true] {
        let x = if m { 100.0 - hx } else { hx };
        cv.fl(&circle(x, hy, 3.0), skin, rgb(0xe3ad98));
    }
}
