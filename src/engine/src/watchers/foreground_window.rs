use crate::errors::*;
use crate::os::prelude::*;
use crate::processor::*;
use tracing::*;

pub struct ForegroundWindowSwitches {
    _hook: hook::WinEventHook,
}

#[derive(Clone, Debug)]
pub struct WindowSwitch {
    pub time: Timestamp,
    pub window: Window,
}

impl ForegroundWindowSwitches {
    pub fn watch(processor: Processor) -> Result<Self> {
        let _hook = hook::WinEventHook::new(
            hook::Range::Single(hook::Event::SystemForeground),
            hook::Locality::Global,
            Box::new(move |args| {
                let time = Timestamp::from_ticks(args.dwms_event_time); // get time first!
                let window = Window::new(args.hwnd)?;

                if args.id_object != winuser::OBJID_WINDOW
                    || unsafe { winuser::IsWindow(args.hwnd) == 0 }
                    || {
                        let cls = window.class_name()?;
                        cls == "ForegroundStaging" ||
                        cls == "LauncherTipWnd" ||
                        cls == "MultitaskingViewFrame" ||
                        cls == "ApplicationManager_DesktopShellWindow"
                    }
                {
                    return Ok(()); // normal response
                }
                let switch = WindowSwitch { time, window };

                processor.process(Message::Switch(switch))?;

                let uwp = window.is_uwp().and_then(|is_uwp| {
                    Ok(if is_uwp {
                        format!("UWP ({})", window.aumid()?)
                    } else {
                        "Win32".to_owned()
                    })
                });
                let title = window
                    .title()
                    .unwrap_or_else(|e| format!("Unable to get title for {:?}: {}", window, e));

                trace!("[{}] switch to ({:?}), title: {:?}", time, uwp, title);

                Ok(())
            }),
        )?;
        Ok(ForegroundWindowSwitches { _hook })
    }
}
