use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::*;
use crate::entities::{TimeFrame, TriggerAction};
use crate::table::{Color, Duration, Real, Score};

/// List of [Ref<T>]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RefVec<T: Table>(pub Vec<Ref<T>>);

impl<T: Table> Deref for RefVec<T> {
    type Target = Vec<Ref<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r, T: Table<Id: FromStr<Err: std::fmt::Debug>>> sqlx::Decode<'r, Sqlite> for RefVec<T> {
    fn decode(
        value: <Sqlite as sqlx::Database>::ValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let str = <String as sqlx::Decode<'r, Sqlite>>::decode(value)?;
        if str.is_empty() {
            return Ok(Self(Vec::new()));
        }

        let inner = str
            .split(',')
            .map(|id| Ref::new(id.parse().unwrap()))
            .collect();

        Ok(Self(inner))
    }
}

impl<T: Table> sqlx::Type<Sqlite> for RefVec<T> {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &<Sqlite as sqlx::Database>::TypeInfo) -> bool {
        <String as sqlx::Type<Sqlite>>::compatible(ty)
    }
}

/// Duration grouped into target
#[derive(Clone, Debug, Default, PartialEq, Eq, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithDuration<T: Table> {
    /// Target Identifier
    pub id: Ref<T>,
    /// Duration value
    pub duration: Duration,
}

/// Duration grouped into target, period chunks
#[derive(Clone, Debug, Default, PartialEq, Eq, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithGroupedDuration<T: Table> {
    /// Target Identifier
    pub id: Ref<T>,
    /// Time Period group
    pub group: crate::table::Timestamp,
    /// Duration value
    pub duration: Duration,
}

/// Duration grouped into target, period chunks
#[derive(Clone, Debug, Default, PartialEq, Eq, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithGroup<T> {
    /// Time Period group
    pub group: crate::table::Timestamp,
    /// Value
    pub value: T,
}

/// Value per common periods
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ValuePerPeriod<T> {
    /// Value today
    pub today: T,
    /// Value this week
    pub week: T,
    /// Value this month
    pub month: T,
}

/// Options to update a [super::App]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedApp {
    /// Identifier
    pub id: Ref<super::App>,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Company
    pub company: String,
    /// Color
    pub color: Color,
    /// Linked [super::Tag]
    pub tag_id: Option<Ref<super::Tag>>,
}

/// [super::App] with additional information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct App {
    #[sqlx(flatten)]
    #[serde(flatten)]
    /// [super::App] itself
    pub inner: super::App,
    /// List of linked [super::App]s
    #[sqlx(flatten)]
    /// Usage Info
    pub usages: ValuePerPeriod<Duration>,
}

/// [super::Tag] with additional information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    #[sqlx(flatten)]
    #[serde(flatten)]
    /// [super::Tag] itself
    pub inner: super::Tag,
    /// List of linked [super::App]s
    pub apps: RefVec<super::App>,
    #[sqlx(flatten)]
    /// Usage Info
    pub usages: ValuePerPeriod<Duration>,
}

/// Status of a [super::Alert]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "tag")]
pub enum AlertTriggerStatus {
    /// Hit
    #[serde(rename_all = "camelCase")]
    Hit {
        /// Timestamp
        timestamp: Timestamp,
    },
    #[serde(rename_all = "camelCase")]
    /// Ignored
    Ignored {
        /// Timestamp
        timestamp: Timestamp,
    },
    /// Not yet hit
    Untriggered,
}

impl FromRow<'_, SqliteRow> for AlertTriggerStatus {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let status = row.get("alert_status");
        let timestamp = row.get("alert_status_timestamp");
        match status {
            Some(0) => Ok(AlertTriggerStatus::Hit { timestamp }),
            Some(1) => Ok(AlertTriggerStatus::Ignored { timestamp }),
            _ => Ok(AlertTriggerStatus::Untriggered),
        }
    }
}

