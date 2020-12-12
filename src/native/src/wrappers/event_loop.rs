use crate::raw::*;
use std::mem;
use std::ptr;
use std::thread;
use util::*;

const WM_RUN_IN_CONTEXT: u32 = winuser::WM_USER + 0x1;

static mut GLOBAL_EVENT_LOOP: mem::MaybeUninit<GlobalEventLoop> = mem::MaybeUninit::uninit();

type ExecFn<'a, T> = &'a mut dyn FnMut() -> Result<T>;

struct Waiter(i64);

impl Waiter {
    pub fn new() -> Waiter {
        Waiter(0)
    }

    pub fn wait(&mut self) -> Result<(), crate::error::Win32Err> {
        let mut ans: u64 = 0;
        win32!(non_zero: synchapi::WaitOnAddress(self as *mut _ as *mut _, &mut ans as *mut _ as *mut _, 8, winbase::INFINITE))?;
        Ok(())
    }

    pub fn signal(&mut self) {
        self.0 = -1;
        unsafe { synchapi::WakeByAddressAll(self as *mut _ as *mut _) }
    }
}

pub struct GlobalEventLoop {
    tid: u32,
}

impl GlobalEventLoop {
    fn new() -> GlobalEventLoop {
        use std::os::windows::io::AsRawHandle;

        let hdl = thread::spawn(|| loop {
            // TODO communicate the exit code to the main exit process
            if let Some(exit) = unsafe { GlobalEventLoop::step() } {
                break Some(exit);
            }
        });

        let tid = unsafe { processthreadsapi::GetThreadId(hdl.as_raw_handle()) };

        GlobalEventLoop { tid }
    }

    pub fn init() {
        unsafe { GLOBAL_EVENT_LOOP.write(GlobalEventLoop::new()) };
    }

    pub fn get() -> &'static mut Self {
        unsafe { GLOBAL_EVENT_LOOP.assume_init_mut() }
    }

    pub fn exec<'a, T>(mut f: ExecFn<'a, T>) -> Result<T> {
        let sel = GlobalEventLoop::get();
        let mut waiter = Waiter::new();

        let mut res: Option<T> = None;
        let resp = &mut res;

        let mut cb: ExecFn<()> = &mut move || -> util::Result<()> {
            *resp = Some(f().with_context(|| "Calling inner function for evloop")?);
            Ok(())
        };

        win32!(non_zero: winuser::PostThreadMessageW(
            sel.tid,
            WM_RUN_IN_CONTEXT,
            &mut waiter as *mut _ as usize,
            &mut cb as *mut _ as isize))
        .with_context(|| "Post callback to thread")?;

        waiter
            .wait()
            .with_context(|| "Error in waiting for signal to fire")?;
        res.with_context(|| "Callback was not called!")
    }

    #[inline(always)]
    unsafe fn step() -> Option<usize> {
        let mut msg = winuser::MSG::default();
        if winuser::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
            if msg.message == winuser::WM_QUIT {
                return Some(msg.wParam);
            }

            if msg.message == WM_RUN_IN_CONTEXT {
                let waiter = &mut *(msg.wParam as *mut Waiter);
                let execf = &mut *(msg.lParam as *mut ExecFn<'static, ()>);
                execf().unwrap_or_exit();
                waiter.signal();
                return None;
            }

            winuser::TranslateMessage(&mut msg as *mut _);
            winuser::DispatchMessageW(&mut msg as *mut _);
        }
        None
    }
}
