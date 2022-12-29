mod app_info_extractor;
mod processor;

use data::db::DatabaseHolder;
use platform::events::TotalWatcher;
use platform::objects::Timestamp;
use utils::channels;
use utils::errors::*;
use utils::tracing::info;

use crate::app_info_extractor::AppInfoExtractor;
use crate::processor::{Event, Processor};

fn main() -> Result<()> {
    utils::setup().context("setup utils")?;
    platform::setup().context("setup platform")?;

    let start = Timestamp::now();
    let conn_str = "./f.db"; // TODO get from config

    let mut db_holder = DatabaseHolder::new(conn_str).context("create db holder")?;
    let db = db_holder.database().context("get db")?;

    info!("🚀 engine started");

    let (events_tx, events_rx) = channels::unbounded();
    let (app_info_tx, app_info_rx) = channels::unbounded();

    let mut processor = Processor::new(db, app_info_tx, start);

    let tx = events_tx.clone();
    let _ = std::thread::spawn(move || {
        let cb = Box::new(move |pev| tx.send(Event::Platform(pev)).context("send platform event"));
        TotalWatcher::new(cb, start)
            .expect("setup total watcher")
            .run();
    });

    let tx = events_tx;
    let _ = std::thread::spawn(move || {
        let mut db_holder = DatabaseHolder::new(conn_str)
            .context("create db holder")
            .unwrap();
        let db = db_holder.database().context("get db").unwrap();

        let app_info_extractor = AppInfoExtractor::new(
            db,
            app_info_rx,
            Box::new(move |app| {
                tx.send(Event::AppInfoUpdate(app))
                    .context("send app info update")
            }),
        );

        app_info_extractor.run();
    });

    for event in events_rx {
        info!(?event);
        processor.process(event).context("process event")?
    }

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
