use crate::os::time::Timestamp;
pub type Color = String;

#[derive(Debug)]
pub struct App {
    pub id: u64,
    pub name: String,
    pub description: String,
    // icon: Blob,
    pub background: Color,
    pub identification: AppIdentification,
}

#[derive(Debug)]
pub enum AppIdentification {
    Win32 { path: String },
    Uwp { aumid: String },
    Java { jar: String },
}

#[derive(Debug)]
pub struct Tag {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub background: Color,
}

#[derive(Debug)]
pub struct AppTag {
    pub app_id: u64,
    pub tag_id: u64,
}

#[derive(Debug)]
pub struct Session {
    pub id: u64,
    pub app_id: u64,
    pub arguments: String,
    pub title: String,
}

#[derive(Debug)]
pub struct Usage {
    pub id: u64,
    pub start: Timestamp,
    pub end: Timestamp,
    pub during_idle: bool,
    pub session_id: u64,
}
