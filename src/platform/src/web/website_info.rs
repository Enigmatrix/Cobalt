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

/// Website URL down to origin
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseWebsiteUrl {
    /// HTTP(S) URL
    Http(String),
    /// file://, chrome://, about:blank, etc.
    NonHttp(String),
}

impl fmt::Display for BaseWebsiteUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseWebsiteUrl::Http(url) => write!(f, "{url}"),
            BaseWebsiteUrl::NonHttp(url) => write!(f, "{url}"),
        }
    }
}

impl WebsiteInfo {
    /// Convert a URL to a base URL. If http/https, remove www. from the host.
    /// else, if a valid url, try to make it as 'basic' as possible.
    pub fn url_to_base_url(url: &str) -> Result<BaseWebsiteUrl> {
        Ok(if let Ok(mut url) = Url::parse(url) {
            // if password is present, remove it
            if url.password().is_some() {
                // ignore any error that pops out
                let _ = url.set_password(Some("<password>"));
            }

            if url.scheme() == "https" || url.scheme() == "http" {
                if let Some(host) = url.host_str() {
                    let host = host.to_string();
                    if let Some(host) = host.strip_prefix("www.") {
                        url.set_host(Some(host))?;
                    }
                }
                let origin = url.origin().unicode_serialization();
                BaseWebsiteUrl::Http(origin)
            } else {
                url.set_fragment(None);
                url.set_query(None);
                BaseWebsiteUrl::NonHttp(url.to_string())
            }
        } else {
            BaseWebsiteUrl::NonHttp(url.to_string())
        })
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
            BaseWebsiteUrl::NonHttp(_) => {
                return Ok(Self::default_from_url(base_url));
            }
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
        let ext = Self::get_icon_ext(&icon_url);

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

        let mime = response.headers().get("Content-Type").cloned();
        let bytes = response.bytes().await?;

        let icon = Icon {
            data: bytes.to_vec(),
            ext,
            mime: mime.and_then(|s| s.to_str().ok().map(|s| s.to_string())),
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
