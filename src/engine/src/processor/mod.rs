use anyhow::*;
use native::wrappers::*;

pub struct Processor {
    msger: Messenger,
    pub recv: flume::Receiver<Message>,
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
        dbg!(msg);
        Ok(())
    }
}
