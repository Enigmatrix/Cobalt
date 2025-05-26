use util::error::{Context, Result};
use windows::core::VARIANT;
use windows::Win32::System::Com::StructuredStorage::{
    PropVariantToStringAlloc, VariantToPropVariant,
};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, TreeScope_Descendants, UIA_ControlTypePropertyId,
    UIA_DocumentControlTypeId, UIA_ValueValuePropertyId,
};

use crate::objects::Window;

/// Detects browser usage information
pub struct BrowserDetector {
    automation: IUIAutomation,
}

impl BrowserDetector {
    /// Create a new [BrowserDetector]
    pub fn new() -> Result<Self> {
        let automation: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)? };
        Ok(Self { automation })
    }

    /// Check if the path is a browser
    pub fn is_browser(path: &str) -> bool {
        let browsers = ["chrome.exe", "msedge.exe"];
        browsers.iter().any(|browser| path.ends_with(browser))
    }

    /// Check if the [Window] is a Chromium browser
    pub fn is_chromium(&self, window: &Window) -> Result<bool> {
        let class = window.class()?;
        Ok(class == "Chrome_WidgetWin_1")
    }

    /// Get the URL of the [Window], assuming it is a Chromium browser
    pub fn chromium_url(&self, window: &Window) -> Result<Option<String>> {
        let element = unsafe { self.automation.ElementFromHandle(window.hwnd)? };

        let variant: VARIANT = UIA_DocumentControlTypeId.0.into();
        let condition = unsafe {
            self.automation
                .CreatePropertyCondition(UIA_ControlTypePropertyId, &variant)?
        };
        // No matching element is Error with S_OK (wtf??????)
        let element = match unsafe { element.FindFirst(TreeScope_Descendants, &condition) } {
            Ok(element) => element,
            Err(err) if err.code().is_ok() => return Ok(None),
            Err(err) => return Err(err).context("find first element"),
        };

        let url_prop = unsafe { element.GetCurrentPropertyValue(UIA_ValueValuePropertyId)? };
        let url_prop = unsafe { VariantToPropVariant(&url_prop)? };
        let url_raw = unsafe { PropVariantToStringAlloc(&url_prop)? };
        // This is a copy, so we are free to CoTaskMemFree the url_raw in the next line
        let url = String::from_utf16_lossy(unsafe { url_raw.as_wide() });
        unsafe { CoTaskMemFree(Some(url_raw.as_ptr().cast())) };

        if url.is_empty() {
            Ok(None)
        } else {
            Ok(Some(url))
        }
    }
}
