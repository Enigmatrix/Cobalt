use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use serde::Serialize;
use sqlx::SqliteExecutor;

use super::*;
use crate::entities::{Duration, TriggerAction};
use crate::table::{AlertVersionedId, VersionedId};

/// Collection of methods to do large, complicated queries against apps etc.
pub struct Repository {
    pub(crate) db: Database,
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

/// Entities with extra information embedded.
pub mod infused {
    use serde::Deserialize;

    use super::*;
    use crate::entities::{TimeFrame, TriggerAction};
    use crate::table::{Color, VersionedId};

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
        usages: ValuePerPeriod<Duration>,
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
        usages: ValuePerPeriod<Duration>,
    }

    /// [super::Alert] with additional information
    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct Alert {
        #[sqlx(flatten)]
        #[serde(flatten)]
        /// [super::Alert] itself
        pub inner: super::Alert,
        /// List of linked [Reminder]s
        pub reminders: Vec<Reminder>,
        /// List of hit [super::AlertEvent]s
        #[sqlx(flatten)]
        pub events: ValuePerPeriod<i64>,
    }

    /// [super::Reminder] with additional information
    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct Reminder {
        #[sqlx(flatten)]
        /// Identifier
        pub id: Ref<super::Reminder>,
        #[sqlx(flatten)]
        /// Link to [Alert]
        pub alert: AlertVersionedId,
        /// Threshold as 0-1 ratio of the Usage Limit
        pub threshold: f64,
        /// Message to send when the threshold is reached
        pub message: String,
        /// List of hit [super::ReminderEvent]s
        #[sqlx(flatten)]
        pub events: ValuePerPeriod<i64>,
    }

