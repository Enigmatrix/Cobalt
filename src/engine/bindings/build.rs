fn main() {
    windows::build! {
        Windows::Win32::Foundation::*,
        Windows::Win32::System::Threading::*,
        Windows::Win32::System::Diagnostics::Debug::*,
        Windows::Win32::UI::WindowsAndMessaging::*,
    };
}