use utils::errors::*;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, UNICODE_STRING, WAIT_TIMEOUT},
    System::Threading::{
        IsImmersiveProcess, NtQueryInformationProcess, OpenProcess, WaitForSingleObject,
        PROCESSINFOCLASS, PROCESS_QUERY_LIMITED_INFORMATION,
    },
};

use crate::{buffers::WideBuffer, errors::NtError, repeat_twice, win32};

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
// ref: https://github.com/winsiderss/systeminformer/blob/d243fcb2f287eca7c01970a332bac9cf4dcb478f/phnt/include/ntpsapi.h#L155
#[allow(non_upper_case_globals)]
const ProcessImageFileNameWin32: PROCESSINFOCLASS = PROCESSINFOCLASS(43);

const APPLICATION_FRAME_HOST: &str = r"C:\Windows\System32\ApplicationFrameHost.exe";

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

    /// Check whether this process is still running
    pub fn exists(&self) -> bool {
        // based on https://stackoverflow.com/a/1238410/8151052
        unsafe { WaitForSingleObject(self.handle, 0) == WAIT_TIMEOUT }
    }

    pub fn is_uwp(&self, path: Option<&str>) -> Result<bool> {
        Ok(unsafe { IsImmersiveProcess(self.handle).as_bool() }
            && if let Some(path) = path {
                path.eq_ignore_ascii_case(APPLICATION_FRAME_HOST)
            } else {
                self.path()
                    .context("get process path for is_uwp")?
                    .eq_ignore_ascii_case(APPLICATION_FRAME_HOST)
            })
    }

    pub fn cmd_line(&self) -> Result<String> {
        self.query_information_string(ProcessCommandLine)
            .context("get process command line")
    }

    pub fn path(&self) -> Result<String> {
        self.query_information_string(ProcessImageFileNameWin32)
            .context("get process image file name")
    }

    #[inline(always)]
    fn query_information_string(&self, cls: PROCESSINFOCLASS) -> Result<String> {
        self.query_information(
            cls,
            |us: &mut UNICODE_STRING| us.to_string_lossy(),
            u16::MAX as u32 + std::mem::size_of::<UNICODE_STRING>() as u32,
        )
    }

    #[inline(always)]
    fn query_information<T, R, F: Fn(&mut T) -> R>(
        &self,
        cls: PROCESSINFOCLASS,
        cb: F,
        max_size: u32,
    ) -> Result<R> {
        Ok(repeat_twice!(len, buf -> unsafe {
            // TODO check when the windows-rs library gives just the NTSTATUS
            match NtQueryInformationProcess(self.handle, cls, buf, len, &mut len) {
                Err(err) => Err(NtError::from(err)),
                Ok(()) => {
                    // unwrap will always succeed as otherwise NtQueryInformationProcess would have failed
                    // and we would have exited.
                    let us = buf.cast::<T>().as_mut().unwrap();
                    Ok(cb(us))
                }
            }
        }, max_size)?)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        win32!(non_zero: unsafe {CloseHandle(self.handle) }).expect("close process handle");
    }
}
