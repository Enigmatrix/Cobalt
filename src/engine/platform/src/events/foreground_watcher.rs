use utils::errors::*;

use super::{CallbackRef, Event};
use crate::objects::{Timestamp, Window};

pub struct ForegroundWatcher {
    window: Window,
}

impl ForegroundWatcher {
    pub fn new(foreground: Window) -> Result<Self> {
        Ok(Self { window: foreground })
    }

    pub fn trigger(&mut self, cb: CallbackRef, now: Timestamp) -> Result<()> {
        let foreground = Window::foreground();
        if let Some(foreground) = foreground && foreground != self.window {
            cb(Event::ForegroundSwitch {
                    at: now,
                    window: foreground.clone(),
                })
                .context("callback foreground switch event")?;
            self.window = foreground;
        }
        Ok(())
    }
}
