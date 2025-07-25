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
    ForegroundEventWatcher, ForegroundWindowSessionInfo, InteractionWatcher, SystemEventWatcher,
};
use platform::objects::{Duration, EventLoop, MessageWindow, Timer, Timestamp, User};
use platform::web;
use resolver::AppInfoResolver;
use sentry::Sentry;
use util::channels::{self, Receiver, Sender};
use util::config::{self, Config};
use util::error::{Context, Result};
use util::future::runtime::{Builder, Handle, Runtime};
use util::future::sync::Mutex;
use util::future::task::JoinHandle;
use util::retry::{ExponentialBuilder, Retryable, RetryableWithContext};
use util::tracing::{ResultTraceExt, error, info};
use util::{Target, future};

use crate::engine::EngineArgs;

/// Desktop State
pub mod desktop;
/// Engine for processing events
pub mod engine;
/// Resolver for app info
pub mod resolver;
/// Sentry for alerting
pub mod sentry;

/// Entry point for the engine
pub fn main() {
    let (config, rt) = match setup() {
        Ok(v) => v,
        Err(e) => {
            error!("fatal error caught in setup: {:?}", e);
            std::process::exit(1);
        }
    };

    let web_state = web::default_state();
    let desktop_state = desktop::new_desktop_state(web_state.clone());

    if let Err(report) = run(&config, rt.handle().clone(), web_state, desktop_state) {
        error!("fatal error caught in main: {:?}", report);
        std::process::exit(1);
    }
}

