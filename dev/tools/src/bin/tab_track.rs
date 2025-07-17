//! List out browser information

use std::sync::Mutex;

use platform::events::WindowTitleWatcher;
use platform::objects::{EventLoop, Target, Window};
use platform::web;
use tools::filters::{ProcessFilter, WindowFilter, match_running_windows};
use util::error::Result;
// use util::tracing::info;
use util::{Target as UtilTarget, config, future as tokio};

// fn perf<T>(f: impl Fn() -> T, act: &str) -> T {
//     let start = std::time::Instant::now();
//     let result = f();
//     info!("{}: {:?}", act, start.elapsed());
//     result
// }

struct UnsafeSyncSendBrowserDetect {
    detect: web::Detect,
}

impl UnsafeSyncSendBrowserDetect {
    fn new() -> Result<Self> {
        let detect = web::Detect::new()?;
        Ok(Self { detect })
    }

    pub fn chromium_url(&self, window: &Window) -> Result<Option<String>> {
        let element = self.detect.get_chromium_element(window)?;
        self.detect.chromium_url(&element)
    }
}

unsafe impl Send for UnsafeSyncSendBrowserDetect {}
unsafe impl Sync for UnsafeSyncSendBrowserDetect {}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    util::set_target(UtilTarget::Tool {
        name: "tab_track".to_string(),
    });
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let window_group = match_running_windows(
        &WindowFilter {
            ..Default::default()
        },
        &ProcessFilter {
            name: Some("Google Chrome".to_string()),
            ..Default::default()
        },
    )?;
    let chrome = window_group.first().unwrap();

    let browser = UnsafeSyncSendBrowserDetect::new()?;

    let m = Mutex::new(());

    let _h = WindowTitleWatcher::new(
        Target::Id(chrome.process.pid),
        Box::new(move |window| {
            let start = std::time::Instant::now();
            let _guard = m.lock().unwrap();

            let url = browser.chromium_url(&window)?;

            let dimset = if let Some(url) = url {
                if !url.contains("youtube") {
                    window.dim(0.5f64)?;
                } else {
                    window.dim(1.0f64)?;
                }
                Some(url)
            } else {
                None
            };
            println!(
                "{:08x}:{:?}, {:?}",
                window.hwnd.0 as usize,
                dimset,
                start.elapsed()
            );
            Ok(())
        }),
    )?;

    EventLoop::new().run();

    Ok(())
}
