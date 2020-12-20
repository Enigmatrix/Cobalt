use native::com::Com;
use native::raw::*;
use native::wrappers::*;
use shobjidl_core::*;
use std::ptr;
use unknwnbase::{IUnknown, IUnknownVtbl};

RIDL! {#[uuid(0x2e941141, 0x7f97, 0x4756, 0xba, 0x1d, 0x9d, 0xec, 0xde, 0x89, 0x4a, 0x3d)]
interface IApplicationActivationManager(IApplicationActivationManagerVtbl): IUnknown(IUnknownVtbl) {
    fn activate_application(
        app_user_model_id: *const u16,
        arguments: *const u16,
        options: std::os::raw::c_int,
        process_id: *mut DWORD,
    ) -> HRESULT,
}}

pub struct App {
    pid: u32,
    process: Process,
}

impl App {
    pub fn spawn(aumid: &str) -> App {
        use std::ffi::OsString;
        use std::os::windows::prelude::*;

        let mut aumid_u16 = OsString::from(aumid);
        aumid_u16.push("\0");
        let aumid_u16: Vec<u16> = aumid_u16.encode_wide().collect();
        let activator =
            Com::<IApplicationActivationManager>::create(ApplicationActivationManager::uuidof())
                .unwrap();
        let mut pid = 0;
        let res = unsafe {
            activator.activate_application(aumid_u16.as_ptr(), ptr::null_mut(), 0, &mut pid)
        };
        if res < 0 {
            let err = std::io::Error::from_raw_os_error(res).to_string();
            println!("ERR activating app {}", aumid);
            println!("NtStatus(0x{:x}): {}", res, err);
            panic!();
        }
        App {
            process: Process::new(
                pid,
                ProcessOptions {
                    terminate: true,
                    ..Default::default()
                },
            )
            .unwrap(),
            pid,
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
        self.process.terminate().unwrap();
    }
}
