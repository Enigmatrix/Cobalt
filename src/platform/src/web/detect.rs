use reqwest::Url;
use url::Host;
use util::error::{Context, Result};
use util::tracing::{debug, info, warn};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance};
use windows::Win32::UI::Accessibility::{
    AutomationElementMode_None, CUIAutomation, IUIAutomation, IUIAutomationCacheRequest,
    IUIAutomationCondition, IUIAutomationElement, IUIAutomationElement9,
    IUIAutomationInvokePattern, TreeScope_Children, TreeScope_Descendants,
    TreeTraversalOptions_LastToFirstOrder, UIA_AutomationIdPropertyId, UIA_ButtonControlTypeId,
    UIA_ClassNamePropertyId, UIA_ControlTypePropertyId, UIA_DocumentControlTypeId,
    UIA_InvokePatternId, UIA_NamePropertyId, UIA_SelectionItemIsSelectedPropertyId,
    UIA_TabItemControlTypeId, UIA_ValueValuePropertyId,
};
use windows::core::{AgileReference, Interface};

use crate::objects::Window;

const DEBUG_LIMIT: std::time::Duration = std::time::Duration::from_millis(10);
const WARN_LIMIT: std::time::Duration = std::time::Duration::from_millis(100);
const BROWSERS: [&str; 2] = [
    // Chrome
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    // Edge
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
];

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
pub struct Detect {
    /// Automation object
    pub automation: AgileReference<IUIAutomation>,
    browser_root_view_cond: AgileReference<IUIAutomationCondition>,
    root_web_area_cond: AgileReference<IUIAutomationCondition>,
    omnibox_cond: AgileReference<IUIAutomationCondition>,
    omnibox_icon_cond: AgileReference<IUIAutomationCondition>,
    cache_request: AgileReference<IUIAutomationCacheRequest>,
}

