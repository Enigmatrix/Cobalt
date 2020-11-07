use crate::com::*;
use crate::raw::*;
use crate::wrappers::stream::Stream;
use anyhow::*;
use pelite::resources::version_info::Language;
use pelite::resources::Resources;
use pelite::{FileMap, PeFile};
use std::io::Write;
use std::path::Path;
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
    pub fn from_classic_app<P: AsRef<Path>>(path: P) -> Result<FileInfo> {
        let file_map = FileMap::open(path.as_ref()).with_context(|| "Unable to open file map")?;
        let image = PeFile::from_bytes(file_map.as_ref())
            .with_context(|| "Unable to open PeFile from file_map")?;

        let resources = image
            .resources()
            .with_context(|| "Unable to open resource section")?;
        let version_info = resources
            .version_info()
            .with_context(|| "Unable to open VersionInfo from resources")?;
        let default_lang = version_info.translation()[0];

        for lang in FileInfo::languages(default_lang) {
            let name = version_info.value(lang, "ProductName");
            let desc = version_info.value(lang, "FileDescription");
            match (name, desc) {
                (Some(name), Some(description)) => {
                    return Ok(FileInfo {
                        name,
                        description,
                        icon: FileInfo::icon(&resources)?,
                    })
                }
                _ => continue,
            }
        }
        Err(anyhow!("Unable to find file info in available languages"))
    }

    pub fn write_icon_from_path<W: Write>(path: impl AsRef<Path>, writer: W) -> Result<W> {
        use std::os::windows::ffi::OsStrExt;

        let mut bufpath: Vec<_> = path.as_ref().as_os_str().encode_wide().collect();
        let mut id = 0u16;
        let icon = win32!(non_null:
            shellapi::ExtractAssociatedIconW(ptr::null_mut(), bufpath.as_mut_ptr(), &mut id as *mut _))
            .with_context(|| "Unable to retrieve HICON handle")?;
        // let icon = win32!(non_null: shellapi::DuplicateIcon(ptr::null_mut(), icon))?;

        let stream = Stream::from(writer).writeable();

        let factory = unsafe {
            Com::<wincodec::IWICImagingFactory2>::create(wincodec::CLSID_WICImagingFactory2)
                .with_context(|| "Unable to create instance of IWICImagingFactory2")?
        };

        let bitmap = unsafe {
            Com::from_fn(|bitmap| factory.CreateBitmapFromHICON(icon, bitmap))
                .with_context(|| "Unable to create bitmap from HICON")?
                .unwrap()
        };

        let encoder = unsafe {
            Com::from_fn(|encoder| {
                factory.CreateEncoder(&wincodec::GUID_ContainerFormatPng, ptr::null_mut(), encoder)
            })
            .with_context(|| "Unable to create PNG encoder")?
            .unwrap()
        };

        hresult!(encoder.Initialize(&stream.as_istream(), wincodec::WICBitmapEncoderNoCache))
            .with_context(|| "Unable to set writer as output stream")?;

        let frame = unsafe {
            Com::from_fn(|frame| encoder.CreateNewFrame(frame, ptr::null_mut()))
                .with_context(|| "Unable to create new frame for encoder")?
                .unwrap()
        };
        hresult!(frame.Initialize(ptr::null_mut()))?;
        hresult!(frame.WriteSource(&*bitmap as *const _ as *const _, ptr::null_mut()))?;

        hresult!(frame.Commit())?;
        hresult!(encoder.Commit()).with_context(|| "Unable to save bitmap to writer")?;

        // win32!(non_zero: winuser::DestroyIcon(icon))?;

        Ok(stream.inner())
    }

    fn icon(resources: &Resources) -> Result<image::DynamicImage> {
        resources
            .icons()
            .filter_map(Result::ok)
            .flat_map(|(_, icons)| {
                icons
                    .entries()
                    .iter()
                    .filter_map(move |entry| icons.image(entry.nId).ok()) // get all accessible images
                    .filter_map(|icon| image::load_from_memory(icon).ok()) // get all valid images
            })
            .next()
            .with_context(|| "Unable to find any valid icons")
    }

    fn languages(default: Language) -> impl Iterator<Item = Language> {
        use std::iter::once;

        once(default).chain(FALLBACK_LANGS.iter().map(|lang| FileInfo::language(lang)))
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

        let path = "C:\\WINDOWS\\system32\\notepad.exe";
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
