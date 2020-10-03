use crate::data::db::*;
use crate::data::entities::*;
use crate::errors::*;
use crate::os::prelude::*;
use tracing::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Processor {
    state: Rc<RefCell<ProcessorState>>,
}

#[derive(Debug)]
pub struct ProcessorState {
    windows: HashMap<Window, (hook::WinEventHook, Session)>,
    db: Database,
    last_switch: WindowSwitch,
}

#[derive(Copy, Clone, Debug)]
pub struct WindowSwitch {
    pub time: Timestamp,
    pub window: Window,
}

#[derive(Debug)]
pub enum Message {
    Switch { switch: WindowSwitch },
    WindowClosed { window: Window },
}

impl Processor {
    pub fn new() -> Result<Self> {
        Ok(Processor {
            state: Rc::new(RefCell::new(ProcessorState {
                windows: HashMap::new(),
                db: Database::new(),
                last_switch: WindowSwitch {
                    window: Window::new(unsafe { winuser::GetForegroundWindow() })?, // TODO should be current fg window
                    time: Timestamp::from_ticks(0),        // TODO should be current time
                },
            })),
        })
    }

    pub fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }

    #[instrument]
    pub fn process(&self, msg: Message) -> Result<()> {
        // info!("BEGIN");
        let mut state = self.state.borrow_mut();
        let last_switch = state.last_switch.clone();
        match msg {
            Message::Switch { switch } => {
                let window = last_switch.window;
                let window_data = state.windows.get_mut(&window);

                let session = if let Some((_, existing_session)) = window_data {
                    existing_session
                } else {
                    let mut new_session = Session {
                        // TODO get this from somewhere
                        id: 0,
                        app_id: 0,
                        arguments: String::new(),
                        title: String::new(),
                    };

                    state.db.insert_session(&mut new_session)?;

                    let (pid, tid) = match switch.window.pid_tid() {
                        Err(Error(ErrorKind::Win32(1400), _)) => {
                            warn!("early return (pid/tid) inaccessible");
                            return Ok(());
                        },
                        x => x
                    }
                    .chain_err(|| format!("Unable to get pid/tid for {:?}", window))?;

                    let self2 = Processor::clone(&self);
                    let closed = hook::WinEventHook::new(
                        hook::Range::Single(hook::Event::ObjectDestroyed),
                        hook::Locality::ProcessThread { pid, tid },
                        Box::new(move |args| {
                            if switch.window == args.hwnd {
                                self2.process(Message::WindowClosed {
                                    window: switch.window
                                })?;
                            }
                            Ok(())
                        }),
                    )
                    .chain_err(|| format!("Unable to set window closed hook for {:?}", window))?;

                    state.windows.insert(window, (closed, new_session));
                    let (_, session) = state.windows.get_mut(&window).unwrap();
                    session
                };
                let mut usage = Usage {
                    id: 0,
                    start: last_switch.time,
                    end: switch.time,
                    during_idle: true, // TODO derive from idle watcher
                    session_id: session.id,
                };
                state.db.insert_usage(&mut usage)?;
                // TODO push to grpc system

                state.last_switch = switch;
            }
            Message::WindowClosed { window } => {
                warn!("window closed: {}", window.title().unwrap_or("NOT_FOUND".to_string()));
                println!("wat");
                state.windows.remove(&window);
            }
        }
        // info!("END");
        Ok(())
    }
}
