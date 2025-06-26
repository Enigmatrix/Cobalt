//! Monitor tab selection changes using UI Automation

use platform::objects::{EventLoop, Target, WinEvent, WinEventHook};
use tools::filters::{match_running_processes, ProcessFilter};
use util::error::Result;
use util::tracing::info;
use util::{config, future as tokio, Target as UtilTarget};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{AccessibleObjectFromEvent, IAccessible, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_VALUECHANGE;

#[tokio::main]
async fn main() -> Result<()> {
    util::set_target(UtilTarget::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;
    info!("Starting Chrome omnibox monitor...");

    let processes = match_running_processes(&ProcessFilter {
        name: Some("Google Chrome".to_string()),
        pid: None,
    })?;
    let process = processes[0].clone();

    let handler = move |_hwineventhook: HWINEVENTHOOK,
                        _event: u32,
                        hwnd: HWND,
                        id_object: i32,
                        id_child: i32,
                        _id_event_thread: u32,
                        _dwms_event_time: u32| {
        let mut acc: Option<IAccessible> = Default::default();
        let mut child: VARIANT = Default::default();
        let res = unsafe {
            AccessibleObjectFromEvent(
                hwnd,
                id_object as u32,
                id_child as u32,
                &mut acc,
                &mut child,
            )
        };
        if let Err(e) = res {
            // error!("AccessibleObjectFromEvent: {:?}", e);
        }
        if let Some(acc) = acc {
            let name = unsafe { acc.get_accName(&child) }
                .ok()
                .map(|r| r.to_string());
            let description = unsafe { acc.get_accDescription(&child) }
                .ok()
                .map(|r| r.to_string());
            let value = unsafe { acc.get_accValue(&child) }
                .ok()
                .map(|r| r.to_string());
            let role = unsafe { acc.get_accRole(&child) }
                .ok()
                .map(|r| r.to_string());
            let state = unsafe { acc.get_accState(&child) }
                .ok()
                .map(|r| r.to_string());

            if state == Some("1048576".to_string()) {
                println!("navigated to: {:?}", value);
            }
        }
    };

    let _hook = WinEventHook::new(
        WinEvent::Event(EVENT_OBJECT_VALUECHANGE),
        Target::Id(process.pid),
        Target::All,
        Box::new(handler),
    )?;

    EventLoop::new().run();

    Ok(())
}
