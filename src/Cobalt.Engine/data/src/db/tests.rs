use sqlx::prelude::FromRow;
use sqlx::{query, Row};
use util::future as tokio;

use super::*;
use crate::entities::Tag;
use crate::table::{AlertVersionedId, ReminderVersionedId};

async fn test_db() -> Result<Database> {
    let conn = SqliteConnectOptions::new()
        .in_memory(true)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .connect()
        .await?;
    let mut ret = Database { conn };
    Migrator::new(&mut ret).migrate().await.context("migrate")?;
    Ok(ret)
}

#[tokio::test]
async fn test_up() -> Result<()> {
    test_db().await?;
    Ok(())
}

#[tokio::test]
async fn insert_new_app() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    let res = writer
        .find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })
        .await?;
    assert_eq!(res, FoundOrInserted::Inserted(Ref::<App>::new(1)));

    let res: Vec<(bool, String)> = query("SELECT * FROM apps")
        .map(|r: SqliteRow| (r.get("initialized"), r.get("identity_path_or_aumid")))
        .fetch_all(&mut writer.db.conn)
        .await?;

    assert_eq!(vec![(false, "notepad.exe".to_string())], res);
    Ok(())
}

#[tokio::test]
async fn found_old_app() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    let res = writer
        .find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })
        .await?;
    assert_eq!(res, FoundOrInserted::Inserted(Ref::<App>::new(1)));
    let res = writer
        .find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })
        .await?;
    assert_eq!(res, FoundOrInserted::Found(Ref::<App>::new(1)));
    Ok(())
}

#[tokio::test]
async fn insert_session() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    writer
        .find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })
        .await?;
    let mut sess = Session {
        id: Default::default(),
        app_id: Ref::new(1),
        title: "TITLE".to_string(),
    };
    writer.insert_session(&mut sess).await?;

    let sess_from_db = query_as("SELECT * FROM sessions")
        .fetch_all(&mut writer.db.conn)
        .await?;

    assert_eq!(vec![sess], sess_from_db);
    Ok(())
}

#[tokio::test]
async fn insert_usage() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    writer
        .find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })
        .await?;
    let mut sess = Session {
        id: Default::default(),
        app_id: Ref::new(1),
        title: "TITLE".to_string(),
    };
    writer.insert_session(&mut sess).await?;
    let mut usage = Usage {
        id: Default::default(),
        session_id: sess.id.clone(),
        start: 42,
        end: 420,
    };
    writer.insert_or_update_usage(&mut usage).await?;

    let usage_from_db = query_as("SELECT * FROM usages")
        .fetch_all(&mut writer.db.conn)
        .await?;
    assert_eq!(vec![usage], usage_from_db);
    Ok(())
}

#[tokio::test]
async fn update_usage_after_insert_usage() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    writer
        .find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })
        .await?;
    let mut sess = Session {
        id: Default::default(),
        app_id: Ref::new(1),
        title: "TITLE".to_string(),
    };
    writer.insert_session(&mut sess).await?;
    let mut usage = Usage {
        id: Default::default(),
        session_id: sess.id.clone(),
        start: 42,
        end: 420,
    };
    writer.insert_or_update_usage(&mut usage).await?;
    usage.end = 1337;
    writer.insert_or_update_usage(&mut usage).await?;

    let usage_from_db = query_as("SELECT * FROM usages")
        .fetch_all(&mut writer.db.conn)
        .await?;
    assert_eq!(vec![usage], usage_from_db);
    assert_ne!(usage_from_db[0].end, 420);
    Ok(())
}

#[tokio::test]
async fn insert_interaction_period() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    writer
        .find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })
        .await?;
    let mut ip = InteractionPeriod {
        id: Default::default(),
        start: 42,
        end: 420,
        mouse_clicks: 23,
        key_strokes: 45,
    };
    writer.insert_interaction_period(&ip).await?;

    let ip_from_db = query_as("SELECT * FROM interaction_periods")
        .fetch_all(&mut writer.db.conn)
        .await?;
    ip.id = Ref::new(1);
    assert_eq!(vec![ip], ip_from_db);
    Ok(())
}

