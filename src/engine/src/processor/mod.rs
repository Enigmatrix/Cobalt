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
    process_exits: (
        mpsc::UnboundedSender<ProcessId>,
        mpsc::UnboundedReceiver<ProcessId>,
    ),
}

#[derive(Debug)]
pub struct SessionData {
    closed_watcher: WindowClosed,
    session: Session,
}

#[derive(Debug)]
pub struct AppData {
    exit_watcher: ProcessExit,
    process: Process,
    app: App,
    arguments: String
}

#[derive(Debug)]
pub enum Message {
    Switch(WindowSwitch),
    WindowClosed(Window),
    ProcessExit(ProcessId)
}

impl Processor {
    pub fn new(taskset: &tokio::task::LocalSet) -> Result<Self> {

        let processor = Processor {
            state: Rc::new(RefCell::new(ProcessorState {
                windows: HashMap::new(),
                processes: HashMap::new(),
                db: Database::new(),
                prev_switch: WindowSwitch {
                    window: Window::new(unsafe { winuser::GetForegroundWindow() })?, // TODO should be current fg window
                    time: Timestamp::from_ticks(0), // TODO should be current time
                },
                process_exits: mpsc::unbounded_channel(),
            })),
        };
        let copy = processor.share();
        let _join_handle = taskset.spawn_local(async move {
            loop {
                let msg = {
                    let ProcessorState { process_exits: (_, recver), ..} = &mut *copy.state.borrow_mut();
                    recver.try_recv()
                };
                match msg {
                    Ok(pid) => copy.process(Message::ProcessExit(pid)).unwrap(),
                    Err(mpsc::error::TryRecvError::Empty) => tokio::task::yield_now().await,
                    Err(mpsc::error::TryRecvError::Closed) => return,
                }
            }
        });
        Ok(processor)
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

    fn get_app_identification(&mut self, window: Window, process: &Process, path: &String, cmdline: &String) -> Result<AppIdentification> {
        let ret = if window.is_uwp()? {
            AppIdentification::Uwp {
                aumid: window.aumid()?
            }
        } else {
            AppIdentification::Win32 {
                path: path.clone()
            }
        }; // TODO check for Java Apps here
        Ok(ret)
    }

    fn create_app(&mut self, _: &Processor, window: Window, process: &Process, path: &String, cmdline: &String) -> Result<App> {
        Ok(App {
            id: 0,
            background: String::new(), // TODO
            description: String::new(), // TODO
            name: String::new(), // TODO
            identification: self.get_app_identification(window, process, path, cmdline)?
        })
    }

    fn create_session(&mut self, processor: &Processor, window: Window /*, app: &AppData*/) -> Result<Session> {
        let (pid, _) = window.pid_tid()?;

        if self.processes.get(&pid).is_none() {
            let process = Process::new(pid, default())?;
            let exit_watcher = ProcessExit::watch(&process, self.process_exits.0.clone())?;
            let (path, cmdline) = process.path_and_cmdline()?;
            let mut app = self.create_app(processor, window, &process, &path, &cmdline)?;
            info!("New App: {:?}", app);
            self.db.insert_app(&mut app)?;
            let appdata = AppData {
                exit_watcher,
                process,
                app,
                arguments: cmdline
            };
            self.processes.insert(pid, appdata);
        }

        let AppData {app, arguments, ..} = self.processes.get(&pid).unwrap();

        Ok(Session {
            id: 0,
            app_id: app.id,
            arguments: arguments.clone(),
            title: window.title()?,
        })
    }

    #[instrument(skip(self, processor))]
    pub fn process(&mut self, msg: Message, processor: &Processor) -> Result<()> {
        match msg {
            Message::Switch(curr_switch) => {
                if self.windows.get(&curr_switch.window).is_none() {
                    let closed_watcher =
                        WindowClosed::watch(processor.share(), curr_switch.window)?;

                    let mut session = self.create_session(processor, curr_switch.window)?;
                    info!("New Session: {:?}", session);
                    self.db.insert_session(&mut session)?;
                    self.windows.insert(
                        curr_switch.window,
                        SessionData {
                            session,
                            closed_watcher,
                        },
                    );
                }

                let SessionData { session, .. } =
                    self.windows.get_mut(&curr_switch.window).unwrap();

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
                self.windows.remove(&window);
            }
            Message::ProcessExit(pid) => {
                info!("Process Exited");
                self.processes.remove(&pid);
            }
        }
        Ok(())
    }
}
