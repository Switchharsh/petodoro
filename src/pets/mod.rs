//! The pets. Every pet is drawn with vector shapes in a 100x100 unit box
//! (feet at y ~ 98) so it scales cleanly to any size.

mod cat;
mod dog;
mod fire;
mod ghost;
mod girl;
mod robot;
mod slime;
mod tomato;

use crate::gfx::{circle, ellipse, mix, rgb, rgba, shade, with_alpha, Canvas, P};
use std::time::Duration;
use tiny_skia::{Color, Transform};

/// Personality: changes faces, animation speed, sleepiness and what the pet says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Persona {
    Sweet,
    Tsundere,
    Angry,
    Sleepy,
    Hyper,
}

impl Persona {
    pub fn name(self) -> &'static str {
        match self {
            Persona::Sweet => "sweet",
            Persona::Tsundere => "tsundere",
            Persona::Angry => "angry",
            Persona::Sleepy => "sleepy",
            Persona::Hyper => "hyper",
        }
    }

    /// Multiplier on animation frame length.
    fn tempo(self) -> f32 {
        match self {
            Persona::Hyper => 0.55,
            Persona::Sleepy => 1.5,
            Persona::Angry => 0.85,
            _ => 1.0,
        }
    }

    /// How long without interaction before dozing off.
    pub fn sleep_after(self) -> Duration {
        Duration::from_secs(match self {
            Persona::Sleepy => 40,
            Persona::Hyper => 900,
            _ => 180,
        })
    }
}

/// Preferred look for the speech bubble and menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    Amoled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetKind {
    Tomo,
    Sakura,
    Yuki,
    Hikari,
    Rin,
    Lilith,
    Akane,
    Nemu,
    Mochi,
    Kuro,
    Pochi,
    Puru,
    Boo,
    Hinote,
    Bolt,
}

pub const ALL: [PetKind; 15] = [
    PetKind::Tomo,
    PetKind::Sakura,
    PetKind::Yuki,
    PetKind::Hikari,
    PetKind::Rin,
    PetKind::Lilith,
    PetKind::Akane,
    PetKind::Nemu,
    PetKind::Mochi,
    PetKind::Kuro,
    PetKind::Pochi,
    PetKind::Puru,
    PetKind::Boo,
    PetKind::Hinote,
    PetKind::Bolt,
];

