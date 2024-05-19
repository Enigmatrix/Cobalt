pub trait ToTicks {
    fn to_ticks(&self) -> u64;
}

pub trait TimeSystem {
    type Ticks: ToTicks;
    fn day_start(&self) -> Self::Ticks;
    fn week_start(&self) -> Self::Ticks;
    fn month_start(&self) -> Self::Ticks;
}
