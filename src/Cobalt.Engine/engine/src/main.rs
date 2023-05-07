#![feature(async_closure)]

mod app_info_resolver;
mod cache;
mod processor;

use std::future::Future;
use std::thread;
use std::thread::JoinHandle;

use common::channels::*;
use common::errors::*;
use common::settings::*;
use common::tracing::*;
use data::*;
use platform::{
    objects::{EventLoop, Process, Timer, Timestamp, Window},
    watchers::{self, InteractionStateChange, WindowSession},
};
use tokio::task::spawn_local;
use tokio::task::LocalSet;

use crate::app_info_resolver::AppInfoResolver;
use crate::processor::ProcessorEvent;

fn main() -> Result<()> {
    let settings = Settings::from_file("appsettings.json").context("fetch settings")?;
    common::setup(&settings).context("setup common")?;
    platform::setup().context("setup platform")?;

    let foreground_window = loop {
        let window = Window::foreground();
        if let Some(window) = window {
            break window;
        }
    };
    let start = Timestamp::now();

    info!("engine is running");

    // TODO from config
    let idle_timeout = platform::objects::Duration::from_millis(5_000);
    let every = platform::objects::Duration::from_millis(1_000);

    let (event_tx, event_rx) = channel();
    let (app_info_tx, mut app_info_rx) = tokio::sync::mpsc::unbounded_channel();

    let watcher = {
        let event_tx = event_tx;
        run_win_event_loop_thread(move |ev| {
            let mut foreground = watchers::Foreground::new(foreground_window.clone())
                .context("create foreground watcher")?;
            let interaction = watchers::Interaction::initialize(idle_timeout, start)
                .context("initialize global interaction watcher")?;
            let _timer = Timer::new(every, every, &mut || {
                let now = Timestamp::now();
                if let Some(change) = foreground.trigger().context("trigger foreground watcher")? {
                    event_tx
                        .send(ProcessorEvent::WindowSession { at: now, change })
                        .context("send window session change")?;
                }
                if let Some(change) = interaction
                    .trigger(now)
                    .context("trigger interaction watcher")?
                {
                    event_tx
                        .send(ProcessorEvent::InteractionStateChange { at: now, change })
                        .context("send interaction state change")?;
                }
                Ok(())
            })
            .context("create timer")?;

            ev.run();
            Ok(())
        })
    };

    let resolver = {
        let settings = settings.clone();
        run_single_threaded_tokio(async move {
            loop {
                let app_info = app_info_rx.recv().await;
                if let Some(app_info) = app_info {
                    let settings = settings.clone();
                    spawn_local(async move {
                        (async move || -> Result<()> {
                            let db = Database::new(&settings).context("create db for app info")?;
                            let resolver = AppInfoResolver::default();
                            resolver
                                .resolve(db, app_info)
                                .await
                                .context("resolve app info")?;
                            Ok(())
                        })()
                        .await
                        .context("spawn task for app info")
                        .unwrap();
                    });
                } else {
                    break;
                }
            }
        })
    };

    let db = Database::new(&settings).context("create db for processor")?;
    for change in event_rx {
        match change {
            ProcessorEvent::WindowSession {
                change: WindowSession { window, title },
                ..
            } => {
                let process = Process::new(window.pid()?)?;
                let path = process.path()?;
                info!(title);
                if process.is_uwp(Some(&path))? {
                    let aumid = window.aumid()?;
                    info!(aumid);
                } else {
                    info!(path = path);
                }
            }
            ProcessorEvent::InteractionStateChange {
                change: InteractionStateChange::Active,
                ..
            } => {
                warn!("Active!")
            }
            ProcessorEvent::InteractionStateChange {
                change:
                    InteractionStateChange::Idle {
                        mouseclicks,
                        keystrokes,
                    },
                ..
            } => warn!("Idle, recorded m={}, k={}", mouseclicks, keystrokes),
        }
    }

    watcher.join().unwrap();
    resolver.join().unwrap();

    Ok(())
}

fn run_win_event_loop_thread<F: Send + 'static + FnMut(&EventLoop) -> Result<()>>(
    mut f: F,
) -> JoinHandle<()> {
    thread::spawn(move || {
        (move || -> Result<()> {
            let ev = EventLoop::new();

            f(&ev)?;

            Ok(())
        })()
        .unwrap();
    })
}

fn run_single_threaded_tokio<F: Send + 'static + Future>(f: F) -> JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        let tasks = LocalSet::new();
        tasks.block_on(&rt, f);
    })
}
