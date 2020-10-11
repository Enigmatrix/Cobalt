mod app_data;
mod session_data;

use crate::data::db::*;
use crate::data::entities::*;
use crate::errors::*;
use crate::os::prelude::*;
use crate::watchers::*;
use app_data::*;
use session_data::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tokio::sync::mpsc;
use tracing::*;

#[derive(Debug)]
pub struct Reactor {
    state: Rc<RefCell<State>>,
}

#[derive(Debug)]
pub struct State {
    sessions: HashMap<Window, SessionData>,
    apps: HashMap<ProcessId, AppData>,
    db: Database,
    prev_switch: WindowSwitch,
    process_exits: (
        mpsc::UnboundedSender<ProcessId>,
        mpsc::UnboundedReceiver<ProcessId>,
    ),
}

#[derive(Debug)]
pub enum Message {
    Switch(WindowSwitch),
    WindowClosed(Window),
    ProcessExit(ProcessId),
}

impl Reactor {
    pub fn new(taskset: &tokio::task::LocalSet) -> Result<Self> {
        let reactor = Reactor {
            state: Rc::new(RefCell::new(State {
                sessions: HashMap::new(),
                apps: HashMap::new(),
                db: Database::new(),
                prev_switch: WindowSwitch {
                    window: Window::new(unsafe { winuser::GetForegroundWindow() })?, // TODO should be current fg window
                    time: Timestamp::from_ticks(0), // TODO should be current time
                },
                process_exits: mpsc::unbounded_channel(),
            })),
        };
        let copy = reactor.share();
        let _join_handle = taskset.spawn_local(async move {
            loop {
                let msg = {
                    let State {
                        process_exits: (_, recver),
                        ..
                    } = &mut *copy.state.borrow_mut();
                    recver.try_recv()
                };
                match msg {
                    Ok(pid) => copy.process(Message::ProcessExit(pid)).unwrap(),
                    Err(mpsc::error::TryRecvError::Empty) => tokio::task::yield_now().await,
                    Err(mpsc::error::TryRecvError::Closed) => return,
                }
            }
        });
        Ok(reactor)
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

impl State {
    fn get_app_identification(
        &mut self,
        window: Window,
        _process: &Process,
        path: &String,
        _cmdline: &String,
    ) -> Result<AppIdentification> {
        let ret = if window.is_uwp()? {
            AppIdentification::Uwp {
                aumid: window.aumid()?,
            }
        } else {
            AppIdentification::Win32 { path: path.clone() }
        }; // TODO check for Java Apps here
        Ok(ret)
    }

    fn create_app(
        &mut self,
        _: &Reactor,
        window: Window,
        process: &Process,
        path: &String,
        cmdline: &String,
    ) -> Result<App> {
        let identification = self.get_app_identification(window, process, path, cmdline)?;
        let app = self
            .db
            .app_by_app_identification(&identification)
            .unwrap_or_else(|| App {
                id: 0,
                background: String::new(),  // TODO
                description: String::new(), // TODO
                name: String::new(),        // TODO
                identification,
            });
        Ok(app)
    }

    fn create_session(&mut self, reactor: &Reactor, window: Window) -> Result<Session> {
        let (pid, _) = window.pid_tid()?;

        if !self.apps.contains_key(&pid) {
            let appdata = AppData::create(self, reactor, window, pid)?;
            self.apps.insert(pid, appdata);
        }

        let AppData { app, arguments, .. } = self.apps.get(&pid).unwrap();

        Ok(Session {
            id: 0,
            app_id: app.id,
            arguments: arguments.clone(),
            title: window.title()?,
        })
    }

    #[instrument(skip(self, reactor))]
    pub fn process(&mut self, msg: Message, reactor: &Reactor) -> Result<()> {
        match msg {
            Message::Switch(curr_switch) => {
                if !self.sessions.contains_key(&curr_switch.window) {
                    let data = SessionData::create(self, reactor, curr_switch.window)?;
                    self.sessions.insert(curr_switch.window, data);
                }

                let SessionData { session, .. } =
                    self.sessions.get_mut(&curr_switch.window).unwrap();

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
                info!("Window Closed");
                self.sessions.remove(&window);
            }
            Message::ProcessExit(pid) => {
                info!("Process Exited");
                self.apps.remove(&pid);
            }
        }
        Ok(())
    }
}
