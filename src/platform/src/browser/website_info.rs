use std::fmt;

use reqwest::{Client, RequestBuilder, Url};
use scraper::{Html, Selector};
use util::error::{Context, Result, bail};
use util::tracing::ResultTraceExt;

use crate::objects::{AppInfo, Icon, random_color};

/// Information about a Website
pub struct WebsiteInfo {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Color
    pub color: String,
    /// Icon
    pub icon: Option<Icon>,
}

/// Web-supported schemes, e.g. file://, chrome://, about:blank, etc.
pub const WEB_SUPPORTED_SCHEMES: [&str; 10] = [
    // Generic URLs, can be seen by user
    "file",
    "data",
    "blob",
    "ftp",
    "wss",
    "ws",
    // Browser-specific
    "chrome",
    "edge",
    "about",
    "view-source",
];

/// URL Base, accounting for http(s), chrome-extension, web-supported, and non-http URLs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseWebsiteUrl {
    /// HTTP(S) URL
    Http(String),
    /// Chrome extension URL
    ChromeExtension {
        /// Extension ID, e.g. "abcdefghijklmnopqrstuvwxyz1234567890"
        extension_id: String,
    },
    /// Web-supported URL (see [WEB_SUPPORTED_SCHEMES])
    WebSupported {
        /// Scheme of the URL (see [WEB_SUPPORTED_SCHEMES])
        scheme: String,
    },
    /// any other non-http URL, including invalid URLs
    NonHttp(String),
}

impl fmt::Display for BaseWebsiteUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseWebsiteUrl::Http(url) => write!(f, "{url}"),
            BaseWebsiteUrl::ChromeExtension { extension_id } => {
                write!(f, "chrome-extension://{extension_id}")
            }
            BaseWebsiteUrl::WebSupported { scheme } => write!(f, "{scheme}:"),
            BaseWebsiteUrl::NonHttp(url) => write!(f, "{url}"),
        }
    }
}

impl WebsiteInfo {
    /// Convert a URL to a base URL. If http/https, remove www. from the host.
    /// else, if a valid url, try to make it as 'basic' as possible.
    pub fn url_to_base_url(url: &str) -> BaseWebsiteUrl {
        if let Ok(mut url) = Url::parse(url) {
            // if password is present, remove it
            if url.password().is_some() {
                // ignore any error that pops out
                let _ = url.set_password(Some("<password>"));
            }

            if url.scheme() == "https" || url.scheme() == "http" {
                if let Some(host) = url.host_str() {
                    let host = host.to_string();
                    if let Some(host) = host.strip_prefix("www.") {
                        url.set_host(Some(host))
                            .with_context(|| format!("cannot strip www. of {url}"))
                            .warn();
                    }
                }
                let origin = url.origin().unicode_serialization();
                BaseWebsiteUrl::Http(origin)
            } else if url.scheme() == "chrome-extension"
                && let Some(extension_id) = url.host_str()
            {
                BaseWebsiteUrl::ChromeExtension {
                    extension_id: extension_id.to_string(),
                }
            } else if WEB_SUPPORTED_SCHEMES.contains(&url.scheme()) {
                BaseWebsiteUrl::WebSupported {
                    scheme: url.scheme().to_string(),
                }
            } else {
                url.set_fragment(None);
                url.set_query(None);
                BaseWebsiteUrl::NonHttp(url.to_string())
            }
        } else {
            BaseWebsiteUrl::NonHttp(url.to_string())
        }
    }

    fn modify_request(rb: RequestBuilder) -> RequestBuilder {
        rb.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/538.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/538.36")
    }

    fn pretty_url_host(url: &str) -> String {
        // Get the host of the URL
        // https://www.google.com -> google.com if http(s), else url

        let url = match Url::parse(url) {
            Ok(url) => url,
            Err(_) => return url.to_string(),
        };

        if url.scheme() == "https" || url.scheme() == "http" {
            let mut host = url.host_str().expect("host is required");
            if host.starts_with("www.") {
                host = &host[4..];
            }
            host.to_string()
        } else {
            url.to_string()
        }
    }

