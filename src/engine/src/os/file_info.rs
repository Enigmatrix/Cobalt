use crate::errors::*;
use pelite::resources::version_info::{Language, VersionInfo};
use pelite::{FileMap, PeFile};
use std::iter::once;

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
        let version_info = resources.version_info()?;
        let default_lang = version_info.translation()[0];
        once(default_lang)
            .chain(FALLBACK_LANGS.iter().map(|x| FileInfo::lang(x)))
            .find_map(|lang| FileInfo::file_info_internal(version_info, lang))
            .ok_or(anyhow!("Unable to parse!"))
    }

    fn file_info_internal(version_info: VersionInfo, lang: Language) -> Option<FileInfo> {
        let name = version_info.value(lang, "ProductName");
        let desc = version_info.value(lang, "FileDescription");
        name.zip(desc)
            .map(|(name, description)| FileInfo { name, description })
    }

    fn lang(s: &'static str) -> pelite::resources::version_info::Language {
        use std::ffi::*;
        use std::os::windows::prelude::*;
        let buf: Vec<_> = OsString::from(s).encode_wide().collect();
        pelite::resources::version_info::Language::parse(&buf[..]).expect("Cannot parse language")
    }
}
