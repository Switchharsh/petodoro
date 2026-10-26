//! Chibi anime girls, built from a shared rig with per-character hair and outfit.

use super::{blush, brows, eyes, fang, mouth, Mood, Persona, Pose};
use crate::gfx::{circle, ellipse, rgb, rgba, rrect, shade, Canvas, P};
use tiny_skia::{Color, Transform};

#[derive(Clone, Copy, PartialEq)]
pub enum Hair {
    TwinTails,
    Hood,
    Ponytail,
    /// Long straight hime cut.
    Long,
    /// Twin buns with short tails.
    Buns,
    /// Short and spiky.
    Short,
    /// Long and messy, with a nightcap.
    Messy,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Outfit {
    Sailor,
    Hoodie,
    Maid,
    Gothic,
    Blazer,
    Jacket,
    Pajamas,
}

pub struct Style {
    pub hair: u32,
    pub eyes: u32,
    pub skin: u32,
    pub hair_kind: Hair,
    pub outfit: Outfit,
    /// Ribbons / bows / book cover.
    pub accent: u32,
    /// Main clothing color.
    pub cloth: u32,
    pub fang: bool,
    pub bandage: bool,
}

const SKIN: u32 = 0xffe7da;
const WHITE: u32 = 0xfbfbff;

pub const SAKURA: Style = Style {
    hair: 0xf7a8c4,
    eyes: 0xa64fd6,
    skin: SKIN,
    hair_kind: Hair::TwinTails,
    outfit: Outfit::Sailor,
    accent: 0xe8475f,
    cloth: 0x2f3e74,
    fang: false,
    bandage: false,
};
pub const YUKI: Style = Style {
    hair: 0xe4e6f7,
    eyes: 0x3aa0e8,
    skin: SKIN,
    hair_kind: Hair::Hood,
    outfit: Outfit::Hoodie,
    accent: 0x6c8cff,
    cloth: 0xa9cdfc,
    fang: false,
    bandage: false,
};
pub const HIKARI: Style = Style {
    hair: 0xffd36e,
    eyes: 0x2fb36b,
    skin: SKIN,
    hair_kind: Hair::Ponytail,
    outfit: Outfit::Maid,
    accent: 0xe23d4f,
    cloth: 0x2b2b3a,
    fang: false,
    bandage: false,
};
pub const RIN: Style = Style {
    hair: 0xf28a46,
    eyes: 0x3d8fe0,
    skin: SKIN,
    hair_kind: Hair::Buns,
    outfit: Outfit::Blazer,
    accent: 0xd8384a,
    cloth: 0xe8d3a2,
    fang: true,
    bandage: false,
};
pub const LILITH: Style = Style {
    hair: 0x3f3159,
    eyes: 0xe8304c,
    skin: 0xfff0ef,
    hair_kind: Hair::Long,
    outfit: Outfit::Gothic,
    accent: 0xc8243c,
    cloth: 0x24212e,
    fang: true,
    bandage: false,
};
pub const AKANE: Style = Style {
    hair: 0xd83a3f,
    eyes: 0xf2b134,
    skin: SKIN,
    hair_kind: Hair::Short,
    outfit: Outfit::Jacket,
    accent: 0xe8475f,
    cloth: 0x2a2c38,
    fang: true,
    bandage: true,
};
pub const NEMU: Style = Style {
    hair: 0xcdb9f4,
    eyes: 0x8a6ad8,
    skin: SKIN,
    hair_kind: Hair::Messy,
    outfit: Outfit::Pajamas,
    accent: 0x6f8fe0,
    cloth: 0xbfd5ff,
    fang: false,
    bandage: false,
};

fn sway(pose: &Pose) -> f32 {
    let f = pose.frame as usize;
    match pose.mood {
        Mood::Held => [3.0, 1.5, -1.5, -3.0, -1.5, 1.5][f % 6],
        Mood::Cheer => [0.0, -1.0, -2.5, -3.5, -3.5, -2.5, -1.0, 0.0][f % 8],
        Mood::Relax | Mood::Walk => [-1.5, 0.0, 1.5, 0.0][f % 4],
        Mood::Dizzy => [-3.0, -1.5, 0.0, 1.5, 3.0, 1.5, 0.0, -1.5][f % 8],
        Mood::Idle => [0.0, 0.8][f % 2],
        _ => 0.0,
    }
}

/// Holding a pillow (Nemu) in calm moods.
fn holds_pillow(st: &Style, pose: &Pose) -> bool {
    st.outfit == Outfit::Pajamas && matches!(pose.mood, Mood::Idle | Mood::Sleep | Mood::Relax | Mood::Walk)
}

pub fn draw(cv: &mut Canvas, st: &Style, pose: &Pose) {
    let hair = rgb(st.hair);
    let hair_back = shade(hair, 0.84);
    let s = sway(pose);
    let accent = rgb(st.accent);

    // 0. Bat wings behind everything.
    if st.outfit == Outfit::Gothic {
        let flap = [0.0, 6.0, 10.0, 6.0][pose.frame as usize % 4]
            * if matches!(pose.mood, Mood::Cheer | Mood::Love | Mood::Held | Mood::Walk) { 1.0 } else { 0.3 };
        for m in [false, true] {
            let a = if m { flap } else { -flap };
            cv.with(Transform::from_rotate_at(a, if m { 62.0 } else { 38.0 }, 64.0), |cv| {
                let wing = P::mirrored(50.0, m)
                    .m(39.0, 64.0)
                    .c(28.0, 50.0, 14.0, 46.0, 4.0, 52.0)
                    .q(9.0, 58.0, 9.0, 64.0)
                    .q(14.0, 60.0, 17.0, 67.0)
                    .q(22.0, 62.0, 26.0, 70.0)
                    .q(31.0, 65.0, 38.0, 70.0)
                    .z()
                    .build();
                cv.fl(&wing, rgb(0x4a3566), rgb(0x9a7fd0));
                for (x, y) in [(9.0, 64.0), (17.0, 67.0), (26.0, 70.0)] {
                    let rib = P::mirrored(50.0, m).m(37.0, 65.0).l(x, y).build();
                    cv.stroke(&rib, rgba(0x9a7fd0, 0.6), 0.6);
                }
            });
        }
    }

    // 1. Hair (and hood) behind the head.
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
        Hair::Buns => {
            for m in [false, true] {
                let tail = P::mirrored(50.0, m)
                    .m(28.0, 20.0)
                    .c(16.0, 26.0, 14.0, 46.0, 19.0 + s, 62.0)
                    .q(22.0 + s, 66.0, 25.0 + s, 63.0)
                    .c(25.0, 52.0, 27.0, 38.0, 34.0, 28.0)
                    .z()
                    .build();
                cv.fs(&tail, hair_back);
            }
            cv.fs(&ellipse(50.0, 40.0, 28.0, 26.0), hair_back);
        }
        Hair::Long => {
            let back = P::new()
                .m(24.0, 36.0)
                .l(21.0 + s * 0.3, 90.0)
                .q(50.0, 93.0, 79.0 + s * 0.3, 90.0)
                .l(76.0, 36.0)
                .z()
                .build();
            cv.fs(&back, hair_back);
            cv.fs(&ellipse(50.0, 40.0, 28.0, 26.0), hair_back);
        }
        Hair::Messy => {
            let back = P::new()
                .m(24.0, 36.0)
                .q(13.0 + s, 60.0, 17.0 + s, 82.0)
                .q(23.0, 78.0, 27.0 + s, 87.0)
                .q(33.0, 80.0, 38.0, 86.0)
                .l(62.0, 86.0)
                .q(67.0, 80.0, 73.0 + s, 87.0)
                .q(77.0, 78.0, 83.0 + s, 82.0)
                .q(87.0 + s, 60.0, 76.0, 36.0)
                .z()
                .build();
            cv.fs(&back, hair_back);
            cv.fs(&ellipse(50.0, 40.0, 28.0, 26.0), hair_back);
        }
        Hair::Short => {
            for m in [false, true] {
                let spikes = P::mirrored(50.0, m)
                    .m(27.0, 36.0)
                    .l(19.0, 56.0 + s)
                    .l(26.0, 52.0)
                    .l(24.0, 64.0 + s)
                    .l(31.0, 56.0)
                    .z()
                    .build();
                cv.fs(&spikes, hair_back);
            }
            // Short spiky ponytail sticking out the back.
            let tail = P::new()
                .m(62.0, 18.0)
                .l(82.0, 6.0 + s)
                .l(77.0, 16.0)
                .l(88.0, 17.0 + s)
                .l(76.0, 23.0)
                .l(84.0, 30.0 + s)
                .l(70.0, 28.0)
                .z()
                .build();
            cv.fs(&tail, hair_back);
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
    let skin = rgb(st.skin);
    let (leg, sock, shoe) = match st.outfit {
        Outfit::Sailor => (skin, rgb(WHITE), rgb(0x5a3426)),
        Outfit::Hoodie => (skin, skin, rgb(0xf0f0f8)),
        Outfit::Maid => (skin, rgb(WHITE), rgb(0x2a2030)),
        Outfit::Gothic => (rgb(0x2a2733), rgb(0x2a2733), rgb(0x1a1820)),
        Outfit::Blazer => (skin, rgb(0x2f3446), rgb(0x5a3426)),
        Outfit::Jacket => (skin, rgb(WHITE), rgb(0x8a2a32)),
        Outfit::Pajamas => (rgb(st.cloth), rgb(st.cloth), rgb(0xffb3cf)),
    };
    for x in [42.0, 52.0] {
        cv.fs(&rrect(x, 82.0, 6.0, 13.0, 2.5), leg);
        cv.fs(&rrect(x, 88.0, 6.0, 7.0, 2.0), sock);
    }
    let fluffy = st.outfit == Outfit::Pajamas;
    let (srx, sry) = if fluffy { (5.6, 3.2) } else { (4.8, 2.6) };
    cv.fs(&ellipse(45.0, 96.2, srx, sry), shoe);
    cv.fs(&ellipse(55.0, 96.2, srx, sry), shoe);

    // 3. Clothes.
    let sleeve = clothes(cv, st);

    // 4. Arms (and the book / pillow).
    let late_hands = arms(cv, st, pose, sleeve, skin);

    // 5. Head and face.
    let face = P::new()
        .m(26.0, 40.0)
        .c(26.0, 26.0, 38.0, 20.0, 50.0, 20.0)
        .c(62.0, 20.0, 74.0, 26.0, 74.0, 40.0)
        .c(74.0, 54.0, 64.0, 64.5, 50.0, 65.0)
        .c(36.0, 64.5, 26.0, 54.0, 26.0, 40.0)
        .z()
        .build();
    cv.fl(&face, skin, rgb(0xe3ad98));
    blush(cv, 33.5, 66.5, 54.5, pose);
    eyes(cv, 39.5, 60.5, 47.0, 5.2, 6.8, rgb(st.eyes), pose);
    mouth(cv, 50.0, 57.5, pose, false);
    if st.fang {
        fang(cv, 50.0, 57.5, pose);
    }
    if st.bandage {
        cv.with(Transform::from_rotate_at(-25.0, 64.5, 55.0), |cv| {
            cv.fl(&rrect(60.5, 53.2, 8.0, 3.6, 1.2), rgb(0xfff4e6), rgb(0xd9b99a));
            cv.stroke(&P::new().m(63.5, 53.6).l(65.5, 56.4).build(), rgb(0xd9b99a), 0.5);
            cv.stroke(&P::new().m(65.5, 53.6).l(63.5, 56.4).build(), rgb(0xd9b99a), 0.5);
        });
    }
    if let Some((hx, hy)) = late_hands {
        for x in [hx, 100.0 - hx] {
            cv.fl(&circle(x, hy, 3.0), skin, rgb(0xe3ad98));
        }
    }

    // 6. Bangs and side locks in front of the face.
    if st.hair_kind == Hair::Long {
        // Hime cut: straight side locks and blunt bangs.
        for m in [false, true] {
            let lock = P::mirrored(50.0, m).m(25.5, 34.0).l(23.5, 72.0).l(31.5, 72.0).l(31.0, 36.0).z().build();
            cv.fs(&lock, hair);
        }
        let bangs = P::new()
            .m(24.0, 46.0)
            .c(20.0, 22.0, 34.0, 11.5, 50.0, 11.5)
            .c(66.0, 11.5, 80.0, 22.0, 76.0, 46.0)
            .l(72.5, 38.5)
            .q(50.0, 41.0, 27.5, 38.5)
            .z()
            .build();
        cv.fs(&bangs, hair);
    } else {
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
    }
    cv.stroke(&P::new().m(35.0, 19.0).q(50.0, 13.5, 65.0, 19.0).build(), rgba(0xffffff, 0.45), 1.8);
    brows(cv, 39.5, 60.5, 37.5, hair, pose);

    // 7. Accessories on top.
    let ahoge = |cv: &mut Canvas| {
        let a = P::new().m(49.0, 12.5).q(50.0, 3.0, 58.0, 5.0).q(53.0, 6.5, 52.0, 12.5).z().build();
        cv.fs(&a, hair);
    };
    match st.hair_kind {
        Hair::TwinTails => {
            bow(cv, 29.0, 24.0, 4.5, accent);
            bow(cv, 71.0, 24.0, 4.5, accent);
            ahoge(cv);
        }
        Hair::Buns => {
            for x in [28.0, 72.0] {
                cv.fs(&circle(x, 17.0, 8.5), hair);
                cv.stroke(&P::new().m(x - 6.0, 13.0).q(x, 10.0, x + 6.0, 13.0).build(), rgba(0xffffff, 0.4), 1.2);
                cv.stroke(&P::new().m(x - 7.5, 21.0).q(x, 25.5, x + 7.5, 21.0).build(), accent, 2.4);
            }
            ahoge(cv);
        }
        Hair::Long => {
            bow(cv, 30.0, 21.0, 4.0, accent);
            bat(cv, 70.0, 21.0, 1.0, rgb(0x2a2236));
        }
        Hair::Short => {
            let a = P::new().m(46.0, 12.5).l(44.0, 2.0).l(50.0, 10.0).l(56.0, 3.0).l(54.0, 12.5).z().build();
            cv.fs(&a, hair);
        }
        Hair::Messy => {
            // Floppy nightcap with a pompom.
            let tip = [0.0, 1.5, 3.0, 1.5][pose.frame as usize % 4];
            let cap = P::new()
                .m(23.0, 30.0)
                .c(24.0, 6.0, 60.0, 0.0, 80.0, 18.0)
                .q(92.0, 30.0 + tip, 90.0, 44.0 + tip)
                .q(82.0, 30.0, 77.0, 30.0)
                .q(50.0, 22.0, 23.0, 30.0)
                .z()
                .build();
            cv.fs(&cap, accent);
            for (x, y) in [(40.0, 14.0), (58.0, 12.0), (74.0, 22.0)] {
                cv.fill(&super::sparkle(x, y, 2.0), rgb(0xffe27a));
            }
            cv.fs(
                &P::new().m(22.5, 30.0).q(50.0, 21.0, 77.5, 30.0).l(77.0, 34.0).q(50.0, 26.0, 23.0, 34.0).z().build(),
                rgb(WHITE),
            );
            cv.fs(&circle(90.0, 45.0 + tip, 4.0), rgb(WHITE));
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
                cv.stroke(&P::new().m(x - dx, y - dy).l(x + dx, y + dy).build(), accent, 1.2);
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
            bow(cv, 64.0, 16.0, 5.0, accent);
        }
    }
}

/// Draws the outfit and returns the sleeve color.
fn clothes(cv: &mut Canvas, st: &Style) -> Color {
    let accent = rgb(st.accent);
    let cloth = rgb(st.cloth);
    let skirt = |flare: f32| {
        P::new()
            .m(39.0, 70.0)
            .l(61.0, 70.0)
            .q(66.0, 80.0, 70.0 + flare, 88.0)
            .q(50.0, 92.0 + flare, 30.0 - flare, 88.0)
            .q(34.0, 80.0, 39.0, 70.0)
            .z()
            .build()
    };
    match st.outfit {
        Outfit::Sailor => {
            cv.fs(&skirt(0.0), cloth);
            for (a, b) in [(44.0, 41.0), (50.0, 50.0), (56.0, 59.0)] {
                cv.stroke(&P::new().m(a, 72.0).l(b, 89.0).build(), shade(cloth, 0.7), 0.8);
            }
            cv.fs(&rrect(38.0, 60.0, 24.0, 14.0, 5.0), rgb(WHITE));
            let collar =
                P::new().m(37.5, 61.0).q(50.0, 64.0, 62.5, 61.0).l(60.0, 68.5).q(50.0, 72.0, 40.0, 68.5).z().build();
            cv.fs(&collar, cloth);
            cv.stroke(&P::new().m(39.5, 64.5).q(50.0, 69.5, 60.5, 64.5).build(), rgb(WHITE), 0.8);
            bow(cv, 50.0, 68.5, 3.6, accent);
            rgb(WHITE)
        }
        Outfit::Hoodie => {
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
        Outfit::Maid | Outfit::Gothic => {
            let gothic = st.outfit == Outfit::Gothic;
            if gothic {
                // Red petticoat peeking out.
                for k in 0..8 {
                    let x = 31.5 + k as f32 * 5.3;
                    cv.fill(&circle(x, 90.0 + if (3..5).contains(&k) { 1.0 } else { 0.0 }, 2.2), accent);
                }
            }
            cv.fs(&skirt(1.0), cloth);
            if !gothic {
                for k in 0..8 {
                    let x = 32.0 + k as f32 * 5.1;
                    cv.fill(&circle(x, 89.4 + if (3..5).contains(&k) { 1.0 } else { 0.0 }, 2.0), rgb(WHITE));
                }
            }
            cv.fs(&rrect(38.0, 60.0, 24.0, 14.0, 5.0), cloth);
            if gothic {
                // Corset lacing.
                let mut lace = P::new().m(47.0, 66.0);
                for k in 0..4 {
                    let y = 67.5 + k as f32 * 2.0;
                    lace = lace.l(if k % 2 == 0 { 53.0 } else { 47.0 }, y);
                }
                cv.stroke(&lace.build(), accent, 0.7);
            } else {
                let apron = P::new()
                    .m(43.0, 66.0)
                    .l(57.0, 66.0)
                    .q(60.0, 77.0, 61.0, 87.0)
                    .q(50.0, 90.0, 39.0, 87.0)
                    .q(40.0, 77.0, 43.0, 66.0)
                    .z()
                    .build();
                cv.fl(&apron, rgb(WHITE), rgb(0xc9c9d9));
            }
            for m in [false, true] {
                let c = P::mirrored(50.0, m).m(50.0, 61.0).l(43.0, 61.5).q(44.0, 65.5, 49.0, 64.5).z().build();
                cv.fl(&c, rgb(WHITE), rgb(0xc9c9d9));
            }
            bow(cv, 50.0, 63.0, 2.6, accent);
            cloth
        }
        Outfit::Blazer => {
            let plaid = rgb(0xc8414b);
            cv.fs(&skirt(0.0), plaid);
            for x in [40.0, 50.0, 60.0] {
                cv.stroke(&P::new().m(x, 71.0).l(x + (x - 50.0) * 0.4, 89.0).build(), rgba(0x5a1820, 0.5), 1.0);
            }
            cv.stroke(&P::new().m(34.0, 80.0).l(66.0, 80.0).build(), rgba(0xffe0a0, 0.6), 0.8);
            cv.fs(&rrect(38.0, 60.0, 24.0, 14.0, 5.0), cloth);
            cv.fl(&P::new().m(44.5, 60.5).l(55.5, 60.5).l(50.0, 71.0).z().build(), rgb(WHITE), rgb(0xc9c9d9));
            let tie = P::new().m(49.0, 61.0).l(51.0, 61.0).l(52.2, 68.0).l(50.0, 70.5).l(47.8, 68.0).z().build();
            cv.fs(&tie, accent);
            cv.fill(&circle(57.0, 70.0, 0.9), rgb(0xb8962e));
            cloth
        }
        Outfit::Jacket => {
            cv.fs(&rrect(43.0, 60.0, 14.0, 13.0, 3.0), rgb(WHITE));
            cv.fs(&P::new().m(43.0, 70.0).l(57.0, 70.0).l(59.0, 80.0).l(41.0, 80.0).z().build(), rgb(0x6b2030));
            // Long open coat.
            for m in [false, true] {
                let panel = P::mirrored(50.0, m)
                    .m(37.0, 61.0)
                    .l(45.0, 60.5)
                    .l(46.0, 72.0)
                    .l(44.0, 92.0)
                    .l(30.0, 92.0)
                    .q(31.0, 76.0, 37.0, 61.0)
                    .z()
                    .build();
                cv.fs(&panel, cloth);
                let lapel = P::mirrored(50.0, m).m(45.0, 60.5).l(41.5, 62.0).l(46.0, 70.0).z().build();
                cv.fs(&lapel, shade(cloth, 1.5));
            }
            cloth
        }
        Outfit::Pajamas => {
            let top = P::new()
                .m(37.0, 60.0)
                .q(50.0, 58.0, 63.0, 60.0)
                .q(67.0, 70.0, 66.0, 82.0)
                .q(50.0, 85.0, 34.0, 82.0)
                .q(33.0, 70.0, 37.0, 60.0)
                .z()
                .build();
            cv.fs(&top, cloth);
            for (x, y) in [(42.0, 66.0), (57.0, 72.0), (45.0, 78.0), (60.0, 64.0)] {
                cv.fill(&super::sparkle(x, y, 1.6), rgb(0xffe27a));
            }
            for y in [64.0, 70.0, 76.0] {
                cv.fill(&circle(50.0, y, 0.8), rgb(WHITE));
            }
            cloth
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

/// A little bat-shaped hair clip.
fn bat(cv: &mut Canvas, x: f32, y: f32, s: f32, c: Color) {
    for m in [false, true] {
        let w = P::mirrored(x, m)
            .m(x, y - 1.0 * s)
            .q(x - 3.0 * s, y - 3.5 * s, x - 7.0 * s, y - 2.5 * s)
            .q(x - 6.0 * s, y, x - 6.5 * s, y + 2.0 * s)
            .q(x - 4.5 * s, y + 0.5 * s, x - 3.0 * s, y + 2.5 * s)
            .q(x - 1.5 * s, y + 1.0 * s, x, y + 2.0 * s)
            .z()
            .build();
        cv.fl(&w, c, rgb(0x9a7fd0));
    }
    cv.fill(&circle(x - 1.0 * s, y - 0.2, 0.5 * s), rgb(0xe8304c));
    cv.fill(&circle(x + 1.0 * s, y - 0.2, 0.5 * s), rgb(0xe8304c));
}

/// Draws arms; returns hand positions to draw after the face (eating).
fn arms(cv: &mut Canvas, st: &Style, pose: &Pose, sleeve: Color, skin: Color) -> Option<(f32, f32)> {
    let f = pose.frame as usize;
    let p = pose.persona;
    let edge = shade(sleeve, 0.62);
    let limb = |cv: &mut Canvas, pts: &[(f32, f32)]| {
        for m in [false, true] {
            let mut path = P::mirrored(50.0, m).m(40.0, 63.5);
            for &(x, y) in pts {
                path = path.l(x, y);
            }
            let path = path.build();
            cv.stroke(&path, edge, 7.2);
            cv.stroke(&path, sleeve, 5.4);
        }
    };
    let hands = |cv: &mut Canvas, hx: f32, hy: f32| {
        for x in [hx, 100.0 - hx] {
            cv.fl(&circle(x, hy, 3.0), skin, rgb(0xe3ad98));
        }
    };

    if pose.mood == Mood::Love && p == Persona::Tsundere {
        // Arms crossed: "hmph!"
        for (a, b) in [((39.0, 64.0), (58.0, 69.5)), ((61.0, 64.0), (42.0, 69.5))] {
            let path = P::new().m(a.0, a.1).l(b.0, b.1).build();
            cv.stroke(&path, edge, 7.2);
            cv.stroke(&path, sleeve, 5.4);
        }
        cv.fl(&circle(59.0, 69.0, 2.8), skin, rgb(0xe3ad98));
        cv.fl(&circle(41.0, 69.0, 2.8), skin, rgb(0xe3ad98));
        return None;
    }
    if holds_pillow(st, pose) {
        limb(cv, &[(39.0, 72.0)]);
        cv.fl(&rrect(35.0, 66.0, 30.0, 15.0, 6.5), rgb(0xfffaf0), rgb(0xcfc6e8));
        let moon = P::new().m(52.0, 69.0).q(44.0, 73.5, 52.0, 78.0).q(47.0, 73.5, 52.0, 69.0).z().build();
        cv.fill(&moon, rgb(0xffe27a));
        hands(cv, 38.5, 73.0);
        return None;
    }
    if pose.mood == Mood::Eat {
        limb(cv, &[(43.0, 63.5)]);
        return Some((43.5, 63.5));
    }
    if matches!(pose.mood, Mood::Idle | Mood::Walk | Mood::Love) && p == Persona::Angry {
        // Hands on hips.
        limb(cv, &[(31.5, 70.0), (38.0, 75.0)]);
        hands(cv, 38.0, 75.0);
        return None;
    }
    let (hx, hy) = match pose.mood {
        Mood::Cheer => (if f % 2 == 0 { 24.0 } else { 22.0 }, if f % 2 == 0 { 50.0 } else { 47.0 }),
        Mood::Idle if p == Persona::Hyper => {
            (if f % 2 == 0 { 24.0 } else { 26.0 }, if f % 2 == 0 { 50.0 } else { 56.0 })
        }
        Mood::Love => (27.0, 56.0),
        Mood::Held => (30.0, 46.0),
        Mood::Focus => (44.0, 76.0),
        Mood::Dizzy => (25.0, 70.0),
        Mood::Walk => (35.0, if f % 2 == 0 { 76.0 } else { 79.0 }),
        _ => (35.0, 78.0),
    };
    limb(cv, &[(hx + (40.0 - hx) * 0.15, hy + (63.5 - hy) * 0.15)]);
    if pose.mood == Mood::Focus {
        let book = rrect(40.0, 69.0, 20.0, 12.5, 1.5);
        cv.fs(&book, rgb(st.accent));
        cv.fl(&rrect(41.5, 70.0, 17.0, 10.5, 1.0), rgb(0xfffdf4), rgb(0xd8cfb8));
        cv.stroke(&P::new().m(50.0, 70.0).l(50.0, 80.5).build(), rgb(0xd8cfb8), 0.8);
        for k in 0..3 {
            let y = 72.5 + k as f32 * 2.4;
            cv.stroke(&P::new().m(43.0, y).l(48.0, y).build(), rgb(0xb7b0a0), 0.6);
            cv.stroke(&P::new().m(52.0, y).l(57.0, y).build(), rgb(0xb7b0a0), 0.6);
        }
    }
    hands(cv, hx, hy);
    None
}
