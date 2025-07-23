use std::sync::{Arc, Mutex};

use util::channels::Sender;
use util::ds::{SmallHashMap, SmallHashSet};
use util::error::{Context, ContextCompat, Result};
use util::tracing::{self, debug, instrument};
use windows::Win32::UI::Accessibility::{
    TreeScope_Element, UIA_AriaRolePropertyId, UIA_NamePropertyId, UIA_PROPERTY_ID,
    UIA_ValueValuePropertyId,
};

use crate::events::{PropertyChange, WindowTitleWatcher};
use crate::objects::{ProcessId, Target, Window};
use crate::web::{self, perf};

/// Watches a browser (by PID) for tab changes. Changes should be reported as fast as possible.
pub struct Watcher {
    processes: SmallHashMap<ProcessId, WindowTitleWatcher>,
    windows: SmallHashMap<Window, PropertyChange>,

    args: Args,
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

            args: Args {
                web_state,
                detect: web::Detect::new()?,
                reentrancy: Arc::new(Mutex::new(())),
                web_change_tx,
            },
        })
    }

    /// Send a tick to recheck all browser windows and update browser PID subscriptions.
    pub fn tick(&mut self) -> Result<()> {
        {
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
        }
        Ok(())
    }

    fn update_browsers_processes(&mut self, pids: &SmallHashSet<ProcessId>) -> Result<()> {
        // Remove any browsers that are no longer in the list
        self.processes.retain(|pid, _| pids.contains(pid));

        // Add any new browsers to the list, watch them for title changes
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
        windows: SmallHashMap<Window, web::ExtractedUIElements>,
    ) -> Result<()> {
        // Remove any browsers that are no longer in the list
        self.windows
            .retain(|window, _| windows.contains_key(window));

        // Add any new browsers to the list, watch them for title changes
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
        // Get the extracted elements from the state.
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
        extracted_elements: web::ExtractedUIElements,
        args: Args,
    ) -> Result<()> {
        Self::central_callback(window, Some(value), extracted_elements, args)
            .context("omnibox text change callback")
    }

    fn central_callback(
        window: Window,
        url: Option<String>,
        extracted_elements: web::ExtractedUIElements,
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

        // Use the given url if available, otherwise get it from the omnibox.
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

        // If a user types in the omnibox, the icon name will be "Search icon". Otherwise it seems to be "View site information" / "Not secure".
        // This can be seen in https://github.com/chromium/chromium/blob/7cfb7a7ce147dc091f752943bd909238d1ad002b/chrome/browser/ui/views/location_bar/location_icon_view.cc#L271
        // where if the icon role/name/desc changed depending on whether the textbox is editing or empty.
        //
        // So we have no false positives - but we can still have false negatives:
        // - e.g. user types in the omnibox, then switches to a different tab, then switches back to the original tab.
        let last_url = if icon_role == "button" && !url.is_empty() {
            let omnibox_icon = extracted_elements.omnibox_icon.resolve()?;
            let icon_text = perf(
                || unsafe { omnibox_icon.GetCurrentPropertyValue(UIA_NamePropertyId) },
                "omnibox_icon - GetCurrentPropertyValue(UIA_NamePropertyId)",
            )?
            .to_string();

            let is_http_hint = web::Detect::is_http_hint_from_icon_text(&icon_text);
            debug!(?url, ?icon_text, "tab changed");
            web::Detect::unelide_omnibox_text(url, is_http_hint)
        } else {
            // Or fetch the url from the document element.
            // TODO: backon retries + timeout to restack + use rt spawn_blocking + throttle
            args.detect
                .chromium_url(&extracted_elements.window_element.resolve()?)
                .context("get chromium url")?
                .context("no element")?
        };
        // we get the title again since the above last_url fetch might have taken a long time to run,
        // causing the title to be stale
        let last_title = window.title()?;

        let (changed, is_incognito) = {
            let mut web_state = args.web_state.blocking_write();
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
            args.web_change_tx.send(Changed {
                window,
                url: last_url,
                is_incognito,
            })?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Args {
    web_state: web::State,
    detect: web::Detect,
    reentrancy: Arc<Mutex<()>>,

    web_change_tx: Sender<Changed>,
}
