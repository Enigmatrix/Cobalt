use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::OnceLock;

use util::error::{bail, Context, Result};
use util::tracing::ResultTraceExt;
use windows::core::HSTRING;
use windows::System::AppDiagnosticInfo;
use windows::Wdk::System::Threading::{
    NtQueryInformationProcess, ProcessImageFileNameWin32, PROCESSINFOCLASS,
};
use windows::Win32::Foundation::{CloseHandle, HANDLE, UNICODE_STRING, WAIT_TIMEOUT};
use windows::Win32::System::ProcessStatus::K32EnumProcesses;
use windows::Win32::System::Threading::{
    IsImmersiveProcess, OpenProcess, TerminateProcess, WaitForSingleObject,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};
use windows::Win32::UI::Shell::DoEnvironmentSubstW;

use crate::adapt_size;
use crate::buf::WideBuffer;
use crate::error::IntoResult;

/// Identifier of a [Process]. May be recycled.
pub type ProcessId = u32;
/// Identifier of a Thread. May be recycled.
pub type ThreadId = u32;

/// [ProcessId] and [ThreadId] that created the [super::Window]. May be recycled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessThreadId {
    /// [Process] that created the [Window].
    pub pid: ProcessId,
    /// [Thread] that created the [Window].
    pub tid: ThreadId,
}

const APPLICATION_FRAME_HOST: &str = r"C:\Windows\System32\ApplicationFrameHost.exe";

/// Representation of a [Process] on the system, namely its handle
pub struct Process {
    handle: HANDLE,
}

// Needed because HANDLE is not Send/Sync - but it really should be
// since it's just a number
unsafe impl Send for Process {}
unsafe impl Sync for Process {}

static BLACKLIST: OnceLock<Vec<OsString>> = OnceLock::new();

/// Check if a path is in the blacklist
fn is_path_in_blacklist(path: impl Into<PathBuf>) -> bool {
    let path = path.into();
    BLACKLIST
        .get_or_init(kill_blacklist)
        .iter()
        .any(|s| s.eq_ignore_ascii_case(path.as_os_str()))
}

/// Get the blacklist for killing processes
fn kill_blacklist() -> Vec<OsString> {
    let blacklist_str = include_str!("../data/kill_blacklist.txt");
    let blacklist = blacklist_str.lines().map(expand_env_str).collect();
    blacklist
}

/// Expand environment variables in a string
fn expand_env_str(s: &str) -> OsString {
    let mut len = 128 + s.len() + 1;
    let wide = s.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let buf = loop {
        let mut buf = vec![0u16; len];
        buf[..wide.len()].copy_from_slice(&wide);

        let res = unsafe { DoEnvironmentSubstW(&mut buf) };
        len = (res & 0xffff) as usize;
        let res = res >> 16;
        if res == 1 {
            unsafe { buf.set_len(len - 1) }; // ignore the null byte
            break buf;
        }
    };
    OsString::from_wide(&buf)
}

impl Process {
    /// Create a new [Process]
    pub fn new(pid: ProcessId) -> Result<Self> {
        let access = PROCESS_QUERY_LIMITED_INFORMATION;
        let handle = unsafe { OpenProcess(access, false, pid)? };
        Ok(Self { handle })
    }

    /// Create a new [Process] with access rights
    pub fn new_killable(pid: ProcessId) -> Result<Self> {
        let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE;
        let handle = unsafe { OpenProcess(access, false, pid)? };
        Ok(Self { handle })
    }

    /// Kill a UWP app with the given AUMID
    pub async fn kill_uwp(aumid: &str) -> Result<()> {
        let infos = AppDiagnosticInfo::RequestInfoForAppUserModelId(&HSTRING::from(aumid))?.await?;
        let infos = infos.into_iter().collect::<Vec<_>>();
        for info in infos {
            let groups = info.GetResourceGroups()?;
            let groups = groups.into_iter().collect::<Vec<_>>();
            for group in groups {
                group.StartTerminateAsync()?.await?;
            }
        }
        Ok(())
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

    /// Kill the [Process]
    pub fn kill(&self, path: Option<&str>) -> Result<()> {
        let path = if let Some(path) = path {
            path
        } else {
            &self.path().context("get process path for kill")?
        };
        if is_path_in_blacklist(path) {
            bail!("path in blacklist, cannot kill: {path}")
        }
        unsafe { TerminateProcess(self.handle, 0)? };
        Ok(())
    }

    /// Get specific information about a process ([`PROCESSINFOCLASS`]) as a [`String`]
    #[inline(always)]
    fn query_information_string(&self, cls: PROCESSINFOCLASS) -> Result<String> {
        self.query_information(
            cls,
            |us: &mut UNICODE_STRING| us.to_string_lossy(),
            u16::MAX as u32 + mem::size_of::<UNICODE_STRING>() as u32,
        )
    }

    /// Get specific information about a process ([`PROCESSINFOCLASS`]) using a buffer
    /// that is sized accordingly, and converts it to a specific type using a callback.
    #[inline(always)]
    fn query_information<T, R, F: Fn(&mut T) -> R>(
        &self,
        cls: PROCESSINFOCLASS,
        cb: F,
        max_size: u32,
    ) -> Result<R> {
        Ok(adapt_size!(len, buf -> unsafe {
            NtQueryInformationProcess(self.handle, cls, buf, len, &mut len).into_result().map(|_| {
                // unwrap will always succeed as otherwise NtQueryInformationProcess would have failed
                // and we would have exited.
                let us = buf.cast::<T>().as_mut().unwrap();
                cb(us)
            })
        }, max_size)?)
    }

    /// List all running processes on the system
    pub fn get_all() -> Result<Vec<ProcessId>> {
        let mut pids = vec![0u32; 1024];
        let mut bytes_returned = 0u32;

        loop {
            let buffer_size = (pids.len() * mem::size_of::<u32>()) as u32;
            unsafe {
                K32EnumProcesses(pids.as_mut_ptr(), buffer_size, &mut bytes_returned).ok()?;
            }

            // If bytes_returned equals buffer_size, the buffer might be too small
            if bytes_returned != buffer_size {
                break;
            }

            // Double the buffer size and try again
            pids.resize(pids.len() * 2, 0);
        }

        let num_processes = bytes_returned as usize / mem::size_of::<u32>();
        pids.truncate(num_processes);
        pids.retain(|pid| *pid != 0);
        Ok(pids)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) }
            .context("close process handle")
            .warn();
    }
}
#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn kill_notepad() -> Result<()> {
        let mut notepad = Command::new("notepad.exe").spawn()?;
        let pid = notepad.id();
        let proc = Process::new_killable(pid)?;
        proc.kill(None)?;

        notepad.kill()?;
        Ok(())
    }

    #[test]
    fn dont_kill_explorer() -> Result<()> {
        let mut explorer = Command::new("explorer.exe").spawn()?;
        let pid = explorer.id();
        let proc = Process::new_killable(pid)?;
        assert!(proc.kill(None).is_err());

        explorer.kill()?;
        Ok(())
    }
}