    /// Create a default [WebsiteInfo] from a base URL
    pub fn default_from_url(base_url: BaseWebsiteUrl) -> Self {
        let url = match base_url {
            BaseWebsiteUrl::Http(url) => url,

            BaseWebsiteUrl::ChromeExtension { extension_id } => {
                return Self {
                    name: format!("Chrome Extension {extension_id}"),
                    description: "Chrome Extension".to_string(),
                    color: random_color(),
                    // maybe in the future we can get the icon of the extension
                    icon: None,
                };
            }
            BaseWebsiteUrl::WebSupported { scheme } => {
                return Self {
                    name: format!("'{scheme}' scheme URL"),
                    description: format!("'{scheme}' scheme URL"),
                    color: random_color(),
                    icon: None,
                };
            }
            BaseWebsiteUrl::NonHttp(url) => url,
        };
        Self {
            name: Self::pretty_url_host(&url),
            description: url,
            color: random_color(),
            icon: None,
        }
    }

    /// Create a new [WebsiteInfo] from a base URL
    pub async fn from_base_url(base_url: BaseWebsiteUrl) -> Result<Self> {
        let url = match base_url {
            BaseWebsiteUrl::Http(url) => url,
            _ => return Ok(Self::default_from_url(base_url)),
        };

        let client = Client::new();

        // Get Open Graph tags + Meta information
        let (description, site_name, icon_url) = {
            let response = Self::modify_request(client.get(&url)).send().await?;
            if !response.status().is_success() {
                bail!(
                    "failed to get website info for {} with status {}",
                    url,
                    response.status()
                );
            }
            let html = response.text().await?;
            let document = Html::parse_document(&html);

            let mut site_name = Self::get_site_name(&url, &document)
                .context("get site name")
                .warn();
            if site_name.is_empty() {
                site_name = Self::pretty_url_host(&url);
            }

            let description = Self::get_description(&document)
                .context("get description")
                .warn();
            let icon_url = Self::get_icon_url(&url, &document).context("get icon url");
            (description, site_name, icon_url)
        };

        // Get icon. If it fails, then icon is None.
        let icon = match icon_url {
            Ok(icon_url) => Self::get_icon(&client, icon_url)
                .await
                .map(Option::Some)
                .context("get icon")
                .warn(),
            icon_url => {
                icon_url.map(|_| "").warn();
                None
            }
        };

        Ok(WebsiteInfo {
            name: site_name,
            description,
            color: random_color(),
            icon,
        })
    }

    /// Get the description of the website
    pub fn get_description(document: &Html) -> Result<String> {
        let og_description = document
            .select(&Selector::parse("meta[property='og:description']").unwrap())
            .next();
        if let Some(og_description) = og_description {
            return Ok(og_description
                .value()
                .attr("content")
                .unwrap_or_default()
                .to_string());
        }

        let meta_description = document
            .select(&Selector::parse("meta[name='description']").unwrap())
            .next();
        if let Some(meta_description) = meta_description {
            return Ok(meta_description
                .value()
                .attr("content")
                .unwrap_or_default()
                .to_string());
        }

        Ok(String::new())
    }

    /// Get the name of the website
    pub fn get_site_name(url: &str, document: &Html) -> Result<String> {
        let og_site_name = document
            .select(&Selector::parse("meta[property='og:site_name']").unwrap())
            .next();
        if let Some(og_site_name) = og_site_name {
            return Ok(og_site_name
                .value()
                .attr("content")
                .unwrap_or_default()
                .to_string());
        }
        let og_title = document
            .select(&Selector::parse("meta[property='og:title']").unwrap())
            .next();
        if let Some(og_site_name) = og_title {
            return Ok(og_site_name
                .value()
                .attr("content")
                .unwrap_or_default()
                .to_string());
        }

        let title = document.select(&Selector::parse("title").unwrap()).next();
        if let Some(title) = title {
            return Ok(title.text().collect::<String>());
        }

        Ok(Self::pretty_url_host(url))
    }

