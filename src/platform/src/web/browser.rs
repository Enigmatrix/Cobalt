use reqwest::Url;
use util::error::{Context, Result};
use util::tracing::{debug, info, warn};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance};
use windows::Win32::UI::Accessibility::{
    AutomationElementMode_None, CUIAutomation, IUIAutomation, IUIAutomationCacheRequest,
    IUIAutomationCondition, IUIAutomationElement, IUIAutomationElement9,
    IUIAutomationInvokePattern, TreeScope_Children, TreeScope_Descendants,
    TreeTraversalOptions_LastToFirstOrder, UIA_AutomationIdPropertyId, UIA_ButtonControlTypeId,
    UIA_ClassNamePropertyId, UIA_ControlTypePropertyId, UIA_InvokePatternId, UIA_NamePropertyId,
    UIA_SelectionItemIsSelectedPropertyId, UIA_TabItemControlTypeId, UIA_ValueValuePropertyId,
};
use windows::core::{AgileReference, Interface};

use crate::objects::Window;

const DEBUG_LIMIT: std::time::Duration = std::time::Duration::from_millis(10);
const WARN_LIMIT: std::time::Duration = std::time::Duration::from_millis(100);

fn perf<T>(f: impl FnOnce() -> T, name: &str) -> T {
    let start = std::time::Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    if elapsed > WARN_LIMIT {
        warn!("slow: {elapsed:?} for {name}");
    } else if elapsed > DEBUG_LIMIT {
        debug!("slow: {elapsed:?} for {name}");
    }
    result
}

/// Detects browser usage information
#[derive(Clone)]
pub struct BrowserDetector {
    automation: AgileReference<IUIAutomation>,
    browser_root_view_cond: AgileReference<IUIAutomationCondition>,
    root_web_area_cond: AgileReference<IUIAutomationCondition>,
    omnibox_cond: AgileReference<IUIAutomationCondition>,
    cache_request: AgileReference<IUIAutomationCacheRequest>,
}

