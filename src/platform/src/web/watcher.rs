use util::channels::Sender;
use util::ds::{SmallHashMap, SmallHashSet};
use util::error::{Context, ContextCompat, Result};

use crate::events::WindowTitleWatcher;
use crate::objects::{ProcessId, Target, Window};
use crate::web;

/// Watches a browser (by PID) for tab changes. Changes should be reported as fast as possible.
pub struct Watcher {
    detect: web::Detect,
    web_state: web::State,

    processes: SmallHashMap<ProcessId, WindowTitleWatcher>,

    web_change_tx: Sender<Changed>,
}

/// Probable change to the focused tab in some browser window.
#[derive(Debug, Clone)]
pub struct Changed {
    /// Window where the tab changed
    pub window: Window,
    /// URL of the tab that the user has switched to
    pub url: String,
    /// Title of the tab that the user has switched to
    pub title: String,
    /// Whether the window is incognito
    pub is_incognito: bool,
}

impl Watcher {
    /// Create a new [Watcher]. tick() needs to be called at least once to start watching.
    pub fn new(web_change_tx: Sender<Changed>, web_state: web::State) -> Result<Self> {
        Ok(Self {
            processes: SmallHashMap::new(),
            detect: web::Detect::new()?,
            web_state,
            web_change_tx,
        })
    }

    /// Send a tick to recheck all browser windows and update browser PID subscriptions.
    pub fn tick(&mut self) -> Result<()> {
        {
            let pids = {
                let state = self.web_state.blocking_read();
                state.browser_processes.clone() // unfortunately we need to clone here
            };
            self.update_browsers(&pids)?;
        }
        Ok(())
    }

    fn update_browsers(&mut self, pids: &SmallHashSet<ProcessId>) -> Result<()> {
        // Remove any browsers that are no longer in the list
        self.processes.retain(|pid, _| pids.contains(pid));

        // Add any new browsers to the list, watch them for title changes
        for pid in pids {
            if !self.processes.contains_key(pid) {
                let web_state = self.web_state.clone();

                let callback = Box::new(move |window: Window| {
                    Self::title_change_callback(window, web_state.clone())
                });
                self.processes
                    .insert(*pid, WindowTitleWatcher::new(Target::Id(*pid), callback)?);
            }
        }
        Ok(())
    }

    fn title_change_callback(window: Window, web_state: web::State) -> Result<()> {
        let title = window.title()?;

        {
            let mut web_state = web_state.blocking_write();
            if let Some(state) = web_state.get_browser_window_mut(&window)? {
                state.last_title = title;
            }
        }

        Ok(())
    }
}
