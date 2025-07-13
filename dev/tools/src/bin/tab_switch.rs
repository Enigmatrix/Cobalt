//! Monitor tab selection changes using UI Automation

use std::io::stdin;

use platform::objects::Window;
use platform::web;
use tools::filters::{ProcessFilter, WindowFilter, match_running_windows};
use util::error::Result;
use util::tracing::info;
use util::{Target as UtilTarget, config, future as tokio};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationEventHandler,
    IUIAutomationEventHandler_Impl, TreeScope_Descendants, UIA_ClassNamePropertyId, UIA_EVENT_ID,
    UIA_SelectionItem_ElementSelectedEventId,
};
use windows::core::{AgileReference, implement};

mod tab_track_handler {

    use super::*;
    #[allow(missing_docs)]
    #[implement(IUIAutomationEventHandler)]
    /// Tab selection event handler
    pub struct TabSelectionEventHandlerCom {
        /// Window
        pub window: Window,
    }

    impl TabSelectionEventHandlerCom {
        /// Create a new tab selection event handler
        pub fn new(window: Window) -> Self {
            Self { window }
        }
    }

    impl IUIAutomationEventHandler_Impl for TabSelectionEventHandlerCom_Impl {
        #[allow(non_snake_case)]
        fn HandleAutomationEvent(
            &self,
            _sender: windows_core::Ref<'_, IUIAutomationElement>,
            _eventid: UIA_EVENT_ID,
        ) -> windows::core::Result<()> {
            let detect = web::Detect::new().expect("Failed to create browser detector");
            let element = detect
                .get_chromium_element(&self.window)
                .expect("Failed to get Chromium element");
            let url = detect
                .chromium_url(&element)
                .expect("Failed to get Chromium URL");

            let mut dim = false;
            if let Some(url) = &url
                && !url.contains("youtube.com")
            {
                dim = true;
            }
            if dim {
                self.window.dim(0.5).unwrap();
            } else {
                self.window.dim(1.0).unwrap();
            }

            info!(
                "Tab selection event: Tab selected in window {:08x} with URL {url:?}",
                self.window.hwnd.0 as usize,
            );
            Ok(())
        }
    }
}
use tab_track_handler::*;

struct TabSelectionEventHandler {
    automation: AgileReference<IUIAutomation>,
    window: Window,
}

impl TabSelectionEventHandler {
    fn new(window: Window) -> Result<Self> {
        let automation: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)? };
        Ok(Self {
            automation: AgileReference::new(&automation)?,
            window,
        })
    }

    fn find_tab_strip_region(&self) -> Result<Option<IUIAutomationElement>> {
        let element = unsafe {
            self.automation
                .resolve()?
                .ElementFromHandle(self.window.hwnd)?
        };

        let tab_strip_cond = unsafe {
            self.automation
                .resolve()?
                .CreatePropertyCondition(UIA_ClassNamePropertyId, &"TabStripRegionView".into())?
        };

        let tab_strip = unsafe { element.FindFirst(TreeScope_Descendants, &tab_strip_cond) };

        match tab_strip {
            Ok(element) => Ok(Some(element)),
            Err(_) => Ok(None),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    util::set_target(UtilTarget::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let window_group = match_running_windows(
        &WindowFilter {
            ..Default::default()
        },
        &ProcessFilter {
            name: Some("Google Chrome".to_string()),
            ..Default::default()
        },
    )?;

    if window_group.is_empty() {
        println!("No Chrome windows found. Please open Chrome and try again.");
        return Ok(());
    }

    let chrome = window_group.first().unwrap();
    println!("Monitoring Chrome window: {:08x}", chrome.process.pid);

    // Get the first window from the process group
    for w in &chrome.windows {
        let chrome_window = w.window.clone();

        let tab_handler = TabSelectionEventHandler::new(chrome_window.clone())?;
        // Create tab selection event handler

        // Check if TabStripRegionView exists
        if let Some(tab_strip) = tab_handler.find_tab_strip_region()? {
            let automation = tab_handler.automation.clone();
            let handler = TabSelectionEventHandlerCom::new(chrome_window.clone());
            unsafe {
                automation.resolve()?.AddAutomationEventHandler(
                    UIA_SelectionItem_ElementSelectedEventId,
                    &tab_strip,
                    TreeScope_Descendants,
                    None,
                    &IUIAutomationEventHandler::from(handler),
                )?;
            }
            println!(
                "Registered UIA event handlers for tabs in window {:08x}",
                chrome_window.hwnd.0 as usize
            );
        } else {
            println!(
                "TabStripRegionView not found for window {:08x}",
                chrome_window.hwnd.0 as usize
            );
        }
    }

    println!(
        "Monitoring tab selection changes via UI Automation event handlers. Press Ctrl+C to exit."
    );

    stdin().read_line(&mut String::new()).unwrap();

    Ok(())
}
