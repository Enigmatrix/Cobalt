use crate::error::Win32Err;
use crate::raw::*;
use crate::wrappers::*;
use anyhow::*;
use std::ptr;

#[derive(Debug)]
pub struct Watcher {
    wait: *mut c_void,
}

impl Watcher {
    pub fn new<F: FnMut(ProcessId) -> Result<()>>(
        process: &Process,
        callback: F,
    ) -> Result<Watcher, Win32Err> {
        let pid = process.pid()?;
        let mut ret = Watcher {
            wait: ptr::null_mut(),
        };
        let dat = Box::into_raw(Box::new((pid, callback)));
        win32!(non_zero: { winbase::RegisterWaitForSingleObject(
            &mut ret.wait,
            process.handle(),
            Some(Watcher::handler::<F>),
            dat.cast::<c_void>(),
            winbase::INFINITE,
            winnt::WT_EXECUTEONLYONCE,
        )})?;
        Ok(ret)
    }

    unsafe extern "system" fn handler<F: FnMut(ProcessId) -> Result<()>>(dat: *mut c_void, _: u8) {
        let mut data = Box::from_raw(dat.cast::<(ProcessId, F)>());
        data.1(data.0)
            .with_context(|| "Error in process exit callback")
            .unwrap();
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        unsafe { winbase::UnregisterWait(self.wait) };
    }
}
