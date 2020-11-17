pub type Timestamp = native::wrappers::Timestamp;
pub type Color = String;
pub type Id = u64;

#[derive(Clone, Debug)]
pub struct App {
    pub id: Id,
    pub name: String,
    pub description: String,
    // pub icon: Blob,
    pub identity: AppIdentity,
    pub color: Color,
}

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum AppIdentity {
    Win32 { path: String },
    UWP { aumid: String },
    // Java { jar_file: String }
}

#[derive(Clone, Debug)]
pub struct Tag {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: Id,
    pub app_id: Id,
    pub arguments: Option<String>,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct Usage {
    pub id: Id,
    pub sess_id: Id,
    pub start: Timestamp,
    pub end: Timestamp,
    pub idle: bool,
}