#[tokio::test]
async fn update_app() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    let identity = AppIdentity::Win32 {
        path: "notepad.exe".to_string(),
    };
    writer.find_or_insert_app(&identity).await?;
    let mut updater = AppUpdater::new(writer.db)?;
    let icon = [42, 233].repeat(50); // 50 * 2 = 100 bytes length
    let app = App {
        id: Ref::new(1),
        name: "name".to_string(),
        description: "desc".to_string(),
        company: "comp".to_string(),
        color: "red".to_string(),
        identity: identity.clone(), // ignored by query
        icon: Some(icon.clone()),
    };
    updater.update_app(&app).await?;

    #[derive(FromRow, PartialEq, Eq, Debug)]
    struct Res {
        initialized: bool,
        #[sqlx(flatten)]
        app: App,
        icon: Vec<u8>,
    }

    let res: Vec<Res> = query_as("SELECT * FROM apps")
        .fetch_all(updater.db.executor())
        .await?;

    assert_eq!(
        vec![Res {
            initialized: true,
            app,
            icon
        }],
        res
    );
    Ok(())
}

async fn insert_app_raw(
    db: &mut Database,
    a0: bool,
    a1: bool,
    a2: &str,
    a3: &str,
    a4: &str,
    a5: &str,
    a6: u32,
    a7: &str,
    a8: Option<Vec<u8>>,
) -> Result<Ref<App>> {
    let res = query("INSERT INTO apps VALUES (NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(a0)
        .bind(a1)
        .bind(a2)
        .bind(a3)
        .bind(a4)
        .bind(a5)
        .bind(a6)
        .bind(a7)
        .bind(a8)
        .execute(db.executor())
        .await?;
    Ok(Ref::new(res.last_insert_rowid()))
}

async fn insert_alert_raw(
    db: &mut Database,
    id: i64,
    vers: i64,
    a0: Option<i64>,
    a1: Option<i64>,
    a2: i64,
    a3: i64,
    a4: i64,
    a5: &str,
    a6: i64,
) -> Result<()> {
    query("INSERT INTO alerts VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(id)
        .bind(vers)
        .bind(a0)
        .bind(a1)
        .bind(a2)
        .bind(a3)
        .bind(a4)
        .bind(a5)
        .bind(a6)
        .execute(db.executor())
        .await?;
    Ok(())
}

async fn insert_reminder_raw(
    db: &mut Database,
    id: i64,
    vers: i64,
    a0: i64,
    a1: i64,
    a2: f64,
    a3: &str,
) -> Result<()> {
    query("INSERT INTO reminders VALUES (?, ?, ?, ?, ?, ?)")
        .bind(id)
        .bind(vers)
        .bind(a0)
        .bind(a1)
        .bind(a2)
        .bind(a3)
        .execute(db.executor())
        .await?;
    Ok(())
}

async fn insert_tag_raw(db: &mut Database, a0: &str, a1: &str) -> Result<Ref<Tag>> {
    let res = query("INSERT INTO tags VALUES (NULL, ?, ?)")
        .bind(a0)
        .bind(a1)
        .execute(db.executor())
        .await?;
    Ok(Ref::new(res.last_insert_rowid()))
}

async fn insert_app_tag_raw(db: &mut Database, a0: i64, a1: i64) -> Result<()> {
    query("INSERT INTO _app_tags VALUES (?, ?)")
        .bind(a0)
        .bind(a1)
        .execute(db.executor())
        .await?;
    Ok(())
}

#[tokio::test]
async fn target_apps() -> Result<()> {
    let mut db = test_db().await?;
    {
        insert_app_raw(
            &mut db, true, true, "name1", "desc1", "comp1", "red1", 1, "path1", None,
        )
        .await?;
        insert_app_raw(
            &mut db, true, true, "name2", "desc2", "comp2", "red2", 0, "aumid2", None,
        )
        .await?;
        insert_app_raw(
            &mut db, true, true, "name3", "desc3", "comp3", "red3", 1, "path3", None,
        )
        .await?;

        insert_tag_raw(&mut db, "tag_name1", "blue1").await?;
        insert_tag_raw(&mut db, "tag_name2", "blue2").await?;
        insert_tag_raw(&mut db, "tag_name3", "blue3").await?;
        insert_tag_raw(&mut db, "tag_name4", "blue4").await?;

        insert_app_tag_raw(&mut db, 1, 1).await?;
        insert_app_tag_raw(&mut db, 2, 1).await?;
        insert_app_tag_raw(&mut db, 2, 2).await?;
        insert_app_tag_raw(&mut db, 3, 2).await?;
        insert_app_tag_raw(&mut db, 1, 3).await?;
    }

    let mut mgr = AlertManager::new(db)?;

    let app1 = App {
        id: Ref::new(1),
        name: "name1".to_string(),
        description: "desc1".to_string(),
        company: "comp1".to_string(),
        color: "red1".to_string(),
        identity: AppIdentity::Win32 {
            path: "path1".to_string(),
        },
        icon: None,
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
        icon: None,
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
        icon: None,
    };

    assert_eq!(
        mgr.target_apps(&Target::App(Ref::new(1))).await?,
        vec![app1.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::App(Ref::new(2))).await?,
        vec![app2.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::App(Ref::new(3))).await?,
        vec![app3.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::Tag(Ref::new(1))).await?,
        vec![app1.id.clone(), app2.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::Tag(Ref::new(2))).await?,
        vec![app2.id.clone(), app3.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::Tag(Ref::new(3))).await?,
        vec![app1.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::Tag(Ref::new(4))).await?,
        Vec::new()
    );

    Ok(())
}

#[tokio::test]
async fn insert_alert_event() -> Result<()> {
    let mut db = test_db().await?;

    let id = 1;
    {
        insert_app_raw(
            &mut db, true, true, "name1", "desc1", "comp1", "red1", 1, "path1", None,
        )
        .await?;
        insert_alert_raw(&mut db, id, 1, Some(1), None, 100, 1, 0, "", 1).await?;
    }

    let mut alert_event = AlertEvent {
        id: Default::default(),
        alert: AlertVersionedId { id, version: 1 },
        timestamp: 1337,
    };

    let mut db = {
        let mut mgr = AlertManager::new(db)?;

        mgr.insert_alert_event(&alert_event).await?;
        mgr.db
    };

    let alert_events_from_db = query_as("SELECT * FROM alert_events")
        .fetch_all(db.executor())
        .await?;
    alert_event.id = Ref::new(1);

    assert_eq!(vec![alert_event], alert_events_from_db);

    Ok(())
}

#[tokio::test]
async fn insert_reminder_event() -> Result<()> {
    let mut db = test_db().await?;

    let alert_id = 1;
    let reminder_id = 1;
    {
        insert_app_raw(
            &mut db, true, true, "name1", "desc1", "comp1", "red1", 1, "path1", None,
        )
        .await?;
        insert_alert_raw(&mut db, alert_id, 1, Some(1), None, 100, 1, 0, "", 1).await?;
        insert_reminder_raw(&mut db, reminder_id, 1, alert_id, 1, 0.75, "hello").await?;
    }

    let mut reminder_event = ReminderEvent {
        id: Default::default(),
        reminder: ReminderVersionedId {
            id: reminder_id,
            version: 1,
        },
        timestamp: 1337,
    };

    let mut db = {
        let mut mgr = AlertManager::new(db)?;

        mgr.insert_reminder_event(&reminder_event).await?;
        mgr.db
    };

    let reminder_events_from_db = query_as("SELECT * FROM reminder_events")
        .fetch_all(db.executor())
        .await?;
    reminder_event.id = Ref::new(1);

    assert_eq!(vec![reminder_event], reminder_events_from_db);

    Ok(())
}

struct Times {
    day_start: u64,
    week_start: u64,
    month_start: u64,
}

struct Tick(u64);

impl ToTicks for Tick {
    fn to_ticks(&self) -> u64 {
        self.0
    }
}

impl TimeSystem for Times {
    type Ticks = Tick;

    fn day_start(&self) -> Self::Ticks {
        Tick(self.day_start)
    }

    fn week_start(&self) -> Self::Ticks {
        Tick(self.week_start)
    }

    fn month_start(&self) -> Self::Ticks {
        Tick(self.month_start)
    }
}

mod arrange {
    use super::*;
    use crate::entities::{TimeFrame, TriggerAction};

    pub async fn app(db: &mut Database, mut app: App) -> Result<App> {
        app.id = insert_app_raw(
            db,
            false,
            false,
            &app.name,
            &app.description,
            &app.company,
            &app.color,
            if let AppIdentity::Win32 { .. } = app.identity {
                1
            } else {
                0
            },
            match &app.identity {
                AppIdentity::Win32 { path } => path,
                AppIdentity::Uwp { aumid } => aumid,
            },
            app.icon.clone(),
        )
        .await?;
        Ok(app)
    }

    pub async fn session(db: &mut Database, mut session: Session) -> Result<Session> {
        let res = query("INSERT INTO sessions VALUES (NULL, ?, ?)")
            .bind(session.app_id.clone())
            .bind(session.title.clone())
            .execute(db.executor())
            .await?;
        session.id = Ref::new(res.last_insert_rowid());
        Ok(session)
    }

    pub async fn usage(db: &mut Database, mut usage: Usage) -> Result<Usage> {
        let res = query("INSERT INTO usages VALUES (NULL, ?, ?, ?)")
            .bind(usage.session_id.clone())
            .bind(usage.start)
            .bind(usage.end)
            .execute(db.executor())
            .await?;
        usage.id = Ref::new(res.last_insert_rowid());
        Ok(usage)
    }

    // pub fn interaction_period(
    //     db: &mut Database,
    //     mut ip: InteractionPeriod,
    // ) -> Result<InteractionPeriod> {
    //     db.conn.execute(
    //         "INSERT INTO interaction_periods VALUES (NULL, ?, ?, ?, ?)",
    //         params![ip.start, ip.end, ip.mouseclicks, ip.keystrokes],
    //     )?;
    //     ip.id = Ref::new(db.conn.last_insert_rowid() as u64);
    //     Ok(ip)
    // }

    pub async fn tag(db: &mut Database, mut tag: Tag) -> Result<Tag> {
        tag.id = insert_tag_raw(db, &tag.name, &tag.color).await?;
        Ok(tag)
    }

    pub async fn app_tags(db: &mut Database, app_id: Ref<App>, tag_id: Ref<Tag>) -> Result<()> {
        insert_app_tag_raw(db, app_id.0, tag_id.0).await?;
        Ok(())
    }

    pub async fn alert(db: &mut Database, alert: Alert) -> Result<Alert> {
        let time_frame = match alert.time_frame {
            TimeFrame::Daily => 0,
            TimeFrame::Weekly => 1,
            TimeFrame::Monthly => 2,
        };

        let (dim_duration, message_content, tag) = match &alert.trigger_action {
            TriggerAction::Kill => (None, None, 0),
            TriggerAction::Dim(dur) => (Some(*dur), None, 1),
            TriggerAction::Message(content) => (None, Some(content.to_string()), 2),
        };

        let (app_id, tag_id) = match alert.target.clone() {
            Target::App(app) => (Some(app.0), None),
            Target::Tag(tag) => (None, Some(tag.0)),
        };

        insert_alert_raw(
            db,
            alert.id.id,
            alert.id.version,
            app_id,
            tag_id,
            alert.usage_limit,
            time_frame,
            dim_duration.unwrap_or(0),
            &message_content.unwrap_or("".to_string()),
            tag,
        )
        .await?;
        Ok(alert)
    }

    pub async fn reminder(db: &mut Database, reminder: Reminder) -> Result<Reminder> {
        insert_reminder_raw(
            db,
            reminder.id.id,
            reminder.id.version,
            reminder.alert.id,
            reminder.alert.version,
            reminder.threshold,
            &reminder.message,
        )
        .await?;
        Ok(reminder)
    }

    pub async fn alert_event(db: &mut Database, mut event: AlertEvent) -> Result<AlertEvent> {
        let res = query("INSERT INTO alert_events VALUES (NULL, ?, ?, ?)")
            .bind(event.alert.id)
            .bind(event.alert.version)
            .bind(event.timestamp)
            .execute(db.executor())
            .await?;
        event.id = Ref::new(res.last_insert_rowid());
        Ok(event)
    }

    pub async fn reminder_event(
        db: &mut Database,
        mut event: ReminderEvent,
    ) -> Result<ReminderEvent> {
        let res = query("INSERT INTO reminder_events VALUES (NULL, ?, ?, ?)")
            .bind(event.reminder.id)
            .bind(event.reminder.version)
            .bind(event.timestamp)
            .execute(db.executor())
            .await?;
        event.id = Ref::new(res.last_insert_rowid());
        Ok(event)
    }

    pub fn to_id(n: i64) -> i64 {
        n
    }
}

mod triggered_alerts {
    use super::*;
    use crate::entities::{TimeFrame, TriggerAction};
    use crate::table::VersionedId;

    #[tokio::test]
    async fn no_alerts() -> Result<()> {
        let db = test_db().await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 0,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(alerts, vec![]);
        Ok(())
    }

    #[tokio::test]
    async fn one_alert_one_app_no_sessions() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let _alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 0,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(alerts, vec![]);
        Ok(())
    }

    #[tokio::test]
    async fn one_alert_one_app_one_session_no_usages() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let _session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        let _alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 0,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(alerts, vec![]);
        Ok(())
    }

    #[tokio::test]
    async fn triggered_correctly() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 0,
                end: 100,
            },
        )
        .await?;

        let alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 0,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            alerts,
            vec![TriggeredAlert {
                alert,
                timestamp: None,
                name: "name".to_string()
            }]
        );
        Ok(())
    }

    #[tokio::test]
    async fn halfway_usage_does_not_trigger() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 0,
                end: 100,
            },
        )
        .await?;

        let alert1 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let _alert2 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(2),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 51,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 50,
                week_start: 50,
                month_start: 50,
            })
            .await?;
        assert_eq!(
            alerts,
            vec![TriggeredAlert {
                alert: alert1,
                timestamp: None,
                name: "name".to_string()
            }]
        ); // alert2 is not triggered
        Ok(())
    }

    #[tokio::test]
    async fn exactly_one_alert_from_one_usage_despite_lower_version_existing() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 0,
                end: 100,
            },
        )
        .await?;

        let _alert1 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 10,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let alert2 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 2,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 0,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            alerts,
            vec![TriggeredAlert {
                alert: alert2,
                timestamp: None,
                name: "name".to_string()
            }]
        );
        Ok(())
    }

    #[tokio::test]
    async fn alert_event_after_start() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 0,
                end: 100,
            },
        )
        .await?;

        let alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let _alert_event = arrange::alert_event(
            &mut db,
            AlertEvent {
                id: Ref::default(),
                alert: alert.id.0.clone().into(),
                timestamp: 75,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 50,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            alerts,
            vec![TriggeredAlert {
                alert,
                timestamp: Some(75),
                name: "name".to_string()
            }]
        );
        Ok(())
    }

    #[tokio::test]
    async fn alert_event_before_start() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 0,
                end: 100,
            },
        )
        .await?;

        let alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let _alert_event = arrange::alert_event(
            &mut db,
            AlertEvent {
                id: Ref::default(),
                alert: alert.id.0.clone().into(),
                timestamp: 25,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 50,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            alerts,
            vec![TriggeredAlert {
                alert,
                timestamp: None,
                name: "name".to_string()
            }]
        );
        Ok(())
    }

    #[tokio::test]
    async fn weekly_event_triggered() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 0,
                end: 100,
            },
        )
        .await?;

        let alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 50,
                time_frame: TimeFrame::Weekly,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 1000,
                week_start: 50,
                month_start: 1000,
            })
            .await?;
        assert_eq!(
            alerts,
            vec![TriggeredAlert {
                alert,
                timestamp: None,
                name: "name".to_string()
            }]
        );
        Ok(())
    }

    #[tokio::test]
    async fn monthly_event_triggered() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 0,
                end: 100,
            },
        )
        .await?;

        let alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 50,
                time_frame: TimeFrame::Monthly,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 1000,
                week_start: 1000,
                month_start: 50,
            })
            .await?;
        assert_eq!(
            alerts,
            vec![TriggeredAlert {
                alert,
                timestamp: None,
                name: "name".to_string()
            }]
        );
        Ok(())
    }

    #[tokio::test]
    async fn multiple_usages() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 0,
                end: 10,
            },
        )
        .await?;

        // 25 units
        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 10,
                end: 75,
            },
        )
        .await?;

        // 25 units
        for i in 0..25 {
            arrange::usage(
                &mut db,
                Usage {
                    id: Ref::default(),
                    session_id: session.id.clone(),
                    start: 200 + i,
                    end: 200 + i + 1,
                },
            )
            .await?;
        }

        let alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 50,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            alerts,
            vec![TriggeredAlert {
                alert,
                timestamp: None,
                name: "name".to_string()
            }]
        );
        Ok(())
    }

    #[tokio::test]
    async fn multiple_usages_less_than_usage_limit() -> Result<()> {
        let mut db = test_db().await?;

        let app = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 0,
                end: 10,
            },
        )
        .await?;

        // 10 units
        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 65,
                end: 75,
            },
        )
        .await?;

        // 25 units
        for i in 0..25 {
            arrange::usage(
                &mut db,
                Usage {
                    id: Ref::default(),
                    session_id: session.id.clone(),
                    start: 200 + i,
                    end: 200 + i + 1,
                },
            )
            .await?;
        }

        let _alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 50,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(alerts, vec![]);
        Ok(())
    }

    #[tokio::test]
    async fn using_tags() -> Result<()> {
        let mut db = test_db().await?;

        let app1 = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let app2 = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name2".to_string(),
                description: "desc2".to_string(),
                company: "comp2".to_string(),
                color: "red2".to_string(),
                identity: AppIdentity::Uwp {
                    aumid: "aumid2".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let empty_tag = arrange::tag(
            &mut db,
            Tag {
                id: Ref::default(),
                name: "emptytag".to_string(),
                color: "e1".to_string(),
            },
        )
        .await?;
        let tag1 = arrange::tag(
            &mut db,
            Tag {
                id: Ref::default(),
                name: "tag_name1".to_string(),
                color: "blue1".to_string(),
            },
        )
        .await?;
        arrange::app_tags(&mut db, app1.id.clone(), tag1.id.clone()).await?;
        let tag2 = arrange::tag(
            &mut db,
            Tag {
                id: Ref::default(),
                name: "tag_name2".to_string(),
                color: "blue2".to_string(),
            },
        )
        .await?;
        arrange::app_tags(&mut db, app1.id.clone(), tag2.id.clone()).await?;
        arrange::app_tags(&mut db, app2.id.clone(), tag2.id.clone()).await?;

        // emptytag = []
        // tag1 = [app1]
        // tag2 = [app1, app2]

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app1.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?
        .id;

        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session,
                start: 0,
                end: 100,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app2.id.clone(),
                title: "title2".to_string(),
            },
        )
        .await?
        .id;

        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session_id: session,
                start: 0,
                end: 100,
            },
        )
        .await?;

        let _empty = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::Tag(empty_tag.id.clone()),
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let _alert1 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(2),
                    version: 1,
                }),
                target: Target::Tag(tag1.id.clone()),
                usage_limit: 120,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let alert2 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(3),
                    version: 1,
                }),
                target: Target::Tag(tag2.id.clone()),
                usage_limit: 200,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let alert3 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(4),
                    version: 1,
                }),
                target: Target::Tag(tag1.id.clone()),
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                day_start: 0,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            alerts,
            vec![
                TriggeredAlert {
                    alert: alert2,
                    timestamp: None,
                    name: "tag_name2".to_string()
                },
                TriggeredAlert {
                    alert: alert3,
                    timestamp: None,
                    name: "tag_name1".to_string()
                }
            ]
        );
        Ok(())
    }
}

