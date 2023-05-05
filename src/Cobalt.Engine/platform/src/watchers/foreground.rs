use crate::objects::Window;
use common::errors::*;

#[derive(Clone)]
pub struct WindowSession {
    pub window: Window,
    pub title: String,
}

/// [Foreground] watcher (window, title)
pub struct Foreground {
    current: WindowSession,
}

impl Foreground {
    /// Create a new [Foreground] watcher
    pub fn new(current_window: Window) -> Result<Self> {
        let title = current_window
            .title()
            .context("get foreground window title")?;
        let current = WindowSession {
            window: current_window,
            title,
        };
        Ok(Foreground { current })
    }

    /// Check if there is [WindowSession] change being triggered
    pub fn trigger(&mut self) -> Result<Option<WindowSession>> {
        let fg = Window::foreground();
        if let Some(fg) = fg {
            let title = fg
                .title()
                .context("get foreground window title in trigger")?;
            if fg != self.current.window || title != self.current.title {
                self.current = WindowSession { window: fg, title };
                Ok(Some(self.current.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
