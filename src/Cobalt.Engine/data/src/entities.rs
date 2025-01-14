use sqlx::prelude::FromRow;
use sqlx::sqlite::SqliteRow;
use sqlx::{Result, Row, Type};

use crate::table::{AlertVersionedId, ReminderVersionedId};
pub use crate::table::{Color, Duration, Id, Ref, Timestamp, VersionedId};

// Documented in the Cobalt.Common.Data/Entities/*.cs files

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct App {
    pub id: Ref<Self>,
    // pub initialized: bool,
    // pub found: bool,
    pub name: String,
    pub description: String,
    pub company: String,
    pub color: Color,
    #[sqlx(flatten)]
    pub identity: AppIdentity,
    // pub icon: Blob
}

/// Unique identity of an App, outside of the Database (on the FileSystem/Registry)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppIdentity {
    Win32 { path: String },
    Uwp { aumid: String },
}

impl FromRow<'_, SqliteRow> for AppIdentity {
    fn from_row(row: &SqliteRow) -> Result<Self> {
        let text0 = row.get("identity_path_or_aumid");
        Ok(if row.get("identity_is_win32") {
            AppIdentity::Win32 { path: text0 }
        } else {
            AppIdentity::Uwp { aumid: text0 }
        })
    }
}

impl Default for AppIdentity {
    fn default() -> Self {
        Self::Win32 {
            path: Default::default(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Tag {
    pub id: Ref<Self>,
    pub name: String,
    pub color: Color,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Session {
    pub id: Ref<Self>,
    pub app_id: Ref<App>,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Usage {
    pub id: Ref<Self>,
    pub session_id: Ref<Session>,
    pub start: Timestamp,
    pub end: Timestamp,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct InteractionPeriod {
    pub id: Ref<Self>,
    pub start: Timestamp,
    pub end: Timestamp,
    pub mouse_clicks: i64,
    pub key_strokes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    App(Ref<App>),
    Tag(Ref<Tag>),
}

impl FromRow<'_, SqliteRow> for Target {
    fn from_row(row: &SqliteRow) -> Result<Self> {
        Ok(if let Some(app_id) = row.get("app_id") {
            Target::App(app_id)
        } else {
            Target::Tag(row.get("tag_id"))
        })
    }
}

impl Default for Target {
    fn default() -> Self {
        Self::App(Default::default())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Type)]
#[repr(i64)]
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

impl FromRow<'_, SqliteRow> for TriggerAction {
    fn from_row(row: &SqliteRow) -> Result<Self> {
        let tag = row.get("trigger_action_tag");
        match tag {
            0 => Ok(Self::Kill),
            1 => Ok(Self::Dim(row.get("trigger_action_dim_duration"))),
            2 => Ok(Self::Message(row.get("trigger_action_message_content"))),
            tag => Err(sqlx::Error::Decode(
                format!("Unknown trigger action tag = {tag}").into(),
            )),
        }
    }
}

impl Default for TriggerAction {
    fn default() -> Self {
        Self::Kill
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Alert {
    #[sqlx(flatten)]
    pub id: Ref<Self>,
    #[sqlx(flatten)]
    pub target: Target,
    pub usage_limit: Duration,
    pub time_frame: TimeFrame,
    #[sqlx(flatten)]
    pub trigger_action: TriggerAction,
}

#[derive(Default, Debug, Clone, FromRow)] // can't impl PartialEq, Eq for f64
pub struct Reminder {
    #[sqlx(flatten)]
    pub id: Ref<Self>,
    #[sqlx(flatten)]
    pub alert: AlertVersionedId,
    pub threshold: f64,
    pub message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct AlertEvent {
    pub id: Ref<Self>,
    #[sqlx(flatten)]
    pub alert: AlertVersionedId,
    pub timestamp: Timestamp,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct ReminderEvent {
    pub id: Ref<Self>,
    #[sqlx(flatten)]
    pub reminder: ReminderVersionedId,
    pub timestamp: Timestamp,
}

macro_rules! table {
    ($t:ty, $name:expr, $fld:ident : $id:ty) => {
        impl crate::table::Table for $t {
            type Id = $id;
            fn id(&self) -> &crate::table::Ref<Self> {
                &self.$fld
            }
            fn name() -> &'static str {
                $name
            }
        }
    };
}

table!(
    App,
    "apps",
    id: Id
);

table!(
    Tag,
    "tags",
    id: Id
);

table!(
    Session,
    "sessions",
    id: Id
);

table!(
    Usage,
    "usages",
    id: Id
);

table!(
    InteractionPeriod,
    "interaction_periods",
    id: Id
);

table!(
    Alert,
    "alerts",
    id: VersionedId
);

table!(
    Reminder,
    "reminders",
    id: VersionedId
);

table!(
    AlertEvent,
    "alert_events",
    id: Id
);

table!(
    ReminderEvent,
    "reminder_events",
    id: Id
);
