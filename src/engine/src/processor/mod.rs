use crate::data::db::Database;
use crate::data::model;
use crate::services;
use native::watchers::*;
use native::wrappers::*;
use std::collections::hash_map::Entry::*;
use std::collections::HashMap;
use util::*;

mod info;
use info::*;

pub type SessionCache = HashMap<Window, SessionInfo>;
pub type AppCache = HashMap<ProcessId, AppInfo>;

#[derive(Debug, Clone)]
pub struct UsageInfo {
    usage: model::Usage,
    info: Info,
}

pub struct Processor {
    sessions: SessionCache,
    apps: AppCache,

    db: Database,

    tx: ProcessorTx,
    recv: channel::Receiver<Message>,
    engine_tx: services::RelayServiceTx,

    current: UsageInfo,
    now_idle: bool,
}

#[derive(Clone)]
pub struct ProcessorTx {
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

impl ProcessorTx {
    pub fn send(&self, msg: Message) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}

impl Processor {
    pub fn new_pair(engine_tx: services::RelayServiceTx) -> Result<(ProcessorTx, Processor)> {
        let (tx, rx) = channel::unbounded();
        let msger = ProcessorTx { sender: tx };

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

            tx: msger.clone(),
            recv: rx,
            engine_tx,

            current: UsageInfo { usage, info },
            now_idle: false,
        };
        Ok((msger, processor))
    }

    pub async fn process_messages(&mut self) -> Result<()> {
        while let Ok(msg) = self.recv.recv_async().await {
            self.process(msg)?
        }
        Ok(())
    }

    pub fn switch_usage(&mut self, end_at: Timestamp, info: Info) -> Result<()> {
        self.current.usage.end = end_at;
        self.db
            .insert_usage(&mut self.current.usage)
            .with_context(|| "Save Usage to Database")?;

        let usage_switch = services::dto::UsageSwitch {
            prev_app_id: self.current.info.app_id,
            prev_sess_id: self.current.info.sess_id,
            prev_usage_id: self.current.usage.id,

            new_app_id: info.app_id,
            new_sess_id: info.sess_id,
        };

        log::trace!(?self.current, ?usage_switch, "recorded usage");

        self.engine_tx
            .push_usage_switch(usage_switch)
            .with_context(|| "Push usage switch to worker")?;

        self.current = UsageInfo {
            usage: model::Usage {
                id: 0,
                sess_id: info.sess_id,
                start: end_at,
                end: end_at,
                idle: self.now_idle,
            },
            info,
        };

        Ok(())
    }

    #[log::instrument(skip(self))]
    pub fn process(&mut self, msg: Message) -> Result<()> {
        log::trace!("processing...");
        match msg {
            Message::ForegroundChanged { window, timestamp } => {
                let info = Info::from(
                    &window,
                    &self.tx,
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

                self.switch_usage(timestamp, info)
                    .with_context(|| "Switch to new Usage")?;
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
            Message::IdleChanged { status } => {
                self.now_idle = match status {
                    idle::IdleStatus::Idle => true,
                    idle::IdleStatus::Active => false,
                };

                self.switch_usage(Timestamp::now(), self.current.info.clone())
                    .with_context(|| "Switch to new Usage")?;
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

                self.engine_tx.push_app_update(app_id).with_context(|| "Push App Update")?;
            }
        }
        Ok(())
    }
}
