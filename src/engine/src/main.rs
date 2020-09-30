#![feature(trait_alias)]
#![feature(async_closure)]
#![feature(default_free_fn)]
#![feature(option_expect_none)]


mod data;
mod os;
mod watchers;

use os::prelude::*;
use std::cell::UnsafeCell;
use tokio::prelude::*;
use tokio::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[LIFECYCLE] Started!");

    //let closes = UnsafeCell::new(watchers::WindowClosedWatcher::new());
    //let closes_ptr = closes.get(); // dirty tricks

    let ev = hook::EventLoop::new();
    let _hook = hook::WinEventHook::new(
        hook::Range::Single(hook::Event::SystemForeground),
        hook::Locality::Global,
        &(1, 2),
        |(i, _x), args| {
            println!("Switch: {}, {:?}", i, args);
            Ok(())
        }
    )?;
    println!("[LIFECYCLE] Set!");

    ev.await;
    Ok(())
    /*
    let _hook = hook::WinEventHook::new(
        hook::Range::Single(hook::Event::SystemForeground),
        hook::Locality::Global,
        move |_win_event_hook: HWINEVENTHOOK,
              _event: DWORD,
              handle: HWND,
              id_object: LONG,
              _id_child: LONG,
              _id_event_thread: DWORD,
              dwms_event_time: DWORD| {
            // test visibility of window as well?
            if id_object != winuser::OBJID_WINDOW || unsafe { winuser::IsWindow(handle) == 0 } {
                return;
            }
            let window = match Window::new(handle) {
                Ok(window) => window,
                Err(e) => { dbg!("Invalid window {}: {}", handle, e); return; }
            };
            unsafe { &*closes_ptr }
                .watch(window)
                .expect(format!("unable to watch for window close for window {:?}", window).as_str());

            let time = Timestamp::from_ticks(dwms_event_time);
            println!(
                "[SWITCH] at ({}) for {}, title: {}",
                time,
                if window.is_uwp().expect(format!("getting path/uwp for {:?}", window).as_str()) {
                    format!("UWP ({})", window.aumid().expect("aumid is readable"))
                } else {
                    "Win32".to_owned()
                },
                window
                    .title()
                    .unwrap_or_else(|e| format!("Unable to get title for {:?}: {}", window, e))
            );
        },
    )?;

    let watch = async move {
        while let Some(item) = unsafe { &mut *closes_ptr }.recver.next().await {
            println!("[CLOSED]: {:?}", item.title());
        }
    };

    tokio::join!(ev, watch);

    println!("[LIFECYCLE] Exited!");
    Ok(())*/
}
