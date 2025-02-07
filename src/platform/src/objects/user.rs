use util::error::*;
use windows::core::PWSTR;
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::WindowsProgramming::GetUserNameW;
use windows::Win32::UI::Shell::IsUserAnAdmin;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub username: String,
    pub is_admin: bool,
}

impl User {
    pub fn current() -> Result<Self> {
        let mut buffer = [0u16; MAX_PATH as usize];
        let mut size = MAX_PATH;
        unsafe { GetUserNameW(PWSTR(buffer.as_mut_ptr()), &mut size)? };
        let username = String::from_utf16_lossy(&buffer[..size as usize - 1]);
        let is_admin = unsafe { IsUserAnAdmin().as_bool() };
        Ok(Self { username, is_admin })
    }
}
