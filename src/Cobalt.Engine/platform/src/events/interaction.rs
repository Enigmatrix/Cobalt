use crate::objects::{Duration, Timestamp};
use util::error::Result;

pub struct InteractionWatcher {
    max_idle_duration: Duration,
    active: bool,

    // the below will be modified by WinEvents
    last_interaction: Timestamp,
    mouse_clicks: u64,
    key_presses: u64,
}

pub enum InteractionChangeEvent {
    BecameIdle {
        recorded_mouse_clicks: u64,
        recorded_key_presses: u64,
    },
    BecameActive,
}

impl InteractionWatcher {
    pub fn poll(&mut self) -> Result<Option<InteractionChangeEvent>> {
        let now = Timestamp::now();
        let interaction_gap_duration = &now - &self.last_interaction;
        if self.active {
            if interaction_gap_duration > self.max_idle_duration {
                let recorded_mouse_clicks = self.mouse_clicks;
                let recorded_key_presses = self.key_presses;
                self.mouse_clicks = 0;
                self.key_presses = 0;
                self.last_interaction = now;
                self.active = false;
                Ok(Some(InteractionChangeEvent::BecameIdle {
                    recorded_mouse_clicks,
                    recorded_key_presses,
                }))
            } else {
                Ok(None)
            }
        } else {
            if interaction_gap_duration < self.max_idle_duration {
                self.active = true;
                Ok(Some(InteractionChangeEvent::BecameActive))
            } else {
                Ok(None)
            }
        }
    }
}
