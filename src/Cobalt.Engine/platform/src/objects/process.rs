use std::mem;

use util::error::{Context, Result};
use windows::{
    Wdk::System::Threading::{
        NtQueryInformationProcess, ProcessImageFileNameWin32, PROCESSINFOCLASS,
    },
    Win32::{
        Foundation::{CloseHandle, HANDLE, UNICODE_STRING, WAIT_TIMEOUT},
        System::Threading::{
            IsImmersiveProcess, OpenProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION,
        },
    },
};

use crate::{adapt_size, buf::WideBuffer, error::NtError};

pub type ProcessId = u32;

const APPLICATION_FRAME_HOST: &str = r"C:\Windows\System32\ApplicationFrameHost.exe";

pub struct Process {
    handle: HANDLE,
}

impl Process {
    /// Create a new [Process]
    pub fn new(pid: ProcessId) -> Result<Self> {
        let access = PROCESS_QUERY_LIMITED_INFORMATION;
        let handle = unsafe { OpenProcess(access, false, pid)? };
        Ok(Self { handle })
    }

    /// Check whether this [Process] is still running
    pub fn exists(&self) -> bool {
        // based on https://stackoverflow.com/a/1238410/8151052
        unsafe { WaitForSingleObject(self.handle, 0) == WAIT_TIMEOUT }
    }

    /// Check whether this [Process] is UWP
    pub fn is_uwp(&self, path: Option<&str>) -> Result<bool> {
        Ok(unsafe { IsImmersiveProcess(self.handle).is_ok() }
            && if let Some(path) = path {
                path.eq_ignore_ascii_case(APPLICATION_FRAME_HOST)
            } else {
                self.path()
                    .context("get process path for is_uwp")?
                    .eq_ignore_ascii_case(APPLICATION_FRAME_HOST)
            })
    }

    /// Get the executable path of this [Process]
    pub fn path(&self) -> Result<String> {
        self.query_information_string(ProcessImageFileNameWin32)
            .context("get process image file name")
    }

    #[inline(always)]
    fn query_information_string(&self, cls: PROCESSINFOCLASS) -> Result<String> {
        //PROCESS_INFORMATION_CLASS
        self.query_information(
            cls,
            |us: &mut UNICODE_STRING| us.to_string_lossy(),
            u16::MAX as u32 + mem::size_of::<UNICODE_STRING>() as u32,
        )
    }

    #[inline(always)]
    fn query_information<T, R, F: Fn(&mut T) -> R>(
        &self,
        cls: PROCESSINFOCLASS,
        cb: F,
        max_size: u32,
    ) -> Result<R> {
        Ok(adapt_size!(len, buf -> unsafe {
            NtError::from(NtQueryInformationProcess(self.handle, cls, buf, len, &mut len)).to_result().map(|_| {
                // unwrap will always succeed as otherwise NtQueryInformationProcess would have failed
                // and we would have exited.
                let us = buf.cast::<T>().as_mut().unwrap();
                cb(us)
            })
        }, max_size)?)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) }.expect("close process handle");
    }
}
