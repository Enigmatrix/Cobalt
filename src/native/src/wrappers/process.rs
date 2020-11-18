use crate::buffer::{self, Buffer};
use crate::error::*;
use crate::raw::*;
use anyhow::*;
use std::default::default;
use std::hash;
use std::mem;
use std::ptr;

pub type ProcessId = u32;

#[derive(Debug, Clone)]
pub struct Process(HANDLE);
unsafe impl Sync for Process {}
unsafe impl Send for Process {}

impl hash::Hash for Process {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.pid().unwrap());
    }
}

impl PartialEq<Process> for Process {
    fn eq(&self, other: &Process) -> bool {
        self.pid().unwrap() == other.pid().unwrap() // check other ways to compare handles
    }
}
impl Eq for Process {}

pub struct ProcessOptions {
    pub readable: bool,
    pub query_information: bool,
    pub sync: bool,
    pub writable: bool,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self {
            readable: true,
            query_information: true,
            sync: true,
            writable: false,
        }
    }
}

impl Process {
    pub fn new(pid: ProcessId, opts: ProcessOptions) -> Result<Self, Win32Err> {
        let handle = win32!(non_null: {
            processthreadsapi::OpenProcess(
                if opts.readable { winnt::PROCESS_VM_READ } else {0} |
                if opts.readable { winnt::PROCESS_QUERY_INFORMATION } else {0} |
                if opts.sync     { winnt::SYNCHRONIZE } else { 0 } |
                if opts.writable { winnt::PROCESS_VM_WRITE | winnt::PROCESS_VM_OPERATION } else { 0 },
                0,
                pid,
            )
        })?;
        Ok(Process(handle))
    }

    pub fn current() -> Result<Process, Win32Err> {
        Ok(Process(win32!(
            non_null: processthreadsapi::GetCurrentProcess()
        )?))
    }

    pub fn handle(&self) -> HANDLE {
        self.0
    }

    pub fn pid(&self) -> Result<ProcessId, Win32Err> {
        win32!(non_zero: {
            processthreadsapi::GetProcessId(self.0)
        })
    }

    pub fn read_process_memory<T: Default>(&self, addr: *mut T) -> Result<T, Win32Err> {
        let mut ret: T = default();
        win32!(non_zero: {
            memoryapi::ReadProcessMemory(
                self.0,
                addr.cast::<c_void>(),
                &mut ret as *mut _ as *mut c_void,
                mem::size_of::<T>(),
                ptr::null_mut(),
            )
        })?;
        Ok(ret)
    }

    pub fn read_string_from_process_memory(
        &self,
        s: ntdef::UNICODE_STRING,
    ) -> Result<String, Win32Err> {
        let mut buf_len = (s.Length / 2) as usize;
        let mut buf = buffer::alloc(buf_len);
        win32!(non_zero: {
            memoryapi::ReadProcessMemory(
                self.0,
                s.Buffer.cast::<c_void>(),
                buf.as_mut_ptr().cast::<c_void>(),
                buf_len * 2,
                &mut buf_len,
            )
        })?;
        Ok(buf.with_length(buf_len / 2).to_string_lossy())
    }

    pub fn path(&self) -> Result<String, Win32Err> {
        let mut buf_len: u32 = 1024;
        let mut buf = buffer::alloc(buf_len as usize);
        while 0
            == unsafe {
                winbase::QueryFullProcessImageNameW(self.0, 0, buf.as_mut_ptr(), &mut buf_len)
            }
        {
            buf_len *= 2;
            buf = buffer::alloc(buf_len as usize);
        }
        Ok(buf.with_length((buf_len) as usize).to_string_lossy())
    }

    pub fn cmd(&self) -> Result<String> {
        let mut info = ntpsapi::PROCESS_BASIC_INFORMATION::default();
        let mut info_len = 0u32;
        ntstatus!(ntpsapi::NtQueryInformationProcess(
            self.handle(),
            0,
            &mut info as *mut _ as *mut c_void,
            std::mem::size_of::<ntpsapi::PROCESS_BASIC_INFORMATION>() as u32,
            &mut info_len as &mut u32,
        ))
        .with_context(|| "Get process information")?;

        let peb = self.read_process_memory(info.PebBaseAddress)?;
        let params = self.read_process_memory(peb.ProcessParameters)?;
        Ok(self.read_string_from_process_memory(params.CommandLine)?)
    }
}
