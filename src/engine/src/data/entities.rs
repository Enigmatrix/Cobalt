use crate::os::time::Timestamp;
pub type Color = String;

pub struct App {
    id: u64,
    name: String,
    description: String,
    // icon: Blob,
    background: Color,
    identification: AppIdentification,
}

pub enum AppIdentification {
    Win32 { path: String },
    Uwp { aumid: String },
    Java { jar: String },
}

pub struct Tag {
    id: u64,
    name: String,
    description: String,
    background: Color,
}

pub struct AppTag {
    app_id: u64,
    tag_id: u64,
}

pub struct Session {
    id: u64,
    app_id: u64,
    arguments: String,
    title: String,
}

pub struct Usage {
    id: u64,
    start: Timestamp,
    end: Timestamp,
    during_idle: bool,
    session_id: u64,
}
