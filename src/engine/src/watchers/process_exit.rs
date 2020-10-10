use crate::os::prelude::*;
use crate::errors::*;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct ProcessExit {
    pid: ProcessId,
    wait: *mut c_void,
    sender: mpsc::UnboundedSender<ProcessId>
}

impl ProcessExit {
    pub fn watch(process: Process, sender: mpsc::UnboundedSender<ProcessId>) -> Result<Self> {
        let mut ret = ProcessExit {
            pid: process.pid()?,
            wait: ptr::null_mut(),
            sender
        };
        expect!(true: winbase::RegisterWaitForSingleObject(
            &mut ret.wait,
            process.handle(),
            Some(ProcessExit::handler),
            &mut ret as *mut _ as *mut c_void,
            winbase::INFINITE,
            winnt::WT_EXECUTEONLYONCE,
        ))?;
        Ok(ret)
    }

    fn signal(&mut self) {
        self.sender.send(self.pid).unwrap();
    }

    pub unsafe extern "system" fn handler(dat: *mut c_void, _: u8) {
        let inst = (dat as *mut ProcessExit).as_mut().unwrap();
        inst.signal();
    }
}