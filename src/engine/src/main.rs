use anyhow::*;
use native::watchers::*;
use native::wrappers::*;

fn main() -> Result<()> {
    native::setup();

    let mut event_loop = EventLoop::new();

    idle::Watcher::begin()?;
    let fg = foreground::Watcher::new(|window, timestamp| {
        println!(
            "switch at {} ({}) to {}",
            timestamp,
            idle::Watcher::last_interaction(),
            window.title()?
        );
        Ok(())
    })?;

    event_loop.run();
    Ok(())
}
