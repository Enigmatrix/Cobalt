use native::watchers::*;
use native::wrappers::*;
use util::*;

mod data;
mod processor;

use processor::*;

fn main() -> Result<()> {
    native::setup()?;

    let (msger, mut processor) = Processor::new_pair().with_context(|| "Create Processor")?;
    let event_loop = EventLoop::new();

    // idle::Watcher::begin()?;

    let fg_msger = msger.clone();
    let _fg = foreground::Watcher::new(|window, timestamp| {
        fg_msger.send(Message::ForegroundChanged { window, timestamp })
    })
    .with_context(|| "Create foreground watcher")?;

    let local = futures::task::LocalSet::new();

    local.spawn_local(async move {
        processor
            .process_messages()
            .await
            .with_context(|| "Error in processing message")
            .unwrap();
    });

    futures::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(local.run_until(event_loop));

    Ok(())
}
