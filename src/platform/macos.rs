//! macOS backend: borderless, transparent, floating Cocoa windows showing
//! the rendered frame in an NSImageView. No Dock icon (accessory app).
//!
//! Cocoa's screen coordinates start at the bottom-left of the main screen;
//! everything here converts to the top-left origin the rest of the app uses.

#![allow(unused_unsafe)]

use super::{Bounds, Res, WinEvent, WinId};
use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSBitmapImageRep, NSColor, NSDeviceRGBColorSpace,
    NSEvent, NSEventMask, NSEventType, NSImage, NSImageScaling, NSImageView, NSScreen, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{NSDate, NSDefaultRunLoopMode, NSPoint, NSRect, NSSize};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;
use tiny_skia::Pixmap;

/// NSFloatingWindowLevel / NSPopUpMenuWindowLevel.
const FLOATING_LEVEL: isize = 3;
const POPUP_LEVEL: isize = 101;

pub struct Display {
    mtm: MainThreadMarker,
    app: Retained<NSApplication>,
    pub screen: Bounds,
    pub work: Bounds,
    /// Extra scale for dense screens (1.0 on desktops).
    pub ui_scale: f32,
    /// Height of the main screen, for flipping y.
    main_h: f64,
    queue: RefCell<VecDeque<WinEvent>>,
    /// Pixel size of each of our windows, for flipping window-relative y.
    sizes: RefCell<Vec<(WinId, f64)>>,
    menu: RefCell<Option<WinId>>,
}

pub struct Surface {
    window: Retained<NSWindow>,
    view: Retained<NSImageView>,
    pub w: u16,
    pub h: u16,
}

impl Surface {
    pub fn id(&self) -> WinId {
        unsafe { self.window.windowNumber() as WinId }
    }
}

impl Display {
    pub fn connect() -> Res<Display> {
        let mtm = MainThreadMarker::new().ok_or("petodoro must run on the main thread")?;
        let app = NSApplication::sharedApplication(mtm);
        unsafe {
            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
            app.finishLaunching();
        }
        let screens = NSScreen::screens(mtm);
        let main = NSScreen::mainScreen(mtm).ok_or("no screen")?;
        let main_frame = main.frame();
        let main_h = main_frame.size.height;
        // Union of all screens, in top-left coordinates.
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for s in screens.iter() {
            let f = s.frame();
            x0 = x0.min(f.origin.x);
            x1 = x1.max(f.origin.x + f.size.width);
            y0 = y0.min(main_h - (f.origin.y + f.size.height));
            y1 = y1.max(main_h - f.origin.y);
        }
        let screen = Bounds { x: x0 as i32, y: y0 as i32, w: (x1 - x0) as i32, h: (y1 - y0) as i32 };
        let v = main.visibleFrame();
        let work = Bounds {
            x: v.origin.x as i32,
            y: (main_h - (v.origin.y + v.size.height)) as i32,
            w: v.size.width as i32,
            h: v.size.height as i32,
        };
        Ok(Display {
            mtm,
            app,
            screen,
            work,
            ui_scale: 1.0,
            main_h,
            queue: RefCell::new(VecDeque::new()),
            sizes: RefCell::new(vec![]),
            menu: RefCell::new(None),
        })
    }

    fn frame(&self, x: i32, y: i32, w: u16, h: u16) -> NSRect {
        NSRect::new(NSPoint::new(x as f64, self.main_h - y as f64 - h as f64), NSSize::new(w as f64, h as f64))
    }

