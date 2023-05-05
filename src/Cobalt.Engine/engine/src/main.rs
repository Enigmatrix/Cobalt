use std::sync::mpsc::channel;
use std::thread;

use common::errors::*;
use common::settings::Settings;
use common::tracing::*;
use platform::{
    objects::{EventLoop, Process, Timer, Timestamp, Window},
    watchers::{self, InteractionStateChange, WindowSession},
};

pub enum Event {
    WindowSession(WindowSession),
    InteractionStateChange(InteractionStateChange),
}

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

    let watcher_event_tx = event_tx;
    let _watcher = thread::spawn(move || {
        (|| -> Result<()> {
            let ev = EventLoop::new();
            let mut foreground = watchers::Foreground::new(foreground_window)
                .context("create foreground watcher")?;
            let interaction = watchers::Interaction::initialize(idle_timeout, start)
                .context("initialize global interaction watcher")?;
            let _timer = Timer::new(every, every, &mut || {
                let now = Timestamp::now();
                if let Some(change) = foreground.trigger().context("trigger foreground watcher")? {
                    watcher_event_tx
                        .send(Event::WindowSession(change))
                        .context("send window session change")?;
                }
                if let Some(change) = interaction
                    .trigger(now)
                    .context("trigger interaction watcher")?
                {
                    watcher_event_tx
                        .send(Event::InteractionStateChange(change))
                        .context("send interaction state change")?;
                }
                Ok(())
            });
            ev.run();
            Ok(())
        })()
        .unwrap()
    });

    for change in event_rx {
        match change {
            Event::WindowSession(WindowSession { window, title }) => {
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
            Event::InteractionStateChange(InteractionStateChange::Active) => warn!("Active!"),
            Event::InteractionStateChange(InteractionStateChange::Idle {
                mouseclicks,
                keystrokes,
            }) => warn!("Idle, recorded m={}, k={}", mouseclicks, keystrokes),
        }
    }

    _watcher.join().unwrap();

    Ok(())
}
