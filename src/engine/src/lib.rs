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
    BrowserTabWatcher, ForegroundEventWatcher, InteractionWatcher, SystemEventWatcher, TabChange,
    WindowSession,
};
use platform::objects::{Duration, EventLoop, MessageWindow, Timer, Timestamp, User};
use platform::web::{self, BrowserDetector};
use resolver::AppInfoResolver;
use sentry::Sentry;
use util::channels::{self, Receiver, Sender};
use util::config::{self, Config};
use util::error::{Context, Result};
use util::future::runtime::{Builder, Handle};
use util::future::sync::Mutex;
use util::future::task::JoinHandle;
use util::tracing::{ResultTraceExt, error, info};
use util::{Target, future};

use crate::engine::EngineArgs;

mod desktop;
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
    util::set_target(Target::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    info!("starting engine");
    platform::setup()?;
    info!("running as {:?}", User::current()?);

    let web_state = web::default_state();
    let desktop = desktop::new_desktop_state(web_state.clone());

    let (event_tx, event_rx) = channels::unbounded();
    let (alert_tx, alert_rx) = channels::unbounded();
    let (tab_change_tx, tab_change_rx) = channels::unbounded();

    let start = Timestamp::now();
    let window_session = foreground_window_session(&config, web_state.clone())?;

    let ev_thread = {
        let config = config.clone();
        let window_session = window_session.clone();
        let web_state = web_state.clone();
        thread::Builder::new()
            .name("event_loop_thread".to_string())
            .spawn(move || {
                event_loop(EventLoopArgs {
                    config,
                    web_state,
                    event_tx,
                    alert_tx,
                    tab_change_tx,
                    window_session,
                    start,
                })
                .context("event loop")
                .error();
            })?
    };

    processor(ProcessorArgs {
        desktop,
        web_state,
        config,
        window_session,
        start,
        event_rx,
        alert_rx,
        tab_change_rx,
    })?;
    // can't turn this to util::error::Result :/
    ev_thread.join().expect("event loop thread");
    Ok(())
}

struct EventLoopArgs {
    web_state: web::State,

    config: Config,

    event_tx: Sender<Event>,
    alert_tx: Sender<Timestamp>,
    tab_change_tx: Sender<TabChange>,

    window_session: WindowSession,
    start: Timestamp,
}

