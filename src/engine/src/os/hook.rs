use crate::os::prelude::*;
use once_cell::sync::OnceCell;
use std::any::Any;
use std::cell::RefCell;
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
    ProcessThread { pid: u32, tid: u32 },
}

pub struct EventArgs {
    win_event_hook: HWINEVENTHOOK,
    event: DWORD,
    hwnd: HWND,
    id_object: LONG,
    id_child: LONG,
    id_event_thread: DWORD,
    dwms_event_time: DWORD,
}

pub struct WinEventHook<C> {
    hook: HWINEVENTHOOK,
    handler: fn(&C, EventArgs) -> Result<(), crate::os::error::Error>,
    context: C
}

type EventContexts = HashMap<HWINEVENTHOOK, *const c_void>;

static mut WIN_EVENT_HOOK_CONTEXTS: Option<EventContexts> = None;

unsafe fn contexts() -> &'static mut EventContexts {
    if WIN_EVENT_HOOK_CONTEXTS.is_none() {
        WIN_EVENT_HOOK_CONTEXTS = Some(HashMap::new());
    }
    WIN_EVENT_HOOK_CONTEXTS.as_mut().unwrap()
}

impl<C> WinEventHook<C> {

    pub fn new(
        ev: Range,
        locality: Locality,
        context: C,
        handler: fn(&C, EventArgs) -> Result<(), crate::os::error::Error>
    ) -> Result<Box<Self>, crate::os::error::Error> {
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
                Some(WinEventHook::<C>::handler),
                id_process,
                id_thread,
                winuser::WINEVENT_OUTOFCONTEXT,
            )
        })?;
        let ret = Box::new(WinEventHook { hook, handler, context });

        unsafe { contexts().insert(hook, &*ret as *const _ as *const c_void).expect_none("Hook already exists") };
        Ok(ret)
    }

    pub fn handle(&self, args: EventArgs) {
        (self.handler)(&self.context, args).expect("Handler threw error");
    }

    unsafe extern "system" fn handler(
        win_event_hook: HWINEVENTHOOK,
        event: DWORD,
        hwnd: HWND,
        id_object: LONG,
        id_child: LONG,
        id_event_thread: DWORD,
        dwms_event_time: DWORD
    ) {
        let hook = &*(contexts().get(&win_event_hook).unwrap() as *const _ as *const Box<WinEventHook<C>>);
        hook.handle(EventArgs {
            win_event_hook,
            event,
            hwnd,
            id_object,
            id_child,
            id_event_thread,
            dwms_event_time,
        });
    }
}

impl<C> Drop for WinEventHook<C> {
    fn drop(&mut self) {
        unsafe { contexts().remove(&self.hook).expect("Handler and Context should already exist") };
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
