use crate::errors::*;
use crate::os::prelude::*;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct ProcessExit {
    wait: *mut c_void,
}

impl ProcessExit {
    pub fn watch(process: &Process, sender: mpsc::UnboundedSender<ProcessId>) -> Result<Self> {
        let pid = process.pid()?;
        let mut ret = ProcessExit { wait: ptr::null_mut() };
        let dat = Box::into_raw(Box::new((sender, pid)));
        win32!(non_zero: { winbase::RegisterWaitForSingleObject(
            &mut ret.wait,
            process.handle(),
            Some(ProcessExit::handler),
            dat.cast::<c_void>(),
            winbase::INFINITE,
            winnt::WT_EXECUTEONLYONCE,
        )})?;
        Ok(ret)
    }

    unsafe extern "system" fn handler(dat: *mut c_void, _: u8) {
        let ptr = dat.cast::<(mpsc::UnboundedSender<ProcessId>, u32)>();
        let (sender, pid) = ptr.as_mut().unwrap();
        sender.send(*pid).unwrap();
        Box::from_raw(ptr); // get the box from the ptr, then it will be dropped
    }
}

impl Drop for ProcessExit {
    fn drop(&mut self) {
        unsafe { winbase::UnregisterWait(self.wait) };
    }
}