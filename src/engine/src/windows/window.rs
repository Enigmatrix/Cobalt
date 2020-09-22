use crate::windows::*;
use widestring::WideString;

pub struct Window {
    handle: wintypes::HWND
}
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

pub struct UWP {
    aumid: WideString
}

impl Window {
    
    pub fn new(handle: wintypes::HWND) -> Self {
        Window { handle }
    }

    pub fn title(&self) -> WideString {
        todo!()
    }

    pub fn process(&self) -> Process {
        
    }

    pub fn uwp(&self) -> Option<UWP> {
        todo!()
    }
}