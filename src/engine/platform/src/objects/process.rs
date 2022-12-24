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
