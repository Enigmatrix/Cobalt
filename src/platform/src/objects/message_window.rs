use util::error::{Context, Result};
use util::tracing::{info, ResultTraceExt};
use windows::core::{Error, HSTRING, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, UnregisterClassW,
    CW_USEDEFAULT, HWND_DESKTOP, WINDOW_STYLE, WM_ENDSESSION, WM_QUERYENDSESSION, WNDCLASSW,
    WS_EX_NOACTIVATE,
};

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_QUERYENDSESSION {
        println!("WM_QUERYENDSESSION {:x}", lparam.0);
        info!("WM_QUERYENDSESSION {:x}", lparam.0);
    } else if msg == WM_ENDSESSION {
        println!("WM_ENDSESSION {:x}", lparam.0);
        info!("WM_ENDSESSION {:x}", lparam.0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

/// Hidden message window
pub struct MessageWindow {
    hwnd: HWND,
    class_name: HSTRING,
}

impl MessageWindow {
    /// Create a hidden message window
    pub fn new() -> Result<Self> {
        let class_name = HSTRING::from("Cobalt.MessageWindow");

        unsafe {
            let wc = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: HINSTANCE::default(),
                lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
                ..Default::default()
            };

            let atom = RegisterClassW(&wc);
            if atom == 0 {
                return Err(Error::from_win32()).context("Failed to register MessageWindow class");
            }

            let hwnd = CreateWindowExW(
                WS_EX_NOACTIVATE,
                PCWSTR::from_raw(class_name.as_ptr()),
                PCWSTR::null(),
                WINDOW_STYLE::default(),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND_DESKTOP,
                None,
                None,
                None,
            );

            if hwnd == HWND::default() {
                return Err(Error::from_win32()).context("Failed to create MessageWindow");
            }

            Ok(Self { hwnd, class_name })
        }
    }

    /// Get the handle of the window
    pub fn handle(&self) -> HWND {
        self.hwnd
    }
}

impl Drop for MessageWindow {
    fn drop(&mut self) {
        unsafe {
            if self.hwnd != HWND::default() {
                DestroyWindow(self.hwnd)
                    .context("Failed to destroy MessageWindow")
                    .error();
            }
            UnregisterClassW(PCWSTR::from_raw(self.class_name.as_ptr()), None)
                .context("Failed to unregister MessageWindow class")
                .error();
        }
    }
}
