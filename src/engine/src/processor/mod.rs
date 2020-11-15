use anyhow::*;
use native::watchers::*;
use native::wrappers::*;
use std::collections::hash_map::Entry::*;
use std::collections::HashMap;

mod app_info;
mod session_info;

use app_info::*;
use session_info::*;

pub struct Processor {
    sessions: HashMap<Window, SessionInfo>,
    apps: HashMap<ProcessId, AppInfo>,

    msger: Messenger,
    recv: flume::Receiver<Message>,
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
        process: Process,
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
        let processor = Processor {
            sessions: HashMap::new(),
            apps: HashMap::new(),

            msger: msger.clone(),
            recv: rx,
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
                let session_info = match self.sessions.entry(window.clone()) {
                    Occupied(occupied) => occupied.into_mut(),
                    Vacant(vacant) => {
                        let msger = self.msger.clone();
                        vacant.insert(
                            SessionInfo::new(
                                window.clone(),
                                window_closed::Watcher::new(window.clone(), move |window| {
                                    msger
                                        .send(Message::WindowClosed { window })
                                        .with_context(|| "Unable to send WindowClosed message")
                                })
                                .with_context(|| {
                                    "Unable to create Window closed watcher for new SessionInfo"
                                })?,
                            )
                            .with_context(|| "Unable to create new SessionInfo")?,
                        )
                    }
                };
            }
            Message::WindowClosed { window } => {
                self.sessions
                    .remove(&window)
                    .with_context(|| "Pre-existing SessionInfo not found for window")?;
            }
            Message::ProcessExit { process } => {
                self.apps
                    .remove(&process.pid()?)
                    .with_context(|| "Pre-existing AppInfo not found for process")?;
            }
        }
        Ok(())
    }
}
