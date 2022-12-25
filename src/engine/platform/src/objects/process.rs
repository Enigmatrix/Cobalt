use utils::errors::*;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, UNICODE_STRING},
    System::Threading::{
        NtQueryInformationProcess, OpenProcess, PROCESSINFOCLASS, PROCESS_QUERY_LIMITED_INFORMATION,
    },
};

use crate::{buffers::Buffer, errors::NtError, repeat_twice, win32};

pub type ProcessId = u32;
pub type ThreadId = u32;

#[derive(PartialEq, Eq, Debug)]
pub struct PidTid {
    pub pid: ProcessId,
    pub tid: ThreadId,
}

impl PidTid {
    pub fn valid(&self) -> bool {
        self != &PidTid { pid: 0, tid: 0 }
    }
}

// ref: https://github.com/winsiderss/systeminformer/blob/d243fcb2f287eca7c01970a332bac9cf4dcb478f/phnt/include/ntpsapi.h#L172
#[allow(non_upper_case_globals)]
const ProcessCommandLine: PROCESSINFOCLASS = PROCESSINFOCLASS(60);

pub struct Process {
    handle: HANDLE,
}

impl Process {
    pub fn new(pid: ProcessId) -> Result<Self> {
        let access = PROCESS_QUERY_LIMITED_INFORMATION; // TODO configurable
                                                        // don't set the handle as inheritable
        let handle = unsafe { OpenProcess(access, false, pid).context("native open process")? };
        Ok(Self { handle })
    }

    pub fn cmd_line(&self) -> Result<String> {
        repeat_twice!(len, buf -> unsafe {
            // TODO check when the windows-rs library gives just the NTSTATUS
            match NtQueryInformationProcess(self.handle, ProcessCommandLine, buf, len, &mut len) {
                Err(err) => Err(NtError::from(err)),
                Ok(()) => {
                    // unwrap will always succeed as otherwise NtQueryInformationProcess would have failed
                    // and we would have exited.
                    let us = buf.cast::<UNICODE_STRING>().as_mut().unwrap();
                    Ok(us.to_string_lossy())
                }
            }
        }, 0x10000)
        .context("get native command line wrapper")
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        win32!(non_zero: unsafe {CloseHandle(self.handle) }).expect("close process handle");
    }
}
