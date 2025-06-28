use std::cell::RefCell;
use std::ffi::c_void;

use util::ds::SmallHashMap;
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS};

use crate::win32;

/// Windows Event Hook
/// ref: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook
pub struct WinEventHook {
    hook: HWINEVENTHOOK,
}

/// Windows Event
pub enum WinEvent {
    /// A single event
    Event(u32),
    /// A range of events
    Range(u32, u32),
}

/// Target for the hook - pid/tid
pub enum Target {
    /// All processes/threads
    All,
    /// A specific process/thread
    Id(u32),
}

type Callback = Box<dyn Fn(HWINEVENTHOOK, u32, HWND, i32, i32, u32, u32) + Send + Sync>;

thread_local! {
    static CALLBACKS: RefCell<SmallHashMap<*mut c_void, Callback>> = RefCell::new(SmallHashMap::new());
}

impl WinEventHook {
    /// Create a new [WinEventHook]
    pub fn new(
        event: WinEvent,
        process: Target,
        thread: Target,
        callback: Callback,
    ) -> Result<Self> {
        let (event_min, event_max) = match event {
            WinEvent::Event(event) => (event, event),
            WinEvent::Range(event_min, event_max) => (event_min, event_max),
        };
        let pid = match process {
            Target::All => 0,
            Target::Id(pid) => pid,
        };
        let tid = match thread {
            Target::All => 0,
            Target::Id(thread) => thread,
        };
        let hook = win32!(ptr: unsafe {
            SetWinEventHook(
                event_min,
                event_max,
                None,
                Some(Self::callback),
                pid,
                tid,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            ).0
        })?;
        let hook = HWINEVENTHOOK(hook);

        CALLBACKS.with_borrow_mut(|map| {
            map.insert(hook.0, callback);
        });
        Ok(Self { hook })
    }

    unsafe extern "system" fn callback(
        hwineventhook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        id_object: i32,
        id_child: i32,
        id_event_thread: u32,
        dwms_event_time: u32,
    ) {
        CALLBACKS.with_borrow(|map| {
            if let Some(callback) = map.get(&hwineventhook.0) {
                callback(
                    hwineventhook,
                    event,
                    hwnd,
                    id_object,
                    id_child,
                    id_event_thread,
                    dwms_event_time,
                );
            }
        });
    }
}

impl Drop for WinEventHook {
    fn drop(&mut self) {
        unsafe {
            UnhookWinEvent(self.hook)
                .ok()
                .context("unhook win event hook")
                .warn();
            CALLBACKS.with_borrow_mut(|map| {
                map.remove(&self.hook.0);
            });
        };
    }
}