    /// Get the icon URL of the website
    pub fn get_icon_url(url: &str, document: &Html) -> Result<Url> {
        let favicon_url = document
            .select(&Selector::parse("link[rel='icon'], link[rel='shortcut icon']").unwrap())
            .next()
            .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
            .unwrap_or_else(|| "/favicon.ico".to_string());

        let icon_url = Url::parse(url)?.join(&favicon_url)?;
        Ok(icon_url)
    }

    /// Get the icon of the website
    pub async fn get_icon(client: &Client, icon_url: Url) -> Result<Icon> {
        let response = Self::modify_request(client.get(icon_url.clone()))
            .send()
            .await?;
        if !response.status().is_success() {
            bail!(
                "failed to get website icon for {} with status {}",
                icon_url,
                response.status()
            );
        }

        let bytes = response.bytes().await?;

        let icon = Icon {
            data: bytes.to_vec(),
        };
        Ok(icon)
    }

    /// Get the extension of the icon
    pub fn get_icon_ext(icon_url: &Url) -> Option<String> {
        let file_name = icon_url.path_segments()?.next_back()?; // last segment
        let file_path = std::path::Path::new(file_name);
        let ext = file_path.extension()?;
        Some(ext.to_string_lossy().to_string())
    }
}

impl From<WebsiteInfo> for AppInfo {
    fn from(website: WebsiteInfo) -> Self {
        AppInfo {
            name: website.name,
            description: website.description,
            // TODO: company name does not exist for websites
            company: "".to_string(),
            color: website.color,
            icon: website.icon,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_extension_url_arb_to_base_url() {
        let url = "chrome-extension://abcdefghijklmnopqrstuvwxyz1234567890/src/css/wee.styles.css";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::ChromeExtension {
                extension_id: "abcdefghijklmnopqrstuvwxyz1234567890".to_string()
            }
        );
    }

    #[test]
    fn test_chrome_extension_url_base_to_base_url() {
        let url = "chrome-extension://abcdefghijklmnopqrstuvwxyz1234567890";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::ChromeExtension {
                extension_id: "abcdefghijklmnopqrstuvwxyz1234567890".to_string()
            }
        );
    }

