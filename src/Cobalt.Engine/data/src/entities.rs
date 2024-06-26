pub use crate::table::{Color, Duration, Id, Ref, Timestamp, VersionedId};

// Documented in the Cobalt.Common.Data/Entities/*.cs files

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub id: Ref<Self>,
    // pub initialized: bool,
    // pub found: bool,
    pub name: String,
    pub description: String,
    pub company: String,
    pub color: Color,
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
    pub color: Color,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: Ref<Self>,
    pub app: Ref<App>,
    pub title: String,
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
    pub mouse_clicks: u64,
    pub key_strokes: u64,
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
    Daily = 0,
    Weekly = 1,
    Monthly = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerAction {
    Kill,
    Dim(Duration),
    Message(String),
}

impl Default for TriggerAction {
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
    pub trigger_action: TriggerAction,
}

#[derive(Default, Debug, Clone)] // can't impl PartialEq, Eq for f64
pub struct Reminder {
    pub id: Ref<Self>,
    pub alert: Ref<Alert>,
    pub threshold: f64,
    pub message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AlertEvent {
    pub id: Ref<Self>,
    pub alert: Ref<Alert>,
    pub timestamp: Timestamp,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ReminderEvent {
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
    "apps",
    id: Id,
    [
        "id",
        "initialized",
        "found", // TODO do i still need this?
        "name",
        "description",
        "company",
        "color",
        "identity_is_win32",
        "identity_path_or_aumid",
        // We do not insert nor query icons, but we put it here anyway
        "icon"
    ]
);

table!(
    Tag,
    "tags",
    id: Id,
    ["id", "name", "color"]
);

table!(
    Session,
    "sessions",
    id: Id,
    [
        "id",
        "app_id",
        "title",
    ]
);

table!(
    Usage,
    "usages",
    id: Id,
    [
        "id",
        "session_id",
        "start",
        "end"
    ]
);

table!(
    InteractionPeriod,
    "interaction_periods",
    id: Id,
    ["id", "start", "end", "mouse_clicks", "key_strokes"]
);

table!(
    Alert,
    "alerts",
    id: VersionedId,
    ["id", "version", "app_id", "tag_id", "usage_limit", "time_frame", "trigger_action_dim_duration", "trigger_action_message_content", "trigger_action_tag"]
);

table!(
    Reminder,
    "reminders",
    id: VersionedId,
    ["id", "version", "alert_id", "alert_version", "threshold", "message"]
);

table!(
    AlertEvent,
    "alert_events",
    id: Id,
    ["id", "alert_id", "alert_version", "timestamp"]
);

table!(
    ReminderEvent,
    "reminder_events",
    id: Id,
    ["id", "reminder_id", "reminder_version", "timestamp"]
);
