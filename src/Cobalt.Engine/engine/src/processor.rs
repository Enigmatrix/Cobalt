use common::channels::*;
use common::errors::*;

use data::Database;
use platform::objects::{Timestamp, Window};
use platform::watchers::{InteractionStateChange, WindowSession};

use crate::app_info_resolver::*;

pub enum ProcessorEvent {
    WindowSession {
        at: Timestamp,
        change: WindowSession,
    },
    InteractionStateChange {
        at: Timestamp,
        change: InteractionStateChange,
    },
}

pub struct Processor {}

impl Processor {
    pub fn new(
        foreground: Window,
        start: Timestamp,
        db: Database,
        app_info_tx: Sender<AppInfoRequest>,
    ) -> Processor {
        unimplemented!()
    }

    pub fn handle(&mut self, event: ProcessorEvent) -> Result<()> {
        match event {
            ProcessorEvent::WindowSession {
                at,
                change: WindowSession { window, title },
            } => {}
            ProcessorEvent::InteractionStateChange {
                at,
                change: InteractionStateChange::Active,
            } => {}
            ProcessorEvent::InteractionStateChange {
                at,
                change:
                    InteractionStateChange::Idle {
                        mouseclicks,
                        keystrokes,
                    },
            } => {}
        }
        Ok(())
    }
}
