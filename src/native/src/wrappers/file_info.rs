use crate::error::*;
use crate::wrappers::stream::WinRTImageStream;
use pelite::resources::version_info::Language;
use pelite::{FileMap, PeFile};
use util::*;

use crate::raw::uwp::windows::storage::file_properties::ThumbnailMode;
use crate::raw::uwp::windows::storage::*;

use std::future::Future;

trait UnsafeFutureExt {
    // TODO use better pattern!
    fn wrap_unsafe<U>(self) -> UnsafeFuture<Self, U>
    where
        Self: Future<Output = U> + Unpin + Sized;
}

impl<T> UnsafeFutureExt for T {
    fn wrap_unsafe<U1>(self) -> UnsafeFuture<Self, U1>
    where
        Self: Future<Output = U1> + Unpin + Sized,
    {
        UnsafeFuture(self)
    }
}

struct UnsafeFuture<T: Future<Output = U>, U>(pub T);
struct UnsafeSend<T>(pub T);
unsafe impl<T> Send for UnsafeSend<T> {}
unsafe impl<T> Sync for UnsafeSend<T> {}
unsafe impl<T: Future<Output = U>, U> Send for UnsafeFuture<T, U> {}
unsafe impl<T: Future<Output = U>, U> Sync for UnsafeFuture<T, U> {}

impl<T: Future<Output = U> + Unpin, U> Future for UnsafeFuture<T, U> {
    type Output = U;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        T::poll(std::pin::Pin::new(&mut self.get_mut().0), cx)
    }
}

pub static FALLBACK_LANGS: &[&str] = &[
    "040904B0", // US English + CP_UNICODE
    "040904E4", // US English + CP_USASCII
    "04090000", // US English + unknown codepage
];

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub description: String,
    pub icon: WinRTImageStream,
}
unsafe impl Send for FileInfo {}
unsafe impl Sync for FileInfo {}

impl FileInfo {
    pub async fn from_uwp(aumid: &str) -> Result<FileInfo> {
        use crate::raw::uwp::windows::application_model::AppInfo;
        use crate::raw::uwp::windows::foundation::Size;

        let app_info = AppInfo::get_from_app_user_model_id(aumid)
            .winrt_with_context(|| "Get UWP AppInfo from WinRT API")?;
        let display_info = app_info
            .display_info()
            .winrt_with_context(|| "DisplayInfo for UWP AppInfo")?;
        let name = display_info
            .display_name()
            .winrt_with_context(|| "Get name from DisplayInfo")?
            .to_string();
        let description = display_info
            .description()
            .winrt_with_context(|| "Get description from DisplayInfo")?
            .to_string();
        let logo = display_info
            .get_logo(Size {
                width: 44.0,
                height: 44.0,
            })
            .winrt_with_context(|| "Get 32x32 logo from DisplayInfo")?;
        let stream = logo
            .open_read_async()
            .winrt_with_context(|| "Open read the RandomAccessStreamReference")?
            .wrap_unsafe();
        let stream = stream
            .await
            .winrt_with_context(|| "async get of underlying RandomAccessStream")?;
        let icon = WinRTImageStream::from(stream);
        Ok(FileInfo {
            name,
            description,
            icon,
        })
    }

    pub async fn from_win32(path: &str) -> Result<FileInfo> {
        let (name, description) = FileInfo::win32_name_desc(path)
            .with_context(|| "Get name and description of exe from its resource sections")?;
        let icon = FileInfo::win32_icon(path)
            .await
            .with_context(|| "Get icon of exe using StorageFile API")?;
        Ok(FileInfo {
            name,
            description,
            icon,
        })
    }

    fn win32_name_desc(path: &str) -> Result<(String, String)> {
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
                    return Ok((name, description));
                }
                _ => continue,
            }
        }
        bail!("Unable to find file info in available languages")
    }

    async fn win32_icon(path: &str) -> Result<WinRTImageStream> {
        let exe = StorageFile::get_file_from_path_async(path)
            .winrt_with_context(|| format!("Get exe in {}", path))?
            .wrap_unsafe();
        let exe = UnsafeSend(
            exe.await
                .winrt_with_context(|| "async of getting StorageFile")?,
        );
        let thumb = exe
            .0
            .get_thumbnail_async_overload_default_size_default_options(ThumbnailMode::SingleItem)
            .winrt_with_context(|| "Get thumbnail of file with default options")?
            .wrap_unsafe();
        let thumb = thumb
            .await
            .winrt_with_context(|| "async of getting thumbail")?;
        Ok(WinRTImageStream::from(thumb))
    }

    fn languages(default: Option<Language>) -> impl Iterator<Item = Language> {
        default
            .into_iter()
            .chain(FALLBACK_LANGS.iter().map(|lang| FileInfo::language(lang)))
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

    fn is_send<T: Send>(a: T) {}

    #[test]
    fn is_1() {
        is_send(FileInfo::from_uwp(""));
    }

    #[test]
    fn storage_file_thumbnail() {
        let path = "C:\\Program Files\\WindowsApps\\Microsoft.WindowsTerminal_1.4.3243.0_x64__8wekyb3d8bbwe\\WindowsTerminal.exe";
        let img = tokio_test::block_on(FileInfo::win32_icon(path)).unwrap();
        // if doesn't fail, it's fine
    }

    #[test]
    fn uwp_test() {
        let aumid = "Microsoft.ZuneVideo_8wekyb3d8bbwe!Microsoft.ZuneVideo";
        let file_info = tokio_test::block_on(FileInfo::from_uwp(aumid)).unwrap();
        /*file_info
        .icon
        .save("C:\\Users\\enigm\\Desktop\\what3.png")
        .unwrap();*/
    }
}
