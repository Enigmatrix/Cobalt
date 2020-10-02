use crate::os::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::*;
use tokio::sync::mpsc::*;

pub struct WindowClosedWatcher {
    windows: HashMap<Window, hook::WinEventHook>,
    recver: UnboundedReceiver<Window>,
    sender: UnboundedSender<Window>,
}

impl WindowClosedWatcher {
    pub fn new() -> WindowClosedWatcher {
        let (sender, recver) = unbounded_channel();
        let windows = HashMap::new();
        WindowClosedWatcher {
            windows,
            recver,
            sender,
        }
    }

    pub fn watch(dis: &Rc<RefCell<Self>>, win: Window) -> Result<(), crate::os::error::Error> {
        let sender = dis.borrow_mut().sender.clone();
        let (pid, tid) = win.pid_tid()?;
        let dis2 = Rc::clone(&dis);
        let hook = hook::WinEventHook::new(
            hook::Range::Single(hook::Event::SystemForeground),
            hook::Locality::ProcessThread { pid, tid },
            &move |args| {
                if win == args.hwnd {
                    sender.send(win).unwrap();
                    dis2.borrow_mut().unwatch(win);
                }
                Ok(())
            },
        )?;
        let _ = dis.borrow_mut().windows.insert(win, hook);
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
}

pub struct CloseFuture<'a> {
    watcher: &'a Rc<RefCell<WindowClosedWatcher>>
}

impl<'a> std::future::Future for CloseFuture<'a> {
    type Output = Option<Window>;
    fn poll(self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<<Self as std::future::Future>::Output> {
        let mut bwatcher = self.watcher.borrow_mut();
        match bwatcher.recver.try_recv() {
            Ok(window) => {
                drop(bwatcher);
                std::task::Poll::Ready(Some(window))
            },
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                drop(bwatcher);
                std::task::Poll::Pending
             }
            Err(tokio::sync::mpsc::error::TryRecvError::Closed) => { std::task::Poll::Ready(None) }
        }
    }
}