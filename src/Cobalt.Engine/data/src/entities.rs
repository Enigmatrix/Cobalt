use crate::table::Ref;

pub type Timestamp = u64;
pub type Duration = u64;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub id: Ref<Self>,
    // pub initialized: bool,
    // pub found: bool,
    pub name: String,
    pub description: String,
    pub company: String,
    pub color: String,
    pub identity: AppIdentity,
    // pub icon: Blob
}

/// Unique identity of an App, outside of the Database (on the FileSystem/Registry)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppIdentity {
    Win32 { path: String },
    Uwp { aumid: String },
}

impl Default for AppIdentity {
    fn default() -> Self {
        Self::Win32 {
            path: Default::default(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: Ref<Self>,
    pub name: String,
    pub color: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: Ref<Self>,
    pub app: Ref<App>,
    pub title: String,
    pub cmd_line: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Usage {
    pub id: Ref<Self>,
    pub session: Ref<Session>,
    pub start: Timestamp,
    pub end: Timestamp,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct InteractionPeriod {
    pub id: Ref<Self>,
    pub start: Timestamp,
    pub end: Timestamp,
    pub mouseclicks: u64,
    pub keystrokes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    App(Ref<App>),
    Tag(Ref<Tag>),
}

impl Default for Target {
    fn default() -> Self {
        Self::App(Default::default())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum TimeFrame {
    #[default]
    Daily,
    Weekly = 1,
    Monthly = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Kill,
    Dim(Duration),
    Message(String),
}

impl Default for Action {
    fn default() -> Self {
        Self::Kill
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub id: Ref<Self>,
    pub target: Target,
    pub usage_limit: Duration,
    pub time_frame: TimeFrame,
    pub action: Action,
}

#[derive(Default, Debug, Clone)]
pub struct Reminder {
    pub id: Ref<Self>,
    pub alert: Ref<Alert>,
    pub threshold: f64,
    pub message: String,
}

#[derive(Default, Debug, Clone)]
pub struct AlertHit {
    pub id: Ref<Self>,
    pub alert: Ref<Alert>,
    pub timestamp: Timestamp,
}

#[derive(Default, Debug, Clone)]
pub struct ReminderHit {
    pub id: Ref<Self>,
    pub reminder: Ref<Reminder>,
    pub timestamp: Timestamp,
}

macro_rules! table {
    ($t:ty, $name:expr, $fld:ident : $id:ty, $cols:expr) => {
        impl crate::table::Table for $t {
            type Id = $id;
            fn id(&self) -> &crate::table::Ref<Self> {
                &self.$fld
            }
            fn name() -> &'static str {
                $name
            }
            fn columns() -> &'static [&'static str] {
                &$cols
            }
        }
    };
}

table!(
    App,
    "app",
    id: u64,
    [
        "id",
        "initialized",
        "found",
        "name",
        "description",
        "company",
        "color",
        "identity_tag",
        "identity_text0",
        // We do not list icon as a column, as we do not insert nor query icons.

        // "icon"
    ]
);

table!(
    Tag,
    "tag",
    id: u64,
    ["id", "name", "color"]
);

table!(
    Session,
    "session",
    id: u64,
    [
        "id",
        "app",
        "title",
        "cmd_line"
    ]
);

table!(
    Usage,
    "usage",
    id: u64,
    [
        "id",
        "session",
        "start",
        "end"
    ]
);

table!(
    InteractionPeriod,
    "interaction_period",
    id: u64,
    ["id", "start", "end", "mouseclicks", "keystrokes"]
);

table!(
    Alert,
    "alert",
    id: u64,
    ["id", "target_is_app", "app", "tag", "usage_limit", "time_frame", "action_tag", "action_int0", "action_text0"]
);

table!(
    Reminder,
    "reminder",
    id: u64,
    ["id", "alert", "threshold", "message"]
);

table!(
    AlertHit,
    "alert_hit",
    id: u64,
    ["id", "alert", "timestamp"]
);

table!(
    ReminderHit,
    "reminder_hit",
    id: u64,
    ["id", "reminder", "timestamp"]
);
