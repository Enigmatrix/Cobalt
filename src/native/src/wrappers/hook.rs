use crate::raw::*;
use crate::wrappers::*;
use crate::error::*;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::ptr;

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

    type EventHandler = Box<dyn Fn(EventArgs) -> anyhow::Result<()>>;
    type EventContexts = HashMap<HWINEVENTHOOK, EventHandler>;

    static mut WIN_EVENT_HOOK_CONTEXTS: MaybeUninit<EventContexts> = MaybeUninit::uninit();

    const WINEVENT_CONTEXT_INIT: usize = 512;

    pub fn init_contexts() {
        unsafe {
            WIN_EVENT_HOOK_CONTEXTS = MaybeUninit::new(HashMap::with_capacity(WINEVENT_CONTEXT_INIT));
        }
    }

    #[allow(clippy::mutable_key_type)]
    unsafe fn contexts() -> &'static mut EventContexts {
        WIN_EVENT_HOOK_CONTEXTS.assume_init_mut()
    }

    impl Hook {
        pub fn new(ev: Range, locality: Locality, handler: EventHandler) -> Result<Self, Win32Err> {
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

            if unsafe { contexts().insert(hook, handler).is_some() } {
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
            let handler = contexts().get(&win_event_hook).unwrap();
            (handler)(EventArgs {
                win_event_hook,
                event,
                hwnd,
                id_object,
                id_child,
                id_event_thread,
                dwms_event_time,
            })
            .expect("Handler threw error");
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

    #[repr(i32)]
    #[derive(Clone, Copy)]
    pub enum Event {
        KeyBoardLL = winuser::WH_KEYBOARD_LL,
        MouseLL = winuser::WH_MOUSE_LL,
    }

    pub struct Hook {
        hook: HHOOK
    }

    pub enum Locality {
        Global,
        Thread(u32)
    }

    impl Hook {
        pub fn new(ev: Event, locality: Locality, cb: winuser::HOOKPROC) -> Result<Hook, Win32Err> {
            let tid = match locality {
                Locality::Thread(x) => x,
                Locality::Global => 0
            };
            let hook = win32!(non_null: winuser::SetWindowsHookExW(ev as i32, cb, ptr::null_mut(), tid))?;
            Ok(Hook { hook })
        }
    }

    impl Drop for Hook {
        fn drop(&mut self) {
            win32!(non_zero: winuser::UnhookWindowsHookEx(self.hook)).unwrap();
        }
    }
}
