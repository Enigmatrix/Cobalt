use crate::raw::*;
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
            Range::Single(Event::ObjectNameChange),
            Locality::ProcessThread { pid, tid },
            Box::new(move |args| {
                if window != &args.hwnd
                    || args.id_object != winuser::OBJID_WINDOW
                    || args.id_child != winuser::CHILDID_SELF
                {
                    return Ok(());
                }
                callback(window.clone())
            }),
        )?;
        Ok(Watcher { _hook })
    }
}
