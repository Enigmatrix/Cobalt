#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)] // disable console window in non-debug

//! Engine running in the background

use std::sync::Arc;
use std::thread;

use data::db::{AppUpdater, DatabasePool};
use engine::{Engine, Event};
use platform::events::{
    ForegroundEventWatcher, InteractionWatcher, SystemEventWatcher, WindowSession,
};
use platform::objects::{EventLoop, MessageWindow, Timer, Timestamp, User, Window};
use resolver::AppInfoResolver;
use sentry::Sentry;
use util::channels::{self, Receiver, Sender};
use util::config::{self, Config};
use util::error::{Context, Result};
use util::future::runtime::{Builder, Handle};
use util::future::sync::Mutex;
use util::tracing::{error, info, ResultTraceExt};
use util::Target;

mod cache;
mod engine;
mod resolver;
mod sentry;

/// Entry point for the engine
pub fn main() {
    if let Err(report) = real_main() {
        error!("fatal error caught in main: {:?}", report);
        std::process::exit(1);
    }
}

fn real_main() -> Result<()> {
    let config = config::get_config()?;
    util::setup(&config, Target::Engine)?;
    info!("starting engine");
    platform::setup()?;
    info!("running as {:?}", User::current()?);

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

    let message_window = MessageWindow::new()?;

    let mut fg_watcher = ForegroundEventWatcher::new(fg)?;
    let it_watcher = InteractionWatcher::init(config, now)?;
    let system_event_tx = event_tx.clone();

    let _it_watcher = it_watcher.clone();
    let _system_watcher = SystemEventWatcher::new(&message_window, move |event| {
        let now = Timestamp::now();
        info!("system state event: {:?}, {:?}", event, now);
        let last_interaction = _it_watcher
            .lock()
            .unwrap()
            .short_circuit(event.state.is_active(), now);
        system_event_tx
            .send(Event::System {
                event,
                last_interaction,
                now,
            })
            .context("send system event to engine")
    })?;

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
            if let Some(event) = it_watcher.lock().unwrap().poll(now)? {
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
    // this is never reached normally because WM_QUIT is never sent.
    Ok(())
}

/// Processing loop for the [Engine].
async fn engine_loop(
    db_pool: DatabasePool,
    cache: Arc<Mutex<cache::Cache>>,
    rx: Receiver<Event>,
    spawner: Handle,
    fg: WindowSession,
    now: Timestamp,
) -> Result<()> {
    let db = db_pool.get_db().await?;
    let mut engine = Engine::new(cache, db_pool, fg, now, db, spawner).await?;
    loop {
        let ev = rx.recv_async().await?;
        engine.handle(ev).await?;
    }
}

/// Processing loop for the [Sentry].
async fn sentry_loop(
    db_pool: DatabasePool,
    cache: Arc<Mutex<cache::Cache>>,
    alert_rx: Receiver<Timestamp>,
) -> Result<()> {
    let db = db_pool.get_db().await?;
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
    let rt = Builder::new_current_thread().enable_time().build()?;
    // let rt = Builder::new_multi_thread().enable_all().build()?;

    let cache = Arc::new(Mutex::new(cache::Cache::new()));

    let handle = rt.handle().clone();
    let res: Result<()> = rt.block_on(async move {
        let db_pool = DatabasePool::new(config).await?;

        // re-run failed app info updates
        let _db_pool = db_pool.clone();
        let _handle = handle.clone();
        handle.clone().spawn(async move {
            update_app_infos(_db_pool, _handle)
                .await
                .context("update app infos")
                .error();
        });

        let sentry_handle = {
            let cache = cache.clone();
            let db_pool = db_pool.clone();
            handle.spawn(async move {
                for attempt in 0.. {
                    if attempt > 0 {
                        info!("restarting sentry loop");
                    }
                    sentry_loop(db_pool.clone(), cache.clone(), alert_rx.clone())
                        .await
                        .context("sentry loop")
                        .error();
                }
            })
        };

        for attempt in 0.. {
            if attempt > 0 {
                info!("restarting engine loop");
            }
            engine_loop(
                db_pool.clone(),
                cache.clone(),
                event_rx.clone(),
                handle.clone(),
                fg.clone(),
                now,
            )
            .await
            .context("engine loop")
            .error();
        }

        sentry_handle.await?;
        Ok(())
    });
    res
}

async fn update_app_infos(db_pool: DatabasePool, handle: Handle) -> Result<()> {
    let db = db_pool.get_db().await?;
    let mut updater = AppUpdater::new(db)?;
    let apps = updater.get_apps_to_update().await?;
    let mut handles = Vec::new();
    for app in apps {
        let _db_pool = db_pool.clone();
        handles.push(handle.spawn(async move {
            AppInfoResolver::update_app(_db_pool, app.id.clone(), app.identity.clone())
                .await
                .with_context(|| {
                    format!(
                        "update app({:?}, {:?}) with info at start",
                        app.id, app.identity
                    )
                })
                .error();
        }));
    }
    for handle in handles {
        handle.await?;
    }
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
