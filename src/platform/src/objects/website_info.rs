use reqwest::{Client, Url};
use scraper::{Html, Selector};
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;

use super::AppInfo;

/// Information about a Website
pub struct WebsiteInfo {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Logo as bytes
    pub logo: Option<Vec<u8>>,
}

/// Website URL down to origin
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseWebsiteUrl {
    /// HTTP(S) URL
    Http(String),
    /// file://, chrome://, about:blank, etc.
    NonHttp(String),
}

impl BaseWebsiteUrl {
    /// Convert a base URL to a string
    pub fn to_string(&self) -> String {
        match self {
            BaseWebsiteUrl::Http(url) => url.clone(),
            BaseWebsiteUrl::NonHttp(url) => url.clone(),
        }
    }
}

impl WebsiteInfo {
    /// Convert a URL to a base URL
    pub fn url_to_base_url(url: &str) -> Result<BaseWebsiteUrl> {
        Ok(if let Ok(url) = Url::parse(url) {
            if url.scheme() == "https" || url.scheme() == "http" {
                BaseWebsiteUrl::Http(url.origin().unicode_serialization())
            } else {
                BaseWebsiteUrl::NonHttp(url.to_string())
            }
        } else {
            BaseWebsiteUrl::NonHttp(url.to_string())
        })
    }

    /// Create a new [WebsiteInfo] from a base URL
    pub async fn from_base_url(base_url: BaseWebsiteUrl) -> Result<Self> {
        let url = match base_url {
            BaseWebsiteUrl::Http(url) => url,
            BaseWebsiteUrl::NonHttp(url) => {
                return Ok(WebsiteInfo {
                    name: url.clone(),
                    description: url,
                    logo: None,
                });
            }
        };

        let client = Client::new();

        // Get Open Graph tags + Meta information
        let (description, site_name, logo_url) = {
            let response = client.get(&url).send().await?;
            let html = response.text().await?;
            let document = Html::parse_document(&html);

            let mut site_name = Self::get_site_name(&url, &document)
                .context("get site name")
                .warn();
            if site_name == String::default() {
                site_name = url.to_string();
            }

            let description = Self::get_description(&document)
                .context("get description")
                .warn();
            let logo_url = Self::get_logo_url(&url, &document).context("get logo url");
            (description, site_name, logo_url)
        };

        // Get logo. If it fails, then logo is None.
        let logo = match logo_url {
            Ok(logo_url) => Self::get_logo(&client, logo_url)
                .await
                .map(|logo| Some(logo))
                .context("get logo")
                .warn(),
            logo_url => {
                logo_url.map(|_| "").warn();
                None
            }
        };

        Ok(WebsiteInfo {
            name: site_name,
            description,
            logo,
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
        return Ok(url.to_string());
    }

    /// Get the logo URL of the website
    pub fn get_logo_url(url: &str, document: &Html) -> Result<Url> {
        let favicon_url = document
            .select(&Selector::parse("link[rel='icon'], link[rel='shortcut icon']").unwrap())
            .next()
            .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
            .unwrap_or_else(|| "/favicon.ico".to_string());

        let logo_url = Url::parse(&url)?.join(&favicon_url)?;
        Ok(logo_url)
    }

    /// Get the logo of the website
    pub async fn get_logo(client: &Client, logo_url: Url) -> Result<Vec<u8>> {
        let response = client.get(logo_url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

impl From<WebsiteInfo> for AppInfo {
    fn from(website: WebsiteInfo) -> Self {
        AppInfo {
            name: website.name,
            description: website.description,
            // TODO: company name does not exist for websites
            company: "".to_string(),
            logo: website.logo,
        }
    }
}
