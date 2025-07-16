use util::channels::Sender;
use util::ds::{SmallHashMap, SmallHashSet};
use util::error::{Context, ContextCompat, Result};

use crate::events::WindowTitleWatcher;
use crate::objects::{ProcessId, Target, Window};
use crate::web;

/// Watches a browser (by PID) for tab changes. Changes should be reported as fast as possible.
pub struct BrowserTabWatcher {
    browser_pids: SmallHashMap<ProcessId, WindowTitleWatcher>,
    detect: web::Detect,
    web_state: web::State,
    tab_change_tx: Sender<TabChange>,
}

/// Probable change to the focused tab in some browser window.
#[derive(Debug, Clone)]
pub enum TabChange {
    /// User has switched to a different tab on a window.
    Tab {
        /// Window where the tab changed
        window: Window,
        /// URL of the tab that the user has switched to
        url: String,
    },
    /// Periodic tick to check for changes in all browser windows.
    Tick,
}

impl BrowserTabWatcher {
    /// Create a new [BrowserTabWatcher]
    pub fn new(tab_change_tx: Sender<TabChange>, web_state: web::State) -> Result<Self> {
        Ok(Self {
            browser_pids: SmallHashMap::new(),
            detect: web::Detect::new()?,
            web_state,
            tab_change_tx,
        })
    }

    fn update_browsers(&mut self, pids: &SmallHashSet<ProcessId>) -> Result<()> {
        // Remove any browsers that are no longer in the list
        self.browser_pids.retain(|pid, _| pids.contains(pid));

        // Add any new browsers to the list, watch them for title changes
        for pid in pids {
            if !self.browser_pids.contains_key(pid) {
                let tab_change_tx = self.tab_change_tx.clone();
                let detect = self.detect.clone();
                let web_state = self.web_state.clone();
                let callback = Box::new(move |window: Window| {
                    let element = {
                        let web_state = web_state.blocking_read();
                        let Some(element) = web_state.get_browser_window_element(&window)? else {
                            return Ok(());
                        };
                        element
                    };

                    // TODO: need to get retry logic here
                    let last_url = detect
                        .chromium_url(&element)
                        .context("get chromium url")?
                        .context("no element")?;
                    let last_title = window.title()?;

                    {
                        let mut web_state = web_state.blocking_write();
                        // only None when the window has been removed and this callback was called right after,
                        // but before the callback was dropped
                        if let Some(state) = web_state.browser_windows.get_mut(&window) {
                            let state = state.as_mut().expect("browser window state exists");
                            state.last_url = last_url.clone();
                            state.last_title = last_title;
                        }
                    }
                    tab_change_tx.send(TabChange::Tab {
                        window,
                        url: last_url,
                    })?;
                    Ok(())
                });
                self.browser_pids
                    .insert(*pid, WindowTitleWatcher::new(Target::Id(*pid), callback)?);
            }
        }
        Ok(())
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

        self.tab_change_tx.send(TabChange::Tick)?;
        Ok(())
    }
}
