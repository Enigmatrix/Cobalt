#[macro_export]
macro_rules! string_buffer {
    ($sz: expr) => {
        vec![0u16; $sz as usize]
    };
}

#[macro_export]
macro_rules! string_from_buffer {
    ($buf: expr) => {{
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide($buf).to_string_lossy().into()
    }};
    ($buf: expr, $len: expr) => {{
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide(&$buf[..$len as usize])
            .to_string_lossy()
            .into()
    }};
}
