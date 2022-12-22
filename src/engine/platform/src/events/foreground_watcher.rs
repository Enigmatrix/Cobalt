use super::{Event, TotalWatcher, WinEventArgs, WinEventHook};
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

    pub fn trigger(&self, total: &TotalWatcher, args: WinEventArgs) -> Result<()> {
        total.sender
            .send(Event::ForegroundSwitch {
                hwnd: args.hwnd,
                timestamp: args.dwmseventtime,
            })
            .context("send foreground switch event")?;
        Ok(())
    }
}