impl PetKind {
    pub fn name(self) -> &'static str {
        match self {
            PetKind::Tomo => "tomo",
            PetKind::Sakura => "sakura",
            PetKind::Yuki => "yuki",
            PetKind::Hikari => "hikari",
            PetKind::Rin => "rin",
            PetKind::Lilith => "lilith",
            PetKind::Akane => "akane",
            PetKind::Nemu => "nemu",
            PetKind::Mochi => "mochi",
            PetKind::Kuro => "kuro",
            PetKind::Pochi => "pochi",
            PetKind::Puru => "puru",
            PetKind::Boo => "boo",
            PetKind::Hinote => "hinote",
            PetKind::Bolt => "bolt",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            PetKind::Tomo => "the Petodoro mascot: a little tomato",
            PetKind::Sakura => "pink twin-tails, sailor uniform",
            PetKind::Yuki => "silver hair, cat-ear hoodie",
            PetKind::Hikari => "blonde ponytail, maid outfit",
            PetKind::Rin => "orange twin buns, school blazer",
            PetKind::Lilith => "gothic vampire girl with bat wings (dark)",
            PetKind::Akane => "red-haired delinquent with a bandage",
            PetKind::Nemu => "lavender hair, pajamas, nightcap and pillow",
            PetKind::Mochi => "orange tabby cat",
            PetKind::Kuro => "pure-black neon cat for AMOLED screens",
            PetKind::Pochi => "shiba inu with endless zoomies",
            PetKind::Puru => "bouncy slime with a sprout",
            PetKind::Boo => "little ghost",
            PetKind::Hinote => "tiny fire spirit with a short fuse",
            PetKind::Bolt => "robot with an LED face (dark)",
        }
    }

    pub fn persona(self) -> Persona {
        match self {
            PetKind::Tomo
            | PetKind::Sakura
            | PetKind::Yuki
            | PetKind::Hikari
            | PetKind::Mochi
            | PetKind::Boo
            | PetKind::Bolt => Persona::Sweet,
            PetKind::Rin | PetKind::Lilith | PetKind::Kuro => Persona::Tsundere,
            PetKind::Akane | PetKind::Hinote => Persona::Angry,
            PetKind::Nemu => Persona::Sleepy,
            PetKind::Pochi | PetKind::Puru => Persona::Hyper,
        }
    }

    pub fn theme(self) -> Theme {
        match self {
            PetKind::Kuro => Theme::Amoled,
            PetKind::Lilith | PetKind::Bolt => Theme::Dark,
            _ => Theme::Light,
        }
    }

    /// Where the mouth is (snacks are drawn there).
    fn mouth(self) -> (f32, f32) {
        match self {
            PetKind::Mochi | PetKind::Kuro => (50.0, 60.0),
            PetKind::Pochi => (50.0, 62.0),
            PetKind::Puru => (50.0, 71.0),
            PetKind::Boo => (50.0, 57.0),
            PetKind::Hinote => (50.0, 75.0),
            PetKind::Bolt => (50.0, 54.0),
            PetKind::Tomo => (50.0, 77.5),
            _ => (50.0, 57.5),
        }
    }

    fn floats(self) -> bool {
        matches!(self, PetKind::Boo)
    }

    pub fn from_name(s: &str) -> Option<PetKind> {
        ALL.iter().copied().find(|p| p.name().eq_ignore_ascii_case(s.trim()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    /// Nothing running.
    Idle,
    /// Focus session or stopwatch running: reading a little book.
    Focus,
    /// On a break: humming.
    Relax,
    /// Ignored for a while.
    Sleep,
    /// A phase just finished: jumping with sparkles.
    Cheer,
    /// Was clicked.
    Love,
    /// Being dragged around.
    Held,
    /// Wandering along the screen.
    Walk,
    /// Was shaken while held.
    Dizzy,
    /// Eating a snack.
    Eat,
}

pub const MOODS: [Mood; 10] = [
    Mood::Idle,
    Mood::Focus,
    Mood::Relax,
    Mood::Sleep,
    Mood::Cheer,
    Mood::Love,
    Mood::Held,
    Mood::Walk,
    Mood::Dizzy,
    Mood::Eat,
];

impl Mood {
    /// How long one animation frame lasts, in seconds.
    pub fn frame_secs(self, p: Persona) -> f32 {
        let base = match self {
            Mood::Idle => 0.7,
            Mood::Focus => 0.6,
            Mood::Relax => 0.45,
            Mood::Sleep => 0.6,
            Mood::Cheer => 0.11,
            Mood::Love => 0.18,
            Mood::Held => 0.16,
            Mood::Walk => 0.14,
            Mood::Dizzy => 0.12,
            Mood::Eat => 0.25,
        };
        match self {
            Mood::Held | Mood::Walk => base,
            _ => base * p.tempo(),
        }
    }

    /// Number of frames after which the animation repeats exactly. Every
    /// `frame % n` used in the drawing code must have n dividing this.
    pub fn cycle(self) -> u32 {
        match self {
            Mood::Focus => 2,
            Mood::Idle | Mood::Relax | Mood::Walk => 4,
            Mood::Sleep => 6,
            Mood::Love | Mood::Dizzy | Mood::Eat => 8,
            Mood::Held => 12,
            Mood::Cheer => 24,
        }
    }

    pub fn blinks(self) -> bool {
        matches!(self, Mood::Idle | Mood::Focus | Mood::Held | Mood::Walk)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pose {
    pub mood: Mood,
    pub frame: u32,
    pub blink: bool,
    pub persona: Persona,
    /// Facing left (mirrored) while walking.
    pub flip: bool,
}

impl Pose {
    pub fn new(mood: Mood, frame: u32, persona: Persona) -> Pose {
        Pose { mood, frame, blink: false, persona, flip: false }
    }
}

pub const LASH: u32 = 0x3a2430;

const CHEER_JUMP: [f32; 8] = [0.0, 5.0, 9.0, 11.0, 11.0, 9.0, 5.0, 0.0];
const HELD_SWING: [f32; 6] = [-7.0, -3.0, 3.0, 7.0, 3.0, -3.0];

/// Vertical lift of the body (for the shadow), in units.
fn lift(pose: &Pose) -> f32 {
    let f = pose.frame as usize;
    match pose.mood {
        Mood::Cheer => CHEER_JUMP[f % CHEER_JUMP.len()],
        Mood::Love => [0.0, 2.0, 3.0, 2.0][f % 4],
        Mood::Walk => [0.0, 2.5, 3.5, 2.5][f % 4],
        Mood::Idle if pose.persona == Persona::Hyper => [0.0, 3.0, 5.0, 3.0][f % 4],
        _ => 0.0,
    }
}

/// Per-mood body motion, applied around the feet (or the scruff when held).
fn body_transform(pose: &Pose) -> Transform {
    let f = pose.frame as usize;
    let around = |sx: f32, sy: f32, px: f32, py: f32| {
        Transform::from_translate(px, py).pre_scale(sx, sy).pre_translate(-px, -py)
    };
    let t = match pose.mood {
        Mood::Idle => match pose.persona {
            Persona::Hyper => Transform::from_translate(0.0, -lift(pose)),
            // Nodding off.
            Persona::Sleepy => Transform::from_rotate_at([0.0, 1.5, 3.0, 1.5][f % 4], 50.0, 98.0),
            // Fuming: a tiny stomp.
            Persona::Angry => Transform::from_translate([0.0, -0.7, 0.0, 0.7][f % 4], 0.0),
            _ => around(if f % 2 == 1 { 1.015 } else { 1.0 }, if f % 2 == 1 { 0.985 } else { 1.0 }, 50.0, 98.0),
        },
        Mood::Focus => Transform::from_translate(0.0, if f % 2 == 1 { 0.8 } else { 0.0 }),
        Mood::Relax => Transform::from_rotate_at([-2.5, 0.0, 2.5, 0.0][f % 4], 50.0, 98.0),
        Mood::Sleep => {
            let s = [1.0, 0.99, 0.98, 0.97, 0.98, 0.99][f % 6];
            around(2.0 - s, s, 50.0, 98.0)
        }
        Mood::Cheer => {
            let squash = if f % CHEER_JUMP.len() == 0 { around(1.06, 0.94, 50.0, 98.0) } else { Transform::identity() };
            Transform::from_translate(0.0, -lift(pose)).pre_concat(squash)
        }
        Mood::Love | Mood::Walk => Transform::from_translate(0.0, -lift(pose)),
        Mood::Held => Transform::from_rotate_at(HELD_SWING[f % HELD_SWING.len()], 50.0, 12.0)
            .pre_concat(around(0.97, 1.04, 50.0, 12.0)),
        Mood::Dizzy => Transform::from_rotate_at([-6.0, -3.0, 0.0, 3.0, 6.0, 3.0, 0.0, -3.0][f % 8], 50.0, 98.0),
        Mood::Eat => Transform::from_translate(0.0, if f % 2 == 1 { 0.6 } else { 0.0 }),
    };
    if pose.flip {
        t.pre_concat(Transform::from_translate(100.0, 0.0).pre_scale(-1.0, 1.0))
    } else {
        t
    }
}

/// Draw `kind` into the 100x100 unit box mapped by `cv.ts`.
pub fn draw(cv: &mut Canvas, kind: PetKind, pose: &Pose) {
    if pose.mood != Mood::Held {
        let l = lift(pose) + if kind.floats() { 6.0 } else { 0.0 };
        let rx = (24.0 - l * 0.8).max(10.0);
        cv.fill(&ellipse(50.0, 97.5, rx, (3.2 - l * 0.1).max(1.5)), rgba(0x000000, 0.16));
    }
    cv.with(body_transform(pose), |cv| match kind {
        PetKind::Tomo => tomato::draw(cv, pose),
        PetKind::Sakura => girl::draw(cv, &girl::SAKURA, pose),
        PetKind::Yuki => girl::draw(cv, &girl::YUKI, pose),
        PetKind::Hikari => girl::draw(cv, &girl::HIKARI, pose),
        PetKind::Rin => girl::draw(cv, &girl::RIN, pose),
        PetKind::Lilith => girl::draw(cv, &girl::LILITH, pose),
        PetKind::Akane => girl::draw(cv, &girl::AKANE, pose),
        PetKind::Nemu => girl::draw(cv, &girl::NEMU, pose),
        PetKind::Mochi => cat::draw(cv, &cat::MOCHI, pose),
        PetKind::Kuro => cat::draw(cv, &cat::KURO, pose),
        PetKind::Pochi => dog::draw(cv, pose),
        PetKind::Puru => slime::draw(cv, pose),
        PetKind::Boo => ghost::draw(cv, pose),
        PetKind::Hinote => fire::draw(cv, pose),
        PetKind::Bolt => robot::draw(cv, pose),
    });
    draw_effects(cv, kind, pose);
}

// ---------------------------------------------------------------- faces

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EyeStyle {
    Open,
    Narrow,
    HappyArc,
    Sleepy,
    Blink,
    Squeeze,
    Wide,
    HalfLid,
    LookAway,
    Sparkle,
    Spiral,
}

pub fn eye_style(pose: &Pose) -> EyeStyle {
    if pose.blink && pose.mood.blinks() {
        return EyeStyle::Blink;
    }
    let p = pose.persona;
    match pose.mood {
        Mood::Idle | Mood::Walk => match p {
            Persona::Sleepy => EyeStyle::HalfLid,
            Persona::Angry => EyeStyle::Narrow,
            Persona::Hyper => EyeStyle::Sparkle,
            _ => EyeStyle::Open,
        },
        Mood::Focus if p == Persona::Sleepy => EyeStyle::HalfLid,
        Mood::Focus => EyeStyle::Narrow,
        Mood::Love => match p {
            Persona::Tsundere => EyeStyle::LookAway,
            Persona::Sleepy => EyeStyle::HappyArc,
            _ => EyeStyle::Squeeze,
        },
        Mood::Relax | Mood::Cheer | Mood::Eat => EyeStyle::HappyArc,
        Mood::Sleep => EyeStyle::Sleepy,
        Mood::Held => EyeStyle::Wide,
        Mood::Dizzy => EyeStyle::Spiral,
    }
}

fn spiral(cx: f32, cy: f32, r: f32) -> tiny_skia::Path {
    let mut p = P::new().m(cx, cy);
    for i in 1..=24 {
        let t = i as f32 / 24.0;
        let a = t * std::f32::consts::TAU * 2.0;
        p = p.l(cx + a.cos() * r * t, cy + a.sin() * r * t);
    }
    p.build()
}

/// Big anime eyes. `w`/`h` are the iris radii.
pub fn eyes(cv: &mut Canvas, lx: f32, rx: f32, y: f32, w: f32, h: f32, iris: Color, pose: &Pose) {
    eyes_ex(cv, lx, rx, y, w, h, iris, rgb(LASH), false, pose);
}

/// Eyes with a custom line color; `led` draws glowing screen-pixel eyes.
pub fn eyes_ex(
    cv: &mut Canvas,
    lx: f32,
    rx: f32,
    y: f32,
    w: f32,
    h: f32,
    iris: Color,
    lash: Color,
    led: bool,
    pose: &Pose,
) {
    let style = eye_style(pose);
    let glow = |cv: &mut Canvas, path: &tiny_skia::Path, width: f32| {
        if led {
            cv.stroke(path, with_alpha(lash, 0.25), width + 2.6);
        }
        cv.stroke(path, lash, width);
    };
    for (i, &cx) in [lx, rx].iter().enumerate() {
        let side = if i == 0 { -1.0 } else { 1.0 };
        match style {
            EyeStyle::Open | EyeStyle::Narrow | EyeStyle::Sparkle | EyeStyle::LookAway | EyeStyle::HalfLid if led => {
                let (hh, dx) = match style {
                    EyeStyle::Narrow => (h * 0.6, 0.0),
                    EyeStyle::HalfLid => (h * 0.4, 0.0),
                    EyeStyle::LookAway => (h, -w * 0.5),
                    _ => (h, 0.0),
                };
                let cy = y + (h - hh) * 0.5;
                let pill = crate::gfx::rrect(cx + dx - w * 0.5, cy - hh, w, hh * 2.0, w * 0.5);
                cv.fill(&pill, with_alpha(iris, 0.3));
                cv.with(Transform::identity(), |cv| cv.stroke(&pill, with_alpha(iris, 0.25), 2.6));
                cv.fill(&crate::gfx::rrect(cx + dx - w * 0.35, cy - hh * 0.85, w * 0.7, hh * 1.7, w * 0.35), iris);
                if style == EyeStyle::Sparkle {
                    cv.fill(&sparkle(cx + dx, cy - hh * 0.2, w * 0.6), rgb(0xffffff));
                }
            }
            EyeStyle::Open | EyeStyle::Narrow | EyeStyle::Sparkle => {
                let hh = if style == EyeStyle::Narrow { h * 0.78 } else { h };
                let cy = y + (h - hh) * 0.6;
                let path = ellipse(cx, cy, w, hh);
                cv.fill_vgrad(&path, cy - hh, shade(iris, 0.45), cy + hh, mix(iris, rgb(0xffffff), 0.35));
                cv.stroke(&path, lash, 0.7);
                cv.fill(&ellipse(cx, cy + hh * 0.12, w * 0.46, hh * 0.52), shade(iris, 0.28));
                if style == EyeStyle::Sparkle {
                    cv.fill(&sparkle(cx - w * 0.2, cy - hh * 0.25, w * 0.62), rgb(0xffffff));
                    cv.fill(&circle(cx + w * 0.35, cy + hh * 0.45, w * 0.17), rgb(0xffffff));
                } else {
                    cv.fill(&circle(cx - w * 0.32, cy - hh * 0.4, w * 0.36), rgb(0xffffff));
                    cv.fill(&circle(cx + w * 0.3, cy + hh * 0.42, w * 0.17), rgba(0xffffff, 0.9));
                }
                let top = P::new()
                    .m(cx - w * 1.2, cy - hh * 0.45)
                    .q(cx, cy - hh * 1.45, cx + w * 1.2, cy - hh * 0.45)
                    .build();
                cv.stroke(&top, lash, 1.7);
                let ox = cx + side * w * 1.15;
                let flick = P::new().m(ox, cy - hh * 0.5).l(ox + side * 1.6, cy - hh * 0.75).build();
                cv.stroke(&flick, lash, 1.1);
            }
            EyeStyle::LookAway => {
                // Eye white with the iris pushed to the side: "I'm not looking at you".
                let white = ellipse(cx, y, w * 1.05, h * 0.9);
                cv.fill(&white, rgb(0xffffff));
                let ix = cx - w * 0.45;
                cv.fill_vgrad(&ellipse(ix, y + 0.5, w * 0.6, h * 0.75), y - h, shade(iris, 0.45), y + h, iris);
                cv.fill(&circle(ix - w * 0.15, y - h * 0.25, w * 0.22), rgb(0xffffff));
                cv.stroke(&white, lash, 0.7);
                let top = P::new().m(cx - w * 1.2, y - h * 0.5).q(cx, y - h * 1.2, cx + w * 1.2, y - h * 0.5).build();
                cv.stroke(&top, lash, 1.7);
            }
            EyeStyle::HalfLid => {
                // Lower half of the eye under a heavy lid.
                let half = P::new().m(cx - w, y).c(cx - w, y + h * 1.3, cx + w, y + h * 1.3, cx + w, y).z().build();
                cv.fill_vgrad(&half, y, shade(iris, 0.5), y + h, mix(iris, rgb(0xffffff), 0.3));
                cv.fill(&circle(cx + w * 0.3, y + h * 0.45, w * 0.18), rgb(0xffffff));
                cv.stroke(&half, lash, 0.7);
                let lid = P::new().m(cx - w * 1.2, y + 0.4).q(cx, y - 1.2, cx + w * 1.2, y + 0.4).build();
                cv.stroke(&lid, lash, 1.8);
            }
            EyeStyle::HappyArc => {
                let p = P::new().m(cx - w, y + 1.5).q(cx, y - h * 1.1, cx + w, y + 1.5).build();
                glow(cv, &p, 1.7);
            }
            EyeStyle::Sleepy => {
                let p = P::new().m(cx - w, y + 0.5).q(cx, y + h * 0.7, cx + w, y + 0.5).build();
                glow(cv, &p, 1.6);
            }
            EyeStyle::Blink => {
                let p = P::new().m(cx - w * 1.1, y + 1.5).q(cx, y + 3.0, cx + w * 1.1, y + 1.0).build();
                glow(cv, &p, 1.6);
            }
            EyeStyle::Squeeze => {
                // ">" on the left eye, "<" on the right.
                let s = -side;
                let p = P::new()
                    .m(cx - s * w * 0.8, y - h * 0.55)
                    .l(cx + s * w * 0.8, y)
                    .l(cx - s * w * 0.8, y + h * 0.55)
                    .build();
                glow(cv, &p, 1.7);
            }
            EyeStyle::Wide if led => glow(cv, &circle(cx, y, w * 0.8), 1.5),
            EyeStyle::Wide => {
                let white = ellipse(cx, y, w * 0.95, h * 0.95);
                cv.fill(&white, rgb(0xffffff));
                cv.stroke(&white, lash, 1.2);
                cv.fill(&circle(cx, y + 0.5, w * 0.45), shade(iris, 0.6));
                cv.fill(&circle(cx - w * 0.15, y - 0.3, w * 0.15), rgb(0xffffff));
            }
            EyeStyle::Spiral => glow(cv, &spiral(cx, y, w * 0.95), 1.0),
        }
    }
}

pub fn brows(cv: &mut Canvas, lx: f32, rx: f32, y: f32, color: Color, pose: &Pose) {
    brows_c(cv, lx, rx, y, shade(color, 0.7), pose)
}

pub fn brows_c(cv: &mut Canvas, lx: f32, rx: f32, y: f32, c: Color, pose: &Pose) {
    let p = pose.persona;
    let angry = match pose.mood {
        Mood::Focus => true,
        Mood::Idle | Mood::Walk | Mood::Love | Mood::Eat => p == Persona::Angry,
        _ => false,
    } || (pose.mood == Mood::Love && p == Persona::Tsundere);
    let worried = matches!(pose.mood, Mood::Held | Mood::Dizzy);
    if angry {
        // Slanted down toward the middle; steeper when actually angry.
        let k = if p == Persona::Angry { 1.8 } else { 1.0 };
        let w = if p == Persona::Angry { 1.7 } else { 1.4 };
        cv.stroke(&P::new().m(lx - 4.0, y - 1.2 * k).l(lx + 3.5, y + 0.6 * k).build(), c, w);
        cv.stroke(&P::new().m(rx + 4.0, y - 1.2 * k).l(rx - 3.5, y + 0.6 * k).build(), c, w);
    } else if worried {
        cv.stroke(&P::new().m(lx - 4.0, y + 0.4).l(lx + 3.0, y - 1.4).build(), c, 1.3);
        cv.stroke(&P::new().m(rx + 4.0, y + 0.4).l(rx - 3.0, y - 1.4).build(), c, 1.3);
    }
}

pub fn blush(cv: &mut Canvas, lx: f32, rx: f32, y: f32, pose: &Pose) {
    let flustered = pose.mood == Mood::Love && matches!(pose.persona, Persona::Tsundere | Persona::Angry);
    let a = match pose.mood {
        _ if flustered => 0.8,
        Mood::Love | Mood::Cheer | Mood::Eat => 0.6,
        Mood::Held => 0.25,
        _ => 0.38,
    };
    let c = rgba(0xff7f9e, a);
    let (bw, bh) = if flustered { (5.5, 2.8) } else { (4.2, 2.2) };
    cv.fill(&ellipse(lx, y, bw, bh), c);
    cv.fill(&ellipse(rx, y, bw, bh), c);
    if matches!(pose.mood, Mood::Love | Mood::Cheer) {
        let line = rgba(0xe0567a, 0.7);
        for cx in [lx, rx] {
            for k in 0..3 {
                let x = cx - 2.4 + k as f32 * 1.8;
                cv.stroke(&P::new().m(x + 0.6, y - 1.0).l(x - 0.4, y + 1.0).build(), line, 0.6);
            }
        }
    }
}

pub fn mouth(cv: &mut Canvas, x: f32, y: f32, pose: &Pose, cat: bool) {
    mouth_c(cv, x, y, pose, cat, rgb(0x8a3a48), rgb(0xb8404f));
}

fn wavy(x: f32, y: f32, w: f32) -> tiny_skia::Path {
    P::new()
        .m(x - w, y)
        .q(x - w * 0.75, y - 1.2, x - w * 0.5, y)
        .q(x - w * 0.25, y + 1.2, x, y)
        .q(x + w * 0.25, y - 1.2, x + w * 0.5, y)
        .q(x + w * 0.75, y + 1.2, x + w, y)
        .build()
}

pub fn mouth_c(cv: &mut Canvas, x: f32, y: f32, pose: &Pose, cat: bool, line: Color, inside: Color) {
    let open_smile = |cv: &mut Canvas, s: f32| {
        let p = P::new().m(x - 3.6 * s, y - 1.0).q(x, y + 6.2 * s, x + 3.6 * s, y - 1.0).z().build();
        cv.fl(&p, inside, line);
        cv.fill(&ellipse(x, y + 2.5 * s, 1.8 * s, 0.9 * s), rgb(0xff8fa3));
    };
    let p = pose.persona;
    let f = pose.frame;
    match pose.mood {
        Mood::Idle | Mood::Walk => match p {
            Persona::Hyper => open_smile(cv, 0.85),
            Persona::Angry => {
                cv.stroke(&P::new().m(x - 2.6, y + 1.4).q(x, y - 0.8, x + 2.6, y + 1.4).build(), line, 1.2)
            }
            Persona::Tsundere => {
                cv.stroke(&P::new().m(x - 1.8, y + 0.9).q(x, y - 0.3, x + 1.8, y + 0.9).build(), line, 1.1)
            }
            Persona::Sleepy if f % 4 == 3 => cv.fl(&ellipse(x, y + 1.0, 1.8, 2.4), inside, line), // yawn
            Persona::Sleepy => cv.fl(&ellipse(x, y + 0.8, 0.9, 1.0), inside, line),
            Persona::Sweet if cat => {
                let m = P::new().m(x - 3.2, y).q(x - 1.6, y + 2.6, x, y).q(x + 1.6, y + 2.6, x + 3.2, y).build();
                cv.stroke(&m, line, 1.0);
            }
            Persona::Sweet => cv.stroke(&P::new().m(x - 2.2, y).q(x, y + 1.9, x + 2.2, y).build(), line, 1.1),
        },
        Mood::Focus if p == Persona::Hyper => cv.fl(&ellipse(x, y + 0.8, 1.6, 1.4), inside, line),
        Mood::Focus => cv.stroke(&P::new().m(x - 1.7, y + 0.7).l(x + 1.7, y + 0.7).build(), line, 1.1),
        Mood::Relax => {
            let m = P::new().m(x - 2.8, y - 0.4).q(x, y + 4.4, x + 2.8, y - 0.4).z().build();
            cv.fl(&m, inside, line);
        }
        Mood::Sleep => cv.fl(&ellipse(x, y + 0.9, 1.1, 1.3), inside, line),
        Mood::Love if matches!(p, Persona::Tsundere | Persona::Angry) => cv.stroke(&wavy(x, y + 0.8, 2.8), line, 1.0),
        Mood::Cheer | Mood::Love => open_smile(cv, 1.0),
        Mood::Held => cv.fl(&ellipse(x, y + 1.0, 1.6, 2.2), inside, line),
        Mood::Dizzy => cv.stroke(&wavy(x, y + 0.8, 3.0), line, 1.0),
        Mood::Eat if f % 2 == 0 => cv.fl(&ellipse(x, y + 0.8, 2.2, 1.8), inside, line),
        Mood::Eat => cv.stroke(&P::new().m(x - 2.4, y + 0.6).q(x, y + 2.2, x + 2.4, y + 0.6).build(), line, 1.1),
    }
}

/// Little white fang at the corner of the mouth (tsundere / angry girls).
pub fn fang(cv: &mut Canvas, x: f32, y: f32, pose: &Pose) {
    if matches!(pose.mood, Mood::Sleep | Mood::Held | Mood::Eat) {
        return;
    }
    let p = P::new().m(x + 0.6, y + 0.5).l(x + 2.6, y + 0.5).l(x + 1.7, y + 2.6).z().build();
    cv.fl(&p, rgb(0xffffff), rgb(0x8a3a48));
}

// --------------------------------------------------------------- effects

pub fn heart(cx: f32, cy: f32, s: f32) -> tiny_skia::Path {
    P::new()
        .m(cx, cy + s * 0.9)
        .c(cx - s * 1.6, cy - s * 0.1, cx - s * 0.9, cy - s * 1.3, cx, cy - s * 0.45)
        .c(cx + s * 0.9, cy - s * 1.3, cx + s * 1.6, cy - s * 0.1, cx, cy + s * 0.9)
        .z()
        .build()
}

pub fn sparkle(cx: f32, cy: f32, s: f32) -> tiny_skia::Path {
    P::new()
        .m(cx, cy - s)
        .q(cx, cy, cx + s, cy)
        .q(cx, cy, cx, cy + s)
        .q(cx, cy, cx - s, cy)
        .q(cx, cy, cx, cy - s)
        .z()
        .build()
}

/// The manga "anger vein" mark.
fn anger_vein(cv: &mut Canvas, x: f32, y: f32, s: f32) {
    let c = rgb(0xe8344a);
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let (ax, ay) = (x + dx * s * 0.35, y + dy * s * 0.35);
        let p = P::new().m(ax + dx * s * 0.9, ay).q(ax, ay, ax, ay + dy * s * 0.9).build();
        cv.stroke(&p, c, s * 0.3);
    }
}

fn cookie(cv: &mut Canvas, x: f32, y: f32, r: f32) {
    cv.fl(&circle(x, y, r), rgb(0xe0a45e), rgb(0x9c6430));
    for (dx, dy) in [(-0.4, -0.3), (0.35, -0.1), (-0.05, 0.4), (0.4, 0.45)] {
        if (dx * dx + dy * dy) * r * r < (r - 1.0) * (r - 1.0) {
            cv.fill(&circle(x + dx * r, y + dy * r, r * 0.13), rgb(0x5a3420));
        }
    }
}

fn draw_effects(cv: &mut Canvas, kind: PetKind, pose: &Pose) {
    let f = pose.frame;
    let p = pose.persona;
    match pose.mood {
        Mood::Sleep => {
            let col = rgb(0x6f7fd8);
            for i in 0..3u32 {
                let q = ((f + i * 2) % 6) as f32 / 6.0;
                let (x, y, s) = (72.0 + q * 16.0, 30.0 - q * 22.0, 3.0 + q * 3.5);
                let z = P::new().m(x, y).l(x + s, y).l(x, y + s).l(x + s, y + s).build();
                cv.stroke(&z, with_alpha(col, 1.0 - q * 0.8), 0.6 + s * 0.22);
            }
            if p == Persona::Sleepy {
                // Snot bubble that grows and pops.
                let (mx, my) = kind.mouth();
                let r = [1.5, 2.5, 3.5, 4.5, 5.5, 0.0][f as usize % 6];
                if r > 0.0 {
                    let b = circle(mx + 3.5 + r * 0.6, my - 4.0, r);
                    cv.fill(&b, rgba(0xbfe6ff, 0.45));
                    cv.stroke(&b, rgba(0x6fb6e6, 0.8), 0.6);
                    cv.fill(&circle(mx + 3.0 + r * 0.4, my - 4.0 - r * 0.4, r * 0.25), rgba(0xffffff, 0.9));
                }
            }
        }
        Mood::Relax => {
            let col = rgb(0xff6f9c);
            for i in 0..2u32 {
                let q = ((f + i * 2) % 4) as f32 / 4.0;
                let (x, y) = (if i == 0 { 76.0 } else { 18.0 } + q * 5.0, 38.0 - q * 18.0);
                let c = with_alpha(col, 1.0 - q * 0.7);
                cv.with(Transform::from_rotate_at(-20.0, x, y), |cv| cv.fill(&ellipse(x, y, 2.4, 1.7), c));
                cv.stroke(
                    &P::new().m(x + 2.0, y - 0.4).l(x + 2.0, y - 8.0).q(x + 5.0, y - 6.5, x + 5.0, y - 3.5).build(),
                    c,
                    1.1,
                );
            }
        }
        Mood::Cheer => {
            let spots = [(14.0, 30.0), (86.0, 24.0), (18.0, 64.0), (84.0, 58.0), (50.0, 4.0)];
            for (i, (x, y)) in spots.iter().enumerate() {
                if (f as usize + i) % 3 != 0 {
                    let s = if (f as usize + i) % 2 == 0 { 4.5 } else { 3.0 };
                    cv.fl(&sparkle(*x, *y, s), rgb(0xffd84d), rgb(0xe0a400));
                }
            }
        }
        Mood::Love if matches!(p, Persona::Tsundere | Persona::Angry) => {
            anger_vein(cv, 78.0, 24.0, if f % 2 == 0 { 6.0 } else { 7.0 });
            let q = (f % 8) as f32 / 8.0;
            cv.fl(&heart(22.0, 34.0 - q * 10.0, 2.8), with_alpha(rgb(0xff5c8a), 1.0 - q * 0.6), rgb(0xc93a63));
        }
        Mood::Love => {
            for i in 0..3u32 {
                let q = ((f + i * 3) % 8) as f32 / 8.0;
                let x = [72.0, 26.0, 82.0][i as usize] + (q * 6.0).sin() * 2.0;
                let y = [30.0, 26.0, 50.0][i as usize] - q * 18.0;
                let c = with_alpha(rgb(0xff5c8a), 1.0 - q * 0.7);
                cv.fl(&heart(x, y, 3.2 + q * 1.5), c, with_alpha(rgb(0xc93a63), 1.0 - q * 0.7));
            }
        }
        Mood::Held => {
            let dy = if f % 2 == 0 { 0.0 } else { 1.5 };
            let (x, y) = (80.0, 26.0 + dy);
            let p = P::new().m(x, y - 3.5).q(x + 3.0, y + 1.0, x, y + 2.5).q(x - 3.0, y + 1.0, x, y - 3.5).z().build();
            cv.fl(&p, rgb(0x9fdcff), rgb(0x4a9fd8));
        }
        Mood::Idle | Mood::Focus if p == Persona::Angry => {
            if f % 2 == 0 {
                anger_vein(cv, 78.0, 22.0, 6.0);
            }
        }
        Mood::Idle if p == Persona::Hyper => {
            let spots = [(14.0, 40.0), (86.0, 36.0), (20.0, 14.0), (82.0, 12.0)];
            for (i, (x, y)) in spots.iter().enumerate() {
                if (f as usize + i) % 2 == 0 {
                    cv.fl(&sparkle(*x, *y, 3.0), rgb(0xffd84d), rgb(0xe0a400));
                }
            }
        }
        Mood::Dizzy => {
            for i in 0..3 {
                let a = (f as f32 * 45.0 + i as f32 * 120.0).to_radians();
                let (x, y) = (50.0 + a.cos() * 20.0, 12.0 + a.sin() * 5.0);
                cv.fl(&sparkle(x, y, 3.2), rgb(0xffd84d), rgb(0xe0a400));
            }
        }
        Mood::Eat => {
            let (mx, my) = kind.mouth();
            let r = [5.5, 5.5, 4.6, 4.6, 3.6, 3.6, 2.4, 2.4][f as usize % 8];
            cookie(cv, mx, my + 5.5, r);
            for k in 0..2u32 {
                let q = ((f + k) % 2) as f32;
                cv.fill(&circle(mx - 6.0 + k as f32 * 11.0, my + 9.0 + q * 3.0, 0.8), rgb(0xc98a45));
            }
        }
        Mood::Walk => {
            // Dust puffs behind the feet.
            let back = if pose.flip { 66.0 } else { 34.0 };
            let q = (f % 4) as f32 / 4.0;
            let dir = if pose.flip { 1.0 } else { -1.0 };
            cv.fill(&circle(back + dir * q * 6.0, 95.0 - q * 3.0, 2.2 + q * 1.5), rgba(0xd8d0c8, 0.55 * (1.0 - q)));
        }
        Mood::Idle | Mood::Focus => {}
    }
}
