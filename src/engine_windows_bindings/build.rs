fn main() {
  windows::build!(
      Windows::Win32::Debug::*,
      Windows::Win32::WindowsAndMessaging::*
    );
}