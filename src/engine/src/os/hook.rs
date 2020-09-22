use crate::os::*;
use libffi::high::*;
use std::future::*;
use std::task::{Context, Poll};

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Event {
    SystemForeground = winuser::EVENT_SYSTEM_FOREGROUND,
    ObjectDestroyed = winuser::EVENT_OBJECT_DESTROY,
}

pub enum Type {
    Single(Event),
    Range { min: Event, max: Event },
}

pub enum Locality {
    Global,
    ProcessThread { pid: u32, tid: u32 },
}

pub trait WinEventCallbackFn = Fn(HWINEVENTHOOK, DWORD, HWND, LONG, LONG, DWORD, DWORD);

struct Callback<'a> {
    _closure: Box<dyn WinEventCallbackFn + 'a>,
    _ffi: Closure7<'a, HWINEVENTHOOK, DWORD, HWND, LONG, LONG, DWORD, DWORD, ()>,
}

impl<'a> Callback<'a> {
    fn new<F>(closure: F) -> Callback<'a>
    where
        F: 'a + WinEventCallbackFn,
    {
        let _closure = Box::new(closure);
        let _ffi = Closure7::new::<F>(unsafe { mem::transmute(&*_closure) });
        Callback { _closure, _ffi }
    }

    fn ptr(&self) -> winuser::WINEVENTPROC {
        Some(unsafe { mem::transmute(*self._ffi.code_ptr()) })
    }
}

pub struct WinEventHook<'a> {
    hook: HWINEVENTHOOK,
    _callback: Callback<'a>,
}

impl<'a> WinEventHook<'a> {
    pub fn new<F>(ev: Type, locality: Locality, handler: F) -> Result<Self, crate::os::error::Error>
    where
        F: 'a + WinEventCallbackFn,
    {
        let (event_min, event_max) = match ev {
            Type::Single(e) => (e as u32, e as u32),
            Type::Range { min, max } => (min as u32, max as u32),
        };
        let (id_process, id_thread) = match locality {
            Locality::Global => (0, 0),
            Locality::ProcessThread { pid, tid } => (pid, tid),
        };
        let _callback = Callback::new(handler);

        let hook = expect!(non_null: {
            winuser::SetWinEventHook(
                event_min,
                event_max,
                ptr::null_mut(),
                _callback.ptr(),
                id_process,
                id_thread,
                winuser::WINEVENT_OUTOFCONTEXT,
            )
        })?;
        Ok(WinEventHook { hook, _callback })
    }
}

impl<'a> Drop for WinEventHook<'a> {
    fn drop(&mut self) {
        expect!(true: winuser::UnhookWinEvent(self.hook)).unwrap();
    }
}

pub struct EventLoop {
    msg: winuser::MSG,
}

impl EventLoop {
    pub fn new() -> EventLoop {
        EventLoop {
            msg: Default::default(),
        }
    }
}

impl Future for EventLoop {
    type Output = usize;
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<<Self as Future>::Output> {
        while unsafe {
            winuser::PeekMessageW(&mut self.msg, ptr::null_mut(), 0, 0, winuser::PM_REMOVE)
        } != 0
        {
            if self.msg.message == winuser::WM_QUIT {
                return Poll::Ready(self.msg.wParam);
            }
            unsafe { winuser::TranslateMessage(&mut self.msg as *mut _) };
            unsafe { winuser::DispatchMessageW(&mut self.msg as *mut _) };
        }
        cx.waker().wake_by_ref(); // yield to scheduler
        Poll::Pending
    }
}
