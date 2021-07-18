fn main() {
    windows::build! {
        Windows::Win32::Foundation::*,                      // basic structures (HWND, BOOL ...)
        Windows::Win32::System::Threading::*,               // timers
        Windows::Win32::System::Diagnostics::Debug::*,      // errors
        Windows::Win32::UI::WindowsAndMessaging::*,         // UI (windows, shell)
        Windows::Win32::System::PropertiesSystem::*,        // window properties
        Windows::Win32::Storage::StructuredStorage::*,      // PROPVARIANT and friends
        Windows::Win32::System::Com::CoTaskMemFree,         // COM Free
    };
}