use std::sync::Arc;
use std::thread;

use data::db::Database;
use engine::{Engine, Event};
use platform::events::{ForegroundEventWatcher, InteractionWatcher, WindowSession};
use platform::objects::{EventLoop, Timer, Timestamp, Window};
use sentry::Sentry;
use util::channels::{self, Receiver, Sender};
use util::config::{self, Config};
use util::error::{Context, Result};
use util::future::runtime::{Builder, Handle};
use util::future::sync::Mutex;
use util::tracing::{error, info, ResultTraceExt};

mod cache;
mod engine;
mod resolver;
mod sentry;

fn main() {
    if let Err(report) = real_main() {
        error!("fatal error caught in main: {:?}", report);
        std::process::exit(1);
    }
}

fn real_main() -> Result<()> {
    let config = config::get_config()?;
    util::setup(&config)?;
    info!("starting engine");
    platform::setup()?;

    let (event_tx, event_rx) = channels::unbounded();
    let (alert_tx, alert_rx) = channels::unbounded();
    let now = Timestamp::now();
    let fg = foreground_window_session()?;

    let ev_thread = {
        let config = config.clone();
        let fg = fg.clone();
        thread::spawn(move || {
            event_loop(&config, event_tx, alert_tx, fg, now)
                .context("event loop")
                .error();
        })
    };

    processor(&config, fg, now, event_rx, alert_rx)?;
    // can't turn this to util::error::Result :/
    ev_thread.join().expect("event loop thread");
    Ok(())
}

/// Win32 [EventLoop] thread to poll for events.
fn event_loop(
    config: &Config,
    event_tx: Sender<Event>,
    alert_tx: Sender<Timestamp>,
    fg: WindowSession,
    now: Timestamp,
) -> Result<()> {
    let ev = EventLoop::new();

    let poll_dur = config.poll_duration().into();
    let alert_dur = config.alert_duration().into();

    let mut fg_watcher = ForegroundEventWatcher::new(fg)?;
    let it_watcher = InteractionWatcher::init(config, now)?;

    let _poll_timer = Timer::new(
        poll_dur,
        poll_dur,
        Box::new(move || {
            let now = Timestamp::now();
            // if there is a switch event, process it. otherwise, tick to update the usage.
            if let Some(event) = fg_watcher.poll(now)? {
                event_tx.send(Event::ForegroundChanged(event))?;
            } else {
                event_tx.send(Event::Tick(now))?;
            }
            if let Some(event) = it_watcher.poll(now)? {
                event_tx.send(Event::InteractionChanged(event))?;
            }
            Ok(())
        }),
    )?;

    let _alert_timer = Timer::new(
        alert_dur,
        alert_dur,
        Box::new(move || {
            let now = Timestamp::now();
            alert_tx.send(now)?;
            Ok(())
        }),
    )?;

    ev.run();
    Ok(())
}

/// Processing loop for the [Engine].
async fn engine_loop(
    config: &Config,
    cache: Arc<Mutex<cache::Cache>>,
    rx: Receiver<Event>,
    spawner: Handle,
    fg: WindowSession,
    now: Timestamp,
) -> Result<()> {
    let db = Database::new(config).await?;
    let mut engine = Engine::new(cache, fg, now, config.clone(), db, spawner).await?;
    loop {
        let ev = rx.recv_async().await?;
        engine.handle(ev).await?;
    }
}

/// Processing loop for the [Sentry].
async fn sentry_loop(
    config: &Config,
    cache: Arc<Mutex<cache::Cache>>,
    alert_rx: Receiver<Timestamp>,
) -> Result<()> {
    let db = Database::new(config).await?;
    let mut sentry = Sentry::new(cache, db)?;
    loop {
        let at = alert_rx.recv_async().await?;
        sentry.run(at).await?;
    }
}

/// Runs the [Engine] and [Sentry] loops in an asynchronous executor.
fn processor(
    config: &Config,
    fg: WindowSession,
    now: Timestamp,
    event_rx: Receiver<Event>,
    alert_rx: Receiver<Timestamp>,
) -> Result<()> {
    let rt = Builder::new_current_thread().build()?;
    // let rt = Builder::new_multi_thread().enable_all().build()?;

    let cache = Arc::new(Mutex::new(cache::Cache::new()));

    let _config = config.clone();
    let _cache = cache.clone();
    rt.spawn(async move {
        sentry_loop(&_config, _cache, alert_rx)
            .await
            .context("sentry loop")
            .error();
    });

    let _config = config.clone();
    let _cache = cache.clone();
    let _handle = rt.handle().clone();
    rt.block_on(async move {
        engine_loop(&_config, _cache, event_rx, _handle, fg, now)
            .await
            .context("engine loop")
            .error();
    });

    Ok(())
}

/// Get the foreground [Window], and makes it into a [WindowSession] blocking until one is present.
fn foreground_window_session() -> Result<WindowSession> {
    loop {
        if let Some(window) = Window::foreground() {
            return WindowSession::new(window);
        }
    }
}
