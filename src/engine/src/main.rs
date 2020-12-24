#![feature(never_type)]
#![feature(async_closure)]
#![feature(seek_convenience)]

use native::watchers::*;
use native::wrappers::*;
use util::*;

mod data;
mod processor;
mod server;

use processor::*;

#[futures::main]
async fn main() -> Result<!> {
    util::setup().with_context(|| "Setup utils")?;
    native::setup().with_context(|| "Setup native dependencies")?;

    log::info!("🚀 Starting engine");

    let (engine_tx, engine) = server::EngineWorker::new();
    let (msger, mut processor) =
        Processor::new_pair(engine_tx).with_context(|| "Create Processor")?;

    idle::Watcher::begin().with_context(|| "Begin monitoring for mouse and keyboard events")?;

    let idle_dur = Duration::from_millis(5_000); // TODO read this from config
    let _idle = idle::Watcher::new(idle_dur, |idle| {
        log::warn!(?idle);
        Ok(())
    })
    .with_context(|| "Create idle watcher")?;

    let _fg = foreground::Watcher::new({
        let msger = msger.clone();
        move |window, timestamp|
            msger.send(Message::ForegroundChanged { window, timestamp })
    })
    .with_context(|| "Create foreground watcher")?;

    futures::spawn(engine.serve());

    processor
        .process_messages()
        .await
        .with_context(|| "Error in processing message")?;

    log::info!("engine exiting");

    std::process::exit(0)
}
