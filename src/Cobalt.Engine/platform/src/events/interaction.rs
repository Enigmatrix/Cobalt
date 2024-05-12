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

pub enum InteractionChangedEvent {
    BecameIdle {
        at: Timestamp,
        recorded_mouse_clicks: u64,
        recorded_key_presses: u64,
    },
    BecameActive { at: Timestamp },
}

impl InteractionWatcher {
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<InteractionChangedEvent>> {
        let interaction_gap_duration = &at - &self.last_interaction;
        if self.active {
            if interaction_gap_duration > self.max_idle_duration {
                let recorded_mouse_clicks = self.mouse_clicks;
                let recorded_key_presses = self.key_presses;
                self.mouse_clicks = 0;
                self.key_presses = 0;
                self.last_interaction = at.clone();
                self.active = false;
                Ok(Some(InteractionChangedEvent::BecameIdle {
                    at,
                    recorded_mouse_clicks,
                    recorded_key_presses,
                }))
            } else {
                Ok(None)
            }
        } else {
            if interaction_gap_duration < self.max_idle_duration {
                self.active = true;
                Ok(Some(InteractionChangedEvent::BecameActive { at }))
            } else {
                Ok(None)
            }
        }
    }
}
