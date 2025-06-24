use reqwest::Url;
use util::error::{Context, Result};
use util::tracing::{debug, info};
use windows::core::{AgileReference, Interface};
use windows::Win32::System::Com::StructuredStorage::{
    PropVariantToStringAlloc, VariantToPropVariant,
};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    AutomationElementMode_None, CUIAutomation, IUIAutomation, IUIAutomationCacheRequest,
    IUIAutomationCondition, IUIAutomationElement, IUIAutomationElement9, TreeScope_Descendants,
    TreeTraversalOptions_LastToFirstOrder, UIA_AutomationIdPropertyId, UIA_ClassNamePropertyId,
    UIA_NamePropertyId, UIA_ValueValuePropertyId,
};

use crate::objects::Window;

fn perf<T>(f: impl FnOnce() -> T, name: &str) -> T {
    let start = std::time::Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    debug!("{elapsed:?} for {name}");
    result
}

/// Detects browser usage information
pub struct BrowserDetector {
    automation: AgileReference<IUIAutomation>,
    browser_root_view_cond: AgileReference<IUIAutomationCondition>,
    root_web_area_cond: AgileReference<IUIAutomationCondition>,
    omnibox_cond: AgileReference<IUIAutomationCondition>,
    cache_request: AgileReference<IUIAutomationCacheRequest>,
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
        let element: IUIAutomationElement9 = perf(
            || unsafe { self.automation.resolve()?.ElementFromHandle(window.hwnd) }?.cast(),
            "ElementFromHandle",
        )?;

        let browser_root_view = self
            .uia_find_result(perf(
                || unsafe {
                    element.FindFirstWithOptionsBuildCache(
                        TreeScope_Descendants,
                        &self.browser_root_view_cond.resolve()?,
                        &self.cache_request.resolve()?,
                        TreeTraversalOptions_LastToFirstOrder,
                        &element,
                    )
                },
                "browser_root_view - FindFirstWithOptionsBuildCache",
            ))
            .context("find browser root view")?;
        let browser_root_view = match browser_root_view {
            Some(browser_root_view) => browser_root_view,
            None => {
                return Ok(BrowserUrl {
                    url: None,
                    incognito: false,
                })
            }
        };
        let name = perf(
            || {
                self.variant_to_string(unsafe {
                    browser_root_view.GetCachedPropertyValue(UIA_NamePropertyId)?
                })
            },
            "browser_root_view - GetCachedPropertyValue(UIA_NamePropertyId)",
        )?;

        // This seems to be the only way to detect incognito mode in Edge
        // Not sure why the \u{200b} is needed, but it works.
        // Have not tested with other languages.
        let incognito = name.ends_with("- Google Chrome (Incognito)")
            || name.ends_with("- Microsoft Edge (InPrivate)");

        let root_web_area = self
            .uia_find_result(perf(
                || unsafe {
                    element.FindFirstWithOptionsBuildCache(
                        TreeScope_Descendants,
                        &self.root_web_area_cond.resolve()?,
                        &self.cache_request.resolve()?,
                        TreeTraversalOptions_LastToFirstOrder,
                        &element,
                    )
                },
                "root_web_area - FindFirstWithOptionsBuildCache",
            ))
            .context("find root web area")?;
        if let Some(root_web_area) = root_web_area {
            let url = perf(
                || {
                    self.variant_to_string(unsafe {
                        root_web_area.GetCachedPropertyValue(UIA_ValueValuePropertyId)?
                    })
                },
                "root_web_area - GetCachedPropertyValue(UIA_ValueValuePropertyId)",
            )?;
            if !url.is_empty() {
                return Ok(BrowserUrl {
                    url: Some(url),
                    incognito,
                });
            }
        }

        let omnibox = self
            .uia_find_result(perf(
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
        let omnibox = match omnibox {
            Some(omnibox) => omnibox,
            None => {
                return Ok(BrowserUrl {
                    url: None,
                    incognito,
                })
            }
        };

        let search_value = perf(
            || unsafe { omnibox.GetCachedPropertyValue(UIA_ValueValuePropertyId) },
            "omnibox - GetCachedPropertyValue(UIA_ValueValuePropertyId)",
        )?;
        let search_value = self.variant_to_string(search_value)?;
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

    fn uia_find_result(
        &self,
        value: windows::core::Result<IUIAutomationElement>,
    ) -> windows::core::Result<Option<IUIAutomationElement9>> {
        match value {
            Ok(value) => Ok(Some(value.cast()?)),
            Err(err) if err.code().is_ok() => Ok(None),
            Err(err) => Err(err),
        }
    }
}
