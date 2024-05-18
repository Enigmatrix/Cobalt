use std::io::Write;

use rusqlite::{params, Connection, Statement};
use util::{
    config::Config,
    error::{Context, Result},
};

use crate::{
    entities::{
        Alert, AlertEvent, App, AppIdentity, InteractionPeriod, Ref, Reminder, ReminderEvent,
        Session, Target, TimeFrame, Timestamp, TriggerAction, Usage, VersionedId,
    },
    migrations::Migrator,
    table::Table,
};

#[derive(Debug, PartialEq, Eq)]
pub enum FoundOrInserted<T: Table> {
    Found(Ref<T>),
    Inserted(Ref<T>),
}

impl<T: Table> From<FoundOrInserted<T>> for Ref<T> {
    fn from(value: FoundOrInserted<T>) -> Self {
        match value {
            FoundOrInserted::Found(id) => id,
            FoundOrInserted::Inserted(id) => id,
        }
    }
}

macro_rules! prepare_stmt {
    ($conn: expr, $sql:expr) => {{
        let sql = $sql;
        $conn
            .prepare(&sql)
            .with_context(|| format!("prepare stmt: {sql}"))
    }};
}

macro_rules! insert_stmt {
    ($conn: expr, $mdl:ty) => {{
        let name = <$mdl as $crate::table::Table>::name();
        let sql = format!(
            "INSERT INTO {} VALUES (NULL{})",
            name,
            ", ?".repeat(<$mdl as $crate::table::Table>::columns().len() - 1)
        );
        prepare_stmt!($conn, sql)
    }};
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(config: &Config) -> Result<Self> {
        let path = config.connection_string();
        let mut conn = Connection::open(path).context("open conn")?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("enable WAL")?;
        Migrator::new(&mut conn).migrate().context("migrate")?;
        Ok(Self { conn })
    }
}

pub struct UsageWriter<'a> {
    conn: &'a Connection,
    find_or_insert_app: Statement<'a>,
    insert_session: Statement<'a>,
    insert_usage: Statement<'a>,
    insert_interaction_period: Statement<'a>,
}

impl<'a> UsageWriter<'a> {
    pub fn new(db: &'a Database) -> Result<Self> {
        let conn = &db.conn;
        let find_or_insert_app = prepare_stmt!(
            conn,
            // We set found=1 to force this query to return a result row regardless of
            // conflict result.
            "INSERT INTO apps (identity_is_win32, identity_path_or_aumid) VALUES (?, ?) ON CONFLICT
                DO UPDATE SET found = 1 RETURNING id, found"
        )?;
        let insert_session = insert_stmt!(conn, Session)?;
        let insert_usage = insert_stmt!(conn, Usage)?;
        let insert_interaction_period = insert_stmt!(conn, InteractionPeriod)?;
        Ok(Self {
            conn,
            find_or_insert_app,
            insert_session,
            insert_usage,
            insert_interaction_period,
        })
    }

    /// Find or insert a [App] by its [AppIdentity]
    pub fn find_or_insert_app(&mut self, identity: &AppIdentity) -> Result<FoundOrInserted<App>> {
        let (tag, text0) = Self::destructure_identity(identity);
        let (app_id, found) = self
            .find_or_insert_app
            .query_row(params![tag, text0], |r| Ok((r.get(0)?, r.get(1)?)))?;

        Ok(if found {
            FoundOrInserted::Found(app_id)
        } else {
            FoundOrInserted::Inserted(app_id)
        })
    }

    /// Insert a [Session] into the [Database]
    pub fn insert_session(&mut self, session: &mut Session) -> Result<()> {
        self.insert_session
            .execute(params![session.app, session.title])?;
        session.id = Ref::new(self.last_insert_id());
        Ok(())
    }

    /// Insert a [Usage] into the [Database]
    pub fn insert_usage(&mut self, usage: &Usage) -> Result<()> {
        self.insert_usage
            .execute(params![usage.session, usage.start, usage.end])?;
        Ok(())
    }

    /// Insert a [InteractionPeriod] into the [Database]
    pub fn insert_interaction_period(
        &mut self,
        interaction_period: &InteractionPeriod,
    ) -> Result<()> {
        self.insert_interaction_period.execute(params![
            interaction_period.start,
            interaction_period.end,
            interaction_period.mouseclicks,
            interaction_period.keystrokes
        ])?;
        Ok(())
    }

    fn last_insert_id(&self) -> u64 {
        self.conn.last_insert_rowid() as u64
    }

