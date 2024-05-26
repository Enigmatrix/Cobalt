use util::config::Config;
use util::error::Result;

use crate::objects::{Duration, Timestamp};

/// Watches for user interaction and notifies when the user becomes idle or active,
/// including counting mouse_clicks and key_presses.
pub struct InteractionWatcher {
    max_idle_duration: Duration,
    active: bool,

    // the below will be modified by WinEvents
    last_interaction: Timestamp,
    mouse_clicks: u64,
    key_presses: u64,
}

/// Event for change in interaction state.
pub enum InteractionChangedEvent {
    BecameIdle {
        at: Timestamp,
        recorded_mouse_clicks: u64,
        recorded_key_presses: u64,
    },
    BecameActive {
        at: Timestamp,
    },
}

impl InteractionWatcher {
    /// Create a new [InteractionWatcher] with the specified [Config] and current [Timestamp].
    pub fn new(config: &Config, at: Timestamp) -> Self {
        Self {
            max_idle_duration: config.max_idle_duration().into(),
            active: true,
            last_interaction: at,
            mouse_clicks: 0,
            key_presses: 0,
        }
    }

    /// Poll for a new [`InteractionChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<InteractionChangedEvent>> {
        let interaction_gap_duration = at - self.last_interaction;
        if self.active {
            if interaction_gap_duration > self.max_idle_duration {
                let recorded_mouse_clicks = self.mouse_clicks;
                let recorded_key_presses = self.key_presses;
                self.mouse_clicks = 0;
                self.key_presses = 0;
                self.active = false;
                Ok(Some(InteractionChangedEvent::BecameIdle {
                    at,
                    recorded_mouse_clicks,
                    recorded_key_presses,
                }))
            } else {
                Ok(None)
            }
        } else if interaction_gap_duration < self.max_idle_duration {
            self.active = true;
            Ok(Some(InteractionChangedEvent::BecameActive { at }))
        } else {
            Ok(None)
        }
    }
}
