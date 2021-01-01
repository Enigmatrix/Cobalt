#![feature(never_type)]
#![feature(async_closure)]
#![feature(seek_convenience)]

use native::watchers::*;
use native::wrappers::*;
use util::*;

mod data;
mod processor;
mod services;

use processor::*;

#[futures::main]
async fn main() -> Result<!> {
    util::setup().with_context(|| "Setup utils")?;
    native::setup().with_context(|| "Setup native dependencies")?;

    log::info!("🚀 Starting engine");

    let (relay_tx, relay) = services::RelayService::new();
    let (processor_tx, mut processor) =
        Processor::new_pair(relay_tx).with_context(|| "Create Processor")?;

    idle::Watcher::begin().with_context(|| "Begin monitoring for mouse and keyboard events")?;

    let idle_dur = config::Config::instance().idle_timeout;
    let _idle = idle::Watcher::new(idle_dur.into(), {
        let processor_tx = processor_tx.clone();
        move |idle| {
            log::warn!(?idle);
            processor_tx.send(Message::IdleChanged { status: idle })
        }
    })
    .with_context(|| "Create idle watcher")?;

    let _fg = foreground::Watcher::new({
        let processor_tx = processor_tx.clone();
        move |window, timestamp| processor_tx.send(Message::ForegroundChanged { window, timestamp })
    })
    .with_context(|| "Create foreground watcher")?;

    futures::spawn(relay.serve());

    processor
        .process_messages()
        .await
        .with_context(|| "Error in processing message")?;

    log::info!("engine exiting");

    std::process::exit(0)
}
