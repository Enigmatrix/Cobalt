//! List out browser information

use std::sync::Mutex;

use platform::objects::{EventLoop, Target, WinEventHook, Window};
use platform::web::BrowserDetector;
use tools::filters::{ProcessFilter, WindowFilter, match_running_windows};
use util::error::Result;
// use util::tracing::info;
use util::{Target as UtilTarget, config, future as tokio};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use windows::Win32::UI::WindowsAndMessaging::{EVENT_OBJECT_NAMECHANGE, OBJID_WINDOW};

// fn perf<T>(f: impl Fn() -> T, act: &str) -> T {
//     let start = std::time::Instant::now();
//     let result = f();
//     info!("{}: {:?}", act, start.elapsed());
//     result
// }

struct UnsafeSyncSendBrowserDetect {
    browser: BrowserDetector,
}

impl UnsafeSyncSendBrowserDetect {
    fn new() -> Result<Self> {
        let browser = BrowserDetector::new()?;
        Ok(Self { browser })
    }

    pub fn chromium_url(&self, window: &Window) -> Result<Option<String>> {
        let element = self.browser.get_chromium_element(window)?;
        self.browser.chromium_url(&element, None)
    }
}

unsafe impl Send for UnsafeSyncSendBrowserDetect {}
unsafe impl Sync for UnsafeSyncSendBrowserDetect {}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    util::set_target(UtilTarget::Engine);
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

    let _hook = WinEventHook::new(
        platform::objects::WinEvent::Event(EVENT_OBJECT_NAMECHANGE),
        Target::Id(chrome.process.pid),
        Target::All,
        Box::new(
            move |_hwineventhook: HWINEVENTHOOK,
                  _event: u32,
                  hwnd: HWND,
                  id_object: i32,
                  _id_child: i32,
                  _id_event_thread: u32,
                  _dwms_event_time: u32| {
                let guard = m.lock().expect("reentrancy lock");
                if id_object != OBJID_WINDOW.0 {
                    return;
                }
                let start = std::time::Instant::now();
                let window = Window::new(hwnd);
                let url = browser.chromium_url(&window).expect("chromium_url");

                let dimset = if let Some(url) = url {
                    if !url.contains("youtube") {
                        window.dim(0.5f64).expect("dim");
                    } else {
                        window.dim(1.0f64).expect("dim");
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

                drop(guard);
            },
        ),
    )?;

    EventLoop::new().run();

    Ok(())
}
