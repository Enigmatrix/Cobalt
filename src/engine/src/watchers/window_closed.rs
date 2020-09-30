use crate::os::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::*;
use tokio::sync::mpsc::*;

pub struct WindowClosedWatcher<'a> {
    windows: Mutex<
        HashMap<
            Window,
            hook::WinEventHook<(
                Rc<RefCell<WindowClosedWatcher<'a>>>,
                UnboundedSender<Window>,
                Window,
            )>,
        >,
    >,
    pub recver: UnboundedReceiver<Window>,
    sender: UnboundedSender<Window>,
}

impl<'a> WindowClosedWatcher<'a> {
    pub fn new() -> WindowClosedWatcher<'a> {
        let (sender, recver) = unbounded_channel();
        let windows = Mutex::new(HashMap::new());
        WindowClosedWatcher {
            windows,
            recver,
            sender,
        }
    }

    pub fn watch(s: &Rc<RefCell<Self>>, win: Window) -> Result<(), crate::os::error::Error> {
        let sender = s.borrow_mut().sender.clone();
        let (pid, tid) = win.pid_tid()?;

        let hook = hook::WinEventHook::new(
            hook::Range::Single(hook::Event::SystemForeground),
            hook::Locality::Global,
            &(Rc::clone(&s), sender, win),
            |(s, sender, win), args| {
                if *win == args.hwnd {
                    sender.send(*win).unwrap();
                    s.borrow_mut().unwatch(*win);
                }
                Ok(())
            },
        )?;
        let s = s.borrow_mut();
        let mut windows = s.windows.lock().unwrap();
        // Don't check whether this is Some or None, because you can watch
        // the same window multiple times. Not that multiple notifications
        // will be sent, of course, only the latest hook registration will
        // be used to get the event
        let _ = windows.insert(win, hook);
        Ok(())
    }

    fn unwatch(&self, win: Window) {
        let mut windows = self.windows.lock().unwrap();
        let existing = windows.remove_entry(&win);
        if existing.is_none() {
            panic!("Key {:?} is not pre-existing", win);
        }
    }
}
