//! Small drawing helpers on top of tiny-skia.

use tiny_skia::{
    Color, FillRule, GradientStop, LineCap, LineJoin, LinearGradient, Paint, Path, PathBuilder, Pixmap, Point, Rect,
    Shader, SpreadMode, Stroke, Transform,
};

pub fn rgb(hex: u32) -> Color {
    rgba(hex, 1.0)
}

pub fn rgba(hex: u32, a: f32) -> Color {
    let r = ((hex >> 16) & 0xff) as f32 / 255.0;
    let g = ((hex >> 8) & 0xff) as f32 / 255.0;
    let b = (hex & 0xff) as f32 / 255.0;
    Color::from_rgba(r, g, b, a.clamp(0.0, 1.0)).unwrap()
}

/// Multiply the RGB channels (f < 1 darkens).
pub fn shade(c: Color, f: f32) -> Color {
    Color::from_rgba(
        (c.red() * f).clamp(0.0, 1.0),
        (c.green() * f).clamp(0.0, 1.0),
        (c.blue() * f).clamp(0.0, 1.0),
        c.alpha(),
    )
    .unwrap()
}

/// Linear blend from `a` to `b`.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let l = |x: f32, y: f32| (x + (y - x) * t).clamp(0.0, 1.0);
    Color::from_rgba(l(a.red(), b.red()), l(a.green(), b.green()), l(a.blue(), b.blue()), l(a.alpha(), b.alpha()))
        .unwrap()
}

pub fn with_alpha(c: Color, a: f32) -> Color {
    let mut c = c;
    c.set_alpha(a.clamp(0.0, 1.0));
    c
}

/// Path builder with an optional horizontal mirror around x = `axis`,
/// which lets us describe symmetric shapes (ears, tails...) once.
pub struct P {
    pb: PathBuilder,
    mirror: Option<f32>,
}

impl P {
    pub fn new() -> Self {
        P { pb: PathBuilder::new(), mirror: None }
    }
    /// Same as `new`, but every x coordinate is reflected around `axis`.
    pub fn mirrored(axis: f32, on: bool) -> Self {
        P { pb: PathBuilder::new(), mirror: if on { Some(axis) } else { None } }
    }
    fn x(&self, x: f32) -> f32 {
        match self.mirror {
            Some(a) => 2.0 * a - x,
            None => x,
        }
    }
    pub fn m(mut self, x: f32, y: f32) -> Self {
        let x = self.x(x);
        self.pb.move_to(x, y);
        self
    }
    pub fn l(mut self, x: f32, y: f32) -> Self {
        let x = self.x(x);
        self.pb.line_to(x, y);
        self
    }
    pub fn q(mut self, x1: f32, y1: f32, x: f32, y: f32) -> Self {
        let (x1, x) = (self.x(x1), self.x(x));
        self.pb.quad_to(x1, y1, x, y);
        self
    }
    #[allow(clippy::too_many_arguments)]
    pub fn c(mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) -> Self {
        let (x1, x2, x) = (self.x(x1), self.x(x2), self.x(x));
        self.pb.cubic_to(x1, y1, x2, y2, x, y);
        self
    }
    pub fn z(mut self) -> Self {
        self.pb.close();
        self
    }
    pub fn build(self) -> Path {
        self.pb.finish().expect("valid path")
    }
}

pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Path {
    PathBuilder::from_oval(Rect::from_xywh(cx - rx, cy - ry, rx * 2.0, ry * 2.0).unwrap()).unwrap()
}

pub fn circle(cx: f32, cy: f32, r: f32) -> Path {
    ellipse(cx, cy, r, r)
}

pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = 0.4477 * r; // cubic approximation of a quarter circle (r - 0.5523r)
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - k, y, x + w, y + k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - k, x + w - k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + k, y + h, x, y + h - k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + k, x + k, y, x + r, y);
    pb.close();
    pb.finish().unwrap()
}

/// A pixmap plus the current transform (unit space -> pixels).
pub struct Canvas<'a> {
    pub pm: &'a mut Pixmap,
    pub ts: Transform,
}

impl<'a> Canvas<'a> {
    pub fn new(pm: &'a mut Pixmap, ts: Transform) -> Self {
        Canvas { pm, ts }
    }

    /// Run `f` with `t` applied on top of the current transform.
    pub fn with(&mut self, t: Transform, f: impl FnOnce(&mut Canvas)) {
        let saved = self.ts;
        self.ts = saved.pre_concat(t);
        f(self);
        self.ts = saved;
    }

    pub fn fill(&mut self, path: &Path, c: Color) {
        let mut p = Paint::default();
        p.set_color(c);
        p.anti_alias = true;
        self.pm.fill_path(path, &p, FillRule::Winding, self.ts, None);
    }

    /// Vertical gradient fill between y0 (color a) and y1 (color b).
    pub fn fill_vgrad(&mut self, path: &Path, y0: f32, a: Color, y1: f32, b: Color) {
        let shader = LinearGradient::new(
            Point::from_xy(0.0, y0),
            Point::from_xy(0.0, y1),
            vec![GradientStop::new(0.0, a), GradientStop::new(1.0, b)],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap_or(Shader::SolidColor(a));
        let p = Paint { shader, anti_alias: true, ..Paint::default() };
        self.pm.fill_path(path, &p, FillRule::Winding, self.ts, None);
    }

    pub fn stroke(&mut self, path: &Path, c: Color, width: f32) {
        let mut p = Paint::default();
        p.set_color(c);
        p.anti_alias = true;
        let s = Stroke { width, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Stroke::default() };
        self.pm.stroke_path(path, &p, &s, self.ts, None);
    }

    /// Fill and outline with a darker version of the fill (sticker/cel look).
    pub fn fs(&mut self, path: &Path, c: Color) {
        self.fill(path, c);
        self.stroke(path, shade(c, 0.62), 1.1);
    }

    /// Fill and outline with an explicit line color.
    pub fn fl(&mut self, path: &Path, c: Color, line: Color) {
        self.fill(path, c);
        self.stroke(path, line, 1.1);
    }
}
