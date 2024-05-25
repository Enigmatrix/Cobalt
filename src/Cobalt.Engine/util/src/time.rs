/// Convert the time to ticks.
pub trait ToTicks {
    fn to_ticks(&self) -> u64;
}

/// Some time system that can be used to get the start of the day, week, month, etc.
pub trait TimeSystem {
    type Ticks: ToTicks;
    /// Get the start of the day
    fn day_start(&self) -> Self::Ticks;
    /// Get the start of the week. The week starts on Sunday.
    fn week_start(&self) -> Self::Ticks;
    /// Get the start of the month
    fn month_start(&self) -> Self::Ticks;
}
