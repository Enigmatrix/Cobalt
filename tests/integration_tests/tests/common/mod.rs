pub mod uwp;
pub mod win32;

use native::raw::*;
use native::wrappers::*;
use std::ptr;

pub trait ProcessContainer {
    fn pid(&self) -> u32;
}

pub fn windows() -> impl Iterator<Item = Window> {
    std::iter::successors(Some(ptr::null_mut()), |child| {
        let child = unsafe {
            winuser::FindWindowExW(ptr::null_mut(), *child, ptr::null_mut(), ptr::null_mut())
        };
        if !child.is_null() {
            Some(child)
        } else {
            None
        }
    })
    .map(|child| Window::new(child).unwrap())
}

pub fn main_window<T: ProcessContainer>(t: &T) -> Option<Window> {
    windows()
        .filter(|win| {
            if let Ok(true) = win.pid_tid().map(|(pid, _)| t.pid() == pid) {
                true
            } else {
                false
            }
        })
        .next()
}

pub fn bring_to_foreground<T: ProcessContainer>(t: &T) {
    let win = main_window(t).unwrap();
    let mut keys = [0u8; 256];
    if unsafe {
        winuser::GetKeyboardState(keys.as_mut_ptr()) != 0
            && keys[winuser::VK_MENU as usize] & 0x80 == 0
    } {
        unsafe {
            winuser::keybd_event(winuser::VK_MENU as u8, 0, winuser::KEYEVENTF_EXTENDEDKEY, 0)
        };
    }

    let res = unsafe { winuser::SetForegroundWindow(win.0) };

    if unsafe {
        winuser::GetKeyboardState(keys.as_mut_ptr()) != 0
            && keys[winuser::VK_MENU as usize] & 0x80 == 0
    } {
        unsafe {
            winuser::keybd_event(
                winuser::VK_MENU as u8,
                0,
                winuser::KEYEVENTF_EXTENDEDKEY | winuser::KEYEVENTF_KEYUP,
                0,
            )
        };
    }

    if res == 0 {
        panic!("fuck.");
    }
    pause();
}

pub fn pause() {
    std::thread::sleep(std::time::Duration::from_millis(500));
}
