/// Trait for converting a time to ticks.
pub trait ToTicks {
    /// Convert the time to ticks.
    fn to_ticks(&self) -> i64;
}

/// Some time system that can be used to get the start of the day, week, month, etc.
pub trait TimeSystem {
    /// Ticks output type.
    type Ticks: ToTicks;
    /// Get the start of the day
    fn day_start(&self, local_tz: bool) -> Self::Ticks;
    /// Get the start of the week. The week starts on Sunday.
    fn week_start(&self, local_tz: bool) -> Self::Ticks;
    /// Get the start of the month
    fn month_start(&self, local_tz: bool) -> Self::Ticks;
}
