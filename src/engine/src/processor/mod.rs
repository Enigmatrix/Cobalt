use crate::data::db::Database;
use crate::data::model;
use anyhow::*;
use native::watchers::*;
use native::wrappers::*;
use std::collections::hash_map::Entry::*;
use std::collections::HashMap;

mod info;
use info::*;

pub type SessionCache = HashMap<Window, SessionInfo>;
pub type AppCache = HashMap<ProcessId, AppInfo>;

pub struct Processor {
    sessions: SessionCache,
    apps: AppCache,

    db: Database,

    msger: Messenger,
    recv: flume::Receiver<Message>,

    current_usage: model::Usage,
}

#[derive(Clone)]
pub struct Messenger {
    sender: flume::Sender<Message>,
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
    ProcessExit {
        pid: ProcessId,
    },
}

impl Messenger {
    pub fn send(&self, msg: Message) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}

impl Processor {
    pub fn new_pair() -> Result<(Messenger, Processor)> {
        let (tx, rx) = flume::unbounded();
        let msger = Messenger { sender: tx };

        let mut sessions = HashMap::new();
        let mut apps = HashMap::new();

        let mut db = Database::new().with_context(|| "Creating database")?;

        let now = Timestamp::now();
        let fg = Window::foreground().with_context(|| "Get foreground window")?;
        let sess_id = Info::session_id(&fg, &msger, &mut db, &mut sessions, &mut apps)
            .with_context(|| "Get Session id")?;

        let current_usage = model::Usage {
            id: 0,
            sess_id,
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

            current_usage,
        };
        Ok((msger, processor))
    }

    pub async fn process_messages(&mut self) -> Result<()> {
        while let Ok(msg) = self.recv.recv_async().await {
            self.process(msg)?
        }
        Ok(())
    }

    pub fn process(&mut self, msg: Message) -> Result<()> {
        match dbg!(msg) {
            Message::ForegroundChanged { window, timestamp } => {
                let sess_id = Info::session_id(
                    &window,
                    &self.msger,
                    &mut self.db,
                    &mut self.sessions,
                    &mut self.apps,
                )
                .with_context(|| "Getting Session id")?;

                if sess_id == self.current_usage.sess_id {
                    // skip processing the rest, as the window hasn't changed
                    return Ok(());
                }

                self.current_usage.end = timestamp;
                self.db
                    .insert_usage(&mut self.current_usage)
                    .with_context(|| "Save Usage to Database")?;
                dbg!(&self.current_usage);

                self.current_usage = model::Usage {
                    id: 0,
                    sess_id,
                    start: timestamp,
                    end: timestamp,
                    idle: false, // TODO idle watcher
                };
            }
            Message::WindowClosed { window } => {
                self.sessions
                    .remove(&window)
                    .with_context(|| "Pre-existing SessionInfo not found for window")?;
            }
            Message::ProcessExit { pid } => {
                // maybe all Sessions associated with this Process/AppInfo can be removed as well.

                self.apps
                    .remove(&pid)
                    .with_context(|| "Pre-existing AppInfo not found for process")?;
            }
        }
        Ok(())
    }
}