    fn destructure_identity(identity: &AppIdentity) -> (u64, &str) {
        match identity {
            AppIdentity::Win32 { path } => (1, path),
            AppIdentity::Uwp { aumid } => (0, aumid),
        }
    }
}

/// Reference to hold statements regarding [App] updates
pub struct AppUpdater<'a> {
    conn: &'a Connection,
    update_app: Statement<'a>,
    update_app_icon_size: Statement<'a>,
}

impl<'a> AppUpdater<'a> {
    /// Initialize a [AppUpdater] from a given [Database]
    pub fn new(db: &'a mut Database) -> Result<Self> {
        let conn = &db.conn;
        Ok(Self {
            update_app: prepare_stmt!(
                conn,
                "UPDATE apps SET
                    name = ?,
                    description = ?,
                    company = ?,
                    color = ?,
                    initialized = 1
                WHERE id = ?"
            )?,
            update_app_icon_size: prepare_stmt!(
                conn,
                "UPDATE apps SET
                    icon = ZEROBLOB(?)
                WHERE id = ?"
            )?,
            conn,
        })
    }

    /// Update the [App] with additional information
    pub fn update_app(&mut self, app: &App) -> Result<()> {
        self.update_app.execute(params![
            app.name,
            app.description,
            app.company,
            app.color,
            app.id
        ])?;
        Ok(())
    }

    /// Get a reference to the [App] icon's writer
    pub fn app_icon_writer(&mut self, app_id: Ref<App>, size: u64) -> Result<impl Write + 'a> {
        self.update_app_icon_size(app_id.clone(), size)?;
        Ok(self.conn.blob_open(
            rusqlite::DatabaseName::Main,
            "apps",
            "icon",
            app_id.inner as i64,
            false,
        )?)
    }

    /// Update the [App] with additional information
    fn update_app_icon_size(&mut self, id: Ref<App>, icon_size: u64) -> Result<()> {
        self.update_app_icon_size.execute(params![icon_size, id])?;
        Ok(())
    }
}

/// Reference to hold statements regarding [Alert] queries
pub struct AlertManager<'a> {
    get_tag_apps: Statement<'a>,
    triggered_alerts: Statement<'a>,
    triggered_reminders: Statement<'a>,
    insert_alert_event: Statement<'a>,
    insert_reminder_event: Statement<'a>,
}

impl<'a> AlertManager<'a> {
    /// Initialize a [AlertManager] from a given [Database]
    pub fn new(db: &'a mut Database) -> Result<Self> {
        let conn = &db.conn;
        let get_tag_apps = prepare_stmt!(conn, "SELECT app_id FROM _app_tags WHERE tag_id = ?")?;
        let triggered_alerts = prepare_stmt!(conn, include_str!("queries/triggered_alerts.sql"))?;
        let triggered_reminders =
            prepare_stmt!(conn, include_str!("queries/triggered_reminders.sql"))?;
        let insert_alert_event = insert_stmt!(conn, AlertEvent)?;
        let insert_reminder_event = insert_stmt!(conn, ReminderEvent)?;
        Ok(Self {
            get_tag_apps,
            triggered_alerts,
            triggered_reminders,
            insert_alert_event,
            insert_reminder_event,
        })
    }

