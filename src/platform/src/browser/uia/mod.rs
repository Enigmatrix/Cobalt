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
use crate::browser::website_info::{BaseWebsiteUrl, WebsiteInfo};
use crate::objects::{Process, Window};

pub(crate) mod detect;
pub(crate) mod state;
mod watcher;

pub use detect::Detect;

/// UIA-based [`Browser`] implementation for Chromium browsers (Chrome, Edge).
pub struct UiaBackend {
    detect: detect::Detect,
    state: state::State,
    actions: Arc<RwLock<HashMap<BaseWebsiteUrl, BrowserAction>>>,
    watcher: std::sync::Mutex<watcher::Watcher>,
}

impl UiaBackend {
    fn new() -> Result<Self> {
        let detect = detect::Detect::new()?;
        let state = state::default_state();
        let actions = Arc::new(RwLock::new(HashMap::new()));
        let watcher = watcher::Watcher::new(state.clone(), detect.clone(), actions.clone())?;
        Ok(Self {
            detect,
            state,
            actions,
            watcher: std::sync::Mutex::new(watcher),
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

        // Enforce on existing browser windows whose current URL matches.
        let state = self.state.blocking_read();
        for (window, bws) in &state.browser_windows {
            let Some(bws) = bws else { continue };
            let base_url = WebsiteInfo::url_to_base_url(&bws.last_url);
            let actions = self.actions.blocking_read();
            if let Some(action) = actions.get(&base_url) {
                match action {
                    BrowserAction::CloseTab => {
                        if let Ok(element) = bws.extracted_elements.window_element.resolve() {
                            self.detect.close_current_tab(&element).warn();
                        }
                    }
                    BrowserAction::Dim { opacity } => {
                        window.dim(*opacity).warn();
                    }
                }
            }
        }
        Ok(())
    }

    fn tick(&self) -> Result<()> {
        {
            let mut watcher = self.watcher.lock().expect("watcher lock poisoned");
            watcher.tick().context("watcher tick")?;
        }

        // Stale entry cleanup
        let mut state = self.state.blocking_write();
        state
            .browser_windows
            .retain(|window, _| window.ptid().is_ok());

        Ok(())
    }
}

/// Create a new UIA-based [`Browser`] backend.
///
/// Returns [`ArcBrowser`] so callers never depend on the concrete type.
pub fn new_uia_backend() -> Result<ArcBrowser> {
    Ok(Arc::new(UiaBackend::new()?))
}
