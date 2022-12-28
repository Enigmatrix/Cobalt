use std::mem::MaybeUninit;

use utils::errors::*;
use windows::core::PCWSTR;
use windows::w;
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoExW, GetFileVersionInfoSizeExW, VerQueryValueW, FILE_VER_GET_LOCALISED,
};

use crate::errors::Win32Error;
use crate::win32;

pub struct FileVersionInfo {
    version_info: Box<[MaybeUninit<u8>]>,
    langid: String,
}

const TRANSLATION_CODEPAGE: PCWSTR = w!("\\VarFileInfo\\Translation\0");

// based off https://referencesource.microsoft.com/#system/services/monitoring/system/diagnosticts/FileVersionInfo.cs
impl FileVersionInfo {
    pub fn new(path: &str) -> Result<Self> {
        let mut pathbuf = Vec::with_capacity(path.len() + 1);
        path.encode_utf16().collect_into(&mut pathbuf);
        pathbuf.push(0); // append null terminator

        let pathptr = PCWSTR(pathbuf.as_ptr());
        let flags = FILE_VER_GET_LOCALISED;
        // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfosizeexw
        let mut _hdl = 0; // docs say this is to be unused

        let size =
            win32!(non_zero_num: unsafe { GetFileVersionInfoSizeExW(flags, pathptr, &mut _hdl) })
                .context("get file version info size")?;

        // version info likely has self-refs, so allocate them all into the heap
        let mut version_info = Box::new_uninit_slice(size as usize);

        win32!(non_zero: unsafe { GetFileVersionInfoExW(flags, pathptr, 0, size, version_info.as_mut_ptr().cast()) })
            .context("get file version info")?;

        let langid_slice = Self::raw_query_value::<u16>(&mut version_info, TRANSLATION_CODEPAGE)
            .context("read translation lang")?;

        // read u32 in little endian
        let langid = ((langid_slice[0] as u32) << 16) + langid_slice[1] as u32;
        let langid = format!("{langid:08X}");

        Ok(Self {
            version_info,
            langid,
        })
    }

    pub fn query_value(&mut self, key: &str) -> Result<String> {
        // constants from: https://referencesource.microsoft.com/#system/services/monitoring/system/diagnosticts/FileVersionInfo.cs,478
        // should be equivalent to how explorer.exe Properties window does it
        let langs = [&self.langid, "040904B0", "040904E4", "04090000"];
        let mut defaulterr = eyre!("all code langs failed for {key}"); // TODO make this cheaper

        for lang in langs {
            let key = format!("\\StringFileInfo\\{lang}\\{key}\0"); // null terminate string
            let mut key = key.encode_utf16().collect::<Vec<_>>();

            let res =
                Self::raw_query_value::<u16>(&mut self.version_info, PCWSTR(key.as_mut_ptr()));
            match res {
                Ok(buf) => {
                    // sometimes there is a null terminator and sometimes there isn't ...
                    let buf = if let Some(0) = buf.last() {
                        &buf[..buf.len() - 1]
                    } else {
                        buf
                    };
                    return Ok(String::from_utf16_lossy(buf));
                }
                Err(err) => defaulterr = defaulterr.error(err),
            }
        }
        Err(defaulterr)
    }

    fn raw_query_value<T>(
        version_info: &mut [MaybeUninit<u8>],
        key: impl Into<PCWSTR>,
    ) -> Result<&[T], Win32Error> {
        // no need to free this ourselves, [docs](https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verqueryvaluew)
        // says it's part of the version info
        let mut lpbuf = std::ptr::null_mut();
        let mut lpsz = 0;
        win32!(non_zero: unsafe { VerQueryValueW(version_info.as_mut_ptr().cast(), key.into(), &mut lpbuf, &mut lpsz) })?;
        Ok(unsafe { std::slice::from_raw_parts(lpbuf.cast(), lpsz as usize) })
    }
}
