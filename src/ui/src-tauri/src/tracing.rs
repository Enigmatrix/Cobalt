use std::str::FromStr;

use util::tracing::*;

macro_rules! dyn_event {
    ($lvl:ident, $($arg:tt)+) => {
        match $lvl {
            Level::TRACE => trace!($($arg)+),
            Level::DEBUG => debug!($($arg)+),
            Level::INFO => info!($($arg)+),
            Level::WARN => warn!($($arg)+),
            Level::ERROR => error!($($arg)+),
        }
    };
}

#[tauri::command]
pub fn log(
    level: String,
    message: String,
    file: String,
    line: u32,
    col: u32,
    args: Option<serde_json::Value>,
) {
    let level = Level::from_str(&level).unwrap_or(Level::INFO);
    dyn_event!(level, target: "ui", "{}:{}:{} {} {}", file, line, col, message, args.map(|v| v.to_string()).unwrap_or("".to_string()));
}