    fn create(&self, x: i32, y: i32, w: u16, h: u16, menu: bool) -> Res<Surface> {
        let mtm = self.mtm;
        unsafe {
            let window = NSWindow::initWithContentRect_styleMask_backing_defer(
                mtm.alloc(),
                self.frame(x, y, w, h),
                NSWindowStyleMask::Borderless,
                NSBackingStoreType::Buffered,
                false,
            );
            window.setReleasedWhenClosed(false);
            window.setOpaque(false);
            window.setBackgroundColor(Some(&NSColor::clearColor()));
            window.setHasShadow(false);
            window.setLevel(if menu { POPUP_LEVEL } else { FLOATING_LEVEL });
            window.setAcceptsMouseMovedEvents(true);
            window.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::Stationary
                    | NSWindowCollectionBehavior::IgnoresCycle,
            );
            let view = NSImageView::new(mtm);
            view.setImageScaling(NSImageScaling::ScaleAxesIndependently);
            window.setContentView(Some(&view));
            window.orderFrontRegardless();
            let s = Surface { window, view, w, h };
            self.sizes.borrow_mut().push((s.id(), h as f64));
            Ok(s)
        }
    }

    pub fn create_pet_window(&self, x: i32, y: i32, w: u16, h: u16) -> Res<Surface> {
        self.create(x, y, w, h, false)
    }

    /// The app is activated with the menu, so clicking elsewhere (which
    /// deactivates it) closes the menu.
    pub fn create_menu_window(&self, x: i32, y: i32, w: u16, h: u16) -> Res<Surface> {
        let s = self.create(x, y, w, h, true)?;
        #[allow(deprecated)]
        unsafe {
            self.app.activateIgnoringOtherApps(true)
        };
        *self.menu.borrow_mut() = Some(s.id());
        Ok(s)
    }

    pub fn destroy(&self, s: Surface) -> Res<()> {
        let id = s.id();
        self.sizes.borrow_mut().retain(|(w, _)| *w != id);
        if *self.menu.borrow() == Some(id) {
            *self.menu.borrow_mut() = None;
        }
        unsafe { s.window.close() };
        Ok(())
    }

    pub fn move_window(&self, s: &Surface, x: i32, y: i32) -> Res<()> {
        let f = self.frame(x, y, s.w, s.h);
        unsafe { s.window.setFrameOrigin(f.origin) };
        Ok(())
    }

    pub fn move_resize(&self, s: &mut Surface, x: i32, y: i32, w: u16, h: u16) -> Res<()> {
        s.w = w;
        s.h = h;
        let id = s.id();
        for e in self.sizes.borrow_mut().iter_mut().filter(|e| e.0 == id) {
            e.1 = h as f64;
        }
        unsafe { s.window.setFrame_display(self.frame(x, y, w, h), true) };
        Ok(())
    }

    pub fn bell(&self) {
        objc2_app_kit::NSBeep();
    }

    pub fn present(&self, s: &mut Surface, pm: &Pixmap) -> Res<()> {
        let (w, h) = (pm.width() as isize, pm.height() as isize);
        unsafe {
            // NSBitmapImageRep's default layout (8-bit RGBA, alpha last,
            // premultiplied) is exactly tiny-skia's.
            let rep = NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
                self.mtm.alloc(),
                std::ptr::null_mut(),
                w,
                h,
                8,
                4,
                true,
                false,
                NSDeviceRGBColorSpace,
                w * 4,
                32,
            )
            .ok_or("NSBitmapImageRep failed")?;
            let data = rep.bitmapData();
            if data.is_null() {
                return Err("no bitmap data".into());
            }
            std::ptr::copy_nonoverlapping(pm.data().as_ptr(), data, pm.data().len());
            let image = NSImage::initWithSize(self.mtm.alloc(), NSSize::new(s.w as f64, s.h as f64));
            image.addRepresentation(&rep);
            s.view.setImage(Some(&image));
        }
        Ok(())
    }

    fn translate(&self, ev: &NSEvent) {
        let ty = unsafe { ev.r#type() };
        let win = unsafe { ev.windowNumber() } as WinId;
        let Some(h) = self.sizes.borrow().iter().find(|e| e.0 == win).map(|e| e.1) else { return };
        let local = unsafe { ev.locationInWindow() };
        let root = NSEvent::mouseLocation();
        let (x, y) = (local.x as i32, (h - local.y) as i32);
        let (rx, ry) = (root.x as i32, (self.main_h - root.y) as i32);
        let out = match ty {
            NSEventType::LeftMouseDown => WinEvent::Press { win, button: 1, x, y, rx, ry },
            NSEventType::RightMouseDown => WinEvent::Press { win, button: 3, x, y, rx, ry },
            NSEventType::OtherMouseDown => WinEvent::Press { win, button: 2, x, y, rx, ry },
            NSEventType::LeftMouseUp => WinEvent::Release { win, button: 1, x, y, rx, ry },
            NSEventType::RightMouseUp => WinEvent::Release { win, button: 3, x, y, rx, ry },
            NSEventType::OtherMouseUp => WinEvent::Release { win, button: 2, x, y, rx, ry },
            NSEventType::MouseMoved | NSEventType::LeftMouseDragged | NSEventType::RightMouseDragged => {
                WinEvent::Motion { win, x, y, rx, ry }
            }
            NSEventType::ScrollWheel => {
                let dy = unsafe { ev.scrollingDeltaY() };
                if dy.abs() < 0.5 {
                    return;
                }
                WinEvent::Press { win, button: if dy > 0.0 { 4 } else { 5 }, x, y, rx, ry }
            }
            _ => return,
        };
        self.queue.borrow_mut().push_back(out);
    }

    /// Pull one event from Cocoa (waiting up to `until`), translate it and
    /// let AppKit handle it too.
    fn pump(&self, until: &NSDate) -> bool {
        let ev = unsafe {
            self.app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                Some(until),
                NSDefaultRunLoopMode,
                true,
            )
        };
        match ev {
            Some(ev) => {
                self.translate(&ev);
                unsafe { self.app.sendEvent(&ev) };
                true
            }
            None => false,
        }
    }

    pub fn poll(&self) -> Res<Option<WinEvent>> {
        if self.queue.borrow().is_empty() {
            let past = NSDate::distantPast();
            while self.queue.borrow().is_empty() && self.pump(&past) {}
        }
        if let Some(menu) = *self.menu.borrow() {
            if !unsafe { self.app.isActive() } {
                self.queue.borrow_mut().push_back(WinEvent::FocusLost(menu));
            }
        }
        Ok(self.queue.borrow_mut().pop_front())
    }

    pub fn flush(&self) -> Res<()> {
        Ok(())
    }

    pub fn wait(&self, timeout: Duration) {
        if !self.queue.borrow().is_empty() {
            return;
        }
        let until = NSDate::dateWithTimeIntervalSinceNow(timeout.as_secs_f64());
        self.pump(&until);
    }
}
