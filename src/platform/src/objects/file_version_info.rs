use std::mem::MaybeUninit;
use std::path::PathBuf;

use util::error::{bail, Context, Result};
use windows::core::{w, PCWSTR};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoExW, GetFileVersionInfoSizeExW, VerQueryValueW, FILE_VER_GET_LOCALISED,
};

use crate::win32;

const TRANSLATION_CODEPAGE: PCWSTR = w!("\\VarFileInfo\\Translation\0");

/// Equivalent to the [FileVersionInfo](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.fileversioninfo) class in .NET.
pub struct FileVersionInfo {
    buffer: Box<[MaybeUninit<u8>]>,
    langid: String,
}

impl FileVersionInfo {
    /// Create a new [FileVersionInfo] for the specified path
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        use std::os::windows::ffi::*;

        let mut path: PathBuf = path.into();
        let mut path: Vec<_> = path.as_mut_os_str().encode_wide().collect();
        path.push(0); // insert null byte
        let path_ref = PCWSTR(path.as_mut_ptr());

        let flags = FILE_VER_GET_LOCALISED;
        // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfosizeexw
        let mut _hdl = 0; // docs say this is to be unused

        let size = win32!(val: unsafe { GetFileVersionInfoSizeExW(flags, path_ref, &mut _hdl) })
            .context("get file version info size")?;

        let mut buffer = Box::new_uninit_slice(size as usize);
        unsafe {
            GetFileVersionInfoExW(
                flags,
                path_ref,
                Some(_hdl),
                size,
                buffer.as_mut_ptr().cast(),
            )
            .context("get file version info")?
        }

        let langid = Self::raw_query_value::<u16>(&mut buffer, TRANSLATION_CODEPAGE)
            .context("get file version translation codepage")?;
        // read u32 in little endian
        let langid = ((langid[0] as u32) << 16) + langid[1] as u32;
        let langid = format!("{langid:08X}");

        Ok(FileVersionInfo { buffer, langid })
    }

    /// Query the [FileVersionInfo] for the specified key, returning the value as a [String]
    pub fn query_value(&mut self, key: &str) -> Result<String> {
        // constants from: https://referencesource.microsoft.com/#system/services/monitoring/system/diagnosticts/FileVersionInfo.cs,478
        // should be equivalent to how explorer.exe Properties window does it
        let langs = [&self.langid, "040904B0", "040904E4", "04090000"];

        for lang in langs {
            let key = format!("\\StringFileInfo\\{lang}\\{key}\0"); // null terminate string
            let mut key = key.encode_utf16().collect::<Vec<_>>();

            let res = Self::raw_query_value::<u16>(&mut self.buffer, PCWSTR(key.as_mut_ptr()));
            if let Ok(buf) = res {
                // sometimes there is a null terminator and sometimes there isn't ...
                let buf = if let Some(0) = buf.last() {
                    &buf[..buf.len() - 1]
                } else {
                    buf
                };
                return Ok(String::from_utf16_lossy(buf));
            }
        }
        bail!(format!("query value for {key} failed"))
    }

    /// Query the [FileVersionInfo] for the specified key, returning the value as the underlying buffer
    fn raw_query_value<T>(
        version_info: &mut [MaybeUninit<u8>],
        key: impl Into<PCWSTR>,
    ) -> Result<&[T]> {
        // no need to free this ourselves, [docs](https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verqueryvaluew)
        // says it's part of the version info
        let mut lpbuf = std::ptr::null_mut();
        let mut lpsz = 0;

        let pwstr: PCWSTR = key.into();

        unsafe {
            VerQueryValueW(
                version_info.as_mut_ptr().cast(),
                pwstr,
                &mut lpbuf,
                &mut lpsz,
            )
            .ok()
            .with_context(|| format!("query value, key: {}", pwstr.display()))?
        }
        Ok(unsafe { std::slice::from_raw_parts(lpbuf.cast(), lpsz as usize) })
    }
}