/// Win32 [EventLoop] thread to poll for events.
/// All the callback and functioons used in this
/// function run on the _same_ thread - so thread-safety
/// is not an issue.
fn event_loop(args: EventLoopArgs) -> Result<()> {
    let ev = EventLoop::new();

    let poll_dur = args.config.poll_duration().into();
    let alert_dur = args.config.alert_duration().into();

    let message_window = MessageWindow::new()?;

    let mut fg_watcher =
        ForegroundEventWatcher::new(args.window_session, &args.config, args.web_state.clone())?;
    let it_watcher = InteractionWatcher::init(&args.config, args.start)?;
    let system_event_tx = args.event_tx.clone();

    let _it_watcher = it_watcher.clone();
    let _system_watcher = SystemEventWatcher::new(&message_window, move |event| {
        let now = Timestamp::now();
        info!("system state event: {:?}, {:?}", event, now);
        let last_interaction = _it_watcher
            .borrow_mut()
            .as_mut()
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
        Box::new(move || {
            let now = Timestamp::now();
            // if there is a switch event, process it. otherwise, tick to update the usage.
            if let Some(event) = fg_watcher.poll(now)? {
                args.event_tx.send(Event::ForegroundChanged(event))?;
            } else {
                args.event_tx.send(Event::Tick(now))?;
            }
            if let Some(event) = it_watcher.borrow_mut().as_mut().unwrap().poll(now)? {
                args.event_tx.send(Event::InteractionChanged(event))?;
            }
            Ok(())
        }),
    )?;

    let _alert_timer = Timer::new(
        alert_dur,
        Box::new(move || {
            let now = Timestamp::now();
            args.alert_tx.send(now)?;
            Ok(())
        }),
    )?;

    let mut browser_tab_watcher = BrowserTabWatcher::new(args.tab_change_tx, args.web_state)?;

    let dim_tick = Duration::from_millis(1000);
    let _browser_tab_tick_timer = Timer::new(
        dim_tick,
        Box::new(move || {
            browser_tab_watcher
                .tick()
                .context("browser tab watcher tick")?;

            Ok(())
        }),
    )?;

    ev.run();
    // this is never reached normally because WM_QUIT is never sent.
    Ok(())
}

/// Processing loop for the [Engine].
async fn engine_loop(options: EngineArgs, rx: Receiver<Event>) -> Result<()> {
    let mut engine = Engine::new(options).await?;
    loop {
        let ev = rx.recv_async().await?;
        engine.handle(ev).await?;
    }
}

/// Processing loop for the [Sentry].
async fn sentry_loop(
    db_pool: DatabasePool,
    desktop: desktop::DesktopState,
    spawner: Handle,
    alert_rx: Receiver<Timestamp>,
    tab_change_rx: Receiver<TabChange>,
) -> Result<()> {
    let db = db_pool.get_db().await?;
    let sentry = Arc::new(Mutex::new(Sentry::new(desktop, db)?));
    // TODO: these two loops must die if the other one dies.

    let _sentry = sentry.clone();
    let join1: JoinHandle<Result<()>> = spawner.spawn(async move {
        loop {
            let tab_change = tab_change_rx.recv_async().await?;
            let mut sentry = _sentry.lock().await;
            sentry.handle_tab_change(tab_change).await?;
        }
    });

    let join2: JoinHandle<Result<()>> = spawner.spawn(async move {
        loop {
            let at = alert_rx.recv_async().await?;
            let mut sentry = sentry.lock().await;
            sentry.run(at).await?;
        }
    });
    let join1_handle = join1.abort_handle();
    let join2_handle = join2.abort_handle();

    // Wait for either join handle to finish, then cancel the other
    let res = future::select! {
        result1 = join1 => {
            join2_handle.abort();
            result1?
        }
        result2 = join2 => {
            join1_handle.abort();
            result2?
        }
    };

    res
}

struct ProcessorArgs {
    desktop: desktop::DesktopState,
    web_state: web::State,

    config: Config,

    event_rx: Receiver<Event>,
    alert_rx: Receiver<Timestamp>,
    tab_change_rx: Receiver<TabChange>,

    window_session: WindowSession,
    start: Timestamp,
}

/// Runs the [Engine] and [Sentry] loops in an asynchronous executor.
fn processor(mut args: ProcessorArgs) -> Result<()> {
    let rt = Builder::new_multi_thread()
        .enable_time()
        .enable_io()
        .build()?;
    // let rt = Builder::new_multi_thread().enable_all().build()?;

    let handle = rt.handle().clone();
    let res: Result<()> = rt.block_on(async move {
        let db_pool = DatabasePool::new(&args.config).await?;

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
            let desktop = args.desktop.clone();
            let db_pool = db_pool.clone();
            let config = args.config.clone();
            let spawner = handle.clone();
            handle.spawn(async move {
                for attempt in 0.. {
                    if attempt > 0 {
                        future::time::sleep(config.alert_duration()).await;
                        info!("restarting sentry loop");
                    }
                    sentry_loop(
                        db_pool.clone(),
                        desktop.clone(),
                        spawner.clone(),
                        args.alert_rx.clone(),
                        args.tab_change_rx.clone(),
                    )
                    .await
                    .context("sentry loop")
                    .error();
                }
            })
        };

        for attempt in 0.. {
            if attempt > 0 {
                future::time::sleep(args.config.poll_duration()).await;
                info!("restarting engine loop");
                args.window_session =
                    foreground_window_session_async(&args.config, args.web_state.clone()).await?;
                args.start = Timestamp::now();
            }
            let event_rx = args.event_rx.clone();
            let args = EngineArgs {
                desktop: args.desktop.clone(),
                config: args.config.clone(),
                web_state: args.web_state.clone(),
                db_pool: db_pool.clone(),
                window_session: args.window_session.clone(),
                start: args.start,
                spawner: handle.clone(),
            };
            engine_loop(args, event_rx)
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
fn foreground_window_session(config: &Config, web_state: web::State) -> Result<WindowSession> {
    let browser = BrowserDetector::new()?;
    loop {
        let session = ForegroundEventWatcher::foreground_window_session(
            &browser,
            web_state.blocking_write(),
            config.track_incognito(),
        )?;
        if let Some(session) = session {
            return Ok(session);
        }
        // This method *MUST* be synchronous, so we use the synchronous version of sleep.
        // There is no blocking or potential for a race condition here because the
        // foreground window is a global resource, seperate from the async runtime.
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// Get the foreground [Window], and makes it into a [WindowSession] blocking until one is present.
async fn foreground_window_session_async(
    config: &Config,
    web_state: web::State,
) -> Result<WindowSession> {
    let browser = BrowserDetector::new()?;
    loop {
        let session = ForegroundEventWatcher::foreground_window_session(
            &browser,
            web_state.write().await,
            config.track_incognito(),
        )?;
        if let Some(session) = session {
            return Ok(session);
        }
        // There is no blocking or potential for a race condition here because the
        // foreground window is a global resource, seperate from the async runtime.
        future::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
