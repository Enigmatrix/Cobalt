use std::sync::Mutex;

use util::error::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use windows::Win32::UI::WindowsAndMessaging::{EVENT_OBJECT_NAMECHANGE, OBJID_WINDOW};

use crate::objects::{Target, WinEvent, WinEventHook, Window};

type ProcessTarget = Target;

/// Watches for name changes of windows
pub struct WindowTitleWatcher {
    _hook: WinEventHook,
}

impl WindowTitleWatcher {
    /// Create a new [WindowTitleWatcher]
    pub fn new(
        target: ProcessTarget,
        on_name_change: Box<dyn Fn(Window) -> Result<()> + Send + Sync>,
    ) -> Result<Self> {
        // prevent reentrancy, needed according to docs
        let m = Mutex::new(());
        let name_change_proc = move |_hwineventhook: HWINEVENTHOOK,
                                     _event: u32,
                                     hwnd: HWND,
                                     id_object: i32,
                                     _id_child: i32,
                                     _id_event_thread: u32,
                                     _dwms_event_time: u32| {
            let guard = m.lock().expect("reentrancy lock");
            if id_object != OBJID_WINDOW.0 {
                return;
            }
            let window = Window::new(hwnd);
            on_name_change(window).expect("on_name_change");
            drop(guard);
        };

        let _hook = WinEventHook::new(
            WinEvent::Event(EVENT_OBJECT_NAMECHANGE),
            target,
            Target::All,
            Box::new(name_change_proc),
        )?;
        Ok(Self { _hook })
    }
}
