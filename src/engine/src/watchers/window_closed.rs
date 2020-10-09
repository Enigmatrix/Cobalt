use crate::errors::*;
use crate::os::prelude::*;
use crate::processor::*;
use tracing::*;

#[derive(Debug)]
pub struct WindowClosedWatcher {
    _hook: hook::WinEventHook,
}

impl WindowClosedWatcher {
    pub fn new(processor: Processor, window: Window) -> Result<Self> {
        let (pid, tid) = match window.pid_tid() {
            Err(Error(ErrorKind::Win32(1400), _)) => {
                warn!("early return (pid/tid) inaccessible for {:?}", window);
                return Err(ErrorKind::WindowAlreadyClosed(window).into());
            }
            x => x,
        }
        .chain_err(|| format!("Unable to get pid/tid for {:?}", window))?;

        let _hook = hook::WinEventHook::new(
            hook::Range::Single(hook::Event::ObjectDestroyed),
            hook::Locality::ProcessThread { pid, tid },
            Box::new(move |args| {
                if window == args.hwnd {
                    processor.process(Message::WindowClosed(window))?;
                }
                Ok(())
            }),
        )
        .chain_err(|| format!("Unable to set window closed hook for {:?}", window))?;

        Ok(WindowClosedWatcher { _hook })
    }
}