    #[test]
    fn test_chrome_extension_url_base_with_slash_to_base_url() {
        let url = "chrome-extension://abcdefghijklmnopqrstuvwxyz1234567890/";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::ChromeExtension {
                extension_id: "abcdefghijklmnopqrstuvwxyz1234567890".to_string()
            }
        );
    }

    #[test]
    fn test_data_to_base_url() {
        let url = "data:text/html,<html><body><h1>Hello, World!</h1></body></html>";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "data".to_string()
            }
        );
    }

    // HTTP/HTTPS URL tests
    #[test]
    fn test_https_url_simple() {
        let url = "https://example.com";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com".to_string())
        );
    }

    #[test]
    fn test_http_url_simple() {
        let url = "http://example.com";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("http://example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_with_www() {
        let url = "https://www.example.com";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com".to_string())
        );
    }

    #[test]
    fn test_http_url_with_www() {
        let url = "http://www.example.com";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("http://example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_with_path() {
        let url = "https://example.com/path/to/page";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_with_query() {
        let url = "https://example.com?key=value&other=param";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_with_fragment() {
        let url = "https://example.com#section";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_with_path_query_fragment() {
        let url = "https://www.example.com/path/to/page?key=value#section";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com".to_string())
        );
    }

    #[test]
    fn test_http_url_with_default_port() {
        let url = "http://example.com:80";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("http://example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_with_default_port2() {
        let url = "https://example.com:443";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com".to_string())
        );
    }

    #[test]
    fn test_http_url_with_default_port_and_path() {
        let url = "http://example.com:80/path";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("http://example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_with_port() {
        let url = "https://example.com:8080";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com:8080".to_string())
        );
    }

    #[test]
    fn test_https_url_with_port_and_path() {
        let url = "https://www.example.com:8080/path";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com:8080".to_string())
        );
    }

    #[test]
    fn test_https_url_with_username() {
        let url = "https://user@example.com";
        // Origin serialization doesn't include username
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_with_username_and_password() {
        let url = "https://user:password@example.com";
        // Password is masked in URL object, but origin() doesn't include credentials
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_with_subdomain() {
        let url = "https://subdomain.example.com";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://subdomain.example.com".to_string())
        );
    }

    #[test]
    fn test_https_url_www_subdomain() {
        let url = "https://www.subdomain.example.com";
        // www. prefix is stripped from the beginning of the host
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::Http("https://subdomain.example.com".to_string())
        );
    }

    // Web-supported scheme tests
    #[test]
    fn test_file_url() {
        let url = "file:///path/to/file.html";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "file".to_string()
            }
        );
    }

    #[test]
    fn test_blob_url() {
        let url = "blob:https://example.com/uuid-here";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "blob".to_string()
            }
        );
    }

    #[test]
    fn test_ftp_url() {
        let url = "ftp://ftp.example.com/file.txt";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "ftp".to_string()
            }
        );
    }

    #[test]
    fn test_ws_url() {
        let url = "ws://example.com/socket";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "ws".to_string()
            }
        );
    }

    #[test]
    fn test_wss_url() {
        let url = "wss://example.com/socket";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "wss".to_string()
            }
        );
    }

    #[test]
    fn test_chrome_url() {
        let url = "chrome://settings/";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "chrome".to_string()
            }
        );
    }

    #[test]
    fn test_edge_url() {
        let url = "edge://settings/";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "edge".to_string()
            }
        );
    }

    #[test]
    fn test_about_url() {
        let url = "about:blank";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "about".to_string()
            }
        );
    }

    #[test]
    fn test_view_source_url() {
        let url = "view-source:https://example.com";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::WebSupported {
                scheme: "view-source".to_string()
            }
        );
    }

    // Non-HTTP URL tests
    #[test]
    fn test_mailto_url() {
        let url = "mailto:user@example.com?subject=Hello";
        let result = WebsiteInfo::url_to_base_url(url);
        assert_eq!(
            result,
            BaseWebsiteUrl::NonHttp("mailto:user@example.com".to_string())
        );
    }

    #[test]
    fn test_tel_url() {
        let url = "tel:+1234567890";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::NonHttp("tel:+1234567890".to_string())
        );
    }

    #[test]
    fn test_mailto_url_with_fragment() {
        let url = "mailto:user@example.com#fragment";
        let result = WebsiteInfo::url_to_base_url(url);
        assert_eq!(
            result,
            BaseWebsiteUrl::NonHttp("mailto:user@example.com".to_string())
        );
    }

    #[test]
    fn test_mailto_url_with_query_and_fragment() {
        let url = "mailto:user@example.com?subject=Hello&body=World#fragment";
        let result = WebsiteInfo::url_to_base_url(url);
        assert_eq!(
            result,
            BaseWebsiteUrl::NonHttp("mailto:user@example.com".to_string())
        );
    }

    // Invalid URL tests
    #[test]
    fn test_invalid_url() {
        let url = "not a valid url";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::NonHttp("not a valid url".to_string())
        );
    }

    #[test]
    fn test_empty_string() {
        let url = "";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::NonHttp("".to_string())
        );
    }

    #[test]
    fn test_invalid_scheme() {
        let url = "invalid-scheme://example.com";
        assert_eq!(
            WebsiteInfo::url_to_base_url(url),
            BaseWebsiteUrl::NonHttp("invalid-scheme://example.com".to_string())
        );
    }

    #[test]
    fn test_just_scheme() {
        let url = "https://";
        let result = WebsiteInfo::url_to_base_url(url);
        assert!(matches!(result, BaseWebsiteUrl::NonHttp(_)));
    }
}
