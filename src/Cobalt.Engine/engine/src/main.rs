use std::thread;
use std::rc::Rc;
use std::cell::RefCell;

use data::db::Database;
use engine::{Engine, Event};
use platform::{
    events::{ForegroundChangedEvent, ForegroundEventWatcher, InteractionWatcher},
    objects::{EventLoop, Timer, Timestamp, Window},
};
use util::{
    channels::{self, Receiver, Sender},
    config::{self, Config},
    error::Result,
    future::{
        executor::{LocalPool, LocalSpawner},
        task::LocalSpawnExt,
    },
};

mod cache;
mod engine;
mod resolver;
mod sentry;

fn main() -> Result<()> {
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let (event_tx, event_rx) = channels::unbounded();
    let now = Timestamp::now();
    let fg = foreground_window();

    let ev_thread = {
        let config = config.clone();
        let fg = fg.clone();
        thread::spawn(move || {
            event_loop(&config, event_tx, fg, now).expect("event loop");
        })
    };

    processor(&config, fg, now, event_rx)?;
    ev_thread.join().expect("event loop thread");
    Ok(())
}

fn event_loop(config: &Config, event_tx: Sender<Event>, fg: Window, now: Timestamp) -> Result<()> {
    let ev = EventLoop::new();

    let mut fg_watcher = ForegroundEventWatcher::new(fg)?;
    let mut it_watcher = InteractionWatcher::new(config, now);

    let every = config.poll_duration().into();
    let _timer = Timer::new(every, every, &mut || {
        let now = Timestamp::now();
        if let Some(ForegroundChangedEvent { at, window, title }) = fg_watcher.poll(now)? {
            event_tx.send(Event::ForegroundChanged(ForegroundChangedEvent {
                at,
                window,
                title,
            }))?;
        }
        if let Some(event) = it_watcher.poll(now)? {
            event_tx.send(Event::InteractionChanged(event))?;
        }
        Ok(())
    })?;

    ev.run();
    Ok(())
}

async fn engine_loop(
    config: &Config,
    rx: Receiver<Event>,
    spawner: &LocalSpawner,
    fg: Window,
    now: Timestamp,
) -> Result<()> {
    let mut db = Database::new(config)?;
    let cache = Rc::new(RefCell::new(cache::Cache::new()));
    let mut engine = Engine::new(cache, fg, now, config.clone(), &mut db, spawner).await?;
    loop {
        let ev = rx.recv_async().await?;
        engine.handle(ev).await?;
    }
}

fn processor(config: &Config, fg: Window, now: Timestamp, rx: Receiver<Event>) -> Result<()> {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    let config = config.clone();
    pool.spawner().spawn_local(async move {
        engine_loop(&config, rx, &spawner, fg, now)
            .await
            .expect("engine loop");
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
