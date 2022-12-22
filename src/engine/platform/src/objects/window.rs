use windows::Win32::Foundation::HWND;

pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub fn new(hwnd: HWND) -> Self {
        Window { hwnd }
    }
}
