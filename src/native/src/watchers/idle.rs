use crate::wrappers::windows_hook::*;
use crate::wrappers::*;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU32, Ordering};
use util::*;

static mut LATEST_INTERACTION: AtomicU32 = AtomicU32::new(0);
static mut MOUSE_HOOK: MaybeUninit<Hook> = MaybeUninit::uninit();
static mut KEYBOARD_HOOK: MaybeUninit<Hook> = MaybeUninit::uninit();

pub struct Watcher(Timer);

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum IdleStatus {
    Active,
    Idle,
}

impl Watcher {
    pub fn new(
        idle_dur: Duration,
        mut callback: impl FnMut(IdleStatus) -> Result<()>,
    ) -> Result<Watcher> {
        let mut last_status = IdleStatus::Active;

        let timer = Timer::new(
            1_005, // duration of 1 second
            Box::new(move |now| {
                let last = Watcher::last_interaction();
                if last_status == IdleStatus::Idle && now - last <= idle_dur {
                    last_status = IdleStatus::Active;
                    callback(last_status.clone()).with_context(|| "Active callback")?;
                }
                if last_status == IdleStatus::Active && now - last > idle_dur {
                    last_status = IdleStatus::Idle;
                    callback(last_status.clone()).with_context(|| "Idle callback")?;
                }
                Ok(())
            }),
        )
        .with_context(|| "Create timer for Idle watcher")?;

        Ok(Watcher(timer))
    }

    pub fn begin() -> Result<()> {
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
            if code == 0 && ev.flags & 0x10 == 0 {
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
            if code == 0 && ev.flags & 0x10 == 0 {
                LATEST_INTERACTION.fetch_max(ev.time, Ordering::SeqCst);
            }
            LowLevelKeyboard::call_next_hook(code, typ, ev)
        }
    }
}
