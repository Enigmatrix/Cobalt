#![feature(trait_alias)]
// use tokio::prelude::*;

mod os;
use os::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Started!");

    let ev = hook::EventLoop::new();
    let hook = hook::WinEventHook::new(
        hook::Type::Single(hook::Event::SystemForeground),
        hook::Locality::Global,
        move |_win_event_hook: HWINEVENTHOOK,
              _event: DWORD,
              handle: HWND,
              _id_object: LONG,
              _id_child: LONG,
              _id_event_thread: DWORD,
              dwms_event_time: DWORD| {
            if unsafe { winuser::IsWindow(handle) == 0 || winuser::IsWindowVisible(handle) == 0 } {
                return;
            }
            let window = Window::new(handle);
            println!("switched to {}", window.title().unwrap());
        },
    )?;

    ev.await;

    println!("Exited!");
    Ok(())
}