    /// Options to create a new [super::Tag]
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct CreateTag {
        /// Name
        pub name: String,
        /// Color
        pub color: String,
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
    #[derive(Debug, Clone, Deserialize)]
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

    /// Options to update a [super::Alert]
    #[derive(Debug, Clone, Deserialize)]
    pub struct UpdatedAlert {
        /// Identifier
        #[serde(flatten)]
        pub id: VersionedId,
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
        #[serde(flatten)]
        pub id: VersionedId,
        /// Threshold
        pub threshold: f64,
        /// Message
        pub message: String,
        /// Whether this reminder is not deleted
        pub active: bool,
    }

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
}

const APP_DUR: &str = "SELECT a.id AS id,
                COALESCE(SUM(MIN(u.end, p.end) - MAX(u.start, p.start)), 0) AS duration
            FROM apps a, (SELECT ? AS start, ? AS end) p
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > p.start AND u.start <= p.end
            GROUP BY a.id";

const TAG_DUR: &str = "SELECT a.tag_id AS id,
                COALESCE(SUM(MIN(u.end, p.end) - MAX(u.start, p.start)), 0) AS duration
            FROM apps a, (SELECT ? AS start, ? AS end) p
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > p.start AND u.start <= p.end
            GROUP BY a.tag_id";

const ALERT_EVENT_COUNT: &str =
    "SELECT e.alert_id, e.alert_version, COUNT(e.id) FROM alert_events e
            WHERE timestamp BETWEEN ? AND ?
            GROUP BY e.alert_id, e.alert_version";

const REMINDER_EVENT_COUNT: &str =
    "SELECT e.reminder_id, e.reminder_version, COUNT(e.id) FROM reminder_events e
            WHERE timestamp BETWEEN ? AND ?
            GROUP BY e.reminder_id, e.reminder_version";

// TODO when we sqlx has named parameters, fixup our queries to use them.

impl Repository {
    /// Initialize a [Repository] from a given [Database]
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self { db })
    }

    /// Update all [Usage]s, such that the last usage's end is equal to `last`
    pub async fn update_usages_set_last(&mut self, last: Timestamp) -> Result<()> {
        let last_usage: i64 = self
            .db
            .executor()
            .fetch_one("SELECT MAX(end) FROM usages")
            .await?
            .get(0);
        let delta = last - last_usage;
        query("UPDATE usages SET start = start + ?, end = end + ?")
            .persistent(false)
            .bind(delta)
            .bind(delta)
            .execute(self.db.executor())
            .await?;
        Ok(())
    }

    /// Gets all [App]s from the database
    pub async fn get_apps(
        &mut self,
        ts: impl TimeSystem,
    ) -> Result<HashMap<Ref<App>, infused::App>> {
        // 1+N (well, 1+1) query pattern here - introduces a little gap for
        // race condition but doesn't matter much. This query pattern doesn't
        // significantly affect the performance of the application for SQLite
        // compared to other DB formats.

        let apps: Vec<infused::App> = query_as(&format!(
            "WITH
                usage_today(id, dur) AS ({APP_DUR}),
                usage_week(id, dur) AS ({APP_DUR}),
                usage_month(id, dur) AS ({APP_DUR})
            SELECT a.*,
                COALESCE(d.dur, 0) AS today,
                COALESCE(w.dur, 0) AS week,
                COALESCE(m.dur, 0) AS month
            FROM apps a
                LEFT JOIN usage_today d ON a.id = d.id
                LEFT JOIN usage_week  w ON a.id = w.id
                LEFT JOIN usage_month m ON a.id = m.id
            WHERE a.initialized = 1
            GROUP BY a.id"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.week_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.month_start(true).to_ticks())
        .bind(i64::MAX)
        .fetch_all(self.db.executor())
        .await?;

        let apps: HashMap<_, _> = apps
            .into_iter()
            .map(|app| (app.inner.id.clone(), app))
            .collect();

        Ok(apps)
    }

    /// Gets all [Tag]s from the database
    pub async fn get_tags(
        &mut self,
        ts: impl TimeSystem,
    ) -> Result<HashMap<Ref<Tag>, infused::Tag>> {
        let tags: Vec<infused::Tag> = query_as(&format!(
            "WITH
                usage_today(id, dur) AS ({TAG_DUR}),
                usage_week(id, dur) AS ({TAG_DUR}),
                usage_month(id, dur) AS ({TAG_DUR})
            SELECT t.*, GROUP_CONCAT(a.id, ',') apps,
                COALESCE(d.dur, 0) AS today,
                COALESCE(w.dur, 0) AS week,
                COALESCE(m.dur, 0) AS month
            FROM tags t
                LEFT JOIN usage_today d ON t.id = d.id
                LEFT JOIN usage_week  w ON t.id = w.id
                LEFT JOIN usage_month m ON t.id = m.id
                LEFT JOIN apps a ON t.id = a.tag_id AND a.initialized = 1
            GROUP BY t.id"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.week_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.month_start(true).to_ticks())
        .bind(i64::MAX)
        .fetch_all(self.db.executor())
        .await?;

        let tags: HashMap<_, _> = tags
            .into_iter()
            .map(|tag| (tag.inner.id.clone(), tag))
            .collect();

        Ok(tags)
    }

    /// Gets all [Alert]s from the database
    pub async fn get_alerts(
        &mut self,
        ts: impl TimeSystem,
    ) -> Result<HashMap<Ref<Alert>, infused::Alert>> {
        #[derive(FromRow)]
        pub struct AlertNoReminders {
            #[sqlx(flatten)]
            inner: super::Alert,
            #[sqlx(flatten)]
            events: infused::ValuePerPeriod<i64>,
        }

        let alerts: Vec<AlertNoReminders> = query_as(&format!(
            "WITH
                events_today(id, version, count) AS ({ALERT_EVENT_COUNT}),
                events_week(id, version, count) AS ({ALERT_EVENT_COUNT}),
                events_month(id, version, count) AS ({ALERT_EVENT_COUNT})
            SELECT al.*,
                COALESCE(t.count, 0) AS today,
                COALESCE(w.count, 0) AS week,
                COALESCE(m.count, 0) AS month
            FROM alerts al
                LEFT JOIN events_today t ON t.id = al.id AND t.version = al.version
                LEFT JOIN events_week w ON w.id = al.id AND w.version = al.version
                LEFT JOIN events_month m ON m.id = al.id AND m.version = al.version
            GROUP BY al.id
            HAVING al.version = MAX(al.version)"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.week_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.month_start(true).to_ticks())
        .bind(i64::MAX)
        .fetch_all(self.db.executor())
        .await?;

        // TODO maybe instad of i64:max for all of these we should use till ::now() (from query_options)
        let reminders: Vec<infused::Reminder> = query_as(&format!(
            "WITH
                events_today(id, version, count) AS ({REMINDER_EVENT_COUNT}),
                events_week(id, version, count) AS ({REMINDER_EVENT_COUNT}),
                events_month(id, version, count) AS ({REMINDER_EVENT_COUNT})
            SELECT r.*,
                COALESCE(t.count, 0) AS today,
                COALESCE(w.count, 0) AS week,
                COALESCE(m.count, 0) AS month
            FROM reminders r
                LEFT JOIN events_today t ON t.id = r.id AND t.version = r.version
                LEFT JOIN events_week w ON w.id = r.id AND w.version = r.version
                LEFT JOIN events_month m ON m.id = r.id AND m.version = r.version
            GROUP BY r.id
            HAVING r.version = MAX(r.version) AND r.active <> 0"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.week_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.month_start(true).to_ticks())
        .bind(i64::MAX)
        .fetch_all(self.db.executor())
        .await?;

        let mut alerts: HashMap<_, _> = alerts
            .into_iter()
            .map(|alert| {
                (
                    alert.inner.id.clone(),
                    infused::Alert {
                        inner: alert.inner,
                        events: alert.events,
                        reminders: Vec::new(),
                    },
                )
            })
            .collect();

        reminders.into_iter().for_each(|reminder| {
            let alert_id = Ref::new(reminder.alert.clone().into());
            if let Some(alert) = alerts.get_mut(&alert_id) {
                alert.reminders.push(reminder);
            }
        });

        Ok(alerts)
    }

    /// Gets all [App]s and its total usage duration in a start-end range.
    /// Assumes start <= end.
    pub async fn get_app_durations(
        &mut self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<HashMap<Ref<App>, WithDuration<App>>> {
        // yes, we need to do this WITH expression and COALESCE
        // once more so that all apps are present in the result.

        let app_durs = query_as(&format!(
            "WITH appdur(id, dur) AS ({APP_DUR})
            SELECT a.*, COALESCE(d.dur, 0) AS duration
            FROM apps a
                LEFT JOIN appdur d ON a.id = d.id"
        ))
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(app_durs
            .into_iter()
            .map(|app_dur: WithDuration<App>| (app_dur.id.clone(), app_dur))
            .collect())
    }

    /// Gets all [App]s and its total usage duration in a start-end range,
    /// grouped per period. Assumes start <= end, and that start and end are
    /// aligned in multiples of period.
    pub async fn get_app_durations_per_period(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        period: Duration,
    ) -> Result<HashMap<Ref<App>, Vec<WithGroupedDuration<App>>>> {
        // This expression is surprisingly slow compared to its previous
        // iteration (tho more correct ofc). Fix it later.
        let app_durs = query_as(
            "WITH RECURSIVE
            params(period, start, end) AS (SELECT ?, ?, ?),
            period_intervals AS (
                SELECT a.id AS id,
                    p.start + (p.period * CAST((u.start - p.start) / p.period AS INT)) AS period_start,
                    p.start + (p.period * (CAST((u.start - p.start) / p.period AS INT) + 1)) AS period_end,
                    u.start AS usage_start,
                    MIN(u.end, p.end) AS usage_end
                FROM apps a, params p
                INNER JOIN sessions s ON a.id = s.app_id
                INNER JOIN usages u ON s.id = u.session_id
                WHERE u.end > p.start AND u.start <= p.end

                UNION ALL

                SELECT id,
                    period_start + p.period AS period_start,
                    period_end + p.period AS period_end,
                    usage_start,
                    usage_end
                FROM period_intervals, params p
                WHERE period_start + p.period < MIN(usage_end, p.end)
            )

            SELECT id,
                period_start AS `group`,
                SUM(MIN(period_end, usage_end) - MAX(period_start, usage_start)) AS duration
            FROM period_intervals
            GROUP BY id, period_start;",
        )
        .bind(period)
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(app_durs.into_iter().fold(
            HashMap::new(),
            |mut acc, app_dur: WithGroupedDuration<App>| {
                acc.entry(app_dur.id.clone()).or_default().push(app_dur);
                acc
            },
        ))
    }

    // TODO write tests

    /// Gets all [Tags]s and its total usage duration in a start-end range,
    /// grouped per period. Assumes start <= end, and that start and end are
    /// aligned in multiples of period.
    pub async fn get_tag_durations_per_period(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        period: Duration,
    ) -> Result<HashMap<Ref<Tag>, Vec<WithGroupedDuration<Tag>>>> {
        // This expression is surprisingly slow compared to its previous
        // iteration (tho more correct ofc). Fix it later.
        let app_durs = query_as(
            "WITH RECURSIVE
            params(period, start, end) AS (SELECT ?, ?, ?),
            period_intervals AS (
                SELECT a.tag_id AS id,
                    p.start + (p.period * CAST((u.start - p.start) / p.period AS INT)) AS period_start,
                    p.start + (p.period * (CAST((u.start - p.start) / p.period AS INT) + 1)) AS period_end,
                    u.start AS usage_start,
                    MIN(u.end, p.end) AS usage_end
                FROM apps a, params p
                INNER JOIN sessions s ON a.id = s.app_id
                INNER JOIN usages u ON s.id = u.session_id
                WHERE u.end > p.start AND u.start <= p.end

                UNION ALL

                SELECT id,
                    period_start + p.period AS period_start,
                    period_end + p.period AS period_end,
                    usage_start,
                    usage_end
                FROM period_intervals, params p
                WHERE period_start + p.period < MIN(usage_end, p.end)
            )

            SELECT id,
                period_start AS `group`,
                SUM(MIN(period_end, usage_end) - MAX(period_start, usage_start)) AS duration
            FROM period_intervals
            GROUP BY id, period_start;",
        )
        .bind(period)
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(app_durs.into_iter().fold(
            HashMap::new(),
            |mut acc, tag_dur: WithGroupedDuration<Tag>| {
                acc.entry(tag_dur.id.clone()).or_default().push(tag_dur);
                acc
            },
        ))
    }

    /// Update the [App] with additional information
    pub async fn update_app(&mut self, app: &infused::UpdatedApp) -> Result<()> {
        query(
            "UPDATE apps SET
                    name = ?,
                    description = ?,
                    company = ?,
                    color = ?,
                    tag_id = ?,
                    initialized = 1
                WHERE id = ?",
        )
        .bind(&app.name)
        .bind(&app.description)
        .bind(&app.company)
        .bind(&app.color)
        .bind(&app.tag_id)
        .bind(&app.id)
        .execute(self.db.executor())
        .await?;
        Ok(())
    }

    /// Update the [Tag] with additional information
    pub async fn update_tag(&mut self, tag: &infused::UpdatedTag) -> Result<()> {
        query(
            "UPDATE tags SET
                    name = ?,
                    color = ?
                WHERE id = ?",
        )
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(&tag.id)
        .execute(self.db.executor())
        .await?;
        Ok(())
    }

    /// Update old and new [App]s with [Tag] atomically
    pub async fn update_tag_apps(
        &mut self,
        tag_id: Ref<Tag>,
        removed_apps: Vec<Ref<App>>,
        added_apps: Vec<Ref<App>>,
    ) -> Result<()> {
        let mut tx = self.db.transaction().await?;
        let updates = removed_apps.into_iter().map(|app| (app, None)).chain(
            added_apps
                .into_iter()
                .map(|app| (app, Some(tag_id.clone()))),
        );
        for (app, tag) in updates {
            query("UPDATE apps SET tag_id = ? WHERE id = ?")
                .bind(tag)
                .bind(app)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    // TODO create and update tag should return a copy of the tag.

    /// Create a new [Tag] from a [infused::CreateTag]
    pub async fn create_tag(&mut self, tag: &infused::CreateTag) -> Result<Ref<Tag>> {
        let res = query("INSERT INTO tags VALUES (NULL, ?, ?)")
            .bind(&tag.name)
            .bind(&tag.color)
            .execute(self.db.executor())
            .await?;
        Ok(Ref::new(res.last_insert_rowid()))
    }

    /// Removes a [Tag]
    pub async fn remove_tag(&mut self, tag_id: Ref<Tag>) -> Result<()> {
        query("DELETE FROM tags WHERE id = ?")
            .bind(tag_id)
            .execute(self.db.executor())
            .await?;
        Ok(())
    }

    fn destructure_target(target: &Target) -> (Option<&Ref<App>>, Option<&Ref<Tag>>) {
        match target {
            Target::App(app_id) => (Some(app_id), None),
            Target::Tag(tag_id) => (None, Some(tag_id)),
        }
    }

    fn destructure_trigger_action(
        trigger_action: &TriggerAction,
    ) -> (Option<i64>, Option<&str>, i64) {
        match &trigger_action {
            TriggerAction::Kill => (None, None, 0),
            TriggerAction::Dim(dur) => (Some(*dur), None, 1),
            TriggerAction::Message(content) => (None, Some(content.as_str()), 2),
        }
    }

    /// Creates a [Alert]
    pub async fn create_alert(&mut self, alert: infused::CreateAlert) -> Result<infused::Alert> {
        let mut tx = self.db.transaction().await?;

        let (app_id, tag_id) = Self::destructure_target(&alert.target);
        let (dim_duration, message_content, tag) =
            Self::destructure_trigger_action(&alert.trigger_action);

        // Insert the alert
        let alert_id: i64 = query(
            "INSERT INTO alerts
             SELECT COALESCE(MAX(id) + 1, 1), 1, ?, ?, ?, ?, ?, ?, ? FROM alerts LIMIT 1 RETURNING id",
        )
        .bind(app_id)
        .bind(tag_id)
        .bind(alert.usage_limit)
        .bind(&alert.time_frame)
        .bind(dim_duration)
        .bind(message_content)
        .bind(tag)
        .fetch_one(&mut *tx)
        .await?
        .get(0);

        let mut reminders = Vec::new();

        // Insert the reminders
        for reminder in alert.reminders {
            let id: i64 = query(
                "INSERT INTO reminders (id, version, alert_id, alert_version, threshold, message)
                 SELECT COALESCE(MAX(id) + 1, 1), 1, ?, 1, ?, ? FROM reminders LIMIT 1 RETURNING id"
            )
            .bind(alert_id)
            .bind(reminder.threshold)
            .bind(&reminder.message)
            .fetch_one(&mut *tx)
            .await?.get(0);
            let reminder = infused::Reminder {
                id: Ref::new(VersionedId { id, version: 1 }),
                alert: AlertVersionedId {
                    id: alert_id,
                    version: 1,
                },
                threshold: reminder.threshold,
                message: reminder.message,
                events: infused::ValuePerPeriod::default(),
            };
            reminders.push(reminder);
        }

        tx.commit().await?;

        let alert = infused::Alert {
            inner: Alert {
                id: Ref::new(VersionedId {
                    id: alert_id,
                    version: 1,
                }),
                target: alert.target,
                usage_limit: alert.usage_limit,
                time_frame: alert.time_frame,
                trigger_action: alert.trigger_action,
            },
            reminders,
            events: infused::ValuePerPeriod::default(),
        };

        Ok(alert)
    }

    /// Updates a [Alert]. Assumes the id of prev and next are the same for alert and reminders.
    pub async fn update_alert(
        &mut self,
        prev: infused::Alert,
        mut next: infused::UpdatedAlert,
    ) -> Result<infused::Alert> {
        let mut tx = self.db.transaction().await?;
        let has_any_alert_events = Self::has_any_alert_events(&mut *tx, &prev.inner.id).await?;

        let should_upgrade_alert = has_any_alert_events
            && (prev.inner.target != next.target
                || prev.inner.usage_limit != next.usage_limit
                || prev.inner.time_frame != next.time_frame);

        let prev_reminders: HashMap<_, _> = prev
            .reminders
            .into_iter()
            .map(|r| (r.id.0.clone(), r))
            .collect();

        let next_alert = Alert {
            id: Ref::new(next.id.clone()),
            target: next.target.clone(),
            usage_limit: next.usage_limit,
            time_frame: next.time_frame.clone(),
            trigger_action: next.trigger_action.clone(),
        };

        let alert = if should_upgrade_alert {
            next.id = Self::upgrade_alert_only(&mut *tx, &next).await?;
            let mut next_reminders = Vec::new();
            for mut reminder in next.reminders {
                if !reminder.active {
                    continue;
                }
                Self::insert_reminder_only(&mut *tx, &next.id, &mut reminder).await?;
                next_reminders.push(infused::Reminder {
                    id: Ref::new(reminder.id),
                    alert: next.id.clone().into(),
                    threshold: reminder.threshold,
                    message: reminder.message,
                    events: infused::ValuePerPeriod::default(), // since this is a new reminder.
                });
            }
            infused::Alert {
                inner: next_alert,
                reminders: next_reminders,
                events: infused::ValuePerPeriod::default(), // since this is a new alert
            }
        } else {
            Self::update_alert_only(&mut *tx, &next).await?;
            let mut next_reminders = Vec::new();
            for mut reminder in next.reminders {
                let has_any_reminder_events =
                    Self::has_any_reminder_events(&mut *tx, &reminder.id).await?;
                let prev_reminder = prev_reminders.get(&reminder.id);
                if let Some(prev_reminder) = prev_reminder {
                    let should_upgrade_reminder =
                        has_any_reminder_events && (reminder.threshold != prev_reminder.threshold);

                    if should_upgrade_reminder {
                        // Even if this reminder is no longer active, if it's threshold changes & has a
                        // event we should insert (upgrade) the reminder.
                        Self::upgrade_reminder_only(&mut *tx, &next.id, &mut reminder).await?;
                        next_reminders.push(infused::Reminder {
                            id: Ref::new(reminder.id),
                            alert: next.id.clone().into(),
                            threshold: reminder.threshold,
                            message: reminder.message,
                            events: infused::ValuePerPeriod::default(), // since this is a new reminder.
                        });
                    } else {
                        Self::update_reminder_only(&mut *tx, &mut reminder).await?;
                        next_reminders.push(infused::Reminder {
                            id: Ref::new(reminder.id),
                            alert: next.id.clone().into(),
                            threshold: reminder.threshold,
                            message: reminder.message,
                            events: prev_reminder.events.clone(), // since this is just the old reminder
                        });
                    }
                } else {
                    Self::insert_reminder_only(&mut *tx, &next.id, &mut reminder).await?;
                    next_reminders.push(infused::Reminder {
                        id: Ref::new(reminder.id),
                        alert: next.id.clone().into(),
                        threshold: reminder.threshold,
                        message: reminder.message,
                        events: infused::ValuePerPeriod::default(), // since this is a new reminder.
                    });
                }
            }
            infused::Alert {
                inner: next_alert,
                reminders: next_reminders,
                events: prev.events,
            }
        };

        tx.commit().await?;

        Ok(alert)
    }

    async fn has_any_alert_events<'a, E: SqliteExecutor<'a>>(
        executor: E,
        alert_id: &Ref<Alert>,
    ) -> Result<bool> {
        let count: i64 =
            query("SELECT COUNT(*) FROM alert_events WHERE alert_id = ? AND alert_version = ?")
                .bind(alert_id.0.id)
                .bind(alert_id.0.version)
                .fetch_one(executor)
                .await?
                .get(0);
        Ok(count > 0)
    }

    async fn has_any_reminder_events<'a, E: SqliteExecutor<'a>>(
        executor: E,
        reminder_id: &VersionedId,
    ) -> Result<bool> {
        let count: i64 = query(
            "SELECT COUNT(*) FROM reminder_events WHERE reminder_id = ? AND reminder_version = ?",
        )
        .bind(reminder_id.id)
        .bind(reminder_id.version)
        .fetch_one(executor)
        .await?
        .get(0);
        Ok(count > 0)
    }

    async fn update_alert_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        alert: &infused::UpdatedAlert,
    ) -> Result<()> {
        let (app_id, tag_id) = Self::destructure_target(&alert.target);
        let (dim_duration, message_content, tag) =
            Self::destructure_trigger_action(&alert.trigger_action);
        query(
            "UPDATE alerts SET
                app_id = ?,
                tag_id = ?,
                usage_limit = ?,
                time_frame = ?,
                dim_duration = ?,
                message_content = ?,
                trigger_action_tag = ?
            WHERE id = ? AND version = ?",
        )
        .bind(app_id)
        .bind(tag_id)
        .bind(alert.usage_limit)
        .bind(&alert.time_frame)
        .bind(dim_duration)
        .bind(message_content)
        .bind(tag)
        .bind(alert.id.id)
        .bind(alert.id.version)
        .execute(executor)
        .await?;
        Ok(())
    }

    async fn upgrade_alert_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        alert: &infused::UpdatedAlert,
    ) -> Result<VersionedId> {
        let new_id = VersionedId {
            id: alert.id.id,
            version: alert.id.version + 1,
        };
        let (app_id, tag_id) = Self::destructure_target(&alert.target);
        let (dim_duration, message_content, tag) =
            Self::destructure_trigger_action(&alert.trigger_action);
        query("INSERT INTO alerts VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(new_id.id)
            .bind(new_id.version)
            .bind(app_id)
            .bind(tag_id)
            .bind(alert.usage_limit)
            .bind(&alert.time_frame)
            .bind(dim_duration)
            .bind(message_content)
            .bind(tag)
            .execute(executor)
            .await?;

        Ok(new_id)
    }

    async fn insert_reminder_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        alert_id: &VersionedId,
        reminder: &mut infused::UpdatedReminder,
    ) -> Result<()> {
        let id: i64 = query(
                "INSERT INTO reminders (id, version, alert_id, alert_version, threshold, message)
                 SELECT COALESCE(MAX(id) + 1, 1), 1, ?, ?, ?, ? FROM reminders LIMIT 1 RETURNING id"
            )
            .bind(alert_id.id)
            .bind(alert_id.version)
            .bind(reminder.threshold)
            .bind(&reminder.message)
            .fetch_one(executor)
            .await?.get(0);
        reminder.id = VersionedId { id, version: 1 };
        Ok(())
    }

    async fn update_reminder_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        reminder: &mut infused::UpdatedReminder,
    ) -> Result<()> {
        let id: i64 = query(
                "UPDATER reminders SET threshold = ?, message = ?, active = ? WHERE id = ? AND version = ?"
            )
            .bind(reminder.threshold)
            .bind(&reminder.message)
            .bind(reminder.active)
            .bind(reminder.id.id)
            .bind(reminder.id.version)
            .fetch_one(executor)
            .await?.get(0);
        reminder.id = VersionedId { id, version: 1 };
        Ok(())
    }

    async fn upgrade_reminder_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        alert_id: &VersionedId,
        reminder: &mut infused::UpdatedReminder,
    ) -> Result<()> {
        reminder.id = VersionedId {
            id: reminder.id.id,
            version: reminder.id.version,
        };
        query(
                "INSERT INTO reminders (id, version, alert_id, alert_version, threshold, message, active) VALUES
                ?, ?, ?, ?, ?, ?, ? FROM reminders"
            )
            .bind(reminder.id.id)
            .bind(reminder.id.version)
            .bind(alert_id.id)
            .bind(alert_id.version)
            .bind(reminder.threshold)
            .bind(&reminder.message)
            .bind(reminder.active)
            .execute(executor)
            .await?;
        Ok(())
    }

    /// Removes a [Alert]
    pub async fn remove_alert(&mut self, alert_id: Ref<Alert>) -> Result<()> {
        // If this a soft delete, then how would deleting a tag work if this alert uses it?
        // e.g. for a non-max version Alert, if it uses a tag and someone tries to delete the tag they won't be able to.
        // Instead, hard delete all versions, should warn the user that this will remove all reminders and *_events.

        // not specifying the version here will delete all versions
        query("DELETE FROM alerts WHERE id = ?")
            .bind(alert_id.0.id)
            .execute(self.db.executor())
            .await?;
        Ok(())
    }

    /// Gets all [Session]s and [Usage]s in a time range
    pub async fn get_app_session_usages(
        &mut self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<infused::AppSessionUsages> {
        #[derive(FromRow)]
        struct AppSessionUsage {
            app_id: Ref<App>,
            session_id: Ref<Session>,
            usage_id: Ref<Usage>,
            session_title: String,
            start: Timestamp,
            end: Timestamp,
        }

        let usages_raw: Vec<AppSessionUsage> = query_as(
            "SELECT
                s.app_id AS app_id,
                u.session_id AS session_id,
                u.id AS usage_id,
                s.title AS session_title,
                MAX(u.start, p.start) AS start,
                MIN(u.end, p.end) AS end
            FROM usages u, (SELECT ? AS start, ? AS end) p
            INNER JOIN sessions s ON u.session_id = s.id 
            WHERE u.end > p.start AND u.start <= p.end
            ORDER BY u.start ASC",
        )
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        let mut usages = HashMap::<Ref<App>, HashMap<Ref<Session>, infused::Session>>::new();
        for usage in usages_raw {
            let session_id = usage.session_id.clone();
            let sessions = usages.entry(usage.app_id.clone()).or_default();
            let session = sessions
                .entry(session_id.clone())
                .or_insert_with(|| infused::Session {
                    id: session_id,
                    title: usage.session_title,
                    start: usage.start,
                    end: usage.end,
                    usages: Vec::new(),
                });
            session.start = session.start.min(usage.start);
            session.end = session.end.max(usage.end);
            session.usages.push(Usage {
                id: usage.usage_id,
                session_id: usage.session_id,
                start: usage.start,
                end: usage.end,
            });
        }

        Ok(usages)
    }

    /// Gets all [InteractionPeriod]s in a time range
    pub async fn get_interaction_periods(
        &mut self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<Vec<InteractionPeriod>> {
        let interactions: Vec<InteractionPeriod> = query_as(
            "SELECT
                i.id,
                MAX(i.start, p.start) AS start,
                MIN(i.end, p.end) AS end,
                i.mouse_clicks,
                i.key_strokes
            FROM interaction_periods i, (SELECT ? AS start, ? AS end) p
            WHERE i.end > p.start AND i.start <= p.end
            ORDER BY i.start ASC",
        )
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(interactions)
    }

    /// Gets all [SystemEvent]s in a time range
    pub async fn get_system_events(
        &mut self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<Vec<SystemEvent>> {
        let events: Vec<SystemEvent> = query_as(
            "SELECT
                e.id,
                e.timestamp,
                e.event
            FROM system_events e, (SELECT ? AS start, ? AS end) p
            WHERE e.timestamp > p.start AND e.timestamp <= p.end
            ORDER BY e.timestamp ASC",
        )
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(events)
    }
}
