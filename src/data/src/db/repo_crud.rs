use std::collections::HashMap;

use sqlx::SqliteExecutor;

use super::repo::Repository;
use super::*;
use crate::entities::{Reason, TriggerAction};

/// SQL expression for getting the duration of all apps in day, week, month range.
pub const APP_DUR: &str = "SELECT s.app_id AS id,
                COALESCE(SUM(MIN(u.end, p.end) - MAX(u.start, p.start)), 0) AS duration
            FROM sessions s, (SELECT ? AS start, ? AS end) p
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > p.start AND u.start <= p.end
            GROUP BY s.app_id";

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
                LEFT JOIN apps a ON t.id = a.tag_id
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
            status: infused::AlertTriggerStatus,
            #[sqlx(flatten)]
            events: infused::ValuePerPeriod<i64>,
        }

        let alerts: Vec<AlertNoReminders> = query_as(&format!(
            "WITH
                range_start(id, range_start) AS (
                    SELECT al.id,
                        CASE
                            WHEN al.time_frame = 0 THEN ?
                            WHEN al.time_frame = 1 THEN ?
                            ELSE ?
                        END range_start
                    FROM alerts al
                ),
                trigger_status(id, status, timestamp) AS (
                    SELECT t.id,
                        e.reason,
                        e.timestamp
                    FROM range_start t
                    LEFT JOIN alert_events e
                        ON t.id = e.alert_id AND e.timestamp = (
                            SELECT MAX(e.timestamp)
                            FROM alert_events e
                            WHERE e.alert_id = t.id AND e.timestamp >= t.range_start)
                ),
                events_today(id, count) AS ({ALERT_EVENT_COUNT}),
                events_week(id, count) AS ({ALERT_EVENT_COUNT}),
                events_month(id, count) AS ({ALERT_EVENT_COUNT})
            SELECT al.*,
                COALESCE(ts.status, 2) AS alert_status,
                ts.timestamp AS alert_status_timestamp,
                COALESCE(t.count, 0) AS today,
                COALESCE(w.count, 0) AS week,
                COALESCE(m.count, 0) AS month
            FROM alerts al
                LEFT JOIN trigger_status ts ON ts.id = al.id
                LEFT JOIN events_today t ON t.id = al.id
                LEFT JOIN events_week w ON w.id = al.id
                LEFT JOIN events_month m ON m.id = al.id
            WHERE al.active <> 0
            GROUP BY al.id"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(ts.week_start(true).to_ticks())
        .bind(ts.month_start(true).to_ticks())
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
                range_start(id, range_start) AS (
                    SELECT r.id,
                        CASE
                            WHEN al.time_frame = 0 THEN ?
                            WHEN al.time_frame = 1 THEN ?
                            ELSE ?
                        END range_start
                    FROM reminders r
                        INNER JOIN alerts al ON r.alert_id = al.id
                ),
                reminder_trigger_status(id, status, timestamp, alert_ignored) AS (
                    SELECT t.id,
                        e.reason,
                        e.timestamp,
                        0 AS alert_ignored
                    FROM range_start t
                    LEFT JOIN reminder_events e
                        ON t.id = e.reminder_id AND e.timestamp = (
                            SELECT MAX(e.timestamp)
                            FROM reminder_events e
                            WHERE e.reminder_id = t.id AND e.timestamp >= t.range_start)
                ),
                -- alert trigger status for matching alert_events that are ignored
                alert_trigger_status(id, status, timestamp, alert_ignored) AS (
                    SELECT t.id,
                        e.reason,
                        e.timestamp,
                        1 AS alert_ignored
                    FROM range_start t
                    INNER JOIN reminders r ON t.id = r.id
                    LEFT JOIN alert_events e
                        ON e.alert_id = r.alert_id
                        AND e.timestamp >= t.range_start
                        AND e.reason = 1
                ),
                events_today(id, count) AS ({REMINDER_EVENT_COUNT}),
                events_week(id, count) AS ({REMINDER_EVENT_COUNT}),
                events_month(id, count) AS ({REMINDER_EVENT_COUNT})
            SELECT r.*,
                COALESCE(rts.status, ats.status, 2) AS reminder_status,
                COALESCE(rts.timestamp, ats.timestamp) AS reminder_status_timestamp,
                COALESCE(rts.alert_ignored, ats.alert_ignored) AS reminder_status_alert_ignored,
                COALESCE(t.count, 0) AS today,
                COALESCE(w.count, 0) AS week,
                COALESCE(m.count, 0) AS month
            FROM reminders r
                LEFT JOIN reminder_trigger_status rts ON rts.id = r.id
                LEFT JOIN alert_trigger_status ats ON ats.id = r.id
                LEFT JOIN events_today t ON t.id = r.id
                LEFT JOIN events_week w ON w.id = r.id
                LEFT JOIN events_month m ON m.id = r.id
            WHERE r.active <> 0
            GROUP BY r.id"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(ts.week_start(true).to_ticks())
        .bind(ts.month_start(true).to_ticks())
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
                        status: alert.status,

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
                    color = ?,
                    score = ?,
                    updated_at = ?
                WHERE id = ?",
        )
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(tag.score)
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
        let res = query("INSERT INTO tags VALUES (NULL, ?, ?, ?, ?, ?)")
            .bind(&tag.name)
            .bind(&tag.color)
            .bind(tag.score)
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
                score: tag.score,
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
    ) -> Result<Ref<Alert>> {
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
            Self::insert_alert_event(
                &mut tx,
                &AlertEvent {
                    id: Ref::new(0),
                    alert_id: alert_id.clone(),
                    timestamp: ts.now().to_ticks(),
                    reason: Reason::Ignored,
                },
            )
            .await?;
        }

        // Insert the reminders
        for reminder in alert.reminders {
            let id: Ref<Reminder> =
                query("INSERT INTO reminders VALUES (NULL, ?, ?, ?, ?, ?, ?) RETURNING id")
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
        }

        tx.commit().await?;

        Ok(alert_id)
    }

    /// Updates a [Alert]. Assumes the id of prev and next are the same for alert and reminders.
    pub async fn update_alert(
        &mut self,
        prev: infused::Alert,
        next: infused::UpdatedAlert,
        ts: impl TimeSystem,
    ) -> Result<infused::Alert> {
        let mut tx = self.db.transaction().await?;

        let should_upgrade_alert = (prev.target != next.target
            || prev.usage_limit != next.usage_limit
            || prev.time_frame != next.time_frame)
            && Self::has_any_alert_events(&mut tx, &prev.id).await?;

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
            status: infused::AlertTriggerStatus::Untriggered,
            events: infused::ValuePerPeriod::default(),
        };

        // only upgrades of alerts/reminders can get an ignore xxx event when ignore_trigger=true

        if should_upgrade_alert {
            next_alert.id = Self::upgrade_alert_only(&mut tx, &prev, &next, &ts).await?;

            if next.ignore_trigger {
                Self::insert_alert_event(
                    &mut tx,
                    &AlertEvent {
                        id: Ref::new(0),
                        alert_id: next.id.clone(),
                        timestamp: ts.now().to_ticks(),
                        reason: Reason::Ignored,
                    },
                )
                .await?;
                next_alert.status = infused::AlertTriggerStatus::Ignored {
                    timestamp: ts.now().to_ticks(),
                };
            }

            for mut reminder in next.reminders {
                // Reminder is deleted from prev if it doesn't appear in next.reminders.
                // We don't need to bother updating the previous reminder during an alert upgrade - the queries won't fetch them.

                let prev_reminder = reminder.id.clone().and_then(|id| prev_reminders.get(&id));
                Self::insert_reminder_only(&mut *tx, prev_reminder, &next.id, &mut reminder, &ts)
                    .await?;
                next_alert.reminders.push(infused::Reminder {
                    id: reminder.id.clone().unwrap(), // insert_reminder updates id to Some
                    alert_id: next_alert.id.clone(),
                    threshold: reminder.threshold,
                    message: reminder.message,
                    created_at: prev_reminder
                        .map(|r| r.created_at)
                        .unwrap_or(ts.now().to_ticks()),
                    updated_at: ts.now().to_ticks(),
                    status: if reminder.ignore_trigger {
                        infused::ReminderTriggerStatus::Ignored {
                            timestamp: ts.now().to_ticks(),
                            // a bit wonky - it's ignored because *this reminder* is ignored as well, but we can't express that.
                            ignored_by_alert: next.ignore_trigger,
                        }
                    } else {
                        infused::ReminderTriggerStatus::Untriggered
                    },
                    events: infused::ValuePerPeriod::default(), // since this is a new reminder.
                });

                if reminder.ignore_trigger {
                    Self::insert_reminder_event(
                        &mut tx,
                        &ReminderEvent {
                            id: Ref::new(0),
                            reminder_id: reminder.id.unwrap(),
                            timestamp: ts.now().to_ticks(),
                            reason: Reason::Ignored,
                        },
                    )
                    .await?;
                }
            }
        } else {
            Self::update_alert_only(&mut *tx, &next, &ts).await?;
            next_alert.events = prev.events;
            next_alert.status = prev.status;

            // Remove reminders that are no longer in next.reminders
            for reminder in prev.reminders.iter() {
                if !next
                    .reminders
                    .iter()
                    .any(|r| r.id.as_ref().map(|id| id == &reminder.id) == Some(true))
                {
                    Self::remove_reminder_only(&mut *tx, &reminder.id, &ts).await?;
                }
            }

            // Update/Upgrade reminders that are still in next.reminders
            for mut reminder in next.reminders {
                let mut next_reminder = infused::Reminder {
                    id: Ref::new(0),
                    alert_id: next.id.clone(),
                    threshold: reminder.threshold,
                    message: reminder.message.clone(),
                    created_at: ts.now().to_ticks(),
                    updated_at: ts.now().to_ticks(),
                    status: infused::ReminderTriggerStatus::Untriggered,
                    events: infused::ValuePerPeriod::default(),
                };
                if let Some(id) = reminder.id.clone() {
                    let has_any_reminder_events =
                        Self::has_any_reminder_events(&mut *tx, &id).await?;
                    let prev_reminder = prev_reminders.get(&id).unwrap(); // unwrap is safe because if the id is Some, then the reminder must exist in prev_reminders.

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
                        next_reminder.id = reminder.id.unwrap(); // upgrade_reminder_only updates id to Some

                        if reminder.ignore_trigger {
                            Self::insert_reminder_event(
                                &mut tx,
                                &ReminderEvent {
                                    id: Ref::new(0),
                                    reminder_id: next_reminder.id.clone(),
                                    timestamp: ts.now().to_ticks(),
                                    reason: Reason::Ignored,
                                },
                            )
                            .await?;
                            next_reminder.status = infused::ReminderTriggerStatus::Ignored {
                                timestamp: ts.now().to_ticks(),
                                // a bit wonky - it's ignored because *this reminder* is ignored as well, but we can't express that.
                                ignored_by_alert: next.ignore_trigger,
                            };
                        }
                    } else {
                        Self::update_reminder_only(&mut *tx, prev_reminder, &mut reminder, &ts)
                            .await?;
                        next_reminder.id = id;
                        next_reminder.created_at = prev_reminder.created_at;
                        next_reminder.events = prev_reminder.events.clone();
                        next_reminder.status = prev_reminder.status.clone();
                        // no change to trigger status, no need to bother looking at ignore_trigger.
                    }
                } else {
                    Self::insert_reminder_only(&mut *tx, None, &next.id, &mut reminder, &ts)
                        .await?;
                    next_reminder.id = reminder.id.unwrap(); // insert_reminder updates id to Some

                    if reminder.ignore_trigger {
                        Self::insert_reminder_event(
                            &mut tx,
                            &ReminderEvent {
                                id: Ref::new(0),
                                reminder_id: next_reminder.id.clone(),
                                timestamp: ts.now().to_ticks(),
                                reason: Reason::Ignored,
                            },
                        )
                        .await?;
                        next_reminder.status = infused::ReminderTriggerStatus::Ignored {
                            timestamp: ts.now().to_ticks(),
                            // a bit wonky - it's ignored because *this reminder* is ignored as well, but we can't express that.
                            ignored_by_alert: next.ignore_trigger,
                        };
                    }
                }
                next_alert.reminders.push(next_reminder);
            }
        };

        tx.commit().await?;

        Ok(next_alert)
    }

    async fn has_any_alert_events(
        executor: &mut sqlx::SqliteConnection,
        alert_id: &Ref<Alert>,
    ) -> Result<bool> {
        let count: i64 = query("SELECT COUNT(*) FROM alert_events WHERE alert_id = ?")
            .bind(alert_id)
            .fetch_one(&mut *executor)
            .await?
            .get(0);
        let reminder_count: i64 = query("SELECT COUNT(*) FROM reminders r INNER JOIN reminder_events re ON r.id = re.reminder_id WHERE r.alert_id = ?")
            .bind(alert_id)
            .fetch_one(executor)
            .await?
            .get(0);
        Ok(count > 0 || reminder_count > 0)
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
                trigger_action_dim_duration = ?,
                trigger_action_message_content = ?,
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
        query("UPDATE reminders SET threshold = ?, message = ?, updated_at = ? WHERE id = ?")
            .bind(reminder.threshold)
            .bind(&reminder.message)
            .bind(ts.now().to_ticks())
            .bind(&reminder.id)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn remove_reminder_only<'a, E: SqliteExecutor<'a>>(
        executor: E,
        id: &Ref<Reminder>,
        ts: &impl TimeSystem,
    ) -> Result<()> {
        query("UPDATE reminders SET active = 0, updated_at = ? WHERE id = ?")
            .bind(ts.now().to_ticks())
            .bind(id)
            .execute(executor)
            .await?;
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

    /// Insert a [AlertEvent]
    pub async fn insert_alert_event(
        executor: &mut sqlx::SqliteConnection,
        event: &AlertEvent,
    ) -> Result<()> {
        query("INSERT INTO alert_events VALUES (NULL, ?, ?, ?)")
            .bind(&event.alert_id)
            .bind(event.timestamp)
            .bind(&event.reason)
            .execute(executor)
            .await?;
        Ok(())
    }

    /// Insert a [ReminderEvent]
    pub async fn insert_reminder_event(
        executor: &mut sqlx::SqliteConnection,
        event: &ReminderEvent,
    ) -> Result<()> {
        query("INSERT INTO reminder_events VALUES (NULL, ?, ?, ?)")
            .bind(&event.reminder_id)
            .bind(event.timestamp)
            .bind(&event.reason)
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
        Self::insert_alert_event(
            &mut self.db.conn,
            &AlertEvent {
                id: Ref::new(0),
                alert_id,
                timestamp,
                reason: Reason::Ignored,
            },
        )
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

    /// Gets all [AlertEvent]s for a specific alert in a time range
    pub async fn get_alert_events(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        alert_id: Ref<Alert>,
    ) -> Result<Vec<AlertEvent>> {
        let events: Vec<AlertEvent> = query_as(
            "SELECT
                e.id,
                e.alert_id,
                e.timestamp,
                e.reason
            FROM alert_events e, (SELECT ? AS start, ? AS end) p
            WHERE e.alert_id = ? 
                AND e.timestamp > p.start 
                AND e.timestamp <= p.end
            ORDER BY e.timestamp ASC",
        )
        .bind(start)
        .bind(end)
        .bind(alert_id)
        .fetch_all(self.db.executor())
        .await?;

        Ok(events)
    }

    /// Gets all [infused::ReminderEvent]s for reminders of a specific alert in a time range
    pub async fn get_alert_reminder_events(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        alert_id: Ref<Alert>,
    ) -> Result<Vec<infused::ReminderEvent>> {
        let events: Vec<infused::ReminderEvent> = query_as(
            "SELECT
                e.id,
                e.reminder_id,
                e.timestamp,
                e.reason,
                r.alert_id,
                r.threshold,
                a.usage_limit * r.threshold AS threshold_duration,
                r.message,
                r.active
            FROM reminder_events e
            INNER JOIN reminders r ON e.reminder_id = r.id
            INNER JOIN alerts a ON r.alert_id = a.id
            WHERE r.alert_id = ?
                AND e.timestamp > ?
                AND e.timestamp <= ?
            ORDER BY e.timestamp ASC",
        )
        .bind(alert_id)
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(events)
    }
}
