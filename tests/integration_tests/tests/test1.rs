use native::wrappers::*;
use std::sync::atomic::*;
use std::thread;

mod common;

static mut KB: AtomicU32 = AtomicU32::new(0);
#[test]
fn it_works() {
    native::setup();

    fn cb(c: i32, w: usize, lparam: &'static mut native::raw::winuser::KBDLLHOOKSTRUCT) -> isize {
        unsafe {
            KB.fetch_add(1, Ordering::SeqCst);
            native::raw::winuser::CallNextHookEx(
                std::ptr::null_mut(),
                c,
                w,
                std::mem::transmute(lparam),
            )
        }
    }

    let _evthread = thread::spawn(|| {
        let mut ev = EventLoop::new();
        let _hook = windows_hook::Hook::new::<windows_hook::LowLevelKeyboard>(
            windows_hook::Locality::Global,
            cb,
        )
        .unwrap();
        ev.run()
    });

    assert_ne!(0, unsafe { KB.load(Ordering::SeqCst) });
}
