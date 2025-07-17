//! Driver to poll the [web::State] as the engine is runningand save it to a file

use std::thread;
use std::time::UNIX_EPOCH;

use engine::desktop;
use platform::objects::Window;
use platform::web;
use util::ds::SmallHashMap;
use util::error::Result;
use util::tracing::error;
use util::{Target, config, future as tokio};

fn main() -> Result<()> {
    util::set_target(Target::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .enable_io()
        .build()?;

    let web_state = web::default_state();
    let desktop_state = desktop::new_desktop_state(web_state.clone());

    let _web_state = web_state.clone();
    thread::spawn(move || {
        let mut driver = Driver::new(_web_state);
        loop {
            let ts = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let events = driver.step(ts);
            for event in events {
                println!("{event:?}");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    if let Err(report) = engine::run(&config, rt.handle().clone(), web_state, desktop_state) {
        error!("fatal error caught in main: {:?}", report);
        std::process::exit(1);
    }
    Ok(())
}

struct Driver {
    current_state: RecordedState,
    state: web::State,
}

impl Driver {
    fn new(state: web::State) -> Self {
        let current_state = RecordedState::from(&*state.blocking_read());
        Self {
            state,
            current_state,
        }
    }

    fn step(&mut self, ts: u64) -> Vec<Event> {
        let new_state = RecordedState::from(&*self.state.blocking_read());
        let events = self.current_state.diff(&new_state, ts);
        self.current_state = new_state;
        events
    }
}

/// Shared inner state of browsers and websites seen in the desktop
#[derive(Debug, Default)]
struct RecordedState {
    pub browser_windows: SmallHashMap<Window, Option<BrowserWindowContext>>,
    // pub browser_processes: SmallHashSet<ProcessId>,
}

impl From<&web::StateInner> for RecordedState {
    fn from(state: &web::StateInner) -> Self {
        Self {
            browser_windows: state
                .browser_windows
                .iter()
                .map(|(window, state)| {
                    (
                        window.clone(),
                        state.clone().map(|state| BrowserWindowContext {
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

impl RecordedState {
    fn diff(&self, new_state: &RecordedState, ts: u64) -> Vec<Event> {
        let mut events = Vec::new();

        for (window, ctx) in &new_state.browser_windows {
            let Some(ctx) = ctx else { continue };
            let old_ctx = self.browser_windows.get(window);
            let change = if let Some(old_state) = old_ctx {
                let old_ctx = old_state
                    .as_ref()
                    .expect("browser window should remain browser window from start");
                old_ctx != ctx
            } else {
                true
            };

            if change {
                events.push(Event::BrowserWindowContextChanged {
                    context: ctx.clone(),
                    timestamp: ts,
                });
            }
        }

        events
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct BrowserWindowContext {
    pub is_incognito: bool,
    pub url: String,
    pub title: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Event {
    BrowserWindowContextChanged {
        context: BrowserWindowContext,
        timestamp: u64,
    },
    // WindowFocused {
    //     window: Window,
    //     timestamp: u64,
    // },
}
