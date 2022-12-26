use utils::{errors::*, tracing::info};
use windows::Foundation::Collections::{IIterable_Impl, IIterable, IIterator, IIterator_Impl};
use windows::Foundation::IReference;
use windows::Win32::Foundation::E_BOUNDS;
use windows::core::{RuntimeType, AsImpl, Interface};
use windows::{Storage::StorageFile, core::HSTRING, h, core::implement};

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: String,
    pub description: String,
    // pub icon:
}

// const PROPS: &[HSTRING] = &[h!("System.FileDescription")];

#[implement(IIterator<T>)]
struct ArrayIterator<T: RuntimeType + 'static>(std::cell::UnsafeCell<(IIterable<T>, usize)>);

#[allow(non_snake_case)]
impl<T: RuntimeType + 'static> IIterator_Impl<T> for ArrayIterator<T> {
    fn Current(&self) -> windows::core::Result<T> {
        unsafe {
            let this = self.0.get();
            let owner = (*this).0.as_impl();

            if owner.0.len() > (*this).1 {
                Ok(owner.0[(*this).1].clone())
            } else {
                Err(windows::core::Error::new(E_BOUNDS, "".into()))
            }
        }
    }

    fn HasCurrent(&self) -> windows::core::Result<bool> {
        unsafe {
            let this = self.0.get();
            let owner = (*this).0.as_impl();
            Ok(owner.0.len() > (*this).1)
        }
    }

    fn MoveNext(&self) -> windows::core::Result<bool> {
        unsafe {
            let this = self.0.get();
            let owner = (*this).0.as_impl();
            (*this).1 += 1;
            Ok(owner.0.len() > (*this).1)
        }
    }

    fn GetMany(&self, _items: &mut [T::DefaultType]) -> windows::core::Result<u32> {
        panic!(); // TODO: arrays still need some work.
    }
}

#[implement(IIterable<T>)]
struct ArrayIterable<T>(&'static [T]) where T: RuntimeType + 'static;

//#[allow(non_snake_case)]
impl<T: RuntimeType + 'static> IIterable_Impl<T> for ArrayIterable<T> {
    fn First(&self) -> windows::core::Result<IIterator<T>> {
        Ok(ArrayIterator::<T>((unsafe { self.cast()? }, 0).into()).into())
    }
}

impl AppInfo {
    pub async fn from_win32(path: &str) -> Result<Self> {
        let file = StorageFile::GetFileFromPathAsync(&path.into())?
            .await
            .with_context(|| format!("get storage file for path: {path}"))?;

        

        let filedesc = "System.FileDescription".into(); 
        let props: [HSTRING; 1] = [filedesc];
        let pp = &props[..];
        let constf = ArrayIterable::<HSTRING>(unsafe { std::mem::transmute(pp) });
        let iter: IIterable<HSTRING> = constf.into();
        let what = file.Properties()?.RetrievePropertiesAsync(&iter)?.await?;
        let ss = what.Lookup(h!("System.FileDescription"))?.cast::<IReference<HSTRING>>()?.GetString()?;

        info!(
            name = file.Name()?.to_string_lossy(),
            display_name = file.DisplayName()?.to_string_lossy(),
            display_type = file.DisplayType()?.to_string_lossy(),
            file_type = file.FileType()?.to_string_lossy(),
            content_type = file.ContentType()?.to_string_lossy(),
            ss = ss.to_string_lossy(),
        );
        Ok(AppInfo {
            name: "".into(),
            description: "".into(),
        })
    }
}
