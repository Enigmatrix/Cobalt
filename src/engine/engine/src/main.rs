use platform::events::TotalWatcher;
use utils::tracing::info;
use utils::{channels, errors::*};

fn main() -> Result<()> {
    utils::setup().context("setup utils")?;
    platform::setup().context("setup platform")?;

    info!("🚀 engine started");

    let (sender, reciever) = channels::unbounded();

    let _ = std::thread::spawn(move || {
        TotalWatcher::new(sender)
            .expect("setup total watcher")
            .run();
    });

    for event in reciever {
        info!("ev = {:?}", event);
    }

    info!("🛑 engine exiting");
    Ok(())
}
