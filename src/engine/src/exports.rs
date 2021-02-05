// Unused warnings will be raised since we import
// all the functions but don't use them here
#![allow(dead_code)]

#![feature(never_type)]
#![feature(async_closure)]
#![feature(seek_convenience)]

mod data;


use rusqlite::Connection;
use util::*;
use crate::data::migrations::Migrator;

#[no_mangle]
pub extern fn migrate(conn_str: ffi::InString) -> ffi::Result {
    fn inner(conn_str: ffi::InString) -> Result<()> {
        let mut conn = Connection::open(conn_str.into_string()?).with_context(|| "Create connection from connection string")?;
        Migrator::migrate(&mut conn).with_context(|| "Run migrations on database")
    }

    inner(conn_str).into()
}

mod ffi {
    use std::{ffi::CStr, os::raw::c_char};
    use util::error::Context;

    /*pub struct String {
        
    }*/

    #[repr(C)]
    pub struct InString(*const c_char);

    impl InString {
        pub fn into_string(self) -> util::Result<std::string::String> {
            let c_str = unsafe {
                assert!(!self.0.is_null());
                CStr::from_ptr(self.0)
            };
            let out = c_str.to_str().with_context(|| "String conversion to UTF-8 failed")?.to_owned();
            Ok(out)
        }
    }

    #[repr(C, u64)]
    pub enum Result {
        Ok(u64),
        Err(String)
    }

    impl From<util::Result<()>> for Result {
        fn from(r: util::Result<()>) -> Self {
            match r {
                Ok(_) => Result::Ok(0),
                Err(e) => Result::Err(e.to_string()) 
            }
        }
    }
}