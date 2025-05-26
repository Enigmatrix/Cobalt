use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::sqlite::SqliteRow;
use sqlx::{Result, Row, Type};

pub use crate::table::{Color, Duration, Id, Period, Ref, Timestamp};

/// An app that has run on the computer.
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    /// Identified
    pub id: Ref<Self>,
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
    /// Created at
    pub created_at: Timestamp,
    // /// set if the app details have finalized.
    // /// else, all fields except id, identity, created_at, updated_at and initialized_at are empty
    // pub initialized_at: Timestamp,
    /// Updated at
    pub updated_at: Timestamp,
}

/// Unique identity of an [App], outside of the Database (on the FileSystem/Registry)
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "tag")]
pub enum AppIdentity {
    /// UWP App
    #[serde(rename_all = "camelCase")]
    Uwp {
        /// AUMID of the UWP App
        aumid: String,
    },
    /// Win32 App
    #[serde(rename_all = "camelCase")]
    Win32 {
        /// Path of the Win32 App
        path: String,
    },
    /// Website
    #[serde(rename_all = "camelCase")]
    Website {
        /// Base URL of the Website
        base_url: String,
    },
}

impl FromRow<'_, SqliteRow> for AppIdentity {
    fn from_row(row: &SqliteRow) -> Result<Self> {
        let text0 = row.get("identity_text0");
        Ok(match row.get("identity_tag") {
            0 => AppIdentity::Uwp { aumid: text0 },
            1 => AppIdentity::Win32 { path: text0 },
            2 => AppIdentity::Website { base_url: text0 },
            _ => panic!("Unknown identity type"),
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
#[serde(rename_all = "camelCase")]
pub struct Tag {
    /// Identifier
    pub id: Ref<Self>,
    /// Name
    pub name: String,
    /// Color
    pub color: Color,
    /// Created at
    pub created_at: Timestamp,
    /// Updated at
    pub updated_at: Timestamp,
}

/// Non-continuous session of [App] usage
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// Identified
    pub id: Ref<Self>,
    /// Link to [App]
    pub app_id: Ref<App>,
    /// Title of Session
    pub title: String,
    /// URL of Session, if app is a browser
    pub url: Option<String>,
}

/// Continuous usage of [App] in a [Session]
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct SystemEvent {
    /// Identifier
    pub id: Ref<Self>,
    /// Timestamp
    pub timestamp: Timestamp,
    /// Event type
    pub event: SystemEventType,
}

/// Target of an [Alert]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "tag")]
pub enum Target {
    /// [App] Target
    #[serde(rename_all = "camelCase")]
    App {
        /// App Identifier
        id: Ref<App>,
    },
    /// [Tag] Target
    #[serde(rename_all = "camelCase")]
    Tag {
        /// Tag Identifier
        id: Ref<Tag>,
    },
}

impl FromRow<'_, SqliteRow> for Target {
    fn from_row(row: &SqliteRow) -> Result<Self> {
        Ok(if let Some(app_id) = row.get("app_id") {
            Target::App { id: app_id }
        } else {
            Target::Tag {
                id: row.get("tag_id"),
            }
        })
    }
}

impl Default for Target {
    fn default() -> Self {
        Self::App {
            id: Default::default(),
        }
    }
}

/// How long the monitoring duration should be for an [Alert]
#[derive(Default, Debug, Clone, PartialEq, Eq, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "tag")]
pub enum TriggerAction {
    /// Kill the offending App / App in Tags
    Kill,
    /// Dim the windows of the offending App / App in Tags
    #[serde(rename_all = "camelCase")]
    Dim {
        /// Duration to dim over
        duration: Duration,
    },
    /// Send a message to the user as a Toast notification
    #[serde(rename_all = "camelCase")]
    Message {
        /// Contents of the Message
        content: String,
    },
}

impl FromRow<'_, SqliteRow> for TriggerAction {
    fn from_row(row: &SqliteRow) -> Result<Self> {
        let tag = row.get("trigger_action_tag");
        match tag {
            0 => Ok(Self::Kill),
            1 => Ok(Self::Dim {
                duration: row.get("trigger_action_dim_duration"),
            }),
            2 => Ok(Self::Message {
                content: row.get("trigger_action_message_content"),
            }),
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
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    /// Identifier
    pub id: Ref<Self>,
    /// Target of this [Alert]
    #[sqlx(flatten)]
    pub target: Target,
    /// Usage Limit
    pub usage_limit: Duration,
    /// Time Frame
    pub time_frame: TimeFrame,
    #[sqlx(flatten)]
    /// Action to take on trigger
    pub trigger_action: TriggerAction,
    /// Whether this alert is not deleted
    pub active: bool,
    /// Created at
    pub created_at: Timestamp,
    /// Updated at
    pub updated_at: Timestamp,
}

/// Notifications to send upon a certain threshold of an [Alert]'s usage_limit
#[derive(Default, Debug, Clone, FromRow, Serialize, Deserialize)] // can't impl PartialEq, Eq for f64
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    /// Identifier
    pub id: Ref<Self>,
    /// Link to [Alert]
    pub alert_id: Ref<Alert>,
    /// Threshold as 0-1 ratio of the Usage Limit
    pub threshold: f64,
    /// Message to send when the threshold is reached
    pub message: String,
    /// Whether this reminder is not deleted
    pub active: bool,
    /// Created at
    pub created_at: Timestamp,
    /// Updated at
    pub updated_at: Timestamp,
}

impl PartialEq<Reminder> for Reminder {
    fn eq(&self, other: &Reminder) -> bool {
        self.id == other.id
            && self.alert_id == other.alert_id
            && (self.threshold - other.threshold).abs() <= f64::EPSILON
            && self.message == other.message
            && self.active == other.active
    }
}

impl Eq for Reminder {}

/// Reason for the [AlertEvent]/[ReminderEvent]
#[derive(Default, Debug, Clone, PartialEq, Eq, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(i64)]
pub enum Reason {
    #[default]
    /// The [Alert]/[Reminder] was triggered
    Hit,
    /// The [Alert]/[Reminder] was ignored
    Ignored,
}

/// An instance of [Alert] triggering.
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertEvent {
    /// Identifier
    pub id: Ref<Self>,
    ///  Link to [Alert]
    pub alert_id: Ref<Alert>,
    /// Timestamp of the [Alert] trigger
    pub timestamp: Timestamp,
    /// Reason for the [AlertEvent]
    pub reason: Reason,
}

/// An instance of [Reminder] triggering.
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderEvent {
    /// Identifier
    pub id: Ref<Self>,
    ///  Link to [Reminder]
    pub reminder_id: Ref<Reminder>,
    /// Timestamp of the [Reminder] trigger
    pub timestamp: Timestamp,
    /// Reason for the [ReminderEvent]
    pub reason: Reason,
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
    id: Id
);

table!(
    Reminder,
    "reminders",
    id: Id
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
