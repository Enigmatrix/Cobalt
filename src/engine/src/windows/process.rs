pub struct Process {
    id: i32
}

impl Process {
    #[inline]
    pub fn from(id: i32) -> Process {
        Process { id }
    }
}