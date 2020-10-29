use crate::errors::*;
use crate::os::prelude::*;
use pelite::resources::version_info::{Language, VersionInfo};
use pelite::{FileMap, PeFile};
use std::iter::once;
use std::ffi::*;
use std::os::windows::prelude::*;

pub struct FileInfo {
    pub name: String,
    pub description: String,
}

pub static FALLBACK_LANGS: &[&str] = &[
    "040904B0", // US English + CP_UNICODE
    "040904E4", // US English + CP_USASCII
    "04090000", // US English + unknown codepage
];

impl FileInfo {
    pub fn new(path: &str) -> Result<FileInfo> {
        let file_map = FileMap::open(&std::path::Path::new(path))?;
        let image = PeFile::from_bytes(file_map.as_ref())?;
        let resources = image.resources()?;
        // resources.manifest()?;
        /*let (_, icon_grp) = resources.icons().filter_map(Result::ok).next().unwrap();
        let icon = icon_grp.entries().into_iter().map(|entry|  icon_grp.image(entry.nId)).next().unwrap()?;*/
        let version_info = resources.version_info()?;
        let default_lang = version_info.translation()[0];
        once(default_lang)
            .chain(FALLBACK_LANGS.iter().map(|x| FileInfo::lang(x)))
            .find_map(|lang| {
                let name = version_info.value(lang, "ProductName");
                let desc = version_info.value(lang, "FileDescription");
                name.zip(desc)
                    .map(|(name, description)| FileInfo { name, description })
            })
            .ok_or(anyhow!("Unable to parse!"))
    }

    /*fn extract_icon(path: &str) {
        let mut id = 0u16;
        let mut path: Vec<_> = OsString::from(path).encode_wide().collect();
        let icon = unsafe { shellapi::ExtractAssociatedIconW(ptr::null_mut(), path.as_mut_ptr(), &mut id as *mut _) };
        
    }*/

    /*fn file_info_internal(version_info: VersionInfo, lang: Language) -> Option<FileInfo> {
        let name = version_info.value(lang, "ProductName");
        let desc = version_info.value(lang, "FileDescription");
        name.zip(desc)
            .map(|(name, description)| FileInfo { name, description })
    }*/

    fn lang(s: &'static str) -> Language {
        let buf: Vec<_> = OsString::from(s).encode_wide().collect();
        Language::parse(&buf[..]).expect("Cannot parse language")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let path = "C:\\Users\\enigm\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe";
        let path2 = "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe";
        let file_map = FileMap::open(&std::path::Path::new(path2)).unwrap();
        let image = PeFile::from_bytes(file_map.as_ref()).unwrap();
        let resources = image.resources().unwrap();

        let (_, icon_grp) = resources.icons()/*.filter_map(Result::ok)*/.next().unwrap().unwrap();
        let img = icon_grp.entries().into_iter().map(|entry| 
            icon_grp.image(entry.nId).unwrap())
            .filter_map(|icon| image::load_from_memory(icon).ok()).next().unwrap();

        let mut out = Vec::with_capacity(4096);
        img.write_to(&mut out, image::ImageOutputFormat::Png).unwrap();

        img.save_with_format(format!("C:\\Users\\enigm\\Desktop\\wtf.png"), image::ImageFormat::Png).unwrap();
    }
}