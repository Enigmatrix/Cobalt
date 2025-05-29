use util::error::{Context, Result};
use windows::core::VARIANT;
use windows::Win32::System::Com::StructuredStorage::{
    PropVariantToStringAlloc, VariantToPropVariant,
};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, TreeScope_Descendants, UIA_ControlTypePropertyId,
    UIA_DocumentControlTypeId, UIA_NamePropertyId, UIA_ValueValuePropertyId,
};

use crate::objects::Window;

/// Detects browser usage information
pub struct BrowserDetector {
    automation: IUIAutomation,
}

/// Detected browser URL with extra information
#[derive(Debug, Clone)]
pub struct BrowserUrl {
    /// The URL
    pub url: Option<String>,
    /// Whether the browser is in incognito mode
    pub incognito: bool,
}

impl BrowserDetector {
    /// Create a new [BrowserDetector]
    pub fn new() -> Result<Self> {
        let automation: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)? };
        Ok(Self { automation })
    }

    /// Check if the path is a browser. Not meant to be super-accurate, but should be good enough.
    pub fn is_browser(path: &str) -> bool {
        let browsers = ["chrome.exe", "msedge.exe"];
        let path_lower = path.to_lowercase();
        browsers.iter().any(|browser| path_lower.ends_with(browser))
    }

    /// Check if the [Window] is a Chromium browser
    pub fn is_chromium(&self, window: &Window) -> Result<bool> {
        let class = window.class()?;
        Ok(class == "Chrome_WidgetWin_1")
    }

    /// Get the URL of the [Window], assuming it is a Chromium browser
    /// This uses UI Automation to get the URL, which is a bit of a hack.
    /// Notably, this only works for Chrome and Edge right now.
    /// This might break in the future if Chrome/Edge team changes the UI.
    pub fn chromium_url(&self, window: &Window) -> Result<BrowserUrl> {
        let element = unsafe { self.automation.ElementFromHandle(window.hwnd)? };

        let name_prop = unsafe { element.GetCurrentPropertyValue(UIA_NamePropertyId)? };
        let name_prop = unsafe { VariantToPropVariant(&name_prop)? };
        let name_raw = unsafe { PropVariantToStringAlloc(&name_prop)? };
        // This is a copy, so we are free to CoTaskMemFree the name_raw in the next line
        let name = String::from_utf16_lossy(unsafe { name_raw.as_wide() });
        unsafe { CoTaskMemFree(Some(name_raw.as_ptr().cast())) };

        // This seems to be the only way to detect incognito mode in Edge
        // Not sure why the \u{200b} is needed, but it works.
        // Have not tested with other languages.
        let incognito = name.ends_with("Google Chrome (Incognito)")
            || name.ends_with("[InPrivate] - Microsoft\u{200b} Edge");

        let variant: VARIANT = UIA_DocumentControlTypeId.0.into();
        let condition = unsafe {
            self.automation
                .CreatePropertyCondition(UIA_ControlTypePropertyId, &variant)?
        };
        // No matching element is Error with S_OK (wtf??????)
        let document = match unsafe { element.FindFirst(TreeScope_Descendants, &condition) } {
            Ok(element) => element,
            Err(err) if err.code().is_ok() => {
                return Ok(BrowserUrl {
                    url: None,
                    incognito,
                })
            }
            Err(err) => return Err(err).context("find first element"),
        };

        let url_prop = unsafe { document.GetCurrentPropertyValue(UIA_ValueValuePropertyId)? };
        let url_prop = unsafe { VariantToPropVariant(&url_prop)? };
        let url_raw = unsafe { PropVariantToStringAlloc(&url_prop)? };
        // This is a copy, so we are free to CoTaskMemFree the url_raw in the next line
        let url = String::from_utf16_lossy(unsafe { url_raw.as_wide() });
        unsafe { CoTaskMemFree(Some(url_raw.as_ptr().cast())) };

        Ok(BrowserUrl {
            url: if url.is_empty() { None } else { Some(url) },
            incognito,
        })
    }
}
