use crate::com::*;
use crate::error::*;
use crate::raw::*;
use crate::wrappers::stream::{RustToWin32StreamAdapter, WinRTStreamToRustAdapter};
use anyhow::*;
use pelite::resources::version_info::Language;
use pelite::{FileMap, PeFile};
use std::io::Write;
use std::ptr;

pub static FALLBACK_LANGS: &[&str] = &[
    "040904B0", // US English + CP_UNICODE
    "040904E4", // US English + CP_USASCII
    "04090000", // US English + unknown codepage
];

pub struct FileInfo {
    pub name: String,
    pub description: String,
    pub icon: image::DynamicImage,
}

impl FileInfo {
    pub fn from_uwp(aumid: &str) -> Result<FileInfo> {
        use crate::raw::uwp::windows::application_model::AppInfo;
        use crate::raw::uwp::windows::foundation::Size;

        let app_info = AppInfo::get_from_app_user_model_id(aumid)
            .map_err(WinRt::from)
            .with_context(|| "Get UWP AppInfo from WinRT API")?;
        let display_info = app_info
            .display_info()
            .map_err(WinRt::from)
            .with_context(|| "DisplayInfo for UWP AppInfo")?;
        let name = display_info
            .display_name()
            .map_err(WinRt::from)
            .with_context(|| "Get name from DisplayInfo")?
            .to_string();
        let description = display_info
            .description()
            .map_err(WinRt::from)
            .with_context(|| "Get description from DisplayInfo")?
            .to_string();
        let logo = display_info
            .get_logo(Size {
                width: 32.0,
                height: 32.0,
            })
            .map_err(WinRt::from)
            .with_context(|| "Get 32x32 logo from DisplayInfo")?;
        let stream = logo
            .open_read_async()
            .map_err(WinRt::from)
            .with_context(|| "Open read the RandomAccessStreamReference")?
            .get()
            .map_err(WinRt::from)
            .with_context(|| "Blocking `get` of underlying RandomAccessStream")?;
        let mut icon_bytes = WinRTStreamToRustAdapter::from(&stream)
            .read_all()
            .with_context(|| "Read bytes out of logo stream")?;
        let icon = image::load_from_memory(&mut icon_bytes)
            .with_context(|| "Load icon bytes as a image")?;
        Ok(FileInfo {
            name,
            description,
            icon,
        })
    }

    pub fn from_classic_app(path: &str) -> Result<FileInfo> {
        let file_map = FileMap::open(path).with_context(|| "Opening FileMap")?;
        let image =
            PeFile::from_bytes(file_map.as_ref()).with_context(|| "Ppen PeFile from FileMap")?;

        let resources = image
            .resources()
            .with_context(|| "Open Resource section from PeFile")?;
        let version_info = resources
            .version_info()
            .with_context(|| "Open VersionInfo from Resources")?;
        let default_lang = version_info.translation().first();

        for lang in FileInfo::languages(default_lang.cloned()) {
            let name = version_info.value(lang, "ProductName");
            let desc = version_info.value(lang, "FileDescription");
            match (name, desc) {
                (Some(name), Some(description)) => {
                    // FIXME don't load the entire image in memory, try to stream it...
                    let buffer = crate::wrappers::stream::HugeExtensibleBuffer::new();
                    let buffer = FileInfo::write_icon_from_path(path, buffer)?;
                    let icon = image::load_from_memory(&buffer.consume()[..])?;
                    return Ok(FileInfo {
                        name,
                        description,
                        icon,
                    });
                }
                _ => continue,
            }
        }
        Err(anyhow!("Unable to find file info in available languages"))
    }

    pub fn write_icon_from_path<W: Write>(path: &str, writer: W) -> Result<W> {
        use std::os::windows::ffi::OsStrExt;

        let mut bufpath: Vec<_> = std::ffi::OsString::from(path).encode_wide().collect();
        let mut id = 0u16;
        let icon = win32!(non_null:
            shellapi::ExtractAssociatedIconW(ptr::null_mut(), bufpath.as_mut_ptr(), &mut id as *mut _))
            .with_context(|| "Retrieve HICON handle")?;
        // let icon = win32!(non_null: shellapi::DuplicateIcon(ptr::null_mut(), icon))?;

        let stream = RustToWin32StreamAdapter::from(writer).writeable();

        let factory =
            Com::<wincodec::IWICImagingFactory2>::create(wincodec::CLSID_WICImagingFactory2)
                .with_context(|| "Create instance of IWICImagingFactory2")?;

        let bitmap = unsafe {
            Com::from_fn(|bitmap| factory.CreateBitmapFromHICON(icon, bitmap))
                .with_context(|| "Create bitmap from HICON")?
                .unwrap()
        };

        let encoder = unsafe {
            Com::from_fn(|encoder| {
                factory.CreateEncoder(&wincodec::GUID_ContainerFormatPng, ptr::null_mut(), encoder)
            })
            .with_context(|| "Create PNG encoder")?
            .unwrap()
        };

        hresult!(encoder.Initialize(&stream.as_istream(), wincodec::WICBitmapEncoderNoCache))
            .with_context(|| "Set writer as output stream")?;

        let frame = unsafe {
            Com::from_fn(|frame| encoder.CreateNewFrame(frame, ptr::null_mut()))
                .with_context(|| "Create new frame for encoder")?
                .unwrap()
        };
        hresult!(frame.Initialize(ptr::null_mut()))?;
        hresult!(frame.WriteSource(&*bitmap as *const _ as *const _, ptr::null_mut()))?;

        hresult!(frame.Commit())?;
        hresult!(encoder.Commit()).with_context(|| "Save bitmap to writer")?;

        // win32!(non_zero: winuser::DestroyIcon(icon))?;

        Ok(stream.inner())
    }

    fn languages(default: Option<Language>) -> impl Iterator<Item = Language> {
        use std::iter::*;

        let e: Box<dyn Iterator<Item = Language>> = match default {
            Some(x) => Box::new(once(x)),
            None => Box::new(empty()),
        };
        e.chain(FALLBACK_LANGS.iter().map(|lang| FileInfo::language(lang)))
    }

    fn language(lang: &'static str) -> Language {
        use std::ffi::*;
        use std::os::windows::prelude::*;

        let buf: Vec<_> = OsString::from(lang).encode_wide().collect();
        Language::parse(&buf[..]).expect("Cannot parse language")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_can_be_retreived() {
        use crate::raw::combaseapi::*;

        /*tracing::subscriber::set_global_default(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
                // .with_thread_ids(true)
                // .compact()
                .finish(),
        )
        .unwrap();
        let span = tracing::trace_span!("main");
        let _ = span.enter();
        tracing::warn!("twefv");*/

        hresult!(CoInitializeEx(NULL, COINITBASE_MULTITHREADED)).unwrap();

        // let path = "C:\\WINDOWS\\system32\\notepad.exe";
        let path = "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe";
        // assert!(FileInfo::from_classic_app(path).is_err());

        let buffer = crate::wrappers::stream::HugeExtensibleBuffer::new();
        let buffer = FileInfo::write_icon_from_path(path, buffer).unwrap();
        let image = image::load_from_memory(&buffer.consume()[..]).unwrap();
        image.save("C:\\Users\\enigm\\Desktop\\what2.png").unwrap();

        /*let mem = FileInfo::write_icon_from_path2(path).unwrap();
        let image = image::load_from_memory(&mem[..]).unwrap();
        image.save("C:\\Users\\enigm\\Desktop\\what.png").unwrap();*/
    }
}
