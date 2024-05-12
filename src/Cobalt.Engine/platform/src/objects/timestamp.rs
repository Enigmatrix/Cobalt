use std::ops;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        todo!()
    }

    pub fn ticks(&self) -> u64 {
        self.0
    }
}

impl ops::Sub for &Timestamp {
    type Output = Duration;

    fn sub(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}
