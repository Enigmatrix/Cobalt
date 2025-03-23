use std::collections::HashMap;

use sqlx::SqliteExecutor;

use super::repo::Repository;
use super::*;
use crate::entities::{Reason, TriggerAction};

/// SQL expression for getting the duration of all apps in day, week, month range.
pub const APP_DUR: &str = "SELECT a.id AS id,
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
            WHERE reason = 0 AND timestamp BETWEEN ? AND ?
            GROUP BY e.alert_id";

const REMINDER_EVENT_COUNT: &str = "SELECT e.reminder_id, COUNT(e.id) FROM reminder_events e
            WHERE reason = 0 AND timestamp BETWEEN ? AND ?
            GROUP BY e.reminder_id";

impl Repository {
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
            WHERE a.initialized_at IS NOT NULL
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
                LEFT JOIN apps a ON t.id = a.tag_id AND a.initialized_at IS NOT NULL
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
                        created_at: alert.inner.created_at,
                        updated_at: alert.inner.updated_at,

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
    /// Update the [App] with additional information
    pub async fn update_app(
        &mut self,
        app: &infused::UpdatedApp,
        ts: impl TimeSystem,
    ) -> Result<()> {
        query(
            "UPDATE apps SET
                    name = ?,
                    description = ?,
                    company = ?,
                    color = ?,
                    tag_id = ?,
                    updated_at = ?
                WHERE id = ?",
        )
        .bind(&app.name)
        .bind(&app.description)
        .bind(&app.company)
        .bind(&app.color)
        .bind(&app.tag_id)
        .bind(ts.now().to_ticks())
        .bind(&app.id)
        .execute(self.db.executor())
        .await?;
        Ok(())
    }

