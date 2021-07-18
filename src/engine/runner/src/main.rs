use std::{io::{Read, stdin}, time::{self, Instant}};

use platform::{timer::Timer, window};
use util::*;

fn main() -> Result<()> {
    util::setup().wrap_err("setup utils")?;

    // TODO fix all these annoying imports in platform
    // TODO re-export window::Window, timer::Timer etc
    // TODO use this the error's Result type instead of normal one in platform

    log::info!("🚀 Starting engine");

    let mut prev = Instant::now();

    let _timer = Timer::new(0, 1000, &mut || {
        let now = Instant::now();
        let since = now.duration_since(prev);
        log::debug!("{:?}: {:?}", since, window::Window::foreground());
        prev = now;
    });

    let mut s = String::new();
    stdin().read_to_string(&mut s);
    Ok(())
}