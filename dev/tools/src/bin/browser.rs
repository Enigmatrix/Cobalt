//! List out browser information

use clap::Parser;
use platform::web::{BrowserDetector, WebsiteInfo};
use tools::filters::{
    ProcessDetails, ProcessFilter, WindowDetails, WindowFilter, match_running_windows,
};
use util::error::Result;
use util::{Target, config, future as tokio};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Browser name
    #[arg(long)]
    app: Option<String>,

    /// Browser window title
    #[arg(long)]
    title: Option<String>,
}

#[derive(Debug, Clone)]
struct Browser {
    pub process: ProcessDetails,
    pub windows: Vec<BrowserWindow>,
}

#[derive(Debug, Clone)]
struct BrowserWindow {
    pub window: WindowDetails,
    pub tabs: Vec<TabDetails>,
}

#[derive(Debug, Clone)]
struct TabDetails {
    pub url: Option<String>,
    pub incognito: bool,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    util::set_target(Target::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let args = Args::parse();
    let window_group = match_running_windows(
        &WindowFilter {
            title: args.title,
            ..Default::default()
        },
        &ProcessFilter {
            name: args.app,
            ..Default::default()
        },
    )?;

    let detect = BrowserDetector::new()?;

    let mut browsers = vec![];
    for window_group in window_group {
        let mut browser_windows = vec![];
        for window in window_group.windows {
            if !detect.is_maybe_chromium_window(&window.window)? {
                continue;
            }
            let element = detect.get_chromium_element(&window.window)?;
            let url = detect.chromium_url(&element)?;
            let incognito = false; // TODO: get incognito from element
            let (name, description) = if let Some(url) = &url {
                let base_url = WebsiteInfo::url_to_base_url(url)?;
                let website_info = WebsiteInfo::from_base_url(base_url.clone())
                    .await
                    .unwrap_or(WebsiteInfo::default_from_url(base_url));
                (Some(website_info.name), Some(website_info.description))
            } else {
                (None, None)
            };

            let browser_window = BrowserWindow {
                window,
                tabs: vec![TabDetails {
                    url,
                    incognito,
                    name,
                    description,
                }],
            };

            browser_windows.push(browser_window);
        }
        if browser_windows.is_empty() {
            continue;
        }
        if !BrowserDetector::is_maybe_chromium_exe(&window_group.process.path) {
            continue;
        }

        let browser = Browser {
            process: window_group.process,
            windows: browser_windows,
        };
        browsers.push(browser);
    }

    // Print out the browsers

    for browser in &browsers {
        for window in &browser.windows {
            for tab in &window.tabs {
                println!("{}", tab_to_string(tab, window, browser));
            }
        }
    }

    Ok(())
}

fn tab_to_string(tab: &TabDetails, window: &BrowserWindow, browser: &Browser) -> String {
    format!(
        "{}{} - (hwnd: {:08x}) - {} (pid: {}){}{}\n",
        tab.url.as_deref().unwrap_or("<UNKNOWN>"),
        if tab.incognito { " [Incognito]" } else { "" },
        // window.window.title,
        window.window.window.hwnd.0 as usize,
        browser.process.name,
        browser.process.pid,
        tab.name
            .as_deref()
            .map(|s| format!("\n\tName: {s}"))
            .unwrap_or_default(),
        tab.description
            .as_deref()
            .map(|s| format!("\n\tDescription: {s}"))
            .unwrap_or_default()
    )
}
