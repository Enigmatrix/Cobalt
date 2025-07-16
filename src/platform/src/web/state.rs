use std::sync::Arc;

use util::ds::{SmallHashMap, SmallHashSet};
use util::error::Result;
use util::future::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use windows::Win32::UI::Accessibility::IUIAutomationElement9;
use windows::core::AgileReference;

use crate::objects::{ProcessId, Window};

/// Shared inner state of browsers and websites seen in the desktop
#[derive(Debug, Default)]
pub struct StateInner {
    /// Cache of whether a window is a browser or not.
    /// Not present means that we don't know if it's a browser or not.
    /// Value = None means it's not a browser window.
    /// Value = Some(BrowserWindowState) means it's a browser window and this is the state.
    pub browser_windows: SmallHashMap<Window, Option<BrowserWindowState>>,
    /// Processes that are known to be browsers.
    pub browser_processes: SmallHashSet<ProcessId>,
}

/// State of a browser window
#[derive(Debug, Clone)]
pub struct BrowserWindowState {
    /// UI Automation element of the window
    pub window_element: AgileReference<IUIAutomationElement9>,
    /// Whether the window is in incognito mode
    pub is_incognito: bool,
    /// Last URL of the window
    pub last_url: String,
    /// Last title of the window
    pub last_title: String,
}

impl StateInner {
    /// Get the UI Automation element for the [Window]
    pub fn get_browser_window_element(
        &self,
        window: &Window,
    ) -> Result<Option<IUIAutomationElement9>> {
        let Some(state) = self.browser_windows.get(window) else {
            return Ok(None);
        };

        let Some(state) = state else {
            return Ok(None);
        };

        Ok(Some(state.window_element.resolve()?))
    }
}

/// Shared state of browsers and websites seen in the desktop
pub type State = Arc<RwLock<StateInner>>;

/// Write locked state of [State]
pub type WriteLockedState<'a> = RwLockWriteGuard<'a, StateInner>;
/// Read locked state of [State]
pub type ReadLockedState<'a> = RwLockReadGuard<'a, StateInner>;

/// Default state of [State]
pub fn default_state() -> State {
    Arc::new(RwLock::new(StateInner::default()))
}
