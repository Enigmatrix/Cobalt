use crate::error::*;
use crate::raw::*;
use crate::wrappers::*;
use std::collections::HashMap;
use std::default::default;
use std::future::Future;
use std::mem::{transmute, MaybeUninit};
use std::pin::Pin;
use std::ptr;
use std::task::{Context as FutContext, Poll};
use util::*;

pub mod winevent {
    use super::*;

    #[repr(u32)]
    #[derive(Clone, Copy)]
    pub enum Event {
        SystemForeground = winuser::EVENT_SYSTEM_FOREGROUND,
        ObjectDestroyed = winuser::EVENT_OBJECT_DESTROY,
    }

    pub enum Range {
        Single(Event),
        // MinMax { min: Event, max: Event },
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
    pub struct Hook {
        hook: HWINEVENTHOOK,
    }
    unsafe impl Send for Hook {}
    unsafe impl Sync for Hook {}

    type EventHandler = Box<dyn Fn(EventArgs) -> util::Result<()>>;
    type EventContexts = HashMap<HWINEVENTHOOK, EventHandler>;

    static mut WIN_EVENT_HOOK_CONTEXTS: MaybeUninit<EventContexts> = MaybeUninit::uninit();

    const WINEVENT_CONTEXT_INIT: usize = 512;

    pub fn init_contexts() {
        unsafe {
            WIN_EVENT_HOOK_CONTEXTS =
                MaybeUninit::new(HashMap::with_capacity(WINEVENT_CONTEXT_INIT));
        }
    }

    #[allow(clippy::mutable_key_type)]
    unsafe fn contexts() -> &'static mut EventContexts {
        WIN_EVENT_HOOK_CONTEXTS.assume_init_mut()
    }

    impl Hook {
        pub fn new<'a>(
            ev: Range,
            locality: Locality,
            handler: Box<dyn 'a + FnMut(EventArgs) -> util::Result<()>>,
        ) -> Result<Self, Win32Err> {
            let (event_min, event_max) = match ev {
                Range::Single(e) => (e as u32, e as u32),
                // Range::MinMax { min, max } => (min as u32, max as u32),
            };
            let (id_process, id_thread) = match locality {
                Locality::Global => (0, 0),
                Locality::ProcessThread { pid, tid } => (pid, tid),
            };

            let hook = win32!(non_null: {
                winuser::SetWinEventHook(
                    event_min,
                    event_max,
                    ptr::null_mut(),
                    Some(Hook::handler),
                    id_process,
                    id_thread,
                    winuser::WINEVENT_OUTOFCONTEXT,
                )
            })?;

            if unsafe { contexts().insert(hook, transmute(handler)).is_some() } {
                panic!("Hook already exists");
            }
            Ok(Hook { hook })
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
            let handler = contexts()
                .get(&win_event_hook)
                .expect("Context found for Hook");
            (handler)(EventArgs {
                win_event_hook,
                event,
                hwnd,
                id_object,
                id_child,
                id_event_thread,
                dwms_event_time,
            })
            .unwrap_or_exit();
        }
    }

    impl Drop for Hook {
        fn drop(&mut self) {
            unsafe {
                let _ = contexts()
                    .remove(&self.hook)
                    .expect("Handler and Context should already exist");
            };
            win32!(non_zero: winuser::UnhookWinEvent(self.hook)).unwrap();
        }
    }
}

pub mod windows_hook {
    use super::*;

    pub trait WindowsHookEvent {
        type WParam;
        type LParam;

        fn event() -> i32;
    }

    pub struct LowLevelKeyboard;
    impl WindowsHookEvent for LowLevelKeyboard {
        type LParam = &'static mut winuser::KBDLLHOOKSTRUCT;
        type WParam = usize;

        fn event() -> i32 {
            winuser::WH_KEYBOARD_LL
        }
    }

    pub struct LowLevelMouse;
    impl WindowsHookEvent for LowLevelMouse {
        type LParam = &'static mut winuser::MSLLHOOKSTRUCT;
        type WParam = usize;

        fn event() -> i32 {
            winuser::WH_MOUSE_LL
        }
    }

    pub struct Hook {
        hook: HHOOK,
    }
    unsafe impl Send for Hook {}
    unsafe impl Sync for Hook {}

    pub enum Locality {
        Global,
        Thread(u32),
    }

    impl Hook {
        pub fn new<E: WindowsHookEvent>(
            locality: Locality,
            cb: fn(i32, E::WParam, E::LParam) -> isize,
        ) -> Result<Hook, Win32Err> {
            let tid = match locality {
                Locality::Thread(x) => x,
                Locality::Global => 0,
            };
            let hook = win32!(
                non_null:
                    winuser::SetWindowsHookExW(
                        E::event(),
                        Some(transmute(cb)),
                        ptr::null_mut(),
                        tid,
                    )
            )?;
            Ok(Hook { hook })
        }
    }

    impl Drop for Hook {
        fn drop(&mut self) {
            win32!(non_zero: winuser::UnhookWindowsHookEx(self.hook))
                .with_context(|| "Unhooking")
                .unwrap_or_exit();
        }
    }

    // TODO callnexthookex definition here
}

pub struct EventLoop {
    msg: winuser::MSG,
}

impl Default for EventLoop {
    fn default() -> Self {
        EventLoop::new()
    }
}

impl EventLoop {
    pub fn new() -> EventLoop {
        EventLoop { msg: default() }
    }
}

impl EventLoop {
    pub fn step_peek(&mut self) -> Option<usize> {
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

    pub fn step(&mut self) -> Option<usize> {
        if unsafe { winuser::GetMessageW(&mut self.msg, ptr::null_mut(), 0, 0) } != 0 {
            if self.msg.message == winuser::WM_QUIT {
                return Some(self.msg.wParam);
            }
            unsafe { winuser::TranslateMessage(&mut self.msg as *mut _) };
            unsafe { winuser::DispatchMessageW(&mut self.msg as *mut _) };
        }
        None
    }

    pub fn run(&mut self) -> Option<usize> {
        loop {
            if let Some(ex) = self.step() {
                return Some(ex);
            }
        }
    }
}

impl Future for EventLoop {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutContext<'_>) -> Poll<usize> {
        if let Some(exit) = self.step_peek() {
            Poll::Ready(exit)
        } else {
            cx.waker().wake_by_ref(); // yield to scheduler
            Poll::Pending
        }
    }
}
