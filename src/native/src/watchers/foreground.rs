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
        mut callback: impl FnMut(Window, Timestamp) -> Result<()>,
    ) -> Result<Watcher, Win32Err> {
        let _hook = Hook::new(
            Range::Single(Event::SystemForeground),
            Locality::Global,
            Box::new(move |args| {
                if args.id_object != winuser::OBJID_WINDOW
                    || unsafe { winuser::GetForegroundWindow() != args.hwnd }
                {
                    return Ok(());
                }
                let window = Window::new(args.hwnd)?;
                let timestamp = Timestamp::from_event_millis(args.dwms_event_time);
                callback(window, timestamp)
            }),
        )?;
        Ok(Watcher { _hook })
    }
}
