use crate::raw::*;
use crate::wrappers::winevent::*;
use crate::wrappers::*;
use util::*;

#[derive(Debug)]
pub struct Watcher {
    _hook: Hook,
}

impl Watcher {
    pub fn new(mut callback: impl FnMut(Window, Timestamp) -> Result<()>) -> Result<Watcher> {
        let _hook = Hook::new(
            Range::Single(Event::SystemForeground),
            Locality::Global,
            Box::new(move |args| {
                if args.id_object != winuser::OBJID_WINDOW
                    || args.id_child != winuser::CHILDID_SELF
                    || unsafe { winuser::GetForegroundWindow() != args.hwnd }
                {
                    return Ok(());
                }
                let window = Window::new(args.hwnd)?;
                let timestamp = Timestamp::from_event_millis(args.dwms_event_time);

                callback(window, timestamp)
            }),
        )?;
        Ok(Watcher { _hook })
    }
}

/*struct Class {
    buffer: buffer::Local::<256>,
    len: usize
}

impl Class {
    pub fn from(win: &Window) -> Result<Class, Win32Err> {
        let mut buf = buffer::local();
        let read = unsafe { winuser::GetClassNameW(win.0, buf.as_mut_ptr(), 256) };
        if read == 0 {
            return Err(Win32Err::last_err());
        }
        Ok(Self { buffer: buf, len: read as usize })
    }

    pub fn to_os_string(&mut self) -> std::ffi::OsString {
        self.buffer.with_length(self.len).to_os_string()
    }
}*/