    /// Gets all [App]s under the [Target]
    pub fn target_apps(&mut self, target: &Target) -> Result<Vec<Ref<App>>> {
        Ok(match target {
            Target::Tag(tag) => self
                .get_tag_apps
                .query_map(params![tag], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?,
            // this will only return one result, but we get a row iterator nonetheless
            Target::App(app) => vec![app.clone()],
        })
    }

    // TODO optim: use a single query to get all triggered alerts and reminders
    // TODO optim: or use a single transaction for the below two.

    /// Get all [Alert]s that are triggered, including when they were triggered
    pub fn triggered_alerts(&mut self) -> Result<Vec<(Alert, Option<Timestamp>)>> {
        let (day_start, week_start, month_start) = Self::time_starts();
        let result = self
            .triggered_alerts
            .query_map(params![day_start, week_start, month_start,], |row| {
                Ok((Self::row_to_alert(row)?, row.get(9)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    /// Get all [Reminder]s that are triggered, except those that are already handled
    pub fn triggered_reminders(&mut self) -> Result<Vec<Reminder>> {
        let (day_start, week_start, month_start) = Self::time_starts();
        let result = self
            .triggered_reminders
            .query_map(params![day_start, week_start, month_start,], |row| {
                Self::row_to_reminder(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    /// Insert a [AlertEvent]
    pub fn insert_alert_event(&mut self, event: &AlertEvent) -> Result<()> {
        self.insert_alert_event.execute(params![
            event.alert.guid,
            event.alert.version,
            event.timestamp,
        ])?;
        Ok(())
    }

    /// Insert a [ReminderEvent]
    pub fn insert_reminder_event(&mut self, event: &ReminderEvent) -> Result<()> {
        self.insert_reminder_event.execute(params![
            event.reminder.guid,
            event.reminder.version,
            event.timestamp,
        ])?;
        Ok(())
    }

    fn row_to_alert(row: &rusqlite::Row) -> Result<Alert, rusqlite::Error> {
        let app_id: Option<Ref<App>> = row.get(2)?;
        Ok(Alert {
            id: Ref::new(VersionedId {
                guid: row.get(0)?,
                version: row.get(1)?,
            }),
            target: if let Some(app_id) = app_id {
                Target::App(app_id)
            } else {
                Target::Tag(row.get(3)?)
            },
            usage_limit: row.get(4)?,
            time_frame: match row.get(5)? {
                0 => TimeFrame::Daily,
                1 => TimeFrame::Weekly,
                2 => TimeFrame::Monthly,
                _ => unreachable!("time frame"),
            },
            trigger_action: match row.get(8)? {
                0 => TriggerAction::Kill,
                1 => TriggerAction::Dim(row.get(6)?),
                2 => TriggerAction::Message(row.get(7)?),
                _ => unreachable!("trigger action"),
            },
        })
    }

    fn row_to_reminder(row: &rusqlite::Row) -> Result<Reminder, rusqlite::Error> {
        Ok(Reminder {
            id: Ref::new(VersionedId {
                guid: row.get(0)?,
                version: row.get(1)?,
            }),
            alert: Ref::new(VersionedId {
                guid: row.get(2)?,
                version: row.get(3)?,
            }),
            threshold: row.get(4)?,
            message: row.get(5)?,
        })
    }

    fn time_starts() -> (Timestamp, Timestamp, Timestamp) {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::entities::Tag;

    use super::*;

    fn test_db() -> Result<Database> {
        let mut conn = Connection::open_in_memory()?;
        Migrator::new(&mut conn).migrate()?;
        Ok(Database { conn })
    }

    #[test]
    fn insert_new_app() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        let res = writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        assert_eq!(res, FoundOrInserted::Inserted(Ref::<App>::new(1)));
        let (init, path): (bool, String) =
            db.conn.query_row("SELECT * FROM apps", params![], |f| {
                Ok((f.get("initialized")?, f.get("identity_path_or_aumid")?))
            })?;
        assert_eq!("notepad.exe", path);
        assert_eq!(false, init);
        Ok(())
    }

    #[test]
    fn found_old_app() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        let res = writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        assert_eq!(res, FoundOrInserted::Inserted(Ref::<App>::new(1)));
        let res = writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        assert_eq!(res, FoundOrInserted::Found(Ref::<App>::new(1)));
        Ok(())
    }

    #[test]
    fn insert_session() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        let mut sess = Session {
            id: Default::default(),
            app: Ref::new(1),
            title: "TITLE".to_string(),
        };
        writer.insert_session(&mut sess)?;
        let sess_from_db = db
            .conn
            .query_row("SELECT * FROM sessions", params![], |f| {
                Ok(Session {
                    id: f.get(0)?,
                    app: f.get(1)?,
                    title: f.get(2)?,
                })
            })?;
        assert_eq!(sess, sess_from_db);
        Ok(())
    }

    #[test]
    fn insert_usage() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        let mut sess = Session {
            id: Default::default(),
            app: Ref::new(1),
            title: "TITLE".to_string(),
        };
        writer.insert_session(&mut sess)?;
        let mut usage = Usage {
            id: Default::default(),
            session: sess.id.clone(),
            start: 42,
            end: 420,
        };
        writer.insert_usage(&mut usage)?;
        let usage_from_db = db.conn.query_row("SELECT * FROM usages", params![], |f| {
            Ok(Usage {
                id: f.get(0)?,
                session: f.get(1)?,
                start: f.get(2)?,
                end: f.get(3)?,
            })
        })?;
        usage.id = Ref::new(1); // not updated by writer
        assert_eq!(usage, usage_from_db);
        Ok(())
    }

    #[test]
    fn insert_interaction_period() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        let mut ip = InteractionPeriod {
            id: Default::default(),
            start: 42,
            end: 420,
            mouseclicks: 23,
            keystrokes: 45,
        };
        writer.insert_interaction_period(&ip)?;
        let ip_from_db =
            db.conn
                .query_row("SELECT * FROM interaction_periods", params![], |f| {
                    Ok(InteractionPeriod {
                        id: f.get(0)?,
                        start: f.get(1)?,
                        end: f.get(2)?,
                        mouseclicks: f.get(3)?,
                        keystrokes: f.get(4)?,
                    })
                })?;
        ip.id = Ref::new(1);
        assert_eq!(ip, ip_from_db);
        Ok(())
    }

    #[test]
    fn update_app() -> Result<()> {
        let mut db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        let identity = AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        };
        writer.find_or_insert_app(&identity)?;
        drop(writer);
        let mut updater = AppUpdater::new(&mut db)?;
        let app = App {
            id: Ref::new(1),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: identity.clone(), // ignored by query
        };
        updater.update_app(&app)?;
        drop(updater);

        let (init, app_from_db): (bool, _) =
            db.conn.query_row("SELECT * FROM apps", params![], |f| {
                Ok((
                    f.get("initialized")?,
                    App {
                        id: f.get("id")?,
                        name: f.get("name")?,
                        description: f.get("description")?,
                        company: f.get("company")?,
                        color: f.get("color")?,
                        identity: if f.get("identity_is_win32")? {
                            AppIdentity::Win32 {
                                path: f.get("identity_path_or_aumid")?,
                            }
                        } else {
                            AppIdentity::Uwp {
                                aumid: f.get("identity_path_or_aumid")?,
                            }
                        },
                    },
                ))
            })?;

        assert_eq!(true, init);
        assert_eq!(app, app_from_db);
        Ok(())
    }

    #[test]
    fn update_app_icon() -> Result<()> {
        let mut db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        let identity = AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        };
        writer.find_or_insert_app(&identity)?;
        drop(writer);

        let icon = &[42, 233].repeat(50); // 50 * 2 = 100 bytes length

        {
            let mut updater = AppUpdater::new(&mut db)?;
            let mut writer = updater.app_icon_writer(Ref::new(1), icon.len() as u64)?;
            writer.write_all(&icon)?;
        }

        let icon_from_db: Vec<u8> = db
            .conn
            .query_row("SELECT icon FROM apps", params![], |f| f.get("icon"))?;

        assert_eq!(icon, &icon_from_db);
        Ok(())
    }

    #[test]
    fn target_apps() -> Result<()> {
        let mut db = test_db()?;
        {
            let mut insert_app = insert_stmt!(db.conn, App)?;
            insert_app.execute(params![
                true, true, "name1", "desc1", "comp1", "red1", 1, "path1", false
            ])?;
            insert_app.execute(params![
                true, true, "name2", "desc2", "comp2", "red2", 0, "aumid2", false
            ])?;
            insert_app.execute(params![
                true, true, "name3", "desc3", "comp3", "red3", 1, "path3", false
            ])?;

            let mut insert_tag = insert_stmt!(db.conn, Tag)?;
            insert_tag.execute(params!["tag_name1", "blue1"])?;
            insert_tag.execute(params!["tag_name2", "blue2"])?;
            insert_tag.execute(params!["tag_name3", "blue3"])?;
            insert_tag.execute(params!["tag_name4", "blue4"])?;

            let mut insert_app_tag = prepare_stmt!(
                db.conn,
                "INSERT INTO _app_tags(app_id, tag_id) VALUES (?, ?)"
            )?;
            insert_app_tag.execute(params![1, 1])?;
            insert_app_tag.execute(params![2, 1])?;
            insert_app_tag.execute(params![2, 2])?;
            insert_app_tag.execute(params![3, 2])?;
            insert_app_tag.execute(params![1, 3])?;
        }

        let mut mgr = AlertManager::new(&mut db)?;

        let app1 = App {
            id: Ref::new(1),
            name: "name1".to_string(),
            description: "desc1".to_string(),
            company: "comp1".to_string(),
            color: "red1".to_string(),
            identity: AppIdentity::Win32 {
                path: "path1".to_string(),
            },
        };
        let app2 = App {
            id: Ref::new(2),
            name: "name2".to_string(),
            description: "desc2".to_string(),
            company: "comp2".to_string(),
            color: "red2".to_string(),
            identity: AppIdentity::Uwp {
                aumid: "aumid2".to_string(),
            },
        };
        let app3 = App {
            id: Ref::new(3),
            name: "name3".to_string(),
            description: "desc3".to_string(),
            company: "comp3".to_string(),
            color: "red3".to_string(),
            identity: AppIdentity::Win32 {
                path: "path3".to_string(),
            },
        };

        assert_eq!(
            mgr.target_apps(&Target::App(Ref::new(1)))?,
            vec![app1.id.clone()],
        );

        assert_eq!(
            mgr.target_apps(&Target::App(Ref::new(2)))?,
            vec![app2.id.clone()],
        );

        assert_eq!(
            mgr.target_apps(&Target::App(Ref::new(3)))?,
            vec![app3.id.clone()],
        );

        assert_eq!(
            mgr.target_apps(&Target::Tag(Ref::new(1)))?,
            vec![app1.id.clone(), app2.id.clone()],
        );

        assert_eq!(
            mgr.target_apps(&Target::Tag(Ref::new(2)))?,
            vec![app2.id.clone(), app3.id.clone()],
        );

        assert_eq!(
            mgr.target_apps(&Target::Tag(Ref::new(3)))?,
            vec![app1.id.clone()],
        );

        assert_eq!(mgr.target_apps(&Target::Tag(Ref::new(4)))?, Vec::new());

        Ok(())
    }

    // get triggered alerts queries
    #[test]
    fn triggered_alerts() -> Result<()> {
        // 1. no usages -> no alerts
        // 2. normal case (1 usage exceeding limit) -> 1 alert + no timestamp
        // 3. normal case + alert event in range -> 1 alert + timestamp
        // 4. 2 usages, one before and one within the range,
        //      total exceeding limit but not just one -> no alerts
        // 5. 2 usages, one halfway in (start before) and one within the range,
        //      total exceeding limit but not just one -> no alerts
        Ok(())
    }

    #[test]
    fn insert_alert_event() -> Result<()> {
        let mut db = test_db()?;

        let guid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        {
            let mut insert_app = insert_stmt!(db.conn, App)?;
            insert_app.execute(params![
                true, true, "name1", "desc1", "comp1", "red1", 1, "path1", false
            ])?;
            let mut insert_alert = prepare_stmt!(
                db.conn,
                "INSERT INTO alerts VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )?;
            insert_alert.execute(params![
                guid,
                1,
                1,
                Option::<Ref<Tag>>::None,
                100,
                1,
                0,
                0,
                1
            ])?;
        }

        let mut alert_event = AlertEvent {
            id: Default::default(),
            alert: Ref::new(VersionedId { guid, version: 1 }),
            timestamp: 1337,
        };

        {
            let mut mgr = AlertManager::new(&mut db)?;

            mgr.insert_alert_event(&alert_event)?;
        }

        let alert_event_from_db =
            db.conn
                .query_row("SELECT * FROM alert_events", params![], |f| {
                    Ok(AlertEvent {
                        id: f.get(0)?,
                        alert: Ref::new(VersionedId {
                            guid: f.get(1)?,
                            version: f.get(2)?,
                        }),
                        timestamp: f.get(3)?,
                    })
                })?;
        alert_event.id = Ref::new(1);

        assert_eq!(alert_event, alert_event_from_db);

        Ok(())
    }

    #[test]
    fn insert_reminder_event() -> Result<()> {
        let mut db = test_db()?;

        let alert_guid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        let reminder_guid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c9");
        {
            let mut insert_app = insert_stmt!(db.conn, App)?;
            insert_app.execute(params![
                true, true, "name1", "desc1", "comp1", "red1", 1, "path1", false
            ])?;
            let mut insert_alert = prepare_stmt!(
                db.conn,
                "INSERT INTO alerts VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )?;
            let mut insert_reminder =
                prepare_stmt!(db.conn, "INSERT INTO reminders VALUES (?, ?, ?, ?, ?, ?)")?;
            insert_alert.execute(params![
                alert_guid,
                1,
                1,
                Option::<Ref<Tag>>::None,
                100,
                1,
                0,
                0,
                1
            ])?;
            insert_reminder.execute(params![reminder_guid, 1, alert_guid, 1, 0.75, "hello",])?;
        }

        let mut reminder_event = ReminderEvent {
            id: Default::default(),
            reminder: Ref::new(VersionedId {
                guid: reminder_guid,
                version: 1,
            }),
            timestamp: 1337,
        };

        {
            let mut mgr = AlertManager::new(&mut db)?;

            mgr.insert_reminder_event(&reminder_event)?;
        }

        let reminder_event_from_db =
            db.conn
                .query_row("SELECT * FROM reminder_events", params![], |f| {
                    Ok(ReminderEvent {
                        id: f.get(0)?,
                        reminder: Ref::new(VersionedId {
                            guid: f.get(1)?,
                            version: f.get(2)?,
                        }),
                        timestamp: f.get(3)?,
                    })
                })?;
        reminder_event.id = Ref::new(1);

        assert_eq!(reminder_event, reminder_event_from_db);

        Ok(())
    }
}
