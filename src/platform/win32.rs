//! Win32 backend: layered windows (per-pixel alpha via UpdateLayeredWindow),
//! always on top, no taskbar button. Transparent pixels are click-through.

use super::{Bounds, Res, WinEvent, WinId};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ptr::{null, null_mut};
use std::time::Duration;
use tiny_skia::Pixmap;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::Diagnostics::Debug::MessageBeep;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

thread_local! {
    static QUEUE: RefCell<VecDeque<WinEvent>> = const { RefCell::new(VecDeque::new()) };
}

const MENU_TAG: isize = 1;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

fn push(ev: WinEvent) {
    QUEUE.with(|q| q.borrow_mut().push_back(ev));
}

fn lo(v: isize) -> i32 {
    (v & 0xffff) as u16 as i16 as i32
}

fn hi(v: isize) -> i32 {
    ((v >> 16) & 0xffff) as u16 as i16 as i32
}

/// Screen position of the cursor when the current message was posted.
fn message_pos() -> (i32, i32) {
    let p = unsafe { GetMessagePos() } as isize;
    (lo(p), hi(p))
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let win = hwnd as usize as WinId;
    let is_menu = GetWindowLongPtrW(hwnd, GWLP_USERDATA) == MENU_TAG;
    let button = match msg {
        WM_LBUTTONDOWN | WM_LBUTTONUP => 1,
        WM_MBUTTONDOWN | WM_MBUTTONUP => 2,
        WM_RBUTTONDOWN | WM_RBUTTONUP => 3,
        _ => 0,
    };
    match msg {
        WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN => {
            if button == 1 {
                SetCapture(hwnd); // keep getting moves while dragging
            }
            let (rx, ry) = message_pos();
            push(WinEvent::Press { win, button, x: lo(lp), y: hi(lp), rx, ry });
            0
        }
        WM_LBUTTONUP | WM_MBUTTONUP | WM_RBUTTONUP => {
            if button == 1 {
                ReleaseCapture();
            }
            let (rx, ry) = message_pos();
            push(WinEvent::Release { win, button, x: lo(lp), y: hi(lp), rx, ry });
            0
        }
        WM_MOUSEMOVE => {
            let (rx, ry) = message_pos();
            push(WinEvent::Motion { win, x: lo(lp), y: hi(lp), rx, ry });
            0
        }
        WM_MOUSEWHEEL => {
            // Wheel coordinates are screen coordinates.
            let (rx, ry) = (lo(lp), hi(lp));
            let mut p = POINT { x: rx, y: ry };
            ScreenToClient(hwnd, &mut p);
            let delta = hi(wp as isize);
            let button = if delta > 0 { 4 } else { 5 };
            push(WinEvent::Press { win, button, x: p.x, y: p.y, rx, ry });
            0
        }
        // Clicking the pet must not steal focus from the user's work.
        WM_MOUSEACTIVATE if !is_menu => MA_NOACTIVATE as LRESULT,
        WM_ACTIVATE => {
            if (wp & 0xffff) as u32 == WA_INACTIVE {
                push(WinEvent::FocusLost(win));
            }
            0
        }
        WM_CLOSE => {
            push(WinEvent::Close);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

pub struct Display {
    pub screen: Bounds,
    pub work: Bounds,
    /// Extra scale for dense screens (1.0 on desktops).
    pub ui_scale: f32,
    class: Vec<u16>,
    hinst: HINSTANCE,
}

pub struct Surface {
    hwnd: HWND,
    pub w: u16,
    pub h: u16,
    memdc: HDC,
    bitmap: HBITMAP,
    bits: *mut u8,
    pos: (i32, i32),
}

impl Surface {
    pub fn id(&self) -> WinId {
        self.hwnd as usize as WinId
    }

    /// (Re)create the 32-bit top-down DIB the frames are copied into.
    fn alloc(&mut self) -> Res<()> {
        unsafe {
            if !self.bitmap.is_null() {
                DeleteObject(self.bitmap);
            }
            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = self.w as i32;
            bmi.bmiHeader.biHeight = -(self.h as i32);
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB;
            let mut bits = null_mut();
            self.bitmap = CreateDIBSection(self.memdc, &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
            if self.bitmap.is_null() {
                return Err("CreateDIBSection failed".into());
            }
            self.bits = bits as *mut u8;
            SelectObject(self.memdc, self.bitmap);
        }
        Ok(())
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            DeleteDC(self.memdc);
            DeleteObject(self.bitmap);
        }
    }
}

impl Display {
    pub fn connect() -> Res<Display> {
        unsafe {
            // Real pixels on high-DPI screens (otherwise Windows blurs us).
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            let hinst = GetModuleHandleW(null());
            let class = wide("PetodoroWindow");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(wndproc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinst,
                hIcon: null_mut(),
                hCursor: LoadCursorW(null_mut(), IDC_HAND),
                hbrBackground: null_mut(),
                lpszMenuName: null(),
                lpszClassName: class.as_ptr(),
                hIconSm: null_mut(),
            };
            if RegisterClassExW(&wc) == 0 {
                return Err("RegisterClassExW failed".into());
            }
            let screen = Bounds {
                x: GetSystemMetrics(SM_XVIRTUALSCREEN),
                y: GetSystemMetrics(SM_YVIRTUALSCREEN),
                w: GetSystemMetrics(SM_CXVIRTUALSCREEN),
                h: GetSystemMetrics(SM_CYVIRTUALSCREEN),
            };
            let mut r: RECT = std::mem::zeroed();
            let work = if SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut r as *mut RECT as *mut _, 0) != 0 {
                Bounds { x: r.left, y: r.top, w: r.right - r.left, h: r.bottom - r.top }
            } else {
                screen
            };
            Ok(Display { screen, work, ui_scale: 1.0, class, hinst })
        }
    }

    fn create(&self, x: i32, y: i32, w: u16, h: u16, menu: bool) -> Res<Surface> {
        unsafe {
            let title = wide("Petodoro");
            let mut ex = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW;
            if !menu {
                ex |= WS_EX_NOACTIVATE;
            }
            let hwnd = CreateWindowExW(
                ex,
                self.class.as_ptr(),
                title.as_ptr(),
                WS_POPUP,
                x,
                y,
                w as i32,
                h as i32,
                null_mut(),
                null_mut(),
                self.hinst,
                null(),
            );
            if hwnd.is_null() {
                return Err("CreateWindowExW failed".into());
            }
            if menu {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, MENU_TAG);
            }
            let screen_dc = GetDC(null_mut());
            let memdc = CreateCompatibleDC(screen_dc);
            ReleaseDC(null_mut(), screen_dc);
            let mut s = Surface { hwnd, w, h, memdc, bitmap: null_mut(), bits: null_mut(), pos: (x, y) };
            s.alloc()?;
            Ok(s)
        }
    }

    pub fn create_pet_window(&self, x: i32, y: i32, w: u16, h: u16) -> Res<Surface> {
        let s = self.create(x, y, w, h, false)?;
        unsafe { ShowWindow(s.hwnd, SW_SHOWNOACTIVATE) };
        Ok(s)
    }

    /// The menu takes focus so that clicking anywhere else closes it.
    pub fn create_menu_window(&self, x: i32, y: i32, w: u16, h: u16) -> Res<Surface> {
        let s = self.create(x, y, w, h, true)?;
        unsafe {
            ShowWindow(s.hwnd, SW_SHOW);
            SetForegroundWindow(s.hwnd);
        }
        Ok(s)
    }

    pub fn destroy(&self, s: Surface) -> Res<()> {
        unsafe { DestroyWindow(s.hwnd) };
        Ok(())
    }

    pub fn move_window(&self, s: &Surface, x: i32, y: i32) -> Res<()> {
        unsafe { SetWindowPos(s.hwnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE) };
        Ok(())
    }

    pub fn move_resize(&self, s: &mut Surface, x: i32, y: i32, w: u16, h: u16) -> Res<()> {
        s.w = w;
        s.h = h;
        s.pos = (x, y);
        s.alloc()?;
        unsafe { SetWindowPos(s.hwnd, HWND_TOPMOST, x, y, w as i32, h as i32, SWP_NOACTIVATE) };
        Ok(())
    }

    pub fn bell(&self) {
        unsafe { MessageBeep(MB_ICONASTERISK) };
    }

    pub fn present(&self, s: &mut Surface, pm: &Pixmap) -> Res<()> {
        let (w, h) = (pm.width().min(s.w as u32) as usize, pm.height().min(s.h as u32) as usize);
        let stride = pm.width() as usize * 4;
        unsafe {
            // Premultiplied RGBA -> premultiplied BGRA, straight into the DIB.
            let dst = std::slice::from_raw_parts_mut(s.bits, s.w as usize * s.h as usize * 4);
            for (y, row) in pm.data().chunks(stride).take(h).enumerate() {
                let out = &mut dst[y * s.w as usize * 4..];
                for (x, px) in row[..w * 4].chunks_exact(4).enumerate() {
                    out[x * 4..x * 4 + 4].copy_from_slice(&[px[2], px[1], px[0], px[3]]);
                }
            }
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let size = SIZE { cx: s.w as i32, cy: s.h as i32 };
            let src = POINT { x: 0, y: 0 };
            let screen_dc = GetDC(null_mut());
            UpdateLayeredWindow(s.hwnd, screen_dc, null(), &size, s.memdc, &src, 0, &blend, ULW_ALPHA);
            ReleaseDC(null_mut(), screen_dc);
        }
        Ok(())
    }

    /// Next pending event, without blocking.
    pub fn poll(&self) -> Res<Option<WinEvent>> {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    push(WinEvent::Close);
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Ok(QUEUE.with(|q| q.borrow_mut().pop_front()))
    }

    pub fn flush(&self) -> Res<()> {
        Ok(())
    }

    /// Sleep until a window message arrives or `timeout` passes.
    pub fn wait(&self, timeout: Duration) {
        if QUEUE.with(|q| !q.borrow().is_empty()) {
            return;
        }
        let ms = timeout.as_millis().clamp(1, 60_000) as u32;
        unsafe { MsgWaitForMultipleObjects(0, null(), 0, ms, QS_ALLINPUT) };
    }
}
