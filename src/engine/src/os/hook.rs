use crate::errors::*;
use crate::os::prelude::*;
use std::collections::HashMap;
use std::future::*;
use std::task::{Context, Poll};

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Event {
    SystemForeground = winuser::EVENT_SYSTEM_FOREGROUND,
    ObjectDestroyed = winuser::EVENT_OBJECT_DESTROY,
}

pub enum Range {
    Single(Event),
    MinMax { min: Event, max: Event },
}

pub enum Locality {
    Global,
    ProcessThread { pid: ProcessId, tid: u32 },
}

#[derive(Debug)]
pub struct EventArgs {
    pub win_event_hook: HWINEVENTHOOK,
    pub event: DWORD,
    pub hwnd: HWND,
    pub id_object: LONG,
    pub id_child: LONG,
    pub id_event_thread: DWORD,
    pub dwms_event_time: DWORD,
}

#[derive(Debug)]
pub struct WinEventHook {
    hook: HWINEVENTHOOK,
}

trait EventHandler = Fn(EventArgs) -> Result<()>;
type EventContexts = HashMap<HWINEVENTHOOK, Box<dyn EventHandler>>;

static mut WIN_EVENT_HOOK_CONTEXTS: mem::MaybeUninit<EventContexts> = mem::MaybeUninit::uninit();

pub fn init_contexts() {
    unsafe {
        WIN_EVENT_HOOK_CONTEXTS = mem::MaybeUninit::new(HashMap::new());
    }
}

#[allow(clippy::mutable_key_type)]
unsafe fn contexts() -> &'static mut EventContexts {
    WIN_EVENT_HOOK_CONTEXTS.assume_init_mut()
}

impl WinEventHook {
    pub fn new(ev: Range, locality: Locality, handler: Box<dyn EventHandler>) -> Result<Self> {
        let (event_min, event_max) = match ev {
            Range::Single(e) => (e as u32, e as u32),
            Range::MinMax { min, max } => (min as u32, max as u32),
        };
        let (id_process, id_thread) = match locality {
            Locality::Global => (0, 0),
            Locality::ProcessThread { pid, tid } => (pid, tid),
        };

        let hook = expect!(non_null: {
            winuser::SetWinEventHook(
                event_min,
                event_max,
                ptr::null_mut(),
                Some(WinEventHook::handler),
                id_process,
                id_thread,
                winuser::WINEVENT_OUTOFCONTEXT,
            )
        })?;

        if unsafe { contexts().insert(hook, handler).is_some() } {
            panic!("Hook already exists");
        }
        Ok(WinEventHook { hook })
    }

    unsafe extern "system" fn handler(
        win_event_hook: HWINEVENTHOOK,
        event: DWORD,
        hwnd: HWND,
        id_object: LONG,
        id_child: LONG,
        id_event_thread: DWORD,
        dwms_event_time: DWORD,
    ) {
        let ret = contexts().get(&win_event_hook).unwrap();
        let handler = ret;
        (handler)(EventArgs {
            win_event_hook,
            event,
            hwnd,
            id_object,
            id_child,
            id_event_thread,
            dwms_event_time,
        })
        .chain_err(|| "Handler threw error")
        .unwrap();
    }
}

impl Drop for WinEventHook {
    fn drop(&mut self) {
        unsafe {
            let _ = contexts()
                .remove(&self.hook)
                .expect("Handler and Context should already exist");
        };
        expect!(true: winuser::UnhookWinEvent(self.hook)).unwrap();
    }
}

pub struct EventLoop {
    msg: winuser::MSG,
}

impl EventLoop {
    pub fn new() -> EventLoop {
        EventLoop { msg: default() }
    }
}

impl EventLoop {
    pub fn step(&mut self) -> Option<usize> {
        while unsafe {
            winuser::PeekMessageW(&mut self.msg, ptr::null_mut(), 0, 0, winuser::PM_REMOVE)
        } != 0
        {
            if self.msg.message == winuser::WM_QUIT {
                return Some(self.msg.wParam);
            }
            unsafe { winuser::TranslateMessage(&mut self.msg as *mut _) };
            unsafe { winuser::DispatchMessageW(&mut self.msg as *mut _) };
        }
        None
    }
}

impl Future for EventLoop {
    type Output = usize;
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<<Self as Future>::Output> {
        if let Some(exit) = self.step() {
            Poll::Ready(exit)
        } else {
            cx.waker().wake_by_ref(); // yield to scheduler
            Poll::Pending
        }
    }
}
