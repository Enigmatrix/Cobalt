use crate::objects::{Timestamp, Window};

use super::{Event, WinEventArgs, WinEventHook};
use utils::channels::Sender;
use utils::errors::*;
use windows::Win32::UI::Accessibility::WINEVENTPROC;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND;

pub struct ForegroundWatcher {
    _hook: WinEventHook,
}

impl ForegroundWatcher {
    pub fn new(proc: WINEVENTPROC) -> Result<Self> {
        let _hook =
            WinEventHook::global(EVENT_SYSTEM_FOREGROUND, proc).context("setup foreground hook")?;
        Ok(Self { _hook })
    }

    pub fn trigger(&self, sender: &mut Sender<Event>, args: WinEventArgs) -> Result<()> {
        sender
            .send(Event::ForegroundSwitch {
                at: Timestamp::from_event_millis(args.dwmseventtime),
                window: Window::new(args.hwnd),
            })
            .context("send foreground switch event")?;
        Ok(())
    }
}
