use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::*;
use crate::entities::{TimeFrame, TriggerAction};
use crate::table::{Color, Duration};

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
pub struct WithDuration<T: Table> {
    /// Target Identifier
    pub id: Ref<T>,
    /// Duration value
    pub duration: Duration,
}

/// Duration grouped into target, period chunks
#[derive(Clone, Debug, Default, PartialEq, Eq, FromRow, Serialize)]
pub struct WithGroupedDuration<T: Table> {
    /// Target Identifier
    pub id: Ref<T>,
    /// Time Period group
    pub group: crate::table::Timestamp,
    /// Duration value
    pub duration: Duration,
}

/// Value per common periods
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
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

/// [super::Alert] with additional information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
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
    /// List of hit [super::AlertEvent]s
    #[sqlx(flatten)]
    pub events: ValuePerPeriod<i64>,
}

/// [super::Reminder] with additional information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Reminder {
    /// Identifier
    pub id: Ref<super::Reminder>,
    /// Link to [Alert]
    pub alert_id: Ref<super::Alert>,
    /// Threshold as 0-1 ratio of the Usage Limit
    pub threshold: f64,
    /// Message to send when the threshold is reached
    pub message: String,
    /// Created at
    pub created_at: Timestamp,
    /// Updated at
    pub updated_at: Timestamp,
    /// List of hit [super::ReminderEvent]s
    #[sqlx(flatten)]
    pub events: ValuePerPeriod<i64>,
}

impl PartialEq<Reminder> for Reminder {
    fn eq(&self, other: &Reminder) -> bool {
        self.id == other.id
            && self.alert_id == other.alert_id
            && (self.threshold - other.threshold).abs() <= f64::EPSILON
            && self.message == other.message
            && self.events == other.events
            && self.created_at == other.created_at
            && self.updated_at == other.updated_at
    }
}

impl Eq for Reminder {}

/// Options to create a new [super::Tag]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTag {
    /// Name
    pub name: String,
    /// Color
    pub color: String,
    /// Apps List
    pub apps: Vec<Ref<super::App>>,
}

/// Options to update a [super::Tag]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdatedTag {
    /// Identifier
    pub id: Ref<super::Tag>,
    /// Name
    pub name: String,
    /// Color
    pub color: String,
}

/// Options to create a new [super::Alert]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
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
}

/// Options to create a new [super::Reminder]
#[derive(Debug, Clone, Deserialize)]
pub struct CreateReminder {
    /// Threshold
    pub threshold: f64,
    /// Message
    pub message: String,
}

impl PartialEq<CreateReminder> for CreateReminder {
    fn eq(&self, other: &CreateReminder) -> bool {
        (self.threshold - other.threshold).abs() <= f64::EPSILON && self.message == other.message
    }
}

impl Eq for CreateReminder {}

/// Options to update a [super::Alert]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
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
}

/// Options to update a [super::Reminder]
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatedReminder {
    /// Identifier
    pub id: Ref<super::Reminder>,
    /// Threshold
    pub threshold: f64,
    /// Message
    pub message: String,
    /// Whether this reminder is not deleted
    pub active: bool,
}

impl PartialEq<UpdatedReminder> for UpdatedReminder {
    fn eq(&self, other: &UpdatedReminder) -> bool {
        self.id == other.id
            && (self.threshold - other.threshold).abs() <= f64::EPSILON
            && self.message == other.message
            && self.active == other.active
    }
}

impl Eq for UpdatedReminder {}

/// [super::Session] with additional information
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Session {
    /// Identifier
    pub id: Ref<super::Session>,
    /// Title of Session
    pub title: String,
    /// Minimum Usage of Usages
    pub start: Timestamp,
    /// Maximum Usage of Usages
    pub end: Timestamp,
    /// Usages
    pub usages: Vec<super::Usage>,
}

/// [Session]s with [Usage]s, partitioned by [App]s
pub type AppSessionUsages = HashMap<Ref<super::App>, HashMap<Ref<super::Session>, Session>>;