mod triggered_reminders {
    use super::*;
    use crate::entities::{TimeFrame, TriggerAction};
    use crate::table::VersionedId;

    // generate an alert which will not fire but the reminder will.
    async fn gen_alert(db: &mut Database) -> Result<Ref<Alert>> {
        let app = arrange::app(
            db,
            App {
                id: Ref::default(),
                name: "name".to_string(),
                description: "desc".to_string(),
                company: "comp".to_string(),
                color: "red".to_string(),
                identity: AppIdentity::Win32 {
                    path: "path".to_string(),
                },
                icon: None,
            },
        )
        .await?;

        let session = arrange::session(
            db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
            },
        )
        .await?;

        let _usage = arrange::usage(
            db,
            Usage {
                id: Ref::default(),
                session_id: session.id.clone(),
                start: 50,
                end: 150,
            },
        )
        .await?;

        let alert = arrange::alert(
            db,
            Alert {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                target: Target::App(app.id.clone()),
                usage_limit: 200,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
            },
        )
        .await?;
        Ok(alert.id)
    }

    #[tokio::test]
    pub async fn no_reminders() -> Result<()> {
        let mut db = test_db().await?;

        let _alert = gen_alert(&mut db).await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                day_start: 50,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            reminders
                .into_iter()
                .map(|r| (r.reminder.id, r.name))
                .collect::<Vec<_>>(),
            vec![]
        );
        Ok(())
    }

    #[tokio::test]
    pub async fn triggered_correctly() -> Result<()> {
        let mut db = test_db().await?;

        let alert = gen_alert(&mut db).await?;
        let hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                alert: alert.0.clone().into(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )
        .await?;

        let _not_hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(2),
                    version: 1,
                }),
                alert: alert.0.clone().into(),
                threshold: 0.51,
                message: "hello".to_string(),
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                day_start: 50,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            reminders
                .into_iter()
                .map(|r| (r.reminder.id, r.name))
                .collect::<Vec<_>>(),
            vec![(hit.id, "name".to_string())]
        );
        Ok(())
    }

    #[tokio::test]
    pub async fn only_max_reminder() -> Result<()> {
        let mut db = test_db().await?;

        let alert = gen_alert(&mut db).await?;

        let _not_hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                alert: alert.0.clone().into(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )
        .await?;
        let hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 2,
                }),
                alert: alert.0.clone().into(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                day_start: 50,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            reminders
                .into_iter()
                .map(|r| (r.reminder.id, r.name))
                .collect::<Vec<_>>(),
            vec![(hit.id, "name".to_string())]
        );
        Ok(())
    }

    #[tokio::test]
    pub async fn reminder_event_before_start_does_not_matter() -> Result<()> {
        let mut db = test_db().await?;

        let alert = gen_alert(&mut db).await?;
        let hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                alert: alert.0.clone().into(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )
        .await?;

        arrange::reminder_event(
            &mut db,
            ReminderEvent {
                id: Default::default(),
                reminder: hit.id.0.clone().into(),
                timestamp: 25,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                day_start: 50,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            reminders
                .into_iter()
                .map(|r| (r.reminder.id, r.name))
                .collect::<Vec<_>>(),
            vec![(hit.id, "name".to_string())]
        );
        Ok(())
    }

    #[tokio::test]
    pub async fn reminder_event_after_start_matters() -> Result<()> {
        let mut db = test_db().await?;

        let alert = gen_alert(&mut db).await?;
        let hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                alert: alert.0.clone().into(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )
        .await?;

        arrange::reminder_event(
            &mut db,
            ReminderEvent {
                id: Default::default(),
                reminder: hit.id.0.clone().into(),
                timestamp: 200,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                day_start: 50,
                week_start: 0,
                month_start: 0,
            })
            .await?;
        assert_eq!(
            reminders
                .into_iter()
                .map(|r| (r.reminder.id, r.name))
                .collect::<Vec<_>>(),
            vec![]
        );
        Ok(())
    }
}
