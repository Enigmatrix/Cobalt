#![feature(trait_alias)]
#![feature(async_closure)]
#![feature(default_free_fn)]
#![feature(option_expect_none)]
#![feature(maybe_uninit_ref)]
#![recursion_limit = "1024"]

mod data;
mod errors;
#[macro_use]
mod os;
mod processor;
mod utils;
mod watchers;

use os::prelude::*;
use tokio::*;
use tracing::*;
use watchers::*;

fn init() -> Result<(runtime::Runtime, Span), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            // .with_thread_ids(true)
            // .compact()
            .finish(),
    )?;

    let runtime = runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .thread_name("cobalt-worker")
        .build()?;

    let span = info_span!("main");

    hook::init_contexts();

    Ok((runtime, span))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut runtime, span) = init()?;
    let _enter = span.enter();

    info!("Start");

    let events = hook::EventLoop::new();
    let processor = processor::Processor::new()?;

    let processor_pump = processor.share();
    let _fg_watcher = ForegroundWindowSwitches::watch(processor_pump)?;

    let main = tokio::task::LocalSet::new();

    /*main.spawn_local(async move {
        loop {
            let next = watchers::WindowClosedWatcher::next_close(&closes_watch);
            if let Some(item) = next.await {
                println!("[CLOSED]: {:?}", item.title());
            }
        }
    });*/

    runtime.block_on(main.run_until(events));

    info!("Exited");
    Ok(())
}
