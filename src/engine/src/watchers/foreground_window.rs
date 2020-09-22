use crate::windows::*;
use tokio::sync::*;

#[derive(Clone, Debug)]
pub struct WindowSwitch {
    pub title: String
}

pub struct ForegroundWindowWatcher<'a> {
    _hook: WinEventHook<'a>,
    pub stream: mpsc::UnboundedReceiver<WindowSwitch>
}

pub fn title(hwnd: wintypes::HWND) -> String {
    use std::os::windows::ffi::OsStringExt;

    let len = unsafe { winuser::GetWindowTextLengthW(hwnd) }; // TODO this could error out
    let mut buf = vec![0u16; len as usize];
    unsafe { winuser::GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1) }; // TODO this could error out
    std::ffi::OsString::from_wide(&buf).into_string().unwrap()
}

impl<'a> ForegroundWindowWatcher<'a> {
    pub fn new() -> Result<Self, error::WinError> {

        let (tx, rx) = mpsc::unbounded_channel::<WindowSwitch>(); // TODO

        let _hook = WinEventHook::new(
            WinEvent::SystemForeground,
            Locality::Global,
            move |
                _win_event_hook: wintypes::HWINEVENTHOOK,
                _event: wintypes::DWORD,
                hwnd: wintypes::HWND,
                _id_object: wintypes::LONG,
                _id_child: wintypes::LONG,
                _id_event_thread: wintypes::DWORD,
                dwms_event_time: wintypes::DWORD |
                {
                    if unsafe { winuser::IsWindow(hwnd) == 0 || winuser::IsWindowVisible(hwnd) == 0 } {
                        return;
                    }
                    let title = title(hwnd);
                    println!("normal: {}", title);

                    tx.send(WindowSwitch { title }).unwrap(); // TODO
                })?;

        Ok(ForegroundWindowWatcher { _hook, stream: rx })
    }

}