use crate::error::Win32Err;
use crate::raw::*;
use crate::wrappers::windows_hook::*;
use crate::wrappers::*;
use std::mem::transmute;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

static mut LATEST_INTERACTION: AtomicU32 = AtomicU32::new(0);
static mut MOUSE_HOOK: MaybeUninit<Hook> = MaybeUninit::uninit();
static mut KEYBOARD_HOOK: MaybeUninit<Hook> = MaybeUninit::uninit();

pub struct Watcher;

impl Watcher {
    pub fn begin() -> Result<(), Win32Err> {
        unsafe {
            MOUSE_HOOK.write(Hook::new::<LowLevelMouse>(
                Locality::Global,
                Watcher::mouse_event_callback,
            )?);
            KEYBOARD_HOOK.write(Hook::new::<LowLevelKeyboard>(
                Locality::Global,
                Watcher::keyboard_event_callback,
            )?);
        }
        Ok(())
    }

    pub fn last_interaction() -> Timestamp {
        Timestamp::from_event_millis(unsafe { LATEST_INTERACTION.load(Ordering::SeqCst) })
    }

    fn mouse_event_callback(
        code: i32,
        typ: <LowLevelMouse as WindowsHookEvent>::WParam,
        ev: <LowLevelMouse as WindowsHookEvent>::LParam,
    ) -> isize {
        unsafe {
            if code == 0 && ev.flags & 0x1 == 0 {
                LATEST_INTERACTION.fetch_max(ev.time, Ordering::SeqCst);
            }
            LowLevelMouse::call_next_hook(code, typ, ev)
        }
    }

    fn keyboard_event_callback(
        code: i32,
        typ: <LowLevelKeyboard as WindowsHookEvent>::WParam,
        ev: <LowLevelKeyboard as WindowsHookEvent>::LParam,
    ) -> isize {
        unsafe {
            if code == 0 && ev.flags & 0x1 == 0 {
                LATEST_INTERACTION.fetch_max(ev.time, Ordering::SeqCst);
            }
            LowLevelKeyboard::call_next_hook(code, typ, ev)
        }
    }
}
