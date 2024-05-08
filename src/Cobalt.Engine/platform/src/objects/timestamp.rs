use std::ops;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp;

impl Timestamp {
    pub fn now() -> Self {
        todo!()
    }
}

impl ops::Sub for &Timestamp {
    type Output = Duration;

    fn sub(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}
