use crate::objects::{Timestamp, Window};

use super::Event;
use utils::channels::Sender;
use utils::errors::*;

pub struct ForegroundWatcher {
    window: Window,
}

impl ForegroundWatcher {
    pub fn new(foreground: Window) -> Result<Self> {
        Ok(Self { window: foreground })
    }

    pub fn trigger(&mut self, sender: &mut Sender<Event>, now: Timestamp) -> Result<()> {
        let foreground = Window::foreground();
        if let Some(foreground) = foreground && foreground != self.window {
            sender
                .send(Event::ForegroundSwitch {
                    at: now,
                    window: foreground.clone(),
                })
                .context("send foreground switch event")?;
            self.window = foreground;
        }
        Ok(())
    }
}
