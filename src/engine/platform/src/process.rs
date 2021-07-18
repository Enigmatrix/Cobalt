use std::{ffi::OsString, hash};

use bindings::Windows::Win32::{Foundation::{CloseHandle, HANDLE}, System::Threading::*};

use crate::{buffer::{self, Buffer}, error::Win32Err, win32};

pub type ProcessId = u32;

#[derive(Debug)]
pub struct Process {
    pub pid: ProcessId,
    pub handle: HANDLE
}

impl hash::Hash for Process {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.pid);
    }
}

impl PartialEq<Process> for Process {
    fn eq(&self, other: &Process) -> bool {
        self.pid == other.pid
    }
}
impl Eq for Process {}

pub struct ProcessOptions {
    pub readable: bool,
    pub query_information: bool,
    pub sync: bool,
    pub writable: bool,
    pub terminate: bool,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self {
            readable: true,
            query_information: true,
            sync: true,
            writable: false,
            terminate: false,
        }
    }
}

impl From<ProcessOptions> for PROCESS_ACCESS_RIGHTS {
    fn from(opts: ProcessOptions) -> Self {
        let mut access = Self::default();

        if opts.readable            { access |= PROCESS_VM_READ }
        if opts.query_information   { access |= PROCESS_QUERY_INFORMATION }
        if opts.sync                { access |= PROCESS_SYNCHRONIZE }
        if opts.writable            { access |= PROCESS_VM_WRITE | PROCESS_VM_OPERATION }
        if opts.terminate           { access |= PROCESS_TERMINATE }

        access
    }
}

impl Process {
    pub fn from_pid(pid: ProcessId, opts: ProcessOptions) -> Result<Self, Win32Err> {
        let handle = win32!(non_null: {
            // we choose not to inherit handle
            OpenProcess(opts.into(), false, pid)
        })?;
        Ok(Process {pid, handle})
    }

    pub fn current() -> Result<Process, Win32Err> {
        let handle = win32!(
            non_null: GetCurrentProcess()
        )?;

        let pid = win32!(non_zero: {
            GetProcessId(handle)
        })?;

        Ok(Process {pid, handle})
    }

    pub fn terminate(&self) -> Result<(), Win32Err> {
        win32!(non_zero: inner {
            TerminateProcess(self.handle, 0)
        })?;
        Ok(())
    }

    pub fn path(&self) -> Result<OsString, Win32Err> {
        let mut buf_len: u32 = 1024;
        let mut buf = buffer::alloc(buf_len as usize);
        while unsafe {
                !QueryFullProcessImageNameW(self.handle, PROCESS_NAME_WIN32, buf.as_pwstr(), &mut buf_len).as_bool()
            }
        {
            buf_len *= 2;
            buf = buffer::alloc(buf_len as usize);
        }
        Ok(buf.with_length((buf_len) as usize).to_os_string())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        win32!(non_zero: inner CloseHandle(self.handle)).expect("Process handle cannot be closed");
    }
}