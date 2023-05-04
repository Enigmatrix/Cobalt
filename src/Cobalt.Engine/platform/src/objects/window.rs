use common::errors::*;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows::Win32::Foundation::HWND;

use crate::win32;
use crate::objects::process::*;

/// Representation of a Window on the user's desktop
pub struct Window {
    inner: HWND
}

impl Window {
    /// Create a new [Window]
    pub fn new(hwnd: HWND) -> Self {
        Self { inner: hwnd }
    }

    /// Get the foreground [Window] shown on the user's desktop
    pub fn foreground() -> Option<Self> {
        let fg = unsafe { GetForegroundWindow() };
        if fg == HWND::default() {
            None
        } else {
            Some(Self::new(fg))
        }
    }

    /// Get [ProcessId] of [Window]
    pub fn pid(&self) -> Result<ProcessId> {
        let mut pid = 0;
        win32!(val: unsafe { GetWindowThreadProcessId(self.inner, Some(&mut pid)) })
            .context("get window pid")?;
        Ok(pid)
    }
}