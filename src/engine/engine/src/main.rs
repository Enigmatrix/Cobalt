mod processor;

use data::db::DatabaseHolder;
use platform::events::{Event as PlatformEvent, TotalWatcher};
use platform::objects::{AppInfo, PidTid, Process, Timestamp};
use utils::channels::{self, select};
use utils::errors::*;
use utils::tracing::info;

use crate::processor::{Processor, Event};

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

    let start = Timestamp::now();

    let mut db_holder = DatabaseHolder::new(":memory:").context("create db holder")?;
    let db = db_holder.database().context("get db")?;

    info!("🚀 engine started");

    let (events_tx, events_rx) = channels::unbounded();

    let (app_info_tx, app_info_rx) = channels::unbounded();
    let (app_info_res_tx, app_info_res_rx) = channels::unbounded();

    let mut processor = Processor::new(db, app_info_tx, start);

    let _ = std::thread::spawn(move || {
        TotalWatcher::new(events_tx, start)
            .expect("setup total watcher")
            .run();
    });

    select::Selector::new()
    .recv(&events_rx, |pev| processor.process(Event::Platform(pev.context("recv platform event")?)))
    .recv(&app_info_res_rx, |app| processor.process(Event::AppInfoUpdate(app.context("recv app info update context")?)))
    .wait();

    // for event in events_rx {
    //     info!(?event);

    //     if let Event::ForegroundSwitch { window, .. } = event {
    //         let PidTid { pid, .. } = window.pid_tid()?;
    //         let process = Process::new(pid)?;
    //         let path = process.path()?;
    //         let info = if process.is_uwp(Some(&path))? {
    //             block_on(AppInfo::from_uwp(&window.aumid()?))
    //         } else {
    //             block_on(AppInfo::from_win32(&path))
    //         }?;
    //         info!(cmd_line = ?process.cmd_line()?, path, is_uwp = process.is_uwp(Some(&path))?, info = ?info);
    //     }
    // }

    info!("🛑 engine exiting");
    Ok(())
}
