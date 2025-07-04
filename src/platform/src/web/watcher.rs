use std::collections::HashMap;

use util::channels::Sender;
use util::ds::SmallHashMap;
use util::error::{ContextCompat, Result};
use util::future::sync::Mutex;
use util::tracing::{ResultTraceExt, debug};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationElement9, IUIAutomationPropertyChangedEventHandler,
    IUIAutomationPropertyChangedEventHandler_Impl, UIA_PROPERTY_ID,
};
use windows::core::{AgileReference, Ref, implement};

use crate::objects::Window;
use crate::web::{self, BrowserDetector, BrowserWindowInfo};

/// Event sent when the URL of a browser window changes.
#[derive(Debug, Clone)]
pub struct UrlChanged {
    /// Browser Window that the URL changed in
    pub window: Window,
    /// New URL of the browser window
    pub url: String,
}

type UrlChangedSender = Sender<UrlChanged>;

/// Watches the URL of a browser windows and emits events when the URL changes.
pub struct UrlWatcher {
    browser_state: web::State,
    watchers: HashMap<Window, WindowUrlWatcher>,
    sender: UrlChangedSender,
    detect: BrowserDetector,
}

impl UrlWatcher {
    /// Creates a new [UrlWatcher]. Assumes that [browser_state] is empty. If not, next tick will update it.
    pub fn new(
        browser_state: web::State,
        sender: UrlChangedSender,
        detect: BrowserDetector,
    ) -> Result<Self> {
        Ok(Self {
            browser_state,
            watchers: HashMap::new(),
            sender,
            detect,
        })
    }

    fn update_browsers(&mut self, windows: &SmallHashMap<Window, BrowserWindowInfo>) -> Result<()> {
        // Remove any browsers that are no longer in the list
        self.watchers
            .retain(|window, _| windows.contains_key(window));

        // Add any new browsers to the list, watch them for title changes
        for (window, info) in windows {
            if !self.watchers.contains_key(window) {
                // Send initial URL if available.
                debug!("url inited for new {:?}: {:?}", window, info.url);
                self.sender.send(UrlChanged {
                    window: window.clone(),
                    url: info.url.clone().unwrap_or_default(), // TODO unwrap properly
                })?;

                self.watchers.insert(
                    window.clone(),
                    WindowUrlWatcher::new(
                        window.clone(),
                        self.browser_state.clone(),
                        self.sender.clone(),
                        self.detect.clone(),
                    )?,
                );
            }
        }

        Ok(())
    }

    /// Send a tick to recheck all browser windows and update browser PID subscriptions.
    pub fn tick(&mut self) -> Result<()> {
        let windows = {
            let state = self.browser_state.blocking_read();
            state
                .browser_windows
                .iter()
                .filter_map(|(window, info)| {
                    info.as_ref().map(|info| (window.clone(), info.clone()))
                })
                .collect()
        };
        self.update_browsers(&windows)?;
        Ok(())
    }
}

/// Watches the URL of a browser window and emits events when the URL changes.
pub struct WindowUrlWatcher {
    omnibox: IUIAutomationElement9,
    detect: BrowserDetector,
    handler: AgileReference<IUIAutomationPropertyChangedEventHandler>,
}

impl WindowUrlWatcher {
    /// Creates a new [ChromiumWindowUrlWatcher] for the given window.
    pub fn new(
        window: Window,
        browser_state: web::State,
        sender: UrlChangedSender,
        detect: BrowserDetector,
    ) -> Result<Self> {
        let window_element = detect.get_chromium_element(&window)?;
        let window_element_ref = AgileReference::new(&window_element)?;

        let handler = OmniboxTextEditHandler::new(
            window.clone(),
            window_element_ref,
            sender,
            detect.clone(),
            browser_state,
        )
        .into();
        let handler = AgileReference::new(&handler)?;

        let omnibox = detect
            .get_chromium_omnibox(&window_element)?
            .with_context(|| format!("no omnibox for window: {window:?}"))?;
        detect.add_omnibox_text_edit_handler(&omnibox, &handler.resolve()?)?;

        Ok(Self {
            omnibox,
            detect,
            handler,
        })
    }
}

impl Drop for WindowUrlWatcher {
    fn drop(&mut self) {
        self.detect
            .remove_omnibox_text_edit_handler(
                &self.omnibox,
                &self.handler.resolve().expect("handler non-null"),
            )
            .warn();
    }
}

use value_changed_handler::*;
mod value_changed_handler {

    use super::*;

    #[allow(missing_docs)]
    #[implement(IUIAutomationPropertyChangedEventHandler)]
    /// Text edit change event handler for omnibox
    pub(crate) struct OmniboxTextEditHandler {
        window: Window,
        element: AgileReference<IUIAutomationElement9>,
        sender: UrlChangedSender,
        detect: BrowserDetector,
        browser_state: web::State,
        reentrant_lock: Mutex<()>,
    }

    impl OmniboxTextEditHandler {
        /// Create a new omnibox text edit event handler
        pub fn new(
            window: Window,
            element: AgileReference<IUIAutomationElement9>,
            sender: UrlChangedSender,
            detect: BrowserDetector,
            browser_state: web::State,
        ) -> Self {
            Self {
                window,
                element,
                sender,
                detect,
                browser_state,
                reentrant_lock: Mutex::new(()),
            }
        }

        fn omnibox_text_edit_changed(&self, omnibox_text: String) -> Result<()> {
            // TODO use a thread pool instead of this scoping?
            let url = std::thread::scope(|s| -> Result<Option<String>> {
                s.spawn(|| -> Result<Option<String>> {
                    let url = self
                        .detect
                        .chromium_url(&self.element.resolve()?, Some(omnibox_text))?;
                    Ok(url)
                })
                .join()
                .expect("join")
            })?;
            debug!("url changed for {:?}: {:?}", self.window, url);

            {
                let mut browser_state = self.browser_state.blocking_write();
                let info = browser_state
                    .browser_windows
                    .get_mut(&self.window)
                    .expect("browser window entry to be present");
                info.as_mut().expect("browser window info to be Some").url = url.clone();
            }

            self.sender.send(UrlChanged {
                window: self.window.clone(),
                url: url.unwrap_or_default(), // TODO: unwrap properly
            })?;
            Ok(())
        }
    }

    impl IUIAutomationPropertyChangedEventHandler_Impl for OmniboxTextEditHandler_Impl {
        #[allow(non_snake_case)]
        fn HandlePropertyChangedEvent(
            &self,
            _sender: Ref<'_, IUIAutomationElement>,
            _property_id: UIA_PROPERTY_ID,
            property_value: &VARIANT,
        ) -> windows_core::Result<()> {
            let _guard = self.reentrant_lock.blocking_lock();
            let omnibox_text = property_value.to_string();
            self.omnibox_text_edit_changed(omnibox_text).warn();
            drop(_guard);
            Ok(())
        }
    }
}
