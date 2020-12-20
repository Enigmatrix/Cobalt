use native::raw::*;
use std::process::*;

pub struct App {
    pid: u32,
    process: Child,
}

impl App {
    pub fn spawn(path: &str) -> App {
        use std::os::windows::io::AsRawHandle;
        let process = Command::new(path).spawn().unwrap();
        unsafe { winuser::WaitForInputIdle(process.as_raw_handle(), winbase::INFINITE) };
        App {
            pid: process.id(),
            process,
        }
    }
}

impl super::ProcessContainer for App {
    fn pid(&self) -> u32 {
        self.pid
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.process.kill().unwrap();
    }
}
