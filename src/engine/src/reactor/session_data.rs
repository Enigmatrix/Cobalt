use super::*;
use crate::errors::*;
use crate::os::prelude::*;

#[derive(Debug)]
pub struct SessionData {
    pub closed_watcher: WindowClosed,
    pub session: Session,
}

impl SessionData {
    pub fn create(state: &mut State, reactor: &Reactor, window: Window) -> Result<Self> {
        let closed_watcher = WindowClosed::watch(reactor.share(), window)?;

        let mut session = state.create_session(reactor, window)?;
        info!("NEW: {:?}", session);
        state.db.insert_session(&mut session)?;

        Ok(SessionData {
            session,
            closed_watcher,
        })
    }
}