impl Detect {
    /// Create a new [BrowserDetector]
    pub fn new() -> Result<Self> {
        let automation: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)? };
        let browser_root_view_cond = unsafe {
            automation
                .CreatePropertyCondition(UIA_ClassNamePropertyId, &"BrowserRootView".into())?
        };
        let root_web_area_cond = unsafe {
            let control_cond = automation.CreatePropertyCondition(
                UIA_ControlTypePropertyId,
                &UIA_DocumentControlTypeId.0.into(),
            )?;
            let class_cond = automation
                .CreatePropertyCondition(UIA_AutomationIdPropertyId, &"RootWebArea".into())?;
            automation.CreateAndCondition(&control_cond, &class_cond)?
        };
        let omnibox_cond = unsafe {
            automation
                .CreatePropertyCondition(UIA_NamePropertyId, &"Address and search bar".into())?
        };
        let omnibox_icon_cond = unsafe {
            automation
                .CreatePropertyCondition(UIA_ClassNamePropertyId, &"LocationIconView".into())?
        };
        let cache_request = unsafe {
            let cache_request = automation.CreateCacheRequest()?;
            cache_request.SetAutomationElementMode(AutomationElementMode_None)?;
            cache_request.AddProperty(UIA_NamePropertyId)?;
            cache_request.AddProperty(UIA_ValueValuePropertyId)?;
            cache_request
        };
        Ok(Self {
            automation: AgileReference::new(&automation)?,
            browser_root_view_cond: AgileReference::new(&browser_root_view_cond)?,
            root_web_area_cond: AgileReference::new(&root_web_area_cond)?,
            omnibox_cond: AgileReference::new(&omnibox_cond)?,
            omnibox_icon_cond: AgileReference::new(&omnibox_icon_cond)?,
            cache_request: AgileReference::new(&cache_request)?,
        })
    }

    /// Check if the path is a browser. Not meant to be super-accurate, but should be good enough.
    pub fn is_chromium_exe(&self, path: &str) -> bool {
        BROWSERS
            .iter()
            .any(|browser| path.to_lowercase() == browser.to_lowercase())
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

    /// Get the UI Automation element for the omnibox in the [Window]
    pub fn get_chromium_omnibox_element(
        &self,
        window_element: &IUIAutomationElement9,
        use_cache: bool,
    ) -> Result<Option<IUIAutomationElement9>> {
        uia_find_result(perf(
            || unsafe {
                window_element.FindFirstBuildCache(
                    TreeScope_Descendants,
                    &self.omnibox_cond.resolve()?,
                    (if use_cache {
                        Some(self.cache_request.resolve()?)
                    } else {
                        None
                    })
                    .as_ref(),
                )
            },
            "omnibox - FindFirstBuildCache",
        ))
        .context("find omnibox")
    }

    /// Get the UI Automation element for the omnibox icon in the [Window]
    pub fn get_chromium_omnibox_icon_element(
        &self,
        window_element: &IUIAutomationElement9,
        use_cache: bool,
    ) -> Result<Option<IUIAutomationElement9>> {
        uia_find_result(perf(
            || unsafe {
                window_element.FindFirstBuildCache(
                    TreeScope_Descendants,
                    &self.omnibox_icon_cond.resolve()?,
                    (if use_cache {
                        Some(self.cache_request.resolve()?)
                    } else {
                        None
                    })
                    .as_ref(),
                )
            },
            "omnibox_icon - FindFirstBuildCache",
        ))
        .context("find omnibox icon")
    }

    /// Check if the [Window] is in incognito mode
    pub fn chromium_incognito(&self, element: &IUIAutomationElement9) -> Result<Option<bool>> {
        let browser_root_view = uia_find_result(perf(
            || unsafe {
                element.FindFirstWithOptionsBuildCache(
                    TreeScope_Descendants,
                    &self.browser_root_view_cond.resolve()?,
                    &self.cache_request.resolve()?,
                    TreeTraversalOptions_LastToFirstOrder,
                    element,
                )
            },
            "browser_root_view - FindFirstWithOptionsBuildCache",
        ))
        .context("find browser root view")?;
        let Some(browser_root_view) = browser_root_view else {
            return Ok(None);
        };
        let name = perf(
            || unsafe { browser_root_view.GetCachedPropertyValue(UIA_NamePropertyId) },
            "browser_root_view - GetCachedPropertyValue(UIA_NamePropertyId)",
        )?
        .to_string();

        // This seems to be the only way to detect incognito mode in Edge
        // Have not tested with other languages.
        let incognito = name.ends_with("- Google Chrome (Incognito)")
            || name.ends_with("- Microsoft Edge (InPrivate)");

        Ok(Some(incognito))
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
                return Ok(Some(Self::elide_document_url(&url)?));
            }
        }

        let omnibox = self.get_chromium_omnibox_element(element, true)?;
        let Some(omnibox) = omnibox else {
            return Ok(None);
        };

        let omnibox_icon = self.get_chromium_omnibox_icon_element(element, true)?;
        let Some(omnibox_icon) = omnibox_icon else {
            return Ok(None);
        };

        let search_value = perf(
            || unsafe { omnibox.GetCachedPropertyValue(UIA_ValueValuePropertyId) },
            "omnibox - GetCachedPropertyValue(UIA_ValueValuePropertyId)",
        )?
        .to_string();

        let icon_text = perf(
            || unsafe { omnibox_icon.GetCachedPropertyValue(UIA_NamePropertyId) },
            "omnibox_icon - GetCachedPropertyValue(UIA_NamePropertyId)",
        )?
        .to_string();
        info!("using omnibox url: {search_value}, icon: {icon_text}");

        // For HTTP urls, the icon is the error icon with text "Not secure".
        // In rare cases of HTTPS url with invalid cert, the icon is the same as above
        // however, the 'search_value' is the full https url so is_http_hint is ignored.
        let is_http_hint = icon_text == "Not secure";
        let url = Self::unelide_omnibox_text(search_value, is_http_hint);

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

    /// Unelide the omnibox text to the create a valid URL.
    pub fn unelide_omnibox_text(mut search_value: String, is_http_hint: bool) -> String {
        // Omnibox URL is used when there is a long loading period (so document isn't loaded), or
        // when an old tab is loaded again from energy saver (so title is set, but no document is loaded).

        // From Omnibox, we get a URL like this:
        // "google.com/search?q=test". note that proto (scheme) is not shown.
        // for chrome://, data:XXX and other non http/https links, it's there tho e.g. chrome://newtab.

        let add_scheme = match Url::parse(&search_value) {
            Ok(url) => {
                if url.cannot_be_a_base() {
                    // localhost:1337/a.html gets parsed as a relative url without base! with schema=localhost, path=1337/a.html
                    // cannot_be_a_base = true. So we have a heuristic to check if the path's first character starts with a number to indicate a port.
                    url.path()
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            // see if we get no proto error (here it's relative url without base)
            // for e.g. google.com/a
            Err(url::ParseError::RelativeUrlWithoutBase) => true,
            _ => false,
        };

        if add_scheme {
            // we have no idea if www. is there or not.
            let mut with_scheme = format!(
                "{}://{search_value}",
                if is_http_hint { "http" } else { "https" }
            );

            // Replace https with http if the host is trustworthy, since if the real URL is http:// chromium would
            // consider it to be trusted and we would have used it as https:// in the above code.
            // These URLs actually are most likely to be HTTP.
            // https://www.w3.org/TR/secure-contexts/#is-origin-trustworthy
            if let Ok(url) = Url::parse(&with_scheme) {
                let trustworthy = match url.host() {
                    Some(Host::Domain(domain)) => {
                        domain == "localhost"
                            || domain == "localhost."
                            || domain.ends_with(".localhost")
                            || domain.ends_with(".localhost.")
                    }
                    Some(Host::Ipv4(ip)) => ip.is_loopback(),
                    Some(Host::Ipv6(ip)) => ip.is_loopback(), // This is only ::1 instead of ::1/128, but it's good enough.
                    _ => false,
                };
                if trustworthy {
                    with_scheme = format!("http://{search_value}");
                }
            }

            search_value = with_scheme;
        }

        Url::parse(&search_value).map_or(search_value, |mut url| {
            // https://google.com -> https://google.com/
            if url.path() == "" {
                url.set_path("/");
            }
            url.to_string()
        })
    }

    /// Elide the document URL to the text that would be shown in the omnibox.
    pub fn elide_document_url(url: &str) -> Result<String> {
        let Ok(mut url) = Url::parse(url) else {
            return Ok(url.to_string());
        };
        if url.scheme() == "http" || url.scheme() == "https" {
            // Strip www. from the host
            if let Some(host) = url.host_str()
                && let Some(new_host) = host.strip_prefix("www.")
            {
                let new_host = new_host.to_string();
                url.set_host(Some(&new_host))?;
            }
        }

        // https://google.com -> https://google.com/
        if url.path() == "" {
            url.set_path("/");
        }

        Ok(url.to_string())
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

#[test]
fn test_unelide_localhost() {
    let search_value = "localhost:33790/diff_url_same_title_1.html";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "http://localhost:33790/diff_url_same_title_1.html");
}

#[test]
fn test_unelide_localhost_hint() {
    let search_value = "localhost:33790/diff_url_same_title_1.html";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), true);
    assert_eq!(url, "http://localhost:33790/diff_url_same_title_1.html");
}

