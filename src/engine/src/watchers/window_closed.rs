use crate::os::*;
use std::collections::HashMap;
use std::sync::*;
use tokio::sync::mpsc::*;

pub struct WindowClosedWatcher<'a> {
    windows: Mutex<HashMap<Window, hook::WinEventHook<'a>>>,
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

    pub fn watch(&'a self, win: Window) -> Result<(), crate::os::Error> {
        let sender = self.sender.clone();
        let (pid, tid) = win.pid_tid()?;
        dbg!("watching...");
        dbg!(pid, tid);
        let hook = hook::WinEventHook::new(
            hook::Type::Single(hook::Event::ObjectDestroyed),
            hook::Locality::ProcessThread { pid, tid },
            move |_win_event_hook: HWINEVENTHOOK,
                  _event: DWORD,
                  handle: HWND,
                  id_object: LONG,
                  _id_child: LONG,
                  _id_event_thread: DWORD,
                  _dwms_event_time: DWORD| {
                if id_object == winuser::OBJID_WINDOW && win == handle {
                    sender.send(win).unwrap();
                    self.unwatch(win);
                }
            },
        )?;
        dbg!("hook");
        let mut windows = self.windows.lock().unwrap();
        // Don't check whether this is Some or None, because you can watch
        // the same window multiple times. Not that multiple notifications
        // will be sent, of course, only the latest hook registration will
        // be used to get the event
        let _ = windows.insert(win, hook);
        dbg!("end");
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
