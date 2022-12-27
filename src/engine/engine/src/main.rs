use data::db::DatabaseHolder;
use platform::events::{Event, TotalWatcher};
use platform::objects::{AppInfo, PidTid, Process};
use utils::tracing::info;
use utils::{channels, errors::*};

static RWVTABLE: std::task::RawWakerVTable =
    std::task::RawWakerVTable::new(|_| make_raw_waker(), |_| {}, |_| {}, |_| {});
fn make_raw_waker() -> std::task::RawWaker {
    std::task::RawWaker::new(&(), &RWVTABLE)
}

fn block_on<T, F: std::future::Future<Output = T>>(fut: F) -> T {
    let mut boxed_fut = Box::pin(fut);
    let waker = unsafe { std::task::Waker::from_raw(make_raw_waker()) };
    let mut ctx = std::task::Context::from_waker(&waker);

    loop {
        if let std::task::Poll::Ready(x) = boxed_fut.as_mut().poll(&mut ctx) {
            break x;
        }
    }
}

fn main() -> Result<()> {
    utils::setup().context("setup utils")?;
    platform::setup().context("setup platform")?;

    let mut db_holder = DatabaseHolder::new(":memory:").context("create db holder")?;
    let db = db_holder.database().context("get db")?;

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
            let info = if process.is_uwp(Some(&path))? {
                block_on(AppInfo::from_uwp(&window.aumid()?))
            } else {
                block_on(AppInfo::from_win32(&path))
            }?;
            info!(cmd_line = ?process.cmd_line()?, path, is_uwp = process.is_uwp(Some(&path))?, info = ?info);
        }
    }

    info!("🛑 engine exiting");
    Ok(())
}
