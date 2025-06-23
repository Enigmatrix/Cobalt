use std::collections::{HashMap, HashSet};

use util::channels::Sender;
use util::error::Result;

use crate::events::WindowTitleWatcher;
use crate::objects::{ProcessId, Target, Window};
use crate::web;

/// Watches a browser (by PID) for tab changes. Changes should be reported as fast as possible.
pub struct BrowserTabWatcher {
    browser_pids: HashMap<ProcessId, WindowTitleWatcher>,
    browser_state: web::State,
    tab_change_tx: Sender<TabChange>,
}

/// Probable change to the focused tab in some browser window.
#[derive(Debug, Clone)]
pub enum TabChange {
    /// User has switched to a different tab on a window.
    Tab {
        /// Window where the tab changed
        window: Window,
    },
    /// Periodic tick to check for changes in all browser windows.
    Tick,
}

impl BrowserTabWatcher {
    /// Create a new [BrowserTabWatcher]
    pub fn new(tab_change_tx: Sender<TabChange>, browser_state: web::State) -> Result<Self> {
        Ok(Self {
            browser_pids: HashMap::new(),
            browser_state,
            tab_change_tx,
        })
    }

    fn update_browsers(&mut self, pids: &HashSet<ProcessId>) -> Result<()> {
        // Remove any browsers that are no longer in the list
        self.browser_pids.retain(|pid, _| pids.contains(pid));

        // Add any new browsers to the list, watch them for title changes
        for pid in pids {
            if !self.browser_pids.contains_key(&pid) {
                let tab_change_tx = self.tab_change_tx.clone();
                let callback = Box::new(move |window: Window| {
                    tab_change_tx.send(TabChange::Tab { window })?;
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
                let state = self.browser_state.blocking_read();
                state.browser_processes.clone() // unfortunately we need to clone here
            };
            self.update_browsers(&pids)?;
        }

        self.tab_change_tx.send(TabChange::Tick)?;
        Ok(())
    }
}
