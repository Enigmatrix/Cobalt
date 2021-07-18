mod bindings {
    windows::include_bindings!();
}
pub mod meta {
    pub use ::windows::*;
}

pub use bindings::*;