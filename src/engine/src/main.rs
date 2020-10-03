#![feature(trait_alias)]
#![feature(async_closure)]
#![feature(default_free_fn)]
#![feature(option_expect_none)]
#![feature(maybe_uninit_ref)]
#![recursion_limit = "1024"]

mod data;
mod errors;
mod os;
mod processor;
mod utils;
mod watchers;

use os::prelude::*;
use tokio::*;
use tracing::*;

fn init() -> Result<(runtime::Runtime, Span), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            // .with_thread_ids(true)
            // .compact()
            .finish(),
    )?;

    let runtime = runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .thread_name("cobalt-worker")
        .build()?;

    let span = info_span!("main");

    hook::init_contexts();

    Ok((runtime, span))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut runtime, span) = init()?;
    let _enter = span.enter();

    info!("Start");

    let events = hook::EventLoop::new();
    let processor = processor::Processor::new()?;
    let processor_pump = processor::Processor::clone(&processor);
    let _hook = hook::WinEventHook::new(
        hook::Range::Single(hook::Event::SystemForeground),
        hook::Locality::Global,
        Box::new(move |args| {
            let time = Timestamp::from_ticks(args.dwms_event_time); // get time first!

            if args.id_object != winuser::OBJID_WINDOW
                || unsafe { winuser::IsWindow(args.hwnd) == 0 }
            {
                return Ok(()); // normal response
            }

            let span = trace_span!("fg_chg");
            let _enter = span.enter();

            let window = Window::new(args.hwnd)?;

            processor_pump.process(processor::Message::Switch {
                switch: processor::WindowSwitch { time, window },
            })?;

            let uwp = window.is_uwp().and_then(|is_uwp| {
                Ok(if is_uwp {
                    format!("UWP ({})", window.aumid()?)
                } else {
                    "Win32".to_owned()
                })
            });
            let title = window
                .title()
                .unwrap_or_else(|e| format!("Unable to get title for {:?}: {}", window, e));
            trace!("[{}] switch to ({:?}), title: {:?}", time, uwp, title);
            Ok(())
        }),
    )?;

    let main = tokio::task::LocalSet::new();

    /*main.spawn_local(async move {
        loop {
            let next = watchers::WindowClosedWatcher::next_close(&closes_watch);
            if let Some(item) = next.await {
                println!("[CLOSED]: {:?}", item.title());
            }
        }
    });*/

    runtime.block_on(main.run_until(events));

    info!("Exited");
    Ok(())
}
