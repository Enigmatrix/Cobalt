use data::db::Database;
use engine::{Engine, Event};
use platform::{
    events::{ForegroundChangedEvent, ForegroundEventWatcher},
    objects::{Timestamp, Window},
};
use resolver::{AppInfoResolver, AppInfoResolverRequest};
use util::{
    channels::{self, Receiver, Sender},
    config::{self, Config},
    error::Result,
    future::{
        executor::{LocalPool, LocalSpawner},
        task::LocalSpawnExt,
    },
};

mod engine;
mod resolver;
mod sentry;

fn main() -> Result<()> {
    let config = config::get_config()?;
    util::setup(&config)?;

    let (event_tx, event_rx) = channels::unbounded();
    let now = Timestamp::now();
    let fg = foreground_window();

    let mut fg_watcher = ForegroundEventWatcher::new(fg.clone())?;
    if let Some(ForegroundChangedEvent { at, window, title }) = fg_watcher.poll(now.clone())? {}

    println!("Hello, world!");
    processor(&config, fg, now, event_rx)?;
    Ok(())
}

async fn resolve_loop(
    config: &Config,
    rx: Receiver<AppInfoResolverRequest>,
    spawner: LocalSpawner,
) -> Result<()> {
    loop {
        let req = rx.recv_async().await?;
        let config = config.clone();
        spawner.spawn_local(async move {
            AppInfoResolver::update_app(&config, req)
                .await
                .expect("update app with info");
        })?;
    }
}

async fn engine_loop(
    config: &Config,
    rx: Receiver<Event>,
    resolve_tx: Sender<AppInfoResolverRequest>,
    fg: Window,
    now: Timestamp,
) -> Result<()> {
    let mut db = Database::new(&config)?;
    let mut engine = Engine::new(fg, now, &mut db, resolve_tx).await?;
    loop {
        let ev = rx.recv_async().await?;
        engine.handle(ev).await?;
    }
}

fn processor(config: &Config, fg: Window, now: Timestamp, rx: Receiver<Event>) -> Result<()> {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();
    let (resolve_tx, resolve_rx) = channels::unbounded();

    let _config = config.clone();
    pool.spawner().spawn_local(async move {
        engine_loop(&_config, rx, resolve_tx, fg, now)
            .await
            .expect("engine loop");
    })?;

    let _config = config.clone();
    pool.spawner().spawn_local(async move {
        resolve_loop(&_config, resolve_rx, spawner)
            .await
            .expect("resolver loop");
    })?;

    pool.run();

    Ok(())
}

fn foreground_window() -> Window {
    loop {
        if let Some(window) = Window::foreground() {
            return window;
        }
    }
}
