use crate::data::db::*;
use crate::data::entities::*;
use crate::errors::*;
use crate::os::prelude::*;
use crate::watchers::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tokio::sync::mpsc;
use tracing::*;

#[derive(Debug)]
pub struct Processor {
    state: Rc<RefCell<ProcessorState>>,
}

#[derive(Debug)]
pub struct ProcessorState {
    windows: HashMap<Window, SessionData>,
    processes: HashMap<ProcessId, AppData>,
    db: Database,
    prev_switch: WindowSwitch,
    process_exits: (mpsc::UnboundedSender<ProcessId>, mpsc::UnboundedReceiver<ProcessId>)
}

#[derive(Debug)]
pub struct SessionData {
    closed_watcher: WindowClosed,
    session: Session
}

#[derive(Debug)]
pub struct AppData {
    exit_watcher: ProcessExit,
    process: Process,
    app: App
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
                processes: HashMap::new(),
                db: Database::new(),
                prev_switch: WindowSwitch {
                    window: Window::new(unsafe { winuser::GetForegroundWindow() })?, // TODO should be current fg window
                    time: Timestamp::from_ticks(0), // TODO should be current time
                },
                process_exits: mpsc::unbounded_channel()
            })),
        })
    }

    pub fn share(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }

    pub fn process(&self, msg: Message) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.process(msg, &self)
    }
}

impl ProcessorState {

    fn create_session(&mut self, window: Window/*, app: &AppData*/) -> Result<Session> {
        Ok(Session {
            // TODO get this from somewhere
            id: 0,
            app_id: 0,
            arguments: String::new(),
            title: String::new(),
        })
    }

    #[instrument(skip(self, processor))]
    pub fn process(&mut self, msg: Message, processor: &Processor) -> Result<()> {
        match msg {
            Message::Switch(curr_switch) => {
                if self.windows.get(&curr_switch.window).is_none() {
                    let mut session = self.create_session(curr_switch.window)?;
                    self.db.insert_session(&mut session)?;

                    let closed_watcher = match WindowClosed::watch(processor.share(), curr_switch.window) {
                        Err(Error(ErrorKind::WindowAlreadyClosed(_), _)) => {
                            return Ok(());
                        }
                        x => x,
                    }?;
                    self.windows.insert(curr_switch.window, SessionData { session, closed_watcher });
                }

                let SessionData { session, ..} = self.windows.get_mut(&curr_switch.window).unwrap();

                let mut usage = Usage {
                    id: 0,
                    start: self.prev_switch.time,
                    end: curr_switch.time,
                    during_idle: true, // TODO derive from idle watcher
                    session_id: session.id,
                };
                self.db.insert_usage(&mut usage)?;
                // TODO push to grpc system


                self.prev_switch = curr_switch;
            }
            Message::WindowClosed(window) => {
                warn!(
                    "window closed: {}",
                    window.title().unwrap_or("NOT_FOUND".to_string())
                );
                self.windows.remove(&window);
            }
        }
        Ok(())
    }
}