/// Trait for converting a time to ticks.
pub trait ToTicks {
    /// Convert the time to ticks.
    fn to_ticks(&self) -> i64;
}

/// Some time system that can be used to get the start of the day, week, month, etc.
pub trait TimeSystem {
    /// Ticks output type.
    type Ticks: ToTicks;
    /// Get the time now
    fn now(&self) -> Self::Ticks;
    /// Get the start of the day
    fn day_start(&self, local_tz: bool) -> Self::Ticks;
    /// Get the start of the week. The week starts on Sunday.
    fn week_start(&self, local_tz: bool) -> Self::Ticks;
    /// Get the start of the month
    fn month_start(&self, local_tz: bool) -> Self::Ticks;
}

const UNITS: [(&str, u128); 6] = [
    ("d", 86400000000),
    ("h", 3600000000),
    ("m", 60000000),
    ("s", 1000000),
    ("ms", 1000),
    ("μs", 1),
];

/// Format a duration as a human readable string.
pub fn human_duration(d: std::time::Duration) -> String {
    let mut map: Vec<(u128, &str)> = Vec::new();
    let mut μs = d.as_micros();
    for (unit, n_μs) in UNITS {
        map.push((μs / n_μs, unit));
        μs %= n_μs;
    }

    match map
        .into_iter()
        .filter_map(|(n, u)| if n > 0 { Some(format!("{n}{u}")) } else { None })
        .take(2)
        .reduce(|acc, item| format!("{acc} {item}"))
    {
        Some(val) => val,
        None => "-".to_string(),
    }
}
