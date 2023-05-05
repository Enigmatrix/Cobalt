use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

/// Representation of the Win32 [EventLoop]
pub struct EventLoop;

impl EventLoop {
    /// Create a new [EventLoop]
    pub fn new() -> Self {
        EventLoop
    }

    /// Start the [EventLoop]
    pub fn run(&self) {
        let mut msg: MSG = Default::default();
        while unsafe { GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() } {
            unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}
