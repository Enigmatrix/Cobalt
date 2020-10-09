use crate::data::db::*;
use crate::data::entities::*;
use crate::errors::*;
use crate::os::prelude::*;
use crate::watchers::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::*;

#[derive(Debug)]
pub struct Processor {
    state: Rc<RefCell<ProcessorState>>,
}

#[derive(Debug)]
pub struct ProcessorState {
    windows: HashMap<Window, SessionData>,
    db: Database,
    last_switch: WindowSwitch,
}

#[derive(Debug)]
pub struct SessionData {
    closed_watcher: WindowClosedWatcher,
    session: Session
}

#[derive(Debug)]
pub enum Message {
    Switch(WindowSwitch),
    WindowClosed(Window),
}

impl Processor {
    pub fn new() -> Result<Self> {
        Ok(Processor {
            state: Rc::new(RefCell::new(ProcessorState {
                windows: HashMap::new(),
                db: Database::new(),
                last_switch: WindowSwitch {
                    window: Window::new(unsafe { winuser::GetForegroundWindow() })?, // TODO should be current fg window
                    time: Timestamp::from_ticks(0), // TODO should be current time
                },
            })),
        })
    }

    pub fn share(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }

    fn create_session(&self, window: Window) -> Result<Session> {
        Ok(Session {
            // TODO get this from somewhere
            id: 0,
            app_id: 0,
            arguments: String::new(),
            title: String::new(),
        })
    }

    fn create_window_closed_watcher(&self, window: Window) -> Result<WindowClosedWatcher> {
        let window_closed = WindowClosedWatcher::new(self.share(), window)?;
        Ok(window_closed)
    }

    #[instrument(skip(self))]
    pub fn process(&self, msg: Message) -> Result<()> {
        let mut state = self.state.borrow_mut();
        let ProcessorState {
            last_switch: prev,
            windows,
            db,
            ..
        } = &mut *state;
        match msg {
            Message::Switch(curr) => {
                let window_data = windows.get_mut(&prev.window);

                let session = if let Some(SessionData { session: existing_session, .. }) = window_data {
                    existing_session
                } else {
                    let mut session = self.create_session(prev.window)?;
                    db.insert_session(&mut session)?;

                    let closed_watcher = match self.create_window_closed_watcher(prev.window) {
                        Err(Error(ErrorKind::WindowAlreadyClosed(_), _)) => {
                            return Ok(());
                        }
                        x => x,
                    }?;
                    windows.insert(prev.window, SessionData { session, closed_watcher });
                    let SessionData { session, .. } = windows.get_mut(&prev.window).unwrap();

                    session
                };

                let mut usage = Usage {
                    id: 0,
                    start: prev.time,
                    end: curr.time,
                    during_idle: true, // TODO derive from idle watcher
                    session_id: session.id,
                };
                db.insert_usage(&mut usage)?;
                // TODO push to grpc system


                *prev = curr;
            }
            Message::WindowClosed(window) => {
                warn!(
                    "window closed: {}",
                    window.title().unwrap_or("NOT_FOUND".to_string())
                );
                windows.remove(&window);
            }
        }
        Ok(())
    }
}
