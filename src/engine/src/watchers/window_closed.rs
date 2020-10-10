use crate::errors::*;
use crate::os::prelude::*;
use crate::processor::*;
use anyhow::Context;

#[derive(Debug)]
pub struct WindowClosed {
    _hook: hook::WinEventHook,
}

impl WindowClosed {
    pub fn watch(processor: Processor, window: Window) -> Result<Self> {
        let (pid, tid) = window.pid_tid()?;
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
        .with_context(|| format!("Unable to set window closed hook for {:?}", window))?;

        Ok(WindowClosed { _hook })
    }
}
