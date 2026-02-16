//! Driver to poll the [browser::State] as the engine is runningand save it to a file

use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

use clap::Parser;
use engine::desktop;
use platform::browser;
use platform::objects::Window;
use serde::Serialize;
use util::ds::SmallHashMap;
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

    let web_state = browser::default_state();
    let desktop_state = desktop::new_desktop_state(web_state.clone());

    let _web_state = web_state.clone();
    thread::spawn(move || {
        let mut driver = Driver::new(_web_state);
        driver
            .run(&args.path, Duration::from_millis(args.delay))
            .expect("failed to run driver");
    });

    if let Err(report) = engine::run(&config, rt.handle().clone(), web_state, desktop_state) {
        error!("fatal error caught in main: {:?}", report);
        std::process::exit(1);
    }
    Ok(())
}

struct Driver {
    current_state: WebStateSnapshot,
    state: browser::State,
}

impl Driver {
    fn new(state: browser::State) -> Self {
        let current_state = WebStateSnapshot::from(&*state.blocking_read());
        Self {
            state,
            current_state,
        }
    }

    fn step(&mut self, ts: u64) -> Vec<Event> {
        let new_state = WebStateSnapshot::from(&*self.state.blocking_read());
        let events = self.current_state.diff(&new_state, ts);
        self.current_state = new_state;
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

/// Shared inner state of browsers and websites seen in the desktop
#[derive(Debug, Default)]
struct WebStateSnapshot {
    pub browser_windows: SmallHashMap<Window, Option<BrowserWindowSnapshot>>,
    // pub browser_processes: SmallHashSet<ProcessId>,
}

impl From<&browser::StateInner> for WebStateSnapshot {
    fn from(state: &browser::StateInner) -> Self {
        Self {
            browser_windows: state
                .browser_windows
                .iter()
                .map(|(window, state)| {
                    (
                        window.clone(),
                        state.clone().map(|state| BrowserWindowSnapshot {
                            is_incognito: state.is_incognito,
                            url: state.last_url.clone(),
                            title: state.last_title.clone(),
                        }),
                    )
                })
                .collect(),
            // browser_processes: state.browser_processes.clone(),
        }
    }
}

impl WebStateSnapshot {
    fn diff(&self, new_state: &WebStateSnapshot, ts: u64) -> Vec<Event> {
        let mut events = Vec::new();

        for (window, snapshot) in &new_state.browser_windows {
            let Some(snapshot) = snapshot else { continue };
            let old_snapshot = self.browser_windows.get(window);
            let change = if let Some(old_snapshot) = old_snapshot {
                let old_snapshot = old_snapshot
                    .as_ref()
                    .expect("browser window should remain browser window from start");
                old_snapshot != snapshot
            } else {
                true
            };

            if change {
                events.push(Event::BrowserWindowSnapshotChange {
                    snapshot: snapshot.clone(),
                    timestamp: ts,
                });
            }
        }

        events
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
    // WindowFocused {
    //     window: Window,
    //     timestamp: u64,
    // },
}

fn time_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
