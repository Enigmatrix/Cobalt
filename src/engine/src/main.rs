#![feature(trait_alias)]
#![feature(default_free_fn)]
#![feature(maybe_uninit_ref)]

mod data;
mod errors;
#[macro_use]
mod os;
mod reactor;
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

    let main = tokio::task::LocalSet::new();

    let events = hook::EventLoop::new();
    let reactor = reactor::Reactor::new(&main)?;

    let copy = reactor.share();
    let _fg_watcher = ForegroundWindowSwitches::watch(copy)?;

    runtime.block_on(main.run_until(events));

    info!("Exited");
    Ok(())
}
