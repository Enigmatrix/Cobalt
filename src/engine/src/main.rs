use anyhow::*;
use native::watchers::*;
use native::wrappers::*;
use tokio::task;

mod data;
mod processor;

use processor::*;

#[tokio::main]
async fn main() -> Result<()> {
    native::setup()?;

    let (msger, mut processor) = Processor::new_pair()?;
    let event_loop = EventLoop::new();

    let fg_msger = msger.clone();

    // idle::Watcher::begin()?;
    let _fg = foreground::Watcher::new(|window, timestamp| {
        fg_msger.send(Message::ForegroundChanged { window, timestamp })
    })?;

    let local = task::LocalSet::new();

    local.spawn_local(async move {
        processor
            .process_messages()
            .await
            .with_context(|| "Error in processing message")
            .unwrap();
    });
    local.run_until(event_loop).await;
    Ok(())
}
