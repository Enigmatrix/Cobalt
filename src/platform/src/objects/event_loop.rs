use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, TranslateMessage,
};

/// Representation of the Win32 [EventLoop]
pub struct EventLoop;

impl EventLoop {
    /// Create a new [EventLoop]
    pub fn new() -> Self {
        EventLoop
    }

    /// Start the [EventLoop]. Runs until WM_QUIT is received.
    pub fn run(&self) {
        let mut msg: MSG = Default::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
            unsafe {
                let _ = TranslateMessage(&msg);
            }
            unsafe { DispatchMessageW(&msg) };
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}
