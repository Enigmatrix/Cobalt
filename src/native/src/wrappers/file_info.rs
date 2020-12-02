use crate::com::*;
use crate::error::*;
use crate::raw::*;
use crate::wrappers::stream::WinRTStreamToRustAdapter;
use pelite::resources::version_info::Language;
use pelite::{FileMap, PeFile};
use std::io::Write;
use std::ptr;
use util::*;

use crate::raw::uwp::windows::storage::file_properties::ThumbnailMode;
use crate::raw::uwp::windows::storage::*;

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
                width: 32.0,
                height: 32.0,
            })
            .winrt_with_context(|| "Get 32x32 logo from DisplayInfo")?;
        let stream = logo
            .open_read_async()
            .winrt_with_context(|| "Open read the RandomAccessStreamReference")?
            .get()
            .winrt_with_context(|| "Blocking `get` of underlying RandomAccessStream")?;
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
        todo!("from_classic_app")
    }

    pub async fn from_win32(path: &str) -> ::winrt::Result<FileInfo> {
        let exe = StorageFile::get_file_from_path_async(path)?.await?;
        let thumb = exe
            .get_thumbnail_async_overload_default_size_default_options(ThumbnailMode::SingleItem)?
            .await?;
        let thumb_sz = thumb.size()?;
        let props = exe.get_basic_properties_async()?.await?;
        // props.retrieve_properties_async();

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_file_thumbnail() {
        use crate::raw::uwp::windows::storage::file_properties::ThumbnailMode;
        use crate::raw::uwp::windows::storage::streams::DataReader;
        use crate::raw::uwp::windows::storage::*;

        let path = "C:\\Program Files\\WindowsApps\\Microsoft.WindowsTerminal_1.4.3243.0_x64__8wekyb3d8bbwe\\WindowsTerminal.exe";
        let file = StorageFile::get_file_from_path_async(path)
            .unwrap()
            .get()
            .unwrap();
        let thumb = file
            .get_thumbnail_async_overload_default_size_default_options(ThumbnailMode::SingleItem)
            .unwrap()
            .get()
            .unwrap();

        let sz = thumb.size().unwrap();
        let mut out = vec![0u8; sz as usize]; // TODO use uninit
        let reader = DataReader::create_data_reader(thumb).unwrap();
        reader.load_async(sz as u32).unwrap().get().unwrap();
        reader.read_bytes(&mut out).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        img.save("C:\\Users\\enigm\\Desktop\\what2.png").unwrap();
        assert_ne!(0, out.len());
    }
}
