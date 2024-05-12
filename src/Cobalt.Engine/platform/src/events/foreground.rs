use crate::objects::{Timestamp, Window};
use util::error::{Context, Result};

pub struct ForegroundEventWatcher {
    window: Window,
    title: String,
}

pub struct ForegroundChangedEvent {
    pub at: Timestamp,
    pub window: Window,
    pub title: String,
}

impl ForegroundEventWatcher {
    pub fn new(window: Window) -> Result<Self> {
        let title = window.title().context("title of initial fg window")?;
        Ok(Self { window, title })
    }

    pub fn poll(&mut self, at: Timestamp) -> Result<Option<ForegroundChangedEvent>> {
        if let Some(fg) = Window::foreground() {
            let title = fg.title().context("title of new fg window")?;
            if fg == self.window && title == self.title {
                return Ok(None);
            }

            self.window = fg.clone();
            self.title = title.clone();
            return Ok(Some(ForegroundChangedEvent { at, window: fg, title }));
        }
        Ok(None)
    }
}
