use crate::wrappers::winevent::*;
use crate::wrappers::*;
use util::*;

#[derive(Debug)]
pub struct Watcher {
    _hook: Hook,
}

impl Watcher {
    pub fn new(window: &Window, mut callback: impl FnMut(Window) -> Result<()>) -> Result<Watcher> {
        let (pid, tid) = window.pid_tid()?;
        let _hook = Hook::new(
            Range::Single(Event::ObjectDestroyed),
            Locality::ProcessThread { pid, tid },
            Box::new(move |args| {
                if window != &args.hwnd
                // TODO find the correct combination to detect when the Window actually closed.
                /*|| unsafe { winuser::IsWindow(args.hwnd) == 0 } || (pid, tid) != window.pid_tid().unwrap_or((0, 0))*/
                {
                    return Ok(());
                }
                callback(window.clone())
            }),
        )?;
        Ok(Watcher { _hook })
    }
}
