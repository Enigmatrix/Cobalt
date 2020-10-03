use crate::errors::*;
use crate::os::prelude::*;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Process(HANDLE);

pub struct ProcessOptions {
    pub readable: bool,
    pub query_information: bool,
    pub sync: bool,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self {
            readable: true,
            query_information: true,
            sync: false,
        }
    }
}

impl Process {
    pub fn new(pid: u32, opts: ProcessOptions) -> Result<Self> {
        let handle = expect! (non_null: {
            processthreadsapi::OpenProcess(
                if opts.readable { winnt::PROCESS_VM_READ } else {0} |
                if opts.readable { winnt::PROCESS_QUERY_INFORMATION } else {0} |
                if opts.sync { winnt::SYNCHRONIZE } else { 0 },
                0,
                pid,
            )
        })?;
        Ok(Process(handle))
    }

    pub fn handle(&self) -> HANDLE {
        self.0
    }

    pub fn read_process_memory<T: Default>(&self, addr: *mut T) -> Result<T> {
        let mut ret: T = default();
        expect!(true: {
            memoryapi::ReadProcessMemory(
                self.0,
                addr as *mut _ as *mut c_void,
                &mut ret as *mut _ as *mut c_void,
                mem::size_of::<T>(),
                ptr::null_mut(),
            )
        })?;
        Ok(ret)
    }

    pub fn read_string_from_process_memory(&self, s: ntdef::UNICODE_STRING) -> Result<String> {
        let mut buf_len = (s.Length / 2) as usize;
        let mut buf = string_buffer!(buf_len);
        expect!(true: {
            memoryapi::ReadProcessMemory(
                self.0,
                s.Buffer as *mut _ as *mut winapi::ctypes::c_void,
                buf.as_mut_ptr() as *mut _ as *mut winapi::ctypes::c_void,
                buf_len * 2,
                &mut buf_len,
            )
        })?;
        Ok(string_from_buffer!(buf, buf_len))
    }

    pub fn path_fast(&self) -> Result<String> {
        let mut len = 1024u32; // TODO macro this pattern
        let buf = loop {
            let mut buf = string_buffer!(len);
            let res = unsafe {
                winbase::QueryFullProcessImageNameW(self.0, 0, buf.as_mut_ptr(), &mut len)
            };
            if res != 0 {
                break string_from_buffer!(buf, len);
            } // TODO check if there is an error other than insufficient buffer!
            len *= 2;
        };
        Ok(buf)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        expect!(true: handleapi::CloseHandle(self.0)).unwrap();
    }
}
