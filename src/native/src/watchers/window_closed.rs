use crate::error::Win32Err;
use crate::raw::*;
use crate::wrappers::winevent::*;
use crate::wrappers::*;
use anyhow::*;

pub struct Watcher {
    _hook: Hook,
}

impl Watcher {
    pub fn new(
        window: Window,
        callback: impl Fn(Window) -> Result<()>,
    ) -> Result<Watcher, Win32Err> {
        let (pid, tid) = window.pid_tid()?;
        let _hook = Hook::new(
            Range::Single(Event::ObjectDestroyed),
            Locality::ProcessThread { pid, tid },
            Box::new(|args| {
                if args.id_object != winuser::OBJID_WINDOW || unsafe { winuser::IsWindow(args.hwnd) != 0 } {
                    return Ok(());
                }
                let window = Window::new(args.hwnd)?;
                callback(window)
            }),
        )?;
        Ok(Watcher { _hook })
    }
}
