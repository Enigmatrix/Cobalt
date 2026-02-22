use std::collections::HashMap;
use std::sync::Arc;

use util::error::{Context, ContextCompat, Result};
use util::future::sync::RwLock;
use util::tracing::ResultTraceExt;
use windows::Win32::UI::Accessibility::UIA_WindowControlTypeId;
use windows::core::AgileReference;

use crate::browser::actions::BrowserAction;
use crate::browser::backend::{
    ArcBrowser, Browser, BrowserWindowInfo, IdentifyResult, ResolvedWindowInfo,
};
use crate::browser::website_info::BaseWebsiteUrl;
use crate::objects::{Process, Window};

pub(crate) mod detect;
pub(crate) mod state;
pub(crate) mod watcher;

// Backward-compat re-exports (temporary, removed in step 6)
pub use detect::*;
pub use state::*;
pub use watcher::*;

/// UIA-based [`Browser`] implementation for Chromium browsers (Chrome, Edge).
///
/// During the transition period (steps 3-4) this wraps a shared [`state::State`]
/// so the existing [`watcher::Watcher`] and this backend see the same data.
pub struct UiaBackend {
    detect: detect::Detect,
    state: state::State,
    actions: Arc<RwLock<HashMap<BaseWebsiteUrl, BrowserAction>>>,
}

impl UiaBackend {
    /// Create a new [`UiaBackend`] wrapping the given shared state.
    pub fn new(state: state::State) -> Result<Self> {
        Ok(Self {
            detect: detect::Detect::new()?,
            state,
            actions: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

impl Browser for UiaBackend {
    fn identify(&self, window: &Window) -> Result<Option<IdentifyResult>> {
        let mut state = self.state.blocking_write();

        // Fast path: window already classified.
        if let Some(entry) = state.browser_windows.get(window) {
            return Ok(entry.as_ref().map(|bws| IdentifyResult {
                info: BrowserWindowInfo {
                    url: bws.last_url.clone(),
                    is_incognito: bws.is_incognito,
                },
                resolved_info: ResolvedWindowInfo::default(),
            }));
        }

        // Cheap heuristic: window class name.
        let maybe_browser = self.detect.is_maybe_chromium_window(window).warn();

        let (is_browser, resolved_info) = if maybe_browser {
            let ptid = window.ptid().context("get ptid")?;
            if state.browser_processes.contains(&ptid.pid) {
                (true, ResolvedWindowInfo::default())
            } else {
                let process = Process::new(ptid.pid).context("get process")?;
                let path = process.path().context("get process path")?;
                let is_browser = self.detect.is_chromium_exe(&path);
                if is_browser {
                    state.browser_processes.insert(ptid.pid);
                }
                (
                    is_browser,
                    ResolvedWindowInfo {
                        exe_path: Some(path),
                    },
                )
            }
        } else {
            (false, ResolvedWindowInfo::default())
        };

        if !is_browser {
            state.browser_windows.insert(window.clone(), None);
            return Ok(None);
        }

        let window_element = self.detect.get_chromium_element(window)?;

        // Filter out dialogs (Ctrl-F, extension popups, etc.)
        if unsafe { window_element.CurrentControlType()? } != UIA_WindowControlTypeId {
            state.browser_windows.insert(window.clone(), None);
            return Ok(None);
        }

        let omnibox = self
            .detect
            .get_chromium_omnibox_element(&window_element, true)?
            .context("no omnibox")?;
        let omnibox_icon = self
            .detect
            .get_chromium_omnibox_icon_element(&window_element, true)?
            .context("no omnibox icon")?;

        let is_incognito = self
            .detect
            .chromium_incognito(&window_element)
            .context("get chromium incognito")?
            .context("no element")?;
        let last_url = self
            .detect
            .chromium_url(&window_element)
            .context("get chromium url")?
            .context("no element")?;
        let last_title = window.title()?;

        let extracted_elements = state::ExtractedUIElements {
            window_element: AgileReference::new(&window_element)?,
            omnibox: AgileReference::new(&omnibox)?,
            omnibox_icon: AgileReference::new(&omnibox_icon)?,
        };

        let bws = state::BrowserWindowState {
            extracted_elements,
            is_incognito,
            last_url: last_url.clone(),
            last_title,
        };
        state.browser_windows.insert(window.clone(), Some(bws));

        Ok(Some(IdentifyResult {
            info: BrowserWindowInfo {
                url: last_url,
                is_incognito,
            },
            resolved_info,
        }))
    }

    fn set_actions(&self, actions: HashMap<BaseWebsiteUrl, BrowserAction>) -> Result<()> {
        *self.actions.blocking_write() = actions;
        Ok(())
    }

    fn tick(&self) -> Result<()> {
        // No-op during transition; watcher is still driven externally.
        Ok(())
    }
}

/// Create a new UIA-based [`Browser`] backend wrapping the given shared state.
///
/// Returns [`ArcBrowser`] so callers never depend on the concrete type.
pub fn new_uia_backend(state: state::State) -> Result<ArcBrowser> {
    Ok(Arc::new(UiaBackend::new(state)?))
}