fn setup() -> Result<(Config, Runtime)> {
    util::set_target(Target::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    info!("starting engine");
    platform::setup()?;
    info!("running as {:?}", User::current()?);

    let rt = Builder::new_multi_thread()
        .enable_time()
        .enable_io()
        .build()?;

    Ok((config, rt))
}

/// Run the engine
pub fn run(
    config: &Config,
    rt: Handle,
    web_state: web::State,
    desktop_state: desktop::DesktopState,
) -> Result<()> {
    let (event_tx, event_rx) = channels::unbounded();
    let (alert_tx, alert_rx) = channels::unbounded();
    let (web_change_tx, web_change_rx) = channels::unbounded();

    let start = Timestamp::now();
    let session = foreground_window_session(config, web_state.clone())?;

    let ev_thread = {
        let config = config.clone();
        let session = session.clone();
        let web_state = web_state.clone();
        thread::Builder::new()
            .name("event_loop_thread".to_string())
            .spawn(move || {
                event_loop(EventLoopArgs {
                    config,
                    web_state,
                    event_tx,
                    alert_tx,
                    web_change_tx,
                    session,
                    start,
                })
                .context("event loop")
                .expect("event loop failed");
            })?
    };

    rt.clone().block_on(processor(ProcessorArgs {
        desktop_state,
        web_state,
        config: config.clone(),
        rt,
        session,
        start,
        event_rx,
        alert_rx,
        web_change_rx,
    }))?;
    // can't turn this to util::error::Result :/
    ev_thread.join().expect("event loop thread");
    Ok(())
}

struct EventLoopArgs {
    web_state: web::State,

    config: Config,

    event_tx: Sender<Event>,
    alert_tx: Sender<Timestamp>,
    web_change_tx: Sender<web::Changed>,

    session: ForegroundWindowSessionInfo,
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

    let mut fg_watcher = ForegroundEventWatcher::new(
        args.session.window_session,
        &args.config,
        args.web_state.clone(),
    )?;
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

    let mut web_watcher = web::Watcher::new(args.web_change_tx, args.web_state)?;

    let dim_tick = Duration::from_millis(1000);
    let _web_tick_timer = Timer::new(
        dim_tick,
        Box::new(move || {
            web_watcher.tick().context("web watcher tick")?;

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
    config: Config,
    db_pool: DatabasePool,
    desktop_state: desktop::DesktopState,
    web_state: web::State,
    spawner: Handle,
    alert_rx: Receiver<Timestamp>,
    web_change_rx: Receiver<web::Changed>,
) -> Result<()> {
    let db = db_pool.get_db().await?;
    let sentry = Arc::new(Mutex::new(Sentry::new(
        config,
        desktop_state,
        web_state,
        db,
    )?));

    let _sentry = sentry.clone();
    let join1: JoinHandle<Result<()>> = spawner.spawn(async move {
        loop {
            let web_change = web_change_rx.recv_async().await?;
            let mut sentry = _sentry.lock().await;
            sentry.handle_web_change(web_change).await?;
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
    desktop_state: desktop::DesktopState,
    web_state: web::State,

    config: Config,
    rt: Handle,

    event_rx: Receiver<Event>,
    alert_rx: Receiver<Timestamp>,
    web_change_rx: Receiver<web::Changed>,

    session: ForegroundWindowSessionInfo,
    start: Timestamp,
}

/// Runs the [Engine] and [Sentry] loops in an asynchronous executor.
async fn processor(args: ProcessorArgs) -> Result<()> {
    let db_pool = DatabasePool::new(&args.config).await?;

    // re-run failed app info updates
    let _db_pool = db_pool.clone();
    let _handle = args.rt.clone();
    args.rt.clone().spawn(async move {
        update_app_infos(_db_pool, _handle)
            .await
            .context("update app infos")
            .error();
    });

    let sentry_handle = {
        let desktop_state = args.desktop_state.clone();
        let web_state = args.web_state.clone();
        let db_pool = db_pool.clone();
        let config = args.config.clone();
        let spawner = args.rt.clone();

        args.rt.clone().spawn(async move {
            (|| async {
                sentry_loop(
                    config.clone(),
                    db_pool.clone(),
                    desktop_state.clone(),
                    web_state.clone(),
                    spawner.clone(),
                    args.alert_rx.clone(),
                    args.web_change_rx.clone(),
                )
                .await
                .context("sentry loop")
            })
            .retry(
                ExponentialBuilder::new()
                    .with_min_delay(config.alert_duration())
                    .without_max_times(),
            )
            .notify(|err, _| {
                error!("sentry loop failed: {:?}", err);
            })
            .await
        })
    };

    (|(first_time, mut args, event_rx): (bool, EngineArgs, Receiver<Event>)| async move {
        // if this is not the first time, we need to get a new window session
        if !first_time {
            match foreground_window_session_async(&args.config, args.web_state.clone()).await {
                Ok(window_session) => {
                    args.start = Timestamp::now();
                    args.session = window_session;
                }
                Err(e) => {
                    return ((false, args.clone(), event_rx.clone()), Err(e));
                }
            }
        }

        (
            (false, args.clone(), event_rx.clone()),
            engine_loop(args, event_rx.clone())
                .await
                .context("engine loop"),
        )
    })
    .retry(
        ExponentialBuilder::new()
            .with_min_delay(args.config.poll_duration())
            .without_max_times(),
    )
    .context((
        true, // first time!
        EngineArgs {
            desktop_state: args.desktop_state.clone(),
            config: args.config.clone(),
            web_state: args.web_state.clone(),
            db_pool: db_pool.clone(),
            session: args.session.clone(),
            start: args.start,
            spawner: args.rt.clone(),
        },
        args.event_rx.clone(),
    ))
    .notify(|err, _| {
        error!("engine loop failed: {:?}", err);
    })
    .await
    .1
    .context("engine loop error")?;

    sentry_handle
        .await
        .context("join sentry loop")?
        .context("sentry loop error")?;
    Ok(())
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
fn foreground_window_session(
    config: &Config,
    web_state: web::State,
) -> Result<ForegroundWindowSessionInfo> {
    let detect = web::Detect::new()?;
    loop {
        let session = ForegroundEventWatcher::foreground_window_session(
            &detect,
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
) -> Result<ForegroundWindowSessionInfo> {
    let detect = web::Detect::new()?;
    loop {
        let session = ForegroundEventWatcher::foreground_window_session(
            &detect,
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
