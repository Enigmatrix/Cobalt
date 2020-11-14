#![feature(default_free_fn)]
#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_extra)]
#![feature(min_const_generics)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_ref)]

pub mod raw;
#[macro_use]
pub mod error;
pub mod buffer;
pub mod com;
pub mod watchers;
pub mod wrappers;

pub fn setup() -> anyhow::Result<()> {
    use crate::raw::combaseapi::*;
    use crate::raw::objbase::*;

    wrappers::Timestamp::calculate_boot_time();
    wrappers::winevent::init_contexts();
    hresult!(CoInitializeEx(
        std::ptr::null_mut(),
        COINIT_APARTMENTTHREADED
    ))?;
    Ok(())
}
