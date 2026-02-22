use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use util::ds::{SmallHashMap, SmallHashSet};
use util::error::{Context, ContextCompat, Result};
use util::future::sync::RwLock;
use util::tracing::{self, ResultTraceExt, debug, instrument};
use windows::Win32::UI::Accessibility::{
    TreeScope_Element, UIA_AriaRolePropertyId, UIA_NamePropertyId, UIA_PROPERTY_ID,
    UIA_ValueValuePropertyId,
};

use super::detect::{self, perf};
use super::state;
use crate::browser::actions::BrowserAction;
use crate::browser::website_info::{BaseWebsiteUrl, WebsiteInfo};
use crate::events::{PropertyChange, WindowTitleWatcher};
use crate::objects::{ProcessId, Target, Window};

/// Watches browser windows for tab changes via UIA property-change events.
pub(super) struct Watcher {
    processes: SmallHashMap<ProcessId, WindowTitleWatcher>,
    windows: SmallHashMap<Window, PropertyChange>,

    args: Args,
}

// SAFETY: The only non-Send field is HWINEVENTHOOK (raw pointer) inside
// WindowTitleWatcher. Win32 event hooks can be safely unhooked from any
// thread, and we never dereference the pointer ourselves.
unsafe impl Send for Watcher {}

impl Watcher {
    /// Create a new [Watcher].
    pub fn new(
        state: state::State,
        detect: detect::Detect,
        actions: Arc<RwLock<HashMap<BaseWebsiteUrl, BrowserAction>>>,
    ) -> Result<Self> {
        Ok(Self {
            processes: SmallHashMap::new(),
            windows: SmallHashMap::new(),

            args: Args {
                web_state: state,
                detect,
                reentrancy: Arc::new(Mutex::new(())),
                actions,
            },
        })
    }

    /// Send a tick to recheck all browser windows and update browser PID subscriptions.
    pub fn tick(&mut self) -> Result<()> {
        let (pids, windows) = {
            let state = self.args.web_state.blocking_read();
            let pids = state.browser_processes.clone();
            let windows = state
                .browser_windows
                .iter()
                .flat_map(|(window, state)| {
                    state
                        .as_ref()
                        .map(|state| (window.clone(), state.extracted_elements.clone()))
                })
                .collect();
            (pids, windows)
        };
        self.update_browser_windows(windows)?;
        self.update_browsers_processes(&pids)?;
        Ok(())
    }

    fn update_browsers_processes(&mut self, pids: &SmallHashSet<ProcessId>) -> Result<()> {
        self.processes.retain(|pid, _| pids.contains(pid));

        for pid in pids {
            if !self.processes.contains_key(pid) {
                let args = self.args.clone();

                let callback = Box::new(move |window: Window| {
                    Self::title_change_callback(window, args.clone())
                });
                self.processes
                    .insert(*pid, WindowTitleWatcher::new(Target::Id(*pid), callback)?);
            }
        }
        Ok(())
    }

    fn update_browser_windows(
        &mut self,
        windows: SmallHashMap<Window, state::ExtractedUIElements>,
    ) -> Result<()> {
        self.windows
            .retain(|window, _| windows.contains_key(window));

        for (window, extracted_elements) in windows {
            if !self.windows.contains_key(&window) {
                let omnibox = extracted_elements.omnibox.clone();

                let args = self.args.clone();

                let callback = {
                    let window = window.clone();
                    let args = args.clone();

                    Box::new(move |_eventid: UIA_PROPERTY_ID, value: String| {
                        Self::omnibox_text_change_callback(
                            window.clone(),
                            value,
                            extracted_elements.clone(),
                            args.clone(),
                        )
                    })
                };

                let handler = PropertyChange::new(
                    self.args.detect.automation.clone(),
                    omnibox,
                    TreeScope_Element,
                    None,
                    &[UIA_ValueValuePropertyId],
                    callback,
                )?;

                self.windows.insert(window, handler);
            }
        }

        Ok(())
    }