#[test]
fn test_unelide_https_www() {
    let search_value = "google.com";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "https://google.com/");
}

#[test]
fn test_unelide_https_www_hint() {
    let search_value = "google.com";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), true);
    assert_eq!(url, "http://google.com/");
}

#[test]
fn test_unelide_https_www_whole() {
    let search_value = "https://google.com";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "https://google.com/");
}

#[test]
fn test_unelide_https_www_whole_hint() {
    let search_value = "https://google.com";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), true);
    assert_eq!(url, "https://google.com/");
}

#[test]
fn test_unelide_https_www_whole2() {
    let search_value = "https://www.google.com";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "https://www.google.com/");
}

#[test]
fn test_unelide_https_www_with_path_query() {
    let search_value = "google.com/search?q=test";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "https://google.com/search?q=test");
}

#[test]
fn test_unelide_non_http() {
    let search_value = "chrome://settings/";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "chrome://settings/");
}

#[test]
fn test_unelide_non_http2() {
    let search_value = "chrome://settings";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "chrome://settings/");
}

#[test]
fn test_unelide_non_http_with_path() {
    let search_value = "chrome://settings/what";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "chrome://settings/what");
}

#[test]
fn test_unelide_non_http_with_query() {
    let search_value = "chrome://settings/?q=test";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "chrome://settings/?q=test");
}

#[test]
fn test_unelide_non_http_with_path_query() {
    let search_value = "chrome://settings/what?q=test";
    let url = Detect::unelide_omnibox_text(search_value.to_string(), false);
    assert_eq!(url, "chrome://settings/what?q=test");
}

#[test]
fn test_elide_https_www() -> Result<()> {
    let url = "https://www.google.com";
    let elided = Detect::elide_document_url(url)?;
    assert_eq!(elided, "https://google.com/");
    Ok(())
}

#[test]
fn test_elide_http_www() -> Result<()> {
    let url = "http://www.google.com";
    let elided = Detect::elide_document_url(url)?;
    assert_eq!(elided, "http://google.com/");
    Ok(())
}

#[test]
fn test_elide_https_www_with_path_query() -> Result<()> {
    let url = "https://www.google.com/search?q=test";
    let elided = Detect::elide_document_url(url)?;
    assert_eq!(elided, "https://google.com/search?q=test");
    Ok(())
}

#[test]
fn test_elide_http_www_with_path_query() -> Result<()> {
    let url = "http://www.google.com/search?q=test";
    let elided = Detect::elide_document_url(url)?;
    assert_eq!(elided, "http://google.com/search?q=test");
    Ok(())
}

#[test]
fn test_elide_non_http() -> Result<()> {
    let url = "chrome://settings";
    let elided = Detect::elide_document_url(url)?;
    assert_eq!(elided, "chrome://settings/");
    Ok(())
}

#[test]
fn test_elide_non_http2() -> Result<()> {
    let url = "chrome://settings/";
    let elided = Detect::elide_document_url(url)?;
    assert_eq!(elided, "chrome://settings/");
    Ok(())
}

#[test]
fn test_elide_non_http_with_path() -> Result<()> {
    let url = "chrome://settings/what";
    let elided = Detect::elide_document_url(url)?;
    assert_eq!(elided, "chrome://settings/what");
    Ok(())
}

#[test]
fn test_elide_non_http_with_query() -> Result<()> {
    let url = "chrome://settings/?q=test";
    let elided = Detect::elide_document_url(url)?;
    assert_eq!(elided, "chrome://settings/?q=test");
    Ok(())
}

#[test]
fn test_elide_non_http_with_path_query() -> Result<()> {
    let url = "chrome://settings/what?q=test";
    let elided = Detect::elide_document_url(url)?;
    assert_eq!(elided, "chrome://settings/what?q=test");
    Ok(())
}
