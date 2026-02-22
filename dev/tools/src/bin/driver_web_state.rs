//! Driver to poll browser window state as the engine is running and save it to a file

use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

use clap::Parser;
use engine::desktop;
use platform::browser::{self, ArcBrowser};
use platform::objects::Window;
use serde::Serialize;
use util::error::Result;
use util::tracing::error;
use util::{Target, config, future as tokio};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to save the events to
    #[arg(long, default_value = "data.json")]
    path: String,

    /// Delay between events polling in milliseconds
    #[arg(long, default_value = "10")]
    delay: u64,
}

fn main() -> Result<()> {
    util::set_target(Target::Tool {
        name: "driver_web_state".to_string(),
    });
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .enable_io()
        .build()?;

    let args = Args::parse();

    let browser: ArcBrowser = browser::uia::new_uia_backend()?;
    let desktop_state = desktop::new_desktop_state();

    let _browser = browser.clone();
    thread::spawn(move || {
        let mut driver = Driver::new(_browser);
        driver
            .run(&args.path, Duration::from_millis(args.delay))
            .expect("failed to run driver");
    });

    if let Err(report) = engine::run(&config, rt.handle().clone(), browser, desktop_state) {
        error!("fatal error caught in main: {:?}", report);
        std::process::exit(1);
    }
    Ok(())
}

struct Driver {
    browser: ArcBrowser,
    last_snapshot: Option<BrowserWindowSnapshot>,
}

impl Driver {
    fn new(browser: ArcBrowser) -> Self {
        Self {
            browser,
            last_snapshot: None,
        }
    }

    fn step(&mut self, ts: u64) -> Vec<Event> {
        let mut events = Vec::new();

        let Some(window) = Window::foreground() else {
            return events;
        };

        let Ok(Some(result)) = self.browser.identify(&window) else {
            return events;
        };

        let snapshot = BrowserWindowSnapshot {
            is_incognito: result.info.is_incognito,
            url: result.info.url,
            title: window.title().unwrap_or_default(),
        };

        let changed = self
            .last_snapshot
            .as_ref()
            .is_none_or(|last| *last != snapshot);

        if changed {
            events.push(Event::BrowserWindowSnapshotChange {
                snapshot: snapshot.clone(),
                timestamp: ts,
            });
            self.last_snapshot = Some(snapshot);
        }

        events
    }

    fn run(&mut self, path: &str, delay: Duration) -> Result<()> {
        let mut file = File::create(path)?;
        loop {
            let ts = time_now();
            let events = self.step(ts);

            for event in events {
                let json = serde_json::to_string(&event)?;
                file.write_all(json.as_bytes())?;
                file.write_all(b"\n")?;
            }

            thread::sleep(delay);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserWindowSnapshot {
    pub is_incognito: bool,
    pub url: String,
    pub title: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum Event {
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "state-change")]
    BrowserWindowSnapshotChange {
        #[serde(flatten)]
        snapshot: BrowserWindowSnapshot,
        timestamp: u64,
    },
}

fn time_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
