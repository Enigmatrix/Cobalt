/*use crate::errors::*;
use crate::os::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::*;
use tokio::sync::mpsc::*;

pub struct WindowClosedWatcher {
    windows: HashMap<Window, hook::WinEventHook>,
    sender: UnboundedSender<Window>
}

pub struct WindowClosedWatcherSink {
    recver: UnboundedReceiver<Window>,
}

impl WindowClosedWatcher {
    pub fn new() -> (WindowClosedWatcher, WindowClosedWatcherSink) {
        let (sender, recver) = unbounded_channel();
        let windows = HashMap::new();
        (WindowClosedWatcher {
            windows,
            sender
        }, WindowClosedWatcherSink {
           recver
        })
    }

    pub fn watch(&mut self, win: Window) -> Result<()> {
        let (pid, tid) = win.pid_tid()?;
        let hook = hook::WinEventHook::new(
            hook::Range::Single(hook::Event::SystemForeground),
            hook::Locality::ProcessThread { pid, tid },
            &move |args| {
                if win == args.hwnd {
                    sender.send(win).unwrap();
                    // dis2.borrow_mut().unwatch(win);
                }
                Ok(())
            },
        )?;
        let _ = self.windows.insert(win, hook);
        Ok(())
    }

    pub fn next_close(dis: &Rc<RefCell<Self>>) -> CloseFuture {
        CloseFuture { watcher: dis }
    }

    fn unwatch(&mut self, win: Window) {
        let existing = self.windows.remove_entry(&win);
        if existing.is_none() {
            panic!("Key {:?} is not pre-existing", win);
        }
    }
}*/
