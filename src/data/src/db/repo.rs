use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use serde::Serialize;
use sqlx::SqliteExecutor;

use super::*;
use crate::entities::{Duration, TriggerAction};

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
    use crate::table::{Color, Duration};

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
            (self.threshold - other.threshold).abs() <= f64::EPSILON
                && self.message == other.message
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

const ALERT_EVENT_COUNT: &str = "SELECT e.alert_id, COUNT(e.id) FROM alert_events e
            WHERE timestamp BETWEEN ? AND ?
            GROUP BY e.alert_id";

const REMINDER_EVENT_COUNT: &str = "SELECT e.reminder_id, COUNT(e.id) FROM reminder_events e
            WHERE timestamp BETWEEN ? AND ?
            GROUP BY e.reminder_id";

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
        .bind(ts.now().to_ticks())
        .bind(ts.week_start(true).to_ticks())
        .bind(ts.now().to_ticks())
        .bind(ts.month_start(true).to_ticks())
        .bind(ts.now().to_ticks())
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
        .bind(ts.now().to_ticks())
        .bind(ts.week_start(true).to_ticks())
        .bind(ts.now().to_ticks())
        .bind(ts.month_start(true).to_ticks())
        .bind(ts.now().to_ticks())
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
                events_today(id, count) AS ({ALERT_EVENT_COUNT}),
                events_week(id, count) AS ({ALERT_EVENT_COUNT}),
                events_month(id, count) AS ({ALERT_EVENT_COUNT})
            SELECT al.*,
                COALESCE(t.count, 0) AS today,
                COALESCE(w.count, 0) AS week,
                COALESCE(m.count, 0) AS month
            FROM alerts al
                LEFT JOIN events_today t ON t.id = al.id
                LEFT JOIN events_week w ON w.id = al.id
                LEFT JOIN events_month m ON m.id = al.id
            GROUP BY al.id
            HAVING al.active <> 0"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(ts.now().to_ticks())
        .bind(ts.week_start(true).to_ticks())
        .bind(ts.now().to_ticks())
        .bind(ts.month_start(true).to_ticks())
        .bind(ts.now().to_ticks())
        .fetch_all(self.db.executor())
        .await?;

        let reminders: Vec<infused::Reminder> = query_as(&format!(
            "WITH
                events_today(id, count) AS ({REMINDER_EVENT_COUNT}),
                events_week(id, count) AS ({REMINDER_EVENT_COUNT}),
                events_month(id, count) AS ({REMINDER_EVENT_COUNT})
            SELECT r.*,
                COALESCE(t.count, 0) AS today,
                COALESCE(w.count, 0) AS week,
                COALESCE(m.count, 0) AS month
            FROM reminders r
                LEFT JOIN events_today t ON t.id = r.id
                LEFT JOIN events_week w ON w.id = r.id
                LEFT JOIN events_month m ON m.id = r.id
            GROUP BY r.id
            HAVING r.active <> 0"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(ts.now().to_ticks())
        .bind(ts.week_start(true).to_ticks())
        .bind(ts.now().to_ticks())
        .bind(ts.month_start(true).to_ticks())
        .bind(ts.now().to_ticks())
        .fetch_all(self.db.executor())
        .await?;

        let mut alerts: HashMap<_, _> = alerts
            .into_iter()
            .map(|alert| {
                (
                    alert.inner.id.clone(),
                    infused::Alert {
                        id: alert.inner.id,
                        target: alert.inner.target,
                        usage_limit: alert.inner.usage_limit,
                        time_frame: alert.inner.time_frame,
                        trigger_action: alert.inner.trigger_action,

                        events: alert.events,
                        reminders: Vec::new(),
                    },
                )
            })
            .collect();

        // ignores inactive reminders
        reminders.into_iter().for_each(|reminder| {
            let alert_id = reminder.alert_id.clone();
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

    /// Create a new [Tag] from a [infused::CreateTag]
    pub async fn create_tag(&mut self, tag: &infused::CreateTag) -> Result<infused::Tag> {
        let res = query("INSERT INTO tags VALUES (NULL, ?, ?)")
            .bind(&tag.name)
            .bind(&tag.color)
            .execute(self.db.executor())
            .await?;
        let id = Ref::new(res.last_insert_rowid());
        Ok(infused::Tag {
            inner: Tag {
                id,
                name: tag.name.clone(),
                color: tag.color.clone(),
            },
            apps: RefVec(Vec::new()),
            usages: infused::ValuePerPeriod::default(),
        })
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
            Target::App { id } => (Some(id), None),
            Target::Tag { id } => (None, Some(id)),
        }
    }

    fn destructure_trigger_action(
        trigger_action: &TriggerAction,
    ) -> (Option<i64>, Option<&str>, i64) {
        match &trigger_action {
            TriggerAction::Kill => (None, None, 0),
            TriggerAction::Dim { duration } => (Some(*duration), None, 1),
            TriggerAction::Message { content } => (None, Some(content.as_str()), 2),
        }
    }

    /// Creates a [Alert]
    pub async fn create_alert(&mut self, alert: infused::CreateAlert) -> Result<infused::Alert> {
        let mut tx = self.db.transaction().await?;

        let (app_id, tag_id) = Self::destructure_target(&alert.target);
        let (dim_duration, message_content, tag) =
            Self::destructure_trigger_action(&alert.trigger_action);

        // Insert the alert
        let alert_id: Ref<Alert> =
            query("INSERT INTO alerts VALUES (NULL, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id")
                .bind(app_id)
                .bind(tag_id)
                .bind(alert.usage_limit)
                .bind(&alert.time_frame)
                .bind(dim_duration)
                .bind(message_content)
                .bind(tag)
                .bind(true)
                .fetch_one(&mut *tx)
                .await?
                .get(0);

        let mut reminders = Vec::new();

        // Insert the reminders
        for reminder in alert.reminders {
            let id = query("INSERT INTO reminders VALUES (NULL, ?, ?, ?, ?) RETURNING id")
                .bind(&alert_id)
                .bind(reminder.threshold)
                .bind(&reminder.message)
                .bind(true)
                .fetch_one(&mut *tx)
                .await?
                .get(0);
            let reminder = infused::Reminder {
                id,
                alert_id: alert_id.clone(),
                threshold: reminder.threshold,
                message: reminder.message,
                events: infused::ValuePerPeriod::default(),
            };
            reminders.push(reminder);
        }

        tx.commit().await?;

        let alert = infused::Alert {
            id: alert_id,
            target: alert.target,
            usage_limit: alert.usage_limit,
            time_frame: alert.time_frame,
            trigger_action: alert.trigger_action,
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
        let has_any_alert_events = Self::has_any_alert_events(&mut *tx, &prev.id).await?;

        let should_upgrade_alert = has_any_alert_events
            && (prev.target != next.target
                || prev.usage_limit != next.usage_limit
                || prev.time_frame != next.time_frame);

        let prev_reminders: HashMap<_, _> = prev
            .reminders
            .into_iter()
            .map(|r| (r.id.clone(), r))
            .collect();

        let mut next_alert = infused::Alert {
            id: next.id.clone(),
            target: next.target.clone(),
            usage_limit: next.usage_limit,
            time_frame: next.time_frame.clone(),
            trigger_action: next.trigger_action.clone(),
            reminders: Vec::new(),
            events: infused::ValuePerPeriod::default(),
        };

        if should_upgrade_alert {
            next.id = Self::upgrade_alert_only(&mut tx, &next).await?;

            for mut reminder in next.reminders {
                if !reminder.active {
                    continue;
                }
                Self::insert_reminder_only(&mut *tx, &next.id, &mut reminder).await?;
                next_alert.reminders.push(infused::Reminder {
                    id: reminder.id,
                    alert_id: next.id.clone(),
                    threshold: reminder.threshold,
                    message: reminder.message,
                    events: infused::ValuePerPeriod::default(), // since this is a new reminder.
                });
            }
        } else {
            Self::update_alert_only(&mut *tx, &next).await?;
            next_alert.events = prev.events;

            for mut reminder in next.reminders {
                let has_any_reminder_events =
                    Self::has_any_reminder_events(&mut *tx, &reminder.id).await?;
                let prev_reminder = prev_reminders.get(&reminder.id);
                let mut next_reminder = infused::Reminder {
                    id: Ref::new(0),
                    alert_id: next.id.clone(),
                    threshold: reminder.threshold,
                    message: reminder.message.clone(),
                    events: infused::ValuePerPeriod::default(),
                };
                if let Some(prev_reminder) = prev_reminder {
                    let should_upgrade_reminder =
                        has_any_reminder_events && (reminder.threshold != prev_reminder.threshold);

                    if should_upgrade_reminder {
                        // Even if this reminder is no longer active, if it's threshold changes & has a
                        // event we should insert (upgrade) the reminder.
                        Self::upgrade_reminder_only(&mut tx, &next.id, &mut reminder).await?;
                        next_reminder.id = reminder.id;
                    } else {
                        Self::update_reminder_only(&mut *tx, &mut reminder).await?;
                        next_reminder.events = prev_reminder.events.clone();
                    }
                } else {
                    Self::insert_reminder_only(&mut *tx, &next.id, &mut reminder).await?;
                    next_reminder.id = reminder.id;
                }
                next_alert.reminders.push(next_reminder);
            }
        };

        tx.commit().await?;

        Ok(next_alert)
    }

    async fn has_any_alert_events<'a, E: SqliteExecutor<'a>>(
        executor: E,
        alert_id: &Ref<Alert>,
    ) -> Result<bool> {
        let count: i64 = query("SELECT COUNT(*) FROM alert_events WHERE alert_id = ?")
            .bind(alert_id)
            .fetch_one(executor)
            .await?
            .get(0);
        Ok(count > 0)
    }

    async fn has_any_reminder_events<'a, E: SqliteExecutor<'a>>(
        executor: E,
        reminder_id: &Ref<Reminder>,
    ) -> Result<bool> {
        let count: i64 = query("SELECT COUNT(*) FROM reminder_events WHERE reminder_id = ?")
            .bind(reminder_id)
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
            WHERE id = ?",
        )
        .bind(app_id)
        .bind(tag_id)
        .bind(alert.usage_limit)
        .bind(&alert.time_frame)
        .bind(dim_duration)
        .bind(message_content)
        .bind(tag)
        .bind(&alert.id)
        .execute(executor)
        .await?;
        Ok(())
    }

    async fn upgrade_alert_only(
        executor: &mut sqlx::SqliteConnection,
        alert: &infused::UpdatedAlert,
    ) -> Result<Ref<Alert>> {
        query("UPDATE alerts SET active = 0 WHERE id = ?")
            .bind(&alert.id)
            .execute(&mut *executor)
            .await?;

        let (app_id, tag_id) = Self::destructure_target(&alert.target);
        let (dim_duration, message_content, tag) =
            Self::destructure_trigger_action(&alert.trigger_action);
        let new_id = query("INSERT INTO alerts VALUES (NULL, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id")
            .bind(app_id)
            .bind(tag_id)
            .bind(alert.usage_limit)
            .bind(&alert.time_frame)
            .bind(dim_duration)
            .bind(message_content)
            .bind(tag)
            .bind(true)
            .fetch_one(&mut *executor)
            .await?
            .get(0);

        Ok(new_id)
    }

    async fn insert_reminder_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        alert_id: &Ref<Alert>,
        reminder: &mut infused::UpdatedReminder,
    ) -> Result<()> {
        reminder.id = query("INSERT INTO reminders VALUES(NULL, ?, ?, ?, ?) RETURNING id")
            .bind(alert_id)
            .bind(reminder.threshold)
            .bind(&reminder.message)
            .bind(true)
            .fetch_one(executor)
            .await?
            .get(0);
        Ok(())
    }

    async fn update_reminder_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        reminder: &mut infused::UpdatedReminder,
    ) -> Result<()> {
        reminder.id =
            query("UPDATE reminders SET threshold = ?, message = ?, active = ? WHERE id = ?")
                .bind(reminder.threshold)
                .bind(&reminder.message)
                .bind(reminder.active)
                .bind(&reminder.id)
                .fetch_one(executor)
                .await?
                .get(0);
        Ok(())
    }

    async fn upgrade_reminder_only(
        executor: &mut sqlx::SqliteConnection,
        alert_id: &Ref<Alert>,
        reminder: &mut infused::UpdatedReminder,
    ) -> Result<()> {
        query("UPDATE reminders SET active = 0 WHERE id = ?")
            .bind(&reminder.id)
            .execute(&mut *executor)
            .await?;

        reminder.id = query("INSERT INTO reminders VALUES(NULL, ?, ?, ?, ?) RETURNING id")
            .bind(alert_id)
            .bind(reminder.threshold)
            .bind(&reminder.message)
            .bind(true)
            .fetch_one(&mut *executor)
            .await?
            .get(0);
        Ok(())
    }

    /// Removes a [Alert]
    pub async fn remove_alert(&mut self, alert_id: Ref<Alert>) -> Result<()> {
        // If this a soft delete, then how would deleting a tag work if this alert uses it?
        // e.g. for a non-max version Alert, if it uses a tag and someone tries to delete the tag they won't be able to.
        // Instead, hard delete all versions, should warn the user that this will remove all reminders and *_events.

        // not specifying the version here will delete all versions
        query("DELETE FROM alerts WHERE id = ?")
            .bind(alert_id)
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
