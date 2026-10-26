//! Raw X11 plumbing: an always-on-top, undecorated, per-pixel transparent
//! window (32-bit ARGB visual) plus an override-redirect popup for the menu.

use super::{Bounds, Res, WinEvent, WinId};
use std::os::unix::io::AsRawFd;
use std::time::Duration;

use tiny_skia::Pixmap;
use x11rb::connection::{Connection, RequestConnection};
use x11rb::properties::{WmHints, WmSizeHints, WmSizeHintsSpecification};
use x11rb::protocol::shape::{self, ConnectionExt as _};
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        UTF8_STRING,
        _NET_WM_NAME,
        _NET_WM_PID,
        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_UTILITY,
        _NET_WM_STATE,
        _NET_WM_STATE_ABOVE,
        _NET_WM_STATE_STICKY,
        _NET_WM_STATE_SKIP_TASKBAR,
        _NET_WM_STATE_SKIP_PAGER,
        _MOTIF_WM_HINTS,
    }
}

pub struct Display {
    conn: RustConnection,
    root: Window,
    atoms: Atoms,
    /// The whole screen (all monitors).
    pub screen: Bounds,
    /// Where the pet starts by default (same as `screen` on X11).
    pub work: Bounds,
    /// Extra scale for dense screens (1.0 on desktops).
    pub ui_scale: f32,
    depth: u8,
    visual: Visualid,
    colormap: Colormap,
    /// A compositor is running, so alpha blending works. Without one we
    /// fall back to cutting the window to shape with the SHAPE extension.
    composited: bool,
    has_shape: bool,
}

pub struct Surface {
    win: Window,
    gc: Gcontext,
    pub w: u16,
    pub h: u16,
    input_shape: Vec<Rectangle>,
    bounding_shape: Vec<Rectangle>,
    buf: Vec<u8>,
}

impl Surface {
    pub fn id(&self) -> WinId {
        self.win as WinId
    }
}

impl Display {
    pub fn connect() -> Res<Display> {
        let (conn, screen_num) = x11rb::connect(None)?;
        let atoms = Atoms::new(&conn)?.reply()?;
        let screen = conn.setup().roots[screen_num].clone();

        // Prefer a 32-bit TrueColor visual for real per-pixel transparency.
        let argb = screen
            .allowed_depths
            .iter()
            .filter(|d| d.depth == 32)
            .flat_map(|d| d.visuals.iter())
            .find(|v| v.class == VisualClass::TRUE_COLOR)
            .map(|v| v.visual_id);
        let (depth, visual) = match argb {
            Some(v) => (32, v),
            None => (screen.root_depth, screen.root_visual),
        };
        let colormap = conn.generate_id()?;
        conn.create_colormap(ColormapAlloc::NONE, colormap, screen.root, visual)?;

        let cm_atom = conn.intern_atom(false, format!("_NET_WM_CM_S{screen_num}").as_bytes())?.reply()?.atom;
        let composited = depth == 32 && conn.get_selection_owner(cm_atom)?.reply()?.owner != x11rb::NONE;
        let has_shape = conn.extension_information(shape::X11_EXTENSION_NAME)?.is_some();

        let screen_bounds = Bounds { x: 0, y: 0, w: screen.width_in_pixels as i32, h: screen.height_in_pixels as i32 };
        Ok(Display {
            root: screen.root,
            screen: screen_bounds,
            work: screen_bounds,
            ui_scale: 1.0,
            conn,
            atoms,
            depth,
            visual,
            colormap,
            composited,
            has_shape,
        })
    }

    /// Next pending event, without blocking.
    pub fn poll(&self) -> Res<Option<WinEvent>> {
        use x11rb::protocol::Event;
        while let Some(ev) = self.conn.poll_for_event()? {
            let out = match ev {
                Event::Expose(e) => WinEvent::Expose(e.window as WinId),
                Event::ButtonPress(e) => WinEvent::Press {
                    win: e.event as WinId,
                    button: e.detail,
                    x: e.event_x as i32,
                    y: e.event_y as i32,
                    rx: e.root_x as i32,
                    ry: e.root_y as i32,
                },
                Event::ButtonRelease(e) => WinEvent::Release {
                    win: e.event as WinId,
                    button: e.detail,
                    x: e.event_x as i32,
                    y: e.event_y as i32,
                    rx: e.root_x as i32,
                    ry: e.root_y as i32,
                },
                Event::MotionNotify(e) => WinEvent::Motion {
                    win: e.event as WinId,
                    x: e.event_x as i32,
                    y: e.event_y as i32,
                    rx: e.root_x as i32,
                    ry: e.root_y as i32,
                },
                Event::ClientMessage(e)
                    if e.type_ == self.atoms.WM_PROTOCOLS
                        && e.format == 32
                        && e.data.as_data32()[0] == self.atoms.WM_DELETE_WINDOW =>
                {
                    WinEvent::Close
                }
                _ => continue,
            };
            return Ok(Some(out));
        }
        Ok(None)
    }

