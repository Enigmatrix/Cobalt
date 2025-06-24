use std::cell::RefCell;
use std::rc::Rc;

use util::error::{Context, Result};
use util::tracing::ResultTraceExt;
use windows::core::{Error, HSTRING, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW, RegisterClassW,
    SetWindowLongPtrW, UnregisterClassW, CW_USEDEFAULT, GWLP_USERDATA, HWND_DESKTOP, WINDOW_STYLE,
    WNDCLASSW, WS_EX_NOACTIVATE,
};

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    {
        let callbacks = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Rc<RefCell<Vec<Callback>>>;
        if let Some(callbacks) = callbacks.as_ref() {
            let mut callbacks = callbacks.borrow_mut();
            for cb in callbacks.iter_mut() {
                if let Some(ret) = cb(hwnd, msg, wparam, lparam) {
                    return ret;
                }
            }
        }
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

type Callback = Box<dyn FnMut(HWND, u32, WPARAM, LPARAM) -> Option<LRESULT>>;

/// Hidden message window
pub struct MessageWindow {
    hwnd: HWND,
    class_name: HSTRING,
    callbacks: &'static Rc<RefCell<Vec<Callback>>>,
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
                Some(HWND_DESKTOP),
                None,
                None,
                None,
            )?;

            let callbacks = Box::leak(Box::new(Rc::new(RefCell::new(Vec::new()))));

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, callbacks as *mut _ as isize);

            Ok(Self {
                hwnd,
                class_name,
                callbacks,
            })
        }
    }

    /// Add a callback to the message window WndProc
    pub fn add_callback(&self, callback: Callback) {
        self.callbacks.borrow_mut().push(callback);
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

            let callbacks =
                GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut Rc<RefCell<Vec<Callback>>>;
            drop(Box::from_raw(callbacks));
        }
    }
}