    /// Update the [Tag] with additional information
    pub async fn update_tag(
        &mut self,
        tag: &infused::UpdatedTag,
        ts: impl TimeSystem,
    ) -> Result<()> {
        query(
            "UPDATE tags SET
                    name = ?,
                    color = ?
                WHERE id = ?",
        )
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(ts.now().to_ticks())
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
        ts: impl TimeSystem,
    ) -> Result<()> {
        let mut tx = self.db.transaction().await?;
        let updates = removed_apps.into_iter().map(|app| (app, None)).chain(
            added_apps
                .into_iter()
                .map(|app| (app, Some(tag_id.clone()))),
        );
        for (app, tag) in updates {
            query("UPDATE apps SET tag_id = ?, updated_at = ? WHERE id = ?")
                .bind(tag)
                .bind(ts.now().to_ticks())
                .bind(app)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Create a new [Tag] from a [infused::CreateTag]
    pub async fn create_tag(
        &mut self,
        tag: &infused::CreateTag,
        ts: impl TimeSystem,
    ) -> Result<infused::Tag> {
        let mut tx = self.db.transaction().await?;
        let res = query("INSERT INTO tags VALUES (NULL, ?, ?, ?, ?)")
            .bind(&tag.name)
            .bind(&tag.color)
            .bind(ts.now().to_ticks())
            .bind(ts.now().to_ticks())
            .execute(&mut *tx)
            .await?;
        let id = Ref::new(res.last_insert_rowid());
        for app in &tag.apps {
            query("UPDATE apps SET tag_id = ?, updated_at = ? WHERE id = ?")
                .bind(&id)
                .bind(ts.now().to_ticks())
                .bind(app)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;

        Ok(infused::Tag {
            inner: Tag {
                id,
                name: tag.name.clone(),
                color: tag.color.clone(),
                created_at: ts.now().to_ticks(),
                updated_at: ts.now().to_ticks(),
            },
            apps: infused::RefVec(tag.apps.clone()),
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
    pub async fn create_alert(
        &mut self,
        alert: infused::CreateAlert,
        ts: impl TimeSystem,
    ) -> Result<infused::Alert> {
        let mut tx = self.db.transaction().await?;

        let (app_id, tag_id) = Self::destructure_target(&alert.target);
        let (dim_duration, message_content, tag) =
            Self::destructure_trigger_action(&alert.trigger_action);

        // Insert the alert
        let alert_id: Ref<Alert> =
            query("INSERT INTO alerts VALUES (NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id")
                .bind(app_id)
                .bind(tag_id)
                .bind(alert.usage_limit)
                .bind(&alert.time_frame)
                .bind(dim_duration)
                .bind(message_content)
                .bind(tag)
                .bind(true)
                .bind(ts.now().to_ticks())
                .bind(ts.now().to_ticks())
                .fetch_one(&mut *tx)
                .await?
                .get(0);

        if alert.ignore_trigger {
            query("INSERT INTO alert_events VALUES (NULL, ?, ?, ?)")
                .bind(&alert_id)
                .bind(ts.now().to_ticks())
                .bind(Reason::Ignored)
                .execute(&mut *tx)
                .await?;
        }

        let mut reminders = Vec::new();

        // Insert the reminders
        for reminder in alert.reminders {
            let id = query("INSERT INTO reminders VALUES (NULL, ?, ?, ?, ?, ?, ?) RETURNING id")
                .bind(&alert_id)
                .bind(reminder.threshold)
                .bind(&reminder.message)
                .bind(true)
                .bind(ts.now().to_ticks())
                .bind(ts.now().to_ticks())
                .fetch_one(&mut *tx)
                .await?
                .get(0);

            if reminder.ignore_trigger {
                query("INSERT INTO reminder_events VALUES (NULL, ?, ?, ?)")
                    .bind(&id)
                    .bind(ts.now().to_ticks())
                    .bind(Reason::Ignored)
                    .execute(&mut *tx)
                    .await?;
            }

            let reminder = infused::Reminder {
                id,
                alert_id: alert_id.clone(),
                threshold: reminder.threshold,
                message: reminder.message,
                created_at: ts.now().to_ticks(),
                updated_at: ts.now().to_ticks(),
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
            created_at: ts.now().to_ticks(),
            updated_at: ts.now().to_ticks(),
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
        ts: impl TimeSystem,
    ) -> Result<infused::Alert> {
        let mut tx = self.db.transaction().await?;
        let has_any_alert_events = Self::has_any_alert_events(&mut *tx, &prev.id).await?;

        let should_upgrade_alert = has_any_alert_events
            && (prev.target != next.target
                || prev.usage_limit != next.usage_limit
                || prev.time_frame != next.time_frame);

        let prev_reminders: HashMap<_, _> = prev
            .reminders
            .iter()
            .cloned()
            .map(|r| (r.id.clone(), r))
            .collect();

        let mut next_alert = infused::Alert {
            id: next.id.clone(),
            target: next.target.clone(),
            usage_limit: next.usage_limit,
            time_frame: next.time_frame.clone(),
            trigger_action: next.trigger_action.clone(),
            created_at: prev.created_at,
            updated_at: ts.now().to_ticks(),
            reminders: Vec::new(),
            events: infused::ValuePerPeriod::default(),
        };

        if should_upgrade_alert {
            next.id = Self::upgrade_alert_only(&mut tx, &prev, &next, &ts).await?;

            for mut reminder in next.reminders {
                if !reminder.active {
                    continue;
                }
                let prev_reminder = prev_reminders.get(&reminder.id);
                Self::insert_reminder_only(&mut *tx, prev_reminder, &next.id, &mut reminder, &ts)
                    .await?;
                next_alert.reminders.push(infused::Reminder {
                    id: reminder.id,
                    alert_id: next.id.clone(),
                    threshold: reminder.threshold,
                    message: reminder.message,
                    created_at: prev_reminder
                        .map(|r| r.created_at)
                        .unwrap_or(ts.now().to_ticks()),
                    updated_at: ts.now().to_ticks(),
                    events: infused::ValuePerPeriod::default(), // since this is a new reminder.
                });
            }
        } else {
            Self::update_alert_only(&mut *tx, &next, &ts).await?;
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
                    created_at: ts.now().to_ticks(),
                    updated_at: ts.now().to_ticks(),
                    events: infused::ValuePerPeriod::default(),
                };
                if let Some(prev_reminder) = prev_reminder {
                    let should_upgrade_reminder =
                        has_any_reminder_events && (reminder.threshold != prev_reminder.threshold);

                    next_reminder.created_at = prev_reminder.created_at;
                    if should_upgrade_reminder {
                        // Even if this reminder is no longer active, if it's threshold changes & has a
                        // event we should insert (upgrade) the reminder.
                        Self::upgrade_reminder_only(
                            &mut tx,
                            prev_reminder,
                            &next.id,
                            &mut reminder,
                            &ts,
                        )
                        .await?;
                        next_reminder.id = reminder.id;
                    } else {
                        Self::update_reminder_only(&mut *tx, prev_reminder, &mut reminder, &ts)
                            .await?;
                        next_reminder.events = prev_reminder.events.clone();
                    }
                } else {
                    Self::insert_reminder_only(&mut *tx, None, &next.id, &mut reminder, &ts)
                        .await?;
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
        ts: &impl TimeSystem,
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
                trigger_action_tag = ?,
                updated_at = ?
            WHERE id = ?",
        )
        .bind(app_id)
        .bind(tag_id)
        .bind(alert.usage_limit)
        .bind(&alert.time_frame)
        .bind(dim_duration)
        .bind(message_content)
        .bind(tag)
        .bind(ts.now().to_ticks())
        .bind(&alert.id)
        .execute(executor)
        .await?;
        Ok(())
    }

    async fn upgrade_alert_only(
        executor: &mut sqlx::SqliteConnection,
        prev: &infused::Alert,
        alert: &infused::UpdatedAlert,
        ts: &impl TimeSystem,
    ) -> Result<Ref<Alert>> {
        query("UPDATE alerts SET active = 0, updated_at = ? WHERE id = ?")
            .bind(ts.now().to_ticks())
            .bind(&alert.id)
            .execute(&mut *executor)
            .await?;

        let (app_id, tag_id) = Self::destructure_target(&alert.target);
        let (dim_duration, message_content, tag) =
            Self::destructure_trigger_action(&alert.trigger_action);
        let new_id =
            query("INSERT INTO alerts VALUES (NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id")
                .bind(app_id)
                .bind(tag_id)
                .bind(alert.usage_limit)
                .bind(&alert.time_frame)
                .bind(dim_duration)
                .bind(message_content)
                .bind(tag)
                .bind(true)
                .bind(prev.created_at)
                .bind(ts.now().to_ticks())
                .fetch_one(&mut *executor)
                .await?
                .get(0);

        Ok(new_id)
    }

    async fn insert_reminder_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        prev_reminder: Option<&infused::Reminder>,
        alert_id: &Ref<Alert>,
        reminder: &mut infused::UpdatedReminder,
        ts: &impl TimeSystem,
    ) -> Result<()> {
        reminder.id = query("INSERT INTO reminders VALUES(NULL, ?, ?, ?, ?, ?, ?) RETURNING id")
            .bind(alert_id)
            .bind(reminder.threshold)
            .bind(&reminder.message)
            .bind(true)
            .bind(
                prev_reminder
                    .map(|r| r.created_at)
                    .unwrap_or(ts.now().to_ticks()),
            )
            .bind(ts.now().to_ticks())
            .fetch_one(executor)
            .await?
            .get(0);
        Ok(())
    }

    async fn update_reminder_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        _prev_reminder: &infused::Reminder,
        reminder: &mut infused::UpdatedReminder,
        ts: &impl TimeSystem,
    ) -> Result<()> {
        reminder.id =
            query("UPDATE reminders SET threshold = ?, message = ?, active = ?, updated_at = ? WHERE id = ?")
                .bind(reminder.threshold)
                .bind(&reminder.message)
                .bind(reminder.active)
                .bind(ts.now().to_ticks())
                .bind(&reminder.id)
                .fetch_one(executor)
                .await?
                .get(0);
        Ok(())
    }

    async fn upgrade_reminder_only(
        executor: &mut sqlx::SqliteConnection,
        prev_reminder: &infused::Reminder,
        alert_id: &Ref<Alert>,
        reminder: &mut infused::UpdatedReminder,
        ts: &impl TimeSystem,
    ) -> Result<()> {
        query("UPDATE reminders SET active = 0, updated_at = ? WHERE id = ?")
            .bind(ts.now().to_ticks())
            .bind(&reminder.id)
            .execute(&mut *executor)
            .await?;

        reminder.id = query("INSERT INTO reminders VALUES(NULL, ?, ?, ?, ?, ?, ?) RETURNING id")
            .bind(alert_id)
            .bind(reminder.threshold)
            .bind(&reminder.message)
            .bind(true)
            .bind(prev_reminder.created_at)
            .bind(ts.now().to_ticks())
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

    /// Create Alert event ignore
    pub async fn create_alert_event_ignore(
        &mut self,
        alert_id: Ref<Alert>,
        timestamp: Timestamp,
    ) -> Result<()> {
        query("INSERT INTO alert_events VALUES (NULL, ?, ?, ?)")
            .bind(alert_id)
            .bind(timestamp)
            .bind(Reason::Ignored)
            .execute(self.db.executor())
            .await?;
        Ok(())
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
