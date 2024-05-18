pub trait ToTicks {
    fn to_ticks(&self) -> u64;
}

pub trait TimeSystem: ToTicks {
    fn day_start(&self) -> Self;
    fn week_start(&self) -> Self;
    fn month_start(&self) -> Self;
}
