use bindings::Windows::Win32::{Foundation::HANDLE};

pub type ProcessId = u32;

pub struct Process {
    pub pid: ProcessId,
    pub handle: HANDLE
}

