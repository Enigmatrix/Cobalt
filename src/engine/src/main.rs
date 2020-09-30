#![feature(trait_alias)]
#![feature(async_closure)]
#![feature(default_free_fn)]
#![feature(option_expect_none)]
#![feature(maybe_uninit_ref)]

mod data;
mod os;
mod watchers;

use os::prelude::*;
use std::cell::UnsafeCell;
use tokio::prelude::*;
use tokio::stream::StreamExt;
use std::rc::Rc;
use std::cell::RefCell;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[LIFECYCLE] Started!");
    hook::init_contexts();

    let closes = Rc::new(RefCell::new(watchers::WindowClosedWatcher::new()));

    let ev = hook::EventLoop::new();
    let _hook = hook::WinEventHook::new(
        hook::Range::Single(hook::Event::SystemForeground),
        hook::Locality::Global,
        &(Rc::clone(&closes)),
        |(closes), args| {
            let time = Timestamp::from_ticks(args.dwms_event_time); // get time first!
            if args.id_object != winuser::OBJID_WINDOW
                || unsafe { winuser::IsWindow(args.hwnd) == 0 }
            {
                return Ok(()); // normal response
            }
            let window = match Window::new(args.hwnd) {
                Ok(window) => window,
                Err(e) => {
                    dbg!("Invalid window {}: {}", args.hwnd, e);
                    return Ok(());
                }
            };
            //let mut re: &mut watchers::WindowClosedWatcher = unsafe { mem::transmute(closes_ptr) };
            {
                let closes = closes.borrow_mut();
                closes.watch(window)
                    .expect(format!("unable to watch for window close for window {:?}", window).as_str());
            }

            println!(
                "[SWITCH] at ({}) for {}, title: {}",
                time,
                if window
                    .is_uwp()
                    .expect(format!("getting path/uwp for {:?}", window).as_str())
                {
                    format!("UWP ({})", window.aumid().expect("aumid is readable"))
                } else {
                    "Win32".to_owned()
                },
                window
                    .title()
                    .unwrap_or_else(|e| format!("Unable to get title for {:?}: {}", window, e))
            );
            Ok(())
        },
    )?;

    ev.await;
    Ok(())

    /*
    let watch = async move {
        while let Some(item) = unsafe { &mut *closes_ptr }.recver.next().await {
            println!("[CLOSED]: {:?}", item.title());
        }
    };

    tokio::join!(ev, watch);

    println!("[LIFECYCLE] Exited!");
    Ok(())*/
}
