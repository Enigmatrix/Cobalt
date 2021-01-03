#![feature(default_free_fn)]
#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_ref)]
#![feature(associated_type_bounds)]
#![feature(seek_convenience)]
#![feature(thread_id_value)]

pub mod raw;
#[macro_use]
pub mod error;
pub mod buffer;
pub mod com;
pub mod watchers;
pub mod wrappers;

pub fn setup() -> util::Result<()> {
    use crate::raw::combaseapi::*;
    use crate::raw::objbase::*;

    wrappers::Timestamp::calculate_boot_time();
    wrappers::winevent::init_contexts();
    hresult!(CoInitializeEx(
        std::ptr::null_mut(),
        COINIT_APARTMENTTHREADED
    ))?;
    wrappers::GlobalEventLoop::init();
    Ok(())
}
