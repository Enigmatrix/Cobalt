use crate::data::db::Database;
use crate::data::model;
use crate::server;
use native::watchers::*;
use native::wrappers::*;
use std::collections::hash_map::Entry::*;
use std::collections::HashMap;
use util::*;

mod info;
use info::*;

pub type SessionCache = HashMap<Window, SessionInfo>;
pub type AppCache = HashMap<ProcessId, AppInfo>;

#[derive(Debug)]
pub struct UsageInfo {
    usage: model::Usage,
    info: Info,
}

pub struct Processor {
    sessions: SessionCache,
    apps: AppCache,

    db: Database,

    msger: Messenger,
    recv: channel::Receiver<Message>,
    engine_tx: server::EngineMessenger,

    current: UsageInfo,
}

#[derive(Clone)]
pub struct Messenger {
    sender: channel::Sender<Message>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ForegroundChanged {
        window: Window,
        timestamp: Timestamp,
    },
    WindowClosed {
        window: Window,
    },
    WindowTitleChanged {
        window: Window,
    },
    ProcessExit {
        pid: ProcessId,
    },
    IdleChanged {
        status: idle::IdleStatus,
    },
    AppUpdate {
        app_id: model::Id,
        file_info: FileInfo,
    },
}

impl Messenger {
    pub fn send(&self, msg: Message) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}

impl Processor {
    pub fn new_pair(engine_tx: server::EngineMessenger) -> Result<(Messenger, Processor)> {
        let (tx, rx) = channel::unbounded();
        let msger = Messenger { sender: tx };

        let mut sessions = HashMap::new();
        let mut apps = HashMap::new();

        let mut db = Database::new().with_context(|| "Creating database")?;

        let now = Timestamp::now();
        let fg = Window::foreground().with_context(|| "Get foreground window")?;
        let info = Info::from(&fg, &msger, &mut db, &mut sessions, &mut apps)
            .with_context(|| "Get Session id")?;

        let usage = model::Usage {
            id: 0,
            sess_id: info.sess_id,
            start: now,
            end: now,
            idle: false,
        };

        let processor = Processor {
            sessions,
            apps,

            db,

            msger: msger.clone(),
            recv: rx,
            engine_tx,

            current: UsageInfo { usage, info },
        };
        Ok((msger, processor))
    }

    pub async fn process_messages(&mut self) -> Result<()> {
        while let Ok(msg) = self.recv.recv_async().await {
            self.process(msg)?
        }
        Ok(())
    }

    #[log::instrument(skip(self))]
    pub fn process(&mut self, msg: Message) -> Result<()> {
        log::trace!("processing...");
        match msg {
            Message::ForegroundChanged { window, timestamp } => {
                let info = Info::from(
                    &window,
                    &self.msger,
                    &mut self.db,
                    &mut self.sessions,
                    &mut self.apps,
                )
                .with_context(|| "Getting Session id")?;

                if info.sess_id == self.current.usage.sess_id {
                    // skip processing the rest, as the window hasn't changed
                    log::trace!("repeated window");
                    return Ok(());
                }

                self.current.usage.end = timestamp;
                self.db
                    .insert_usage(&mut self.current.usage)
                    .with_context(|| "Save Usage to Database")?;

                let usage_switch = crate::server::UsageSwitch {
                    // TODO send this to the server
                    prev_app_id: self.current.info.app_id,
                    prev_sess_id: self.current.info.sess_id,
                    prev_usage_id: self.current.usage.id,

                    new_app_id: info.app_id,
                    new_sess_id: info.sess_id,
                };
                self.engine_tx
                    .push_usage_switch(usage_switch.clone())
                    .with_context(|| "Push usage switch to worker")?;

                log::trace!(?self.current, ?usage_switch, "recorded usage");

                self.current = UsageInfo {
                    usage: model::Usage {
                        id: 0,
                        sess_id: info.sess_id,
                        start: timestamp,
                        end: timestamp,
                        idle: false, // TODO idle watcher
                    },
                    info,
                };
            }
            Message::WindowClosed { window } => {
                self.sessions.remove(&window);
            }
            Message::WindowTitleChanged { window } => {
                todo!("action when the Window {:?} title changed", window)
            }
            Message::ProcessExit { pid } => {
                // TODO maybe all Sessions associated with this Process/AppInfo can
                // be removed as well? just in case, even thought by right they should
                // be removed since WindowClosed will be fired as well.

                self.apps.remove(&pid);
            }
            Message::IdleChanged { status: _ } => {
                // TODO split self.current
            }
            Message::AppUpdate { app_id, file_info } => {
                self.db
                    .update_app(
                        app_id,
                        file_info.name,
                        file_info.description,
                        file_info.icon,
                    )
                    .with_context(|| "Update App information in the Database")?;

                // TODO send the new update app id to clients
            }
        }
        Ok(())
    }
}