impl BrowserDetector {
    /// Create a new [BrowserDetector]
    pub fn new() -> Result<Self> {
        let automation: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)? };
        let browser_root_view_cond = unsafe {
            automation
                .CreatePropertyCondition(UIA_ClassNamePropertyId, &"BrowserRootView".into())?
        };
        let root_web_area_cond = unsafe {
            automation.CreatePropertyCondition(UIA_AutomationIdPropertyId, &"RootWebArea".into())?
        };
        let omnibox_cond = unsafe {
            automation
                .CreatePropertyCondition(UIA_NamePropertyId, &"Address and search bar".into())?
        };
        let cache_request = unsafe {
            let cache_requesst = automation.CreateCacheRequest()?;
            cache_requesst.SetAutomationElementMode(AutomationElementMode_None)?;
            cache_requesst.AddProperty(UIA_NamePropertyId)?;
            cache_requesst.AddProperty(UIA_ValueValuePropertyId)?;
            cache_requesst
        };
        Ok(Self {
            automation: AgileReference::new(&automation)?,
            browser_root_view_cond: AgileReference::new(&browser_root_view_cond)?,
            root_web_area_cond: AgileReference::new(&root_web_area_cond)?,
            omnibox_cond: AgileReference::new(&omnibox_cond)?,
            cache_request: AgileReference::new(&cache_request)?,
        })
    }

    /// Check if the path is a browser. Not meant to be super-accurate, but should be good enough.
    pub fn is_maybe_chromium_exe(path: &str) -> bool {
        let browsers = ["chrome.exe", "msedge.exe"];
        let path_lower = path.to_lowercase();
        browsers.iter().any(|browser| path_lower.ends_with(browser))
    }

    /// Check if the [Window] is a Chromium browser
    pub fn is_maybe_chromium_window(&self, window: &Window) -> Result<bool> {
        let class = window.class()?;
        Ok(class == "Chrome_WidgetWin_1")
    }

    /// Get the UI Automation element for the [Window]
    pub fn get_chromium_element(&self, window: &Window) -> Result<IUIAutomationElement9> {
        let element: IUIAutomationElement9 = perf(
            || unsafe { self.automation.resolve()?.ElementFromHandle(window.hwnd) }?.cast(),
            "ElementFromHandle",
        )?;
        Ok(element)
    }

    /// Get the URL of the [Window], assuming it is a Chromium browser
    /// This uses UI Automation to get the URL, which is a bit of a hack.
    /// Notably, this only works for Chrome and Edge right now.
    /// This might break in the future if Chrome/Edge team changes the UI.
    pub fn chromium_url(&self, element: &IUIAutomationElement9) -> Result<Option<String>> {
        let root_web_area = uia_find_result(perf(
            || unsafe {
                element.FindFirstWithOptionsBuildCache(
                    TreeScope_Descendants,
                    &self.root_web_area_cond.resolve()?,
                    &self.cache_request.resolve()?,
                    TreeTraversalOptions_LastToFirstOrder,
                    element,
                )
            },
            "root_web_area - FindFirstWithOptionsBuildCache",
        ))
        .context("find root web area")?;
        if let Some(root_web_area) = root_web_area {
            let url = perf(
                || unsafe { root_web_area.GetCachedPropertyValue(UIA_ValueValuePropertyId) },
                "root_web_area - GetCachedPropertyValue(UIA_ValueValuePropertyId)",
            )?
            .to_string();
            if !url.is_empty() {
                return Ok(Some(url));
            }
        }

        let omnibox = uia_find_result(perf(
            || unsafe {
                element.FindFirstBuildCache(
                    TreeScope_Descendants,
                    &self.omnibox_cond.resolve()?,
                    &self.cache_request.resolve()?,
                )
            },
            "omnibox - FindFirstBuildCache",
        ))
        .context("find omnibox")?;
        let Some(omnibox) = omnibox else {
            return Ok(None);
        };

        let search_value = perf(
            || unsafe { omnibox.GetCachedPropertyValue(UIA_ValueValuePropertyId) },
            "omnibox - GetCachedPropertyValue(UIA_ValueValuePropertyId)",
        )?
        .to_string();
        info!("using omnibox url: {search_value}");

        // Omnibox URL is used when there is a long loading period (so document isn't loaded), or
        // when an old tab is loaded again from energy saver (so title is set, but no document is loaded).

        // From Omnibox, we get a URL like this:
        // "google.com/search?q=test". note that proto is not shown.
        // for chrome://, data:XXX and other non http/https links, it's there tho e.g. chrome://newtab.

        let url = match Url::parse(&search_value) {
            // see if we get no proto error (here it's relative url without base)
            // if so, add https:// to the front and see if we get a valid url.
            // if thats finally valid, then we use that url.
            // in all other cases, use original value
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                // The URL could really be http://
                // There might be way to detect this by checking the button next to the omnibox and seeing if it's lock / not-secure.
                let url = format!("https://{search_value}");
                if Url::parse(&url).is_ok() {
                    url
                } else {
                    search_value
                }
            }
            _ => search_value,
        };

        Ok(if url.is_empty() { None } else { Some(url) })
    }

    /// Close the current tab in the given [Window]
    pub fn close_current_tab(&self, element: &IUIAutomationElement9) -> Result<()> {
        let tab_condition = unsafe {
            let is_tab = self.automation.resolve()?.CreatePropertyCondition(
                UIA_ControlTypePropertyId,
                &UIA_TabItemControlTypeId.0.into(),
            )?;
            let is_selected = self
                .automation
                .resolve()?
                .CreatePropertyCondition(UIA_SelectionItemIsSelectedPropertyId, &true.into())?;
            self.automation
                .resolve()?
                .CreateAndCondition(&is_tab, &is_selected)?
        };

        let tab = uia_find_result(perf(
            || unsafe { element.FindFirst(TreeScope_Descendants, &tab_condition) },
            "tab - FindFirst",
        ))?;

        let Some(tab) = tab else {
            return Ok(());
        };

        let close_button_condition = unsafe {
            self.automation.resolve()?.CreatePropertyCondition(
                UIA_ControlTypePropertyId,
                &UIA_ButtonControlTypeId.0.into(),
            )
        }?;

        let close_buttons = perf(
            || unsafe { tab.FindAll(TreeScope_Children, &close_button_condition) },
            "close_button - FindFirst",
        )?;

        let mut close_button = None;
        let len = unsafe { close_buttons.Length()? };
        for i in 0..len {
            let button: IUIAutomationElement9 = unsafe { close_buttons.GetElement(i)?.cast()? };
            let class_name = perf(
                || unsafe { button.GetCurrentPropertyValue(UIA_ClassNamePropertyId) },
                "button - GetCurrentPropertyValue(UIA_ClassNamePropertyId)",
            )?
            .to_string();
            if class_name.contains("CloseButton") {
                close_button = Some(button);
                break;
            }
        }

        let Some(close_button) = close_button else {
            return Ok(());
        };

        // Click the close button
        let invoke_pattern: IUIAutomationInvokePattern = perf(
            || unsafe { close_button.GetCurrentPatternAs(UIA_InvokePatternId) },
            "close_button - GetCurrentPatternAs(UIA_InvokePatternId)",
        )?;
        perf(
            || unsafe { invoke_pattern.Invoke() },
            "invoke_pattern - Invoke",
        )?;

        Ok(())
    }
}

fn uia_find_result(
    value: windows::core::Result<IUIAutomationElement>,
) -> windows::core::Result<Option<IUIAutomationElement9>> {
    match value {
        Ok(value) => Ok(Some(value.cast()?)),
        Err(err) if err.code().is_ok() => Ok(None),
        Err(err) => Err(err),
    }
}
