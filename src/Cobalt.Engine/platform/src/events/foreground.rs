use crate::objects::Window;
use util::error::{Context, Result};

pub struct ForegroundEventWatcher {
    window: Window,
    title: String,
}

pub struct ForegroundChangedEvent {
    window: Window,
    title: String,
}

impl ForegroundEventWatcher {
    pub fn new() -> Result<Self> {
        let window = loop {
            if let Some(window) = Window::foreground() {
                break window;
            }
        };
        let title = window.title().context("title of initial fg window")?;
        Ok(Self { window, title })
    }

    pub fn poll(&mut self) -> Result<Option<ForegroundChangedEvent>> {
        if let Some(fg) = Window::foreground() {
            let title = fg.title().context("title of new fg window")?;
            if fg == self.window && title == self.title {
                return Ok(None);
            }

            self.window = fg.clone();
            self.title = title.clone();
            return Ok(Some(ForegroundChangedEvent { window: fg, title }));
        }
        Ok(None)
    }
}