    #[instrument(level = "trace", skip(args))]
    fn title_change_callback(window: Window, args: Args) -> Result<()> {
        let extracted_elements = {
            let state = args.web_state.blocking_read();
            let Some(state) = state.get_browser_window(&window)? else {
                return Ok(());
            };
            state.extracted_elements.clone()
        };

        Self::central_callback(window, None, extracted_elements, args)
            .context("title change callback")
    }

    #[instrument(level = "trace", skip(extracted_elements, args))]
    fn omnibox_text_change_callback(
        window: Window,
        value: String,
        extracted_elements: state::ExtractedUIElements,
        args: Args,
    ) -> Result<()> {
        Self::central_callback(window, Some(value), extracted_elements, args)
            .context("omnibox text change callback")
    }

    fn central_callback(
        window: Window,
        url: Option<String>,
        extracted_elements: state::ExtractedUIElements,
        args: Args,
    ) -> Result<()> {
        let _guard = args.reentrancy.lock().expect("reentrancy lock poisoned");

        let icon_role = perf(
            || unsafe {
                extracted_elements
                    .omnibox_icon
                    .resolve()?
                    .GetCurrentPropertyValue(UIA_AriaRolePropertyId)
            },
            "omnibox_icon - GetCurrentPropertyValue(UIA_AriaRolePropertyId)",
        )?
        .to_string();

        let url = if let Some(url) = url {
            url
        } else {
            let omnibox = extracted_elements.omnibox.resolve()?;
            perf(
                || unsafe { omnibox.GetCurrentPropertyValue(UIA_ValueValuePropertyId) },
                "omnibox - GetCurrentPropertyValue(UIA_ValueValuePropertyId)",
            )?
            .to_string()
        };

        let new_url = if icon_role == "button" && !url.is_empty() {
            let omnibox_icon = extracted_elements.omnibox_icon.resolve()?;
            let icon_text = perf(
                || unsafe { omnibox_icon.GetCurrentPropertyValue(UIA_NamePropertyId) },
                "omnibox_icon - GetCurrentPropertyValue(UIA_NamePropertyId)",
            )?
            .to_string();

            let is_http_hint = detect::Detect::is_http_hint_from_icon_text(&icon_text);
            debug!(?url, ?icon_text, "tab changed");
            detect::Detect::unelide_omnibox_text(url, is_http_hint)
        } else {
            args.detect
                .chromium_url(&extracted_elements.window_element.resolve()?)
                .context("get chromium url")?
                .context("no element")?
        };
        let last_title = window.title()?;

        let prev_url = {
            let mut web_state = args.web_state.blocking_write();
            let Some(state) = web_state.get_browser_window_mut(&window)? else {
                return Ok(());
            };

            let prev_url = state.last_url.clone();

            state.last_url = new_url.clone();
            state.last_title = last_title;

            prev_url
        };

        if prev_url != new_url {
            Self::enforce_actions(
                &args.actions,
                &args.detect,
                &new_url,
                &window,
                &extracted_elements,
            );
        }
        Ok(())
    }

    /// Check the action map and enforce matching actions for the given URL.
    fn enforce_actions(
        actions: &RwLock<HashMap<BaseWebsiteUrl, BrowserAction>>,
        detect: &detect::Detect,
        url: &str,
        window: &Window,
        extracted_elements: &state::ExtractedUIElements,
    ) {
        let actions = actions.blocking_read();
        let base_url = WebsiteInfo::url_to_base_url(url);
        if let Some(action) = actions.get(&base_url) {
            match action {
                BrowserAction::CloseTab => {
                    if let Ok(element) = extracted_elements.window_element.resolve() {
                        detect.close_current_tab(&element).warn();
                    }
                }
                BrowserAction::Dim { opacity } => {
                    window.dim(*opacity).warn();
                }
            }
        }
    }
}

#[derive(Clone)]
struct Args {
    web_state: state::State,
    detect: detect::Detect,
    reentrancy: Arc<Mutex<()>>,
    actions: Arc<RwLock<HashMap<BaseWebsiteUrl, BrowserAction>>>,
}
