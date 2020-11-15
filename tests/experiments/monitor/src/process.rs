use anyhow::*;
use native::raw::*;
use native::wrappers::*;

pub struct WindowEnumerator {
    windows: *mut Vec<Window>,
}

impl WindowEnumerator {
    pub fn new() -> Result<WindowEnumerator> {
        let windows = Box::into_raw(Box::new(Vec::new()));
        let enumerator = WindowEnumerator { windows };
        unsafe { winuser::EnumWindows(Some(WindowEnumerator::handler), windows as isize) };
        Ok(enumerator)
    }

    pub fn into_windows(self) -> Vec<Window> {
        unsafe { *Box::from_raw(self.windows) }
    }

    unsafe extern "system" fn handler(hwnd: HWND, lparam: isize) -> i32 {
        let windows = &mut *(lparam as *mut Vec<Window>);
        windows.push(Window::new(hwnd).unwrap());
        1
    }
}

#[derive(Clone)]
pub struct App {
    pub pid: u32,
    pub name: String,
    pub desc: String,
    pub path: String,
    pub icon: image::DynamicImage,

    pub selected_state: iced::button::State,
}

pub struct AppInfo {
    pub process: Process,
}

impl AppInfo {
    pub fn new(pid: u32) -> AppInfo {
        AppInfo {
            process: Process::new(pid, Default::default()).unwrap(),
        }
    }

    fn user_params(
        &self,
    ) -> Result<(
        ntrtl::RTL_USER_PROCESS_PARAMETERS,
        *mut ntrtl::RTL_USER_PROCESS_PARAMETERS,
    )> {
        let mut info = ntpsapi::PROCESS_BASIC_INFORMATION::default();
        let mut info_len = 0u32;
        unsafe {
            ntpsapi::NtQueryInformationProcess(
                self.process.handle(),
                0,
                &mut info as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<ntpsapi::PROCESS_BASIC_INFORMATION>() as u32,
                &mut info_len as &mut u32,
            )
        };

        let peb = self.process.read_process_memory(info.PebBaseAddress)?;
        let params = self.process.read_process_memory(peb.ProcessParameters)?;
        Ok((params, peb.ProcessParameters))
    }

    pub fn path_and_cmd(&self) -> (String, String) {
        let user_params = self.user_params().unwrap().0;
        let path = self
            .process
            .read_string_from_process_memory(user_params.ImagePathName)
            .unwrap();
        let cmd_line = self
            .process
            .read_string_from_process_memory(user_params.CommandLine)
            .unwrap();
        (path, cmd_line)
    }

    pub fn write_cmd(&self, s: String) {
        let (user_params, user_params_ptr) = self.user_params().unwrap();
        let user_params_ptr = unsafe { &mut *user_params_ptr };
        let mut new_value = user_params.CommandLine.Buffer;
        let mut written = 0u32;
        if unsafe {
            memoryapi::WriteProcessMemory(
                self.process.handle(),
                &mut user_params_ptr.CommandLine.Buffer as *mut _ as *mut _,
                &mut new_value as *mut _ as *mut _,
                std::mem::size_of_val(&new_value),
                &mut written as *mut _ as *mut _,
            )
        } == 0
        {
            panic!("wroite failed {:?}", std::io::Error::last_os_error());
        }
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut _core::fmt::Formatter<'_>) -> _core::fmt::Result {
        f.debug_struct("App")
            .field("pid", &self.pid)
            .field("name", &self.name)
            .field("desc", &self.desc)
            .field("path", &self.path)
            .field("icon", &format!("{} bytes", self.icon.to_bytes().len()))
            .finish()
    }
}

impl App {
    pub fn new(pid: u32) -> Result<App> {
        let process = Process::new(pid, ProcessOptions::default())?;
        let path = process.path()?;
        let file = FileInfo::from_classic_app(path.clone())?;
        Ok(App {
            pid,
            name: file.name,
            desc: file.description,
            path,
            icon: file.icon,
            selected_state: iced::button::State::new(),
        })
    }

    pub fn new_from_window(win: Window) -> Result<App> {
        let (pid, _) = win.pid_tid()?;
        App::new(pid)
    }
}