/// Status of a [super::Reminder]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "tag")]
pub enum ReminderTriggerStatus {
    /// Hit
    #[serde(rename_all = "camelCase")]
    Hit {
        /// Timestamp
        timestamp: Timestamp,
    },
    /// Ignored
    #[serde(rename_all = "camelCase")]
    Ignored {
        /// Timestamp
        timestamp: Timestamp,
        /// Ignored because the alert itself was ignored.
        ignored_by_alert: bool,
    },
    /// Not yet hit
    Untriggered,
}

impl FromRow<'_, SqliteRow> for ReminderTriggerStatus {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let status = row.get("reminder_status");
        let timestamp = row.get("reminder_status_timestamp");
        let ignored_by_alert = row.get("reminder_status_alert_ignored");
        match status {
            Some(0) => Ok(ReminderTriggerStatus::Hit { timestamp }),
            Some(1) => Ok(ReminderTriggerStatus::Ignored {
                timestamp,
                ignored_by_alert,
            }),
            _ => Ok(ReminderTriggerStatus::Untriggered),
        }
    }
}

/// [super::Alert] with additional information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    /// Identifier
    pub id: Ref<super::Alert>,
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
    /// Created at
    pub created_at: Timestamp,
    /// Updated at
    pub updated_at: Timestamp,
    /// List of linked [Reminder]s
    pub reminders: Vec<Reminder>,
    /// Status of the [super::Alert]
    #[sqlx(flatten)]
    pub status: AlertTriggerStatus,
    /// List of hit [super::AlertEvent]s
    #[sqlx(flatten)]
    pub events: ValuePerPeriod<i64>,
}

/// [super::Reminder] with additional information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    /// Identifier
    pub id: Ref<super::Reminder>,
    /// Link to [Alert]
    pub alert_id: Ref<super::Alert>,
    /// Threshold as 0-1 ratio of the Usage Limit
    pub threshold: Real,
    /// Message to send when the threshold is reached
    pub message: String,
    /// Created at
    pub created_at: Timestamp,
    /// Updated at
    pub updated_at: Timestamp,
    /// Status of the [super::Reminder]
    #[sqlx(flatten)]
    pub status: ReminderTriggerStatus,
    /// List of hit [super::ReminderEvent]s
    #[sqlx(flatten)]
    pub events: ValuePerPeriod<i64>,
}

/// Options to create a new [super::Tag]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTag {
    /// Name
    pub name: String,
    /// Color
    pub color: String,
    /// Score as a real number from -100 to 100
    pub score: Real,
    /// Apps List
    pub apps: Vec<Ref<super::App>>,
}

/// Options to update a [super::Tag]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedTag {
    /// Identifier
    pub id: Ref<super::Tag>,
    /// Name
    pub name: String,
    /// Score as a real number from -100 to 100
    pub score: Real,
    /// Color
    pub color: String,
}

/// Options to create a new [super::Alert]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlert {
    /// Target of this [Alert]
    pub target: Target,
    /// Usage Limit
    pub usage_limit: Duration,
    /// Time Frame
    pub time_frame: TimeFrame,
    /// Action to take on trigger
    pub trigger_action: TriggerAction,
    /// Reminders
    pub reminders: Vec<CreateReminder>,
    /// Whether to ignore the trigger
    pub ignore_trigger: bool,
}

/// Options to create a new [super::Reminder]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReminder {
    /// Threshold
    pub threshold: Real,
    /// Message
    pub message: String,
    /// Whether to ignore the trigger
    pub ignore_trigger: bool,
}

/// Options to update a [super::Alert]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedAlert {
    /// Identifier
    pub id: Ref<super::Alert>,
    /// Target of this [Alert]
    pub target: Target,
    /// Usage Limit
    pub usage_limit: Duration,
    /// Time Frame
    pub time_frame: TimeFrame,
    /// Action to take on trigger
    pub trigger_action: TriggerAction,
    /// Reminders
    pub reminders: Vec<UpdatedReminder>,
    /// Whether to ignore the trigger
    pub ignore_trigger: bool,
}

