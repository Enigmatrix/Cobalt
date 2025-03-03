use serde::Serialize;
use sqlx::prelude::FromRow;
use sqlx::sqlite::SqliteRow;
use sqlx::{Result, Row, Type};

use crate::table::{AlertVersionedId, ReminderVersionedId};
pub use crate::table::{Color, Duration, Id, Ref, Timestamp, VersionedId};

/// An app that has run on the computer.
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct App {
    /// Identified
    pub id: Ref<Self>,
    // /// true if the app details have finalized.
    // /// else, all fields except id, identity and initialized are empty
    // pub initialized: bool,
    // /// only used for [UsageWriter::find_or_insert_app]
    // pub found: bool,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Company
    pub company: String,
    /// Color
    pub color: Color,
    #[sqlx(flatten)]
    /// Unique identity of an [App]
    pub identity: AppIdentity,
    /// Icon, in bytes
    pub icon: Option<Vec<u8>>,
    /// Link to [Tag]
    pub tag_id: Option<Ref<Tag>>,
}

/// Unique identity of an [App], outside of the Database (on the FileSystem/Registry)
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum AppIdentity {
    /// Win32 App
    Win32 {
        /// Path of the Win32 App
        path: String,
    },
    /// UWP App
    Uwp {
        /// AUMID of the UWP App
        aumid: String,
    },
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

/// Collection of [App] with a name
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Tag {
    /// Identifier
    pub id: Ref<Self>,
    /// Name
    pub name: String,
    /// Color
    pub color: Color,
}

/// Non-continuous session of [App] usage
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Session {
    /// Identified
    pub id: Ref<Self>,
    /// Link to [App]
    pub app_id: Ref<App>,
    /// Title of Session
    pub title: String,
}

/// Continuous usage of [App] in a [Session]
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Usage {
    /// Identifier
    pub id: Ref<Self>,
    /// Link to [Session]
    pub session_id: Ref<Session>,
    /// Start timestamp
    pub start: Timestamp,
    /// End timestamp
    pub end: Timestamp,
}

/// A period of continuous usage without idling
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct InteractionPeriod {
    /// Identifier
    pub id: Ref<Self>,
    /// Start timestamp
    pub start: Timestamp,
    /// End timestamp
    pub end: Timestamp,
    /// Number of mouse clicks in the interaction period
    pub mouse_clicks: i64,
    /// Number of key strokes in the interaction period
    pub key_strokes: i64,
}

type SystemEventType = i64; // TODO define enum

/// A system event
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct SystemEvent {
    /// Identifier
    pub id: Ref<Self>,
    /// Timestamp
    pub timestamp: Timestamp,
    /// Event type
    pub event: SystemEventType,
}

/// Target of an [Alert]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Target {
    /// [App] Target
    App(Ref<App>),
    /// [Tag] Target
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

/// How long the monitoring duration should be for an [Alert]
#[derive(Default, Debug, Clone, PartialEq, Eq, Type, Serialize)]
#[repr(i64)]
pub enum TimeFrame {
    #[default]
    /// Daily
    Daily = 0,
    /// Weekly
    Weekly = 1,
    /// Monthly
    Monthly = 2,
}

/// Action to take once the [Alert]'s Usage Limit has been reached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TriggerAction {
    /// Kill the offending App / App in Tags
    Kill,
    /// Dim the windows of the offending App / App in Tags
    Dim(Duration),
    /// Send a message to the user as a Toast notification
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

/// Monitoring record describing an usage limit for how long you use an [App]
/// or a collection of [App]s under a [Tag], the actions to take when that
/// limit is reached, and the reminders
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Alert {
    #[sqlx(flatten)]
    /// Identifier
    pub id: Ref<Self>,
    #[sqlx(flatten)]
    /// Target of this [Alert]
    pub target: Target,
    /// Usage Limit
    pub usage_limit: Duration,
    /// Time Frame
    pub time_frame: TimeFrame,
    #[sqlx(flatten)]
    /// Action to take on trigger
    pub trigger_action: TriggerAction,
}

/// Notifications to send upon a certain threshold of an [Alert]'s usage_limit
#[derive(Default, Debug, Clone, FromRow, Serialize)] // can't impl PartialEq, Eq for f64
pub struct Reminder {
    #[sqlx(flatten)]
    /// Identifier
    pub id: Ref<Self>,
    #[sqlx(flatten)]
    /// Link to [Alert]
    pub alert: AlertVersionedId,
    /// Threshold as 0-1 ratio of the Usage Limit
    pub threshold: f64,
    /// Message to send when the threshold is reached
    pub message: String,
}

/// An instance of [Alert] triggering.
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct AlertEvent {
    /// Identifier
    pub id: Ref<Self>,
    #[sqlx(flatten)]
    ///  Link to [Alert]
    pub alert: AlertVersionedId,
    /// Timestamp of the [Alert] trigger
    pub timestamp: Timestamp,
}

/// An instance of [Reminder] triggering.
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct ReminderEvent {
    /// Identifier
    pub id: Ref<Self>,
    #[sqlx(flatten)]
    ///  Link to [Reminder]
    pub reminder: ReminderVersionedId,
    /// Timestamp of the [Reminder] trigger
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
    SystemEvent,
    "system_events",
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