    pub fn flush(&self) -> Res<()> {
        self.conn.flush()?;
        Ok(())
    }

    /// Sleep until the X server has something for us or `timeout` passes.
    pub fn wait(&self, timeout: Duration) {
        let mut pfd = libc::pollfd { fd: self.conn.stream().as_raw_fd(), events: libc::POLLIN, revents: 0 };
        unsafe { libc::poll(&mut pfd, 1, timeout.as_millis().clamp(1, 60_000) as i32) };
    }

    fn create(&self, x: i32, y: i32, w: u16, h: u16, override_redirect: bool, events: EventMask) -> Res<Surface> {
        let win = self.conn.generate_id()?;
        let aux = CreateWindowAux::new()
            .background_pixel(0)
            .border_pixel(0)
            .colormap(self.colormap)
            .override_redirect(u32::from(override_redirect))
            .event_mask(events);
        self.conn.create_window(
            self.depth,
            win,
            self.root,
            x as i16,
            y as i16,
            w,
            h,
            0,
            WindowClass::INPUT_OUTPUT,
            self.visual,
            &aux,
        )?;
        let gc = self.conn.generate_id()?;
        self.conn.create_gc(gc, win, &CreateGCAux::new().graphics_exposures(0))?;
        Ok(Surface { win, gc, w, h, input_shape: vec![], bounding_shape: vec![], buf: vec![] })
    }

    /// The floating pet window: managed by the WM (so it stays on top and
    /// skips the taskbar) but without decorations or keyboard focus.
    pub fn create_pet_window(&self, x: i32, y: i32, w: u16, h: u16) -> Res<Surface> {
        let ev = EventMask::EXPOSURE
            | EventMask::BUTTON_PRESS
            | EventMask::BUTTON_RELEASE
            | EventMask::POINTER_MOTION
            | EventMask::STRUCTURE_NOTIFY;
        let s = self.create(x, y, w, h, false, ev)?;
        let (c, a, win) = (&self.conn, &self.atoms, s.win);

        c.change_property8(PropMode::REPLACE, win, AtomEnum::WM_NAME, AtomEnum::STRING, b"Petodoro")?;
        c.change_property8(PropMode::REPLACE, win, a._NET_WM_NAME, a.UTF8_STRING, b"Petodoro")?;
        c.change_property8(PropMode::REPLACE, win, AtomEnum::WM_CLASS, AtomEnum::STRING, b"petodoro\0Petodoro\0")?;
        c.change_property32(PropMode::REPLACE, win, a._NET_WM_PID, AtomEnum::CARDINAL, &[std::process::id()])?;
        c.change_property32(PropMode::REPLACE, win, a.WM_PROTOCOLS, AtomEnum::ATOM, &[a.WM_DELETE_WINDOW])?;
        // flags = MWM_HINTS_DECORATIONS, decorations = 0
        c.change_property32(PropMode::REPLACE, win, a._MOTIF_WM_HINTS, a._MOTIF_WM_HINTS, &[2, 0, 0, 0, 0])?;
        c.change_property32(
            PropMode::REPLACE,
            win,
            a._NET_WM_WINDOW_TYPE,
            AtomEnum::ATOM,
            &[a._NET_WM_WINDOW_TYPE_UTILITY],
        )?;
        c.change_property32(
            PropMode::REPLACE,
            win,
            a._NET_WM_STATE,
            AtomEnum::ATOM,
            &[a._NET_WM_STATE_ABOVE, a._NET_WM_STATE_STICKY, a._NET_WM_STATE_SKIP_TASKBAR, a._NET_WM_STATE_SKIP_PAGER],
        )?;
        let mut hints = WmHints::new();
        hints.input = Some(false);
        hints.set(c, win)?;
        let size = WmSizeHints {
            position: Some((WmSizeHintsSpecification::UserSpecified, x, y)),
            size: Some((WmSizeHintsSpecification::UserSpecified, w as i32, h as i32)),
            min_size: Some((w as i32, h as i32)),
            max_size: Some((w as i32, h as i32)),
            ..WmSizeHints::default()
        };
        size.set_normal_hints(c, win)?;
        c.map_window(win)?;
        // Some WMs ignore the requested position at map time; insist.
        self.move_window(&s, x, y)?;
        Ok(s)
    }

