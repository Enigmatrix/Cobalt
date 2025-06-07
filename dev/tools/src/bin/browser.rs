//! List out browser information

use clap::Parser;
use platform::objects::BrowserDetector;
use tools::filters::{
    match_running_windows, ProcessDetails, ProcessFilter, WindowDetails, WindowFilter,
};
use util::error::Result;
use util::{config, Target};

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
}

fn main() -> Result<()> {
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
            if !detect.is_chromium(&window.window)? {
                continue;
            }
            let info = detect.chromium_url(&window.window)?;

            let browser_window = BrowserWindow {
                window,
                tabs: vec![TabDetails {
                    url: info.url,
                    incognito: info.incognito,
                }],
            };

            browser_windows.push(browser_window);
        }
        if browser_windows.is_empty() {
            continue;
        }
        if !BrowserDetector::is_browser(&window_group.process.path) {
            continue;
        }

        let browser = Browser {
            process: window_group.process,
            windows: browser_windows,
        };
        browsers.push(browser);
    }

    dbg!(&browsers);

    Ok(())
}
