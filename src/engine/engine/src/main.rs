use platform::events::{Event, TotalWatcher};
use platform::objects::{PidTid, Process};
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
        info!(?event);

        if let Event::ForegroundSwitch { window, .. } = event {
            let PidTid { pid, .. } = window.pid_tid()?;
            let process = Process::new(pid)?;
            let path = process.path()?;
            info!(cmd_line = ?process.cmd_line()?, path, is_uwp = process.is_uwp(Some(&path))?);
        }
    }

    info!("🛑 engine exiting");
    Ok(())
}