    /// The right-click menu: an unmanaged popup that grabs the pointer so a
    /// click anywhere else closes it.
    pub fn create_menu_window(&self, x: i32, y: i32, w: u16, h: u16) -> Res<Surface> {
        let ev = EventMask::EXPOSURE | EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION;
        let s = self.create(x, y, w, h, true, ev)?;
        self.conn.map_window(s.win)?;
        self.conn.configure_window(s.win, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE))?;
        let mask = EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION;
        // A failed grab only means outside clicks will not close the menu.
        let _ = self
            .conn
            .grab_pointer(
                false,
                s.win,
                mask,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
                x11rb::NONE,
                x11rb::NONE,
                x11rb::CURRENT_TIME,
            )?
            .reply();
        Ok(s)
    }

    pub fn destroy(&self, s: Surface) -> Res<()> {
        self.conn.ungrab_pointer(x11rb::CURRENT_TIME)?;
        self.conn.free_gc(s.gc)?;
        self.conn.destroy_window(s.win)?;
        Ok(())
    }

    pub fn move_window(&self, s: &Surface, x: i32, y: i32) -> Res<()> {
        self.conn.configure_window(s.win, &ConfigureWindowAux::new().x(x).y(y))?;
        Ok(())
    }

    pub fn move_resize(&self, s: &mut Surface, x: i32, y: i32, w: u16, h: u16) -> Res<()> {
        let size = WmSizeHints {
            position: Some((WmSizeHintsSpecification::UserSpecified, x, y)),
            size: Some((WmSizeHintsSpecification::UserSpecified, w as i32, h as i32)),
            min_size: Some((w as i32, h as i32)),
            max_size: Some((w as i32, h as i32)),
            ..WmSizeHints::default()
        };
        size.set_normal_hints(&self.conn, s.win)?;
        let aux = ConfigureWindowAux::new().x(x).y(y).width(w as u32).height(h as u32);
        self.conn.configure_window(s.win, &aux)?;
        s.w = w;
        s.h = h;
        s.input_shape.clear();
        s.bounding_shape.clear();
        Ok(())
    }

    pub fn bell(&self) {
        let _ = self.conn.bell(0);
    }

    /// Upload a frame and update the click-through / cut-out shapes.
    pub fn present(&self, s: &mut Surface, pm: &Pixmap) -> Res<()> {
        let (w, h) = (pm.width().min(s.w as u32) as usize, pm.height().min(s.h as u32) as usize);
        let stride = pm.width() as usize * 4;
        // tiny-skia is premultiplied RGBA; X wants premultiplied BGRA.
        s.buf.clear();
        s.buf.reserve(w * h * 4);
        for row in pm.data().chunks(stride).take(h) {
            for px in row[..w * 4].chunks_exact(4) {
                s.buf.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
            }
        }
        // Split into requests that fit the server's maximum request size.
        let max_rows = ((self.conn.maximum_request_bytes() - 64) / (w * 4)).max(1);
        let mut y = 0;
        while y < h {
            let rows = max_rows.min(h - y);
            self.conn.put_image(
                ImageFormat::Z_PIXMAP,
                s.win,
                s.gc,
                w as u16,
                rows as u16,
                0,
                y as i16,
                0,
                self.depth,
                &s.buf[y * w * 4..(y + rows) * w * 4],
            )?;
            y += rows;
        }

        if self.has_shape {
            // Clicks on transparent areas fall through to whatever is below.
            let input = alpha_rects(pm, 4, 24);
            if !same_rects(&input, &s.input_shape) {
                self.conn.shape_rectangles(
                    shape::SO::SET,
                    shape::SK::INPUT,
                    ClipOrdering::UNSORTED,
                    s.win,
                    0,
                    0,
                    &input,
                )?;
                s.input_shape = input;
            }
            if !self.composited {
                let bounding = alpha_rects(pm, 1, 110);
                if !same_rects(&bounding, &s.bounding_shape) {
                    self.conn.shape_rectangles(
                        shape::SO::SET,
                        shape::SK::BOUNDING,
                        ClipOrdering::UNSORTED,
                        s.win,
                        0,
                        0,
                        &bounding,
                    )?;
                    s.bounding_shape = bounding;
                }
            }
        }
        Ok(())
    }
}

fn same_rects(a: &[Rectangle], b: &[Rectangle]) -> bool {
    let key = |r: &Rectangle| (r.x, r.y, r.width, r.height);
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| key(a) == key(b))
}

/// Rectangles (on a `block`-pixel grid) covering every pixel whose alpha
/// exceeds `threshold`.
fn alpha_rects(pm: &Pixmap, block: u32, threshold: u8) -> Vec<Rectangle> {
    let (w, h) = (pm.width(), pm.height());
    let data = pm.data();
    let solid = |x: u32, y: u32| data[((y * w + x) * 4 + 3) as usize] > threshold;
    let mut rects = Vec::new();
    let mut by = 0;
    while by < h {
        let bh = block.min(h - by);
        let mut bx = 0;
        let mut run_start: Option<u32> = None;
        while bx < w {
            let bw = block.min(w - bx);
            let hit = (by..by + bh).any(|y| (bx..bx + bw).any(|x| solid(x, y)));
            match (hit, run_start) {
                (true, None) => run_start = Some(bx),
                (false, Some(s)) => {
                    rects.push(Rectangle { x: s as i16, y: by as i16, width: (bx - s) as u16, height: bh as u16 });
                    run_start = None;
                }
                _ => {}
            }
            bx += bw;
        }
        if let Some(s) = run_start {
            rects.push(Rectangle { x: s as i16, y: by as i16, width: (w - s) as u16, height: bh as u16 });
        }
        by += bh;
    }
    rects
}
