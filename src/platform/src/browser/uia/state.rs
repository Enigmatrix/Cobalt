use std::sync::Arc;

use util::ds::{SmallHashMap, SmallHashSet};
use util::error::Result;
use util::future::sync::RwLock;
use windows::Win32::UI::Accessibility::IUIAutomationElement9;
use windows::core::AgileReference;

use crate::objects::{ProcessId, Window};

/// Shared inner state of browsers and websites seen in the desktop
#[derive(Debug, Default)]
pub(super) struct StateInner {
    /// Cache of whether a window is a browser or not.
    pub(super) browser_windows: SmallHashMap<Window, Option<BrowserWindowState>>,
    /// Processes that are known to be browsers.
    pub(super) browser_processes: SmallHashSet<ProcessId>,
}

/// State of a browser window
#[derive(Debug, Clone)]
pub(super) struct BrowserWindowState {
    /// Extracted UI Automation elements of the window
    pub(super) extracted_elements: ExtractedUIElements,
    /// Whether the window is in incognito mode
    pub(super) is_incognito: bool,
    /// Last URL of the window
    pub(super) last_url: String,
    /// Last title of the window
    pub(super) last_title: String,
}

/// Extracted UI Automation elements of a browser window
#[derive(Debug, Clone)]
pub(super) struct ExtractedUIElements {
    /// UI Automation element of the window
    pub(super) window_element: AgileReference<IUIAutomationElement9>,
    /// UI Automation element of the omnibox
    pub(super) omnibox: AgileReference<IUIAutomationElement9>,
    /// UI Automation element of the omnibox icon
    pub(super) omnibox_icon: AgileReference<IUIAutomationElement9>,
}

impl StateInner {
    pub(super) fn get_browser_window(
        &self,
        window: &Window,
    ) -> Result<Option<&BrowserWindowState>> {
        let Some(state) = self.browser_windows.get(window) else {
            return Ok(None);
        };
        let Some(state) = state else {
            return Ok(None);
        };
        Ok(Some(state))
    }

    pub(super) fn get_browser_window_mut(
        &mut self,
        window: &Window,
    ) -> Result<Option<&mut BrowserWindowState>> {
        let Some(state) = self.browser_windows.get_mut(window) else {
            return Ok(None);
        };
        let Some(state) = state.as_mut() else {
            return Ok(None);
        };
        Ok(Some(state))
    }
}

/// Shared state of browsers and websites seen in the desktop
pub(super) type State = Arc<RwLock<StateInner>>;

/// Default state of [State]
pub(super) fn default_state() -> State {
    Arc::new(RwLock::new(StateInner::default()))
}
