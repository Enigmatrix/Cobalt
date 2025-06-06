use reqwest::Url;
use util::error::{Context, Result};
use util::tracing::info;
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
        let name = self.variant_to_string(name_prop)?;

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
        let documents = unsafe { element.FindAll(TreeScope_Descendants, &condition)? };
        let len = unsafe { documents.Length()? };
        for i in 0..len {
            let document = unsafe { documents.GetElement(i)? };

            let url_prop = unsafe { document.GetCurrentPropertyValue(UIA_ValueValuePropertyId)? };
            let url = self.variant_to_string(url_prop)?;
            if url.is_empty() {
                continue;
            }

            return Ok(BrowserUrl {
                url: Some(url),
                incognito,
            });
        }

        let condition = unsafe {
            self.automation
                .CreatePropertyCondition(UIA_NamePropertyId, &"Address and search bar".into())?
        };
        let search_bar = match unsafe { element.FindFirst(TreeScope_Descendants, &condition) } {
            Ok(search_bar) => search_bar,
            Err(err) if err.code().is_ok() => {
                return Ok(BrowserUrl {
                    url: None,
                    incognito,
                })
            }
            Err(err) => return Err(err).context("find first element"),
        };

        let search_value = unsafe { search_bar.GetCurrentPropertyValue(UIA_ValueValuePropertyId)? };
        let search_value = self.variant_to_string(search_value)?;
        info!("using omnibox url: {search_value}");

        // Omnibox URL is used when there is a long loading period (so document isn't loaded), or
        // when an old tab is loaded again from energy saver (so title is set, but no document is loaded).

        // From Omnibox, we get a URL like this:
        // "www.google.com/search?q=test". note that proto is not shown.
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

        Ok(BrowserUrl {
            url: if url.is_empty() { None } else { Some(url) },
            incognito,
        })
    }

    fn variant_to_string(&self, value: VARIANT) -> Result<String> {
        let value = unsafe { VariantToPropVariant(&value)? };
        let value_raw = unsafe { PropVariantToStringAlloc(&value)? };
        let value = String::from_utf16_lossy(unsafe { value_raw.as_wide() });
        unsafe { CoTaskMemFree(Some(value_raw.as_ptr().cast())) };
        Ok(value)
    }
}