/// Options to update a [super::Reminder]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedReminder {
    /// Identifier
    pub id: Option<Ref<super::Reminder>>,
    /// Threshold
    pub threshold: Real,
    /// Message
    pub message: String,
    /// Whether to ignore the trigger
    pub ignore_trigger: bool,
}

/// [super::Session] with additional information
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// Identifier
    pub id: Ref<super::Session>,
    /// Title of Session
    pub title: String,
    /// URL of Session
    pub url: Option<String>,
    /// Minimum Usage of Usages
    pub start: Timestamp,
    /// Maximum Usage of Usages
    pub end: Timestamp,
    /// Usages
    pub usages: Vec<super::Usage>,
}

/// [Session]s with [Usage]s, partitioned by [App]s
pub type AppSessionUsages = HashMap<Ref<super::App>, HashMap<Ref<super::Session>, Session>>;

/// Represents a streak of time that is either focused or distractive.
#[derive(FromRow, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Streak {
    /// The start time of the streak.
    pub start: Timestamp,
    /// The end time of the streak.
    pub end: Timestamp,
    /// Whether the streak is a focused streak. If false, it is a distractive streak.
    pub is_focused: bool,
}

/// Settings for focus streaks.
#[derive(FromRow, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusStreakSettings {
    /// The minimum score of a focused app.
    pub min_focus_score: Score,
    /// The minimum duration of a focused usage.
    pub min_focus_usage_dur: Duration,
    /// The maximum gap between two focused streaks.
    pub max_focus_gap: Duration,
}

impl From<util::config::FocusStreakSettings> for FocusStreakSettings {
    fn from(settings: util::config::FocusStreakSettings) -> Self {
        Self {
            min_focus_score: settings.min_focus_score.into(),
            min_focus_usage_dur: to_ticks(settings.min_focus_usage_dur),
            max_focus_gap: to_ticks(settings.max_focus_gap),
        }
    }
}

impl From<FocusStreakSettings> for util::config::FocusStreakSettings {
    fn from(settings: FocusStreakSettings) -> Self {
        Self {
            min_focus_score: settings.min_focus_score.into(),
            min_focus_usage_dur: from_ticks(settings.min_focus_usage_dur),
            max_focus_gap: from_ticks(settings.max_focus_gap),
        }
    }
}

/// Settings for distractive streaks.
#[derive(FromRow, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistractiveStreakSettings {
    /// The maximum score of a distractive app.
    pub max_distractive_score: Score,
    /// The minimum duration of a distractive usage.
    pub min_distractive_usage_dur: Duration,
    /// The maximum gap between two distractive streaks.
    pub max_distractive_gap: Duration,
}

impl From<util::config::DistractiveStreakSettings> for DistractiveStreakSettings {
    fn from(settings: util::config::DistractiveStreakSettings) -> Self {
        Self {
            max_distractive_score: settings.max_distractive_score.into(),
            min_distractive_usage_dur: to_ticks(settings.min_distractive_usage_dur),
            max_distractive_gap: to_ticks(settings.max_distractive_gap),
        }
    }
}

impl From<DistractiveStreakSettings> for util::config::DistractiveStreakSettings {
    fn from(settings: DistractiveStreakSettings) -> Self {
        Self {
            max_distractive_score: settings.max_distractive_score.into(),
            min_distractive_usage_dur: from_ticks(settings.min_distractive_usage_dur),
            max_distractive_gap: from_ticks(settings.max_distractive_gap),
        }
    }
}

fn to_ticks(duration: std::time::Duration) -> i64 {
    (duration.as_nanos() / 100) as i64
}

fn from_ticks(ticks: i64) -> std::time::Duration {
    std::time::Duration::from_nanos((ticks * 100) as u64)
}
