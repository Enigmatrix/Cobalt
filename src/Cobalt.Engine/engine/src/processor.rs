use common::channels::*;
use common::errors::*;

use data::Database;
use platform::objects::{Timestamp, Window};
use platform::watchers::{InteractionStateChange, WindowSession};

use crate::app_info_resolver::*;

pub enum ProcessorEvent {
    WindowSession(WindowSession),
    InteractionStateChange(InteractionStateChange),
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

    pub fn handle(event: ProcessorEvent) -> Result<()> {
        unimplemented!()
    }
}
