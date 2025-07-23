use util::channels::Sender;
use util::ds::{SmallHashMap, SmallHashSet};
use util::error::{Context, ContextCompat, Result};
use util::tracing::{self, instrument};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement9, TreeScope_Element, UIA_AriaRolePropertyId, UIA_NamePropertyId,
    UIA_PROPERTY_ID, UIA_ValueValuePropertyId,
};
use windows::core::AgileReference;

use crate::events::{PropertyChange, WindowTitleWatcher};
use crate::objects::{ProcessId, Target, Window};
use crate::web::{self, perf};

/// Watches a browser (by PID) for tab changes. Changes should be reported as fast as possible.
pub struct Watcher {
    detect: web::Detect,
    web_state: web::State,

    processes: SmallHashMap<ProcessId, WindowTitleWatcher>,
    windows: SmallHashMap<Window, PropertyChange>,

    web_change_tx: Sender<Changed>,
}

/// Probable change to the focused tab in some browser window.
#[derive(Debug, Clone)]
pub struct Changed {
    /// Window where the tab changed
    pub window: Window,
    /// URL of the tab that the user has switched to
    pub url: String,
    /// Whether the window is incognito
    pub is_incognito: bool,
}

impl Watcher {
    /// Create a new [Watcher]. tick() needs to be called at least once to start watching.
    pub fn new(web_change_tx: Sender<Changed>, web_state: web::State) -> Result<Self> {
        Ok(Self {
            processes: SmallHashMap::new(),
            windows: SmallHashMap::new(),

            detect: web::Detect::new()?,
            web_state,
            web_change_tx,
        })
    }

    /// Send a tick to recheck all browser windows and update browser PID subscriptions.
    pub fn tick(&mut self) -> Result<()> {
        {
            let (pids, windows) = {
                let state = self.web_state.blocking_read();
                let pids = state.browser_processes.clone();
                let windows = state
                    .browser_windows
                    .iter()
                    .flat_map(|(window, state)| {
                        state
                            .as_ref()
                            .map(|state| (window.clone(), state.window_element.clone()))
                    })
                    .collect();
                (pids, windows)
            };
            self.update_browsers_processes(&pids)?;
            self.update_browser_windows(windows)?;
        }
        Ok(())
    }

    fn update_browsers_processes(&mut self, pids: &SmallHashSet<ProcessId>) -> Result<()> {
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

    fn update_browser_windows(
        &mut self,
        windows: SmallHashMap<Window, AgileReference<IUIAutomationElement9>>,
    ) -> Result<()> {
        // Remove any browsers that are no longer in the list
        self.windows
            .retain(|window, _| windows.contains_key(window));

        // Add any new browsers to the list, watch them for title changes
        for (window, window_element) in windows {
            if !self.windows.contains_key(&window) {
                let window_element = window_element.clone();

                let omnibox = self
                    .detect
                    .get_chromium_omnibox_element(&window_element.resolve()?, true)?
                    .context("no omnibox")?;
                let omnibox_icon = self
                    .detect
                    .get_chromium_omnibox_icon_element(&window_element.resolve()?, true)?
                    .context("no omnibox icon")?;

                let callback = {
                    let window = window.clone();

                    let web_state = self.web_state.clone();
                    let detect = self.detect.clone();
                    let web_change_tx = self.web_change_tx.clone();

                    Box::new(move |_eventid: UIA_PROPERTY_ID, value: String| {
                        Self::omnibox_text_change_callback(
                            window.clone(),
                            window_element.clone(),
                            AgileReference::new(&omnibox_icon)?,
                            value,
                            web_state.clone(),
                            detect.clone(),
                            web_change_tx.clone(),
                        )
                    })
                };
                self.windows.insert(
                    window,
                    PropertyChange::new(
                        self.detect.automation.clone(),
                        AgileReference::new(&omnibox)?,
                        TreeScope_Element,
                        None,
                        &[UIA_ValueValuePropertyId],
                        callback,
                    )?,
                );
            }
        }

        Ok(())
    }

    #[instrument(level = "trace", skip(web_state))]
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

    #[instrument(
        level = "trace",
        skip(window_element, omnibox_icon, web_state, detect, web_change_tx)
    )]
    fn omnibox_text_change_callback(
        window: Window,
        window_element: AgileReference<IUIAutomationElement9>,
        omnibox_icon: AgileReference<IUIAutomationElement9>,
        value: String,

        web_state: web::State,
        detect: web::Detect,

        web_change_tx: Sender<Changed>,
    ) -> Result<()> {
        // TODO: need to call window title

        // TODO: mutex for reentrancy
        //

        let icon_role = perf(
            || unsafe {
                omnibox_icon
                    .resolve()?
                    .GetCurrentPropertyValue(UIA_AriaRolePropertyId)
            },
            "omnibox_icon - GetCurrentPropertyValue(UIA_AriaRolePropertyId)",
        )?
        .to_string();

        // If a user types in the omnibox, the icon name will be "Search icon". Otherwise it seems to be "View site information" / "Not secure".
        // This can be seen in https://github.com/chromium/chromium/blob/7cfb7a7ce147dc091f752943bd909238d1ad002b/chrome/browser/ui/views/location_bar/location_icon_view.cc#L271
        // where if the icon role/name/desc changed depending on whether the textbox is editing or empty.
        //
        // So we have no false positives - but we can still have false negatives:
        // - e.g. user types in the omnibox, then switches to a different tab, then switches back to the original tab.
        let last_url = if icon_role == "button" && !value.is_empty() {
            let icon_text = perf(
                || unsafe {
                    omnibox_icon
                        .resolve()?
                        .GetCurrentPropertyValue(UIA_NamePropertyId)
                },
                "omnibox_icon - GetCurrentPropertyValue(UIA_NamePropertyId)",
            )?
            .to_string();

            let is_http_hint = web::Detect::is_http_hint_from_icon_text(&icon_text);
            web::Detect::unelide_omnibox_text(value, is_http_hint)
        } else {
            detect
                .chromium_url(&window_element.resolve()?)
                .context("get chromium url")?
                .context("no element")?
        };
        // we get the title again since the above last_url fetch might have taken a long time to run,
        // causing the title to be stale / too early.
        let last_title = window.title()?;

        let (changed, is_incognito) = {
            let mut web_state = web_state.blocking_write();
            let Some(state) = web_state.get_browser_window_mut(&window)? else {
                return Ok(());
            };

            let changed = (&state.last_url, &state.last_title) != (&last_url, &last_title);

            if changed {
                state.last_url = last_url.clone();
                state.last_title = last_title;
            }

            (changed, state.is_incognito)
        };

        if changed {
            web_change_tx.send(Changed {
                window,
                url: last_url,
                is_incognito,
            })?;
        }
        Ok(())
    }
}
