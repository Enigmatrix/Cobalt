use sqlx::prelude::FromRow;
use sqlx::{Row, query};
use util::future as tokio;

use super::*;
use crate::entities::Reason;
use crate::table::Real;

pub async fn test_db() -> Result<Database> {
    let conn = SqliteConnectOptions::new()
        .in_memory(true)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePool::connect_with(conn).await?;
    let conn = pool.acquire().await?;
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
        .find_or_insert_app(
            &AppIdentity::Win32 {
                path: "notepad.exe".to_string(),
            },
            Times {
                now: 400,
                ..Default::default()
            },
        )
        .await?;
    assert_eq!(res, FoundOrInserted::Inserted(Ref::<App>::new(1)));

    let res: Vec<(Option<i64>, i64, String)> = query("SELECT * FROM apps")
        .map(|r: SqliteRow| {
            (
                r.get("initialized_at"),
                r.get("created_at"),
                r.get("identity_text0"),
            )
        })
        .fetch_all(writer.db.executor())
        .await?;

    assert_eq!(vec![(None, 400, "notepad.exe".to_string())], res);
    Ok(())
}

#[tokio::test]
async fn found_old_app() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    let res = writer
        .find_or_insert_app(
            &AppIdentity::Win32 {
                path: "notepad.exe".to_string(),
            },
            Times {
                now: 400,
                ..Default::default()
            },
        )
        .await?;
    assert_eq!(res, FoundOrInserted::Inserted(Ref::<App>::new(1)));
    let res = writer
        .find_or_insert_app(
            &AppIdentity::Win32 {
                path: "notepad.exe".to_string(),
            },
            Times {
                now: 500,
                ..Default::default()
            },
        )
        .await?;
    assert_eq!(res, FoundOrInserted::Found(Ref::<App>::new(1)));

    let res: Vec<(Option<i64>, i64, String)> = query("SELECT * FROM apps")
        .map(|r: SqliteRow| {
            (
                r.get("initialized_at"),
                r.get("created_at"),
                r.get("identity_text0"),
            )
        })
        .fetch_all(writer.db.executor())
        .await?;

    assert_eq!(vec![(None, 400, "notepad.exe".to_string())], res);
    Ok(())
}

#[tokio::test]
async fn insert_session() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    writer
        .find_or_insert_app(
            &AppIdentity::Win32 {
                path: "notepad.exe".to_string(),
            },
            Times::default(),
        )
        .await?;
    let mut sess = Session {
        id: Default::default(),
        app_id: Ref::new(1),
        title: "TITLE".to_string(),
        url: None,
    };
    writer.insert_session(&mut sess).await?;

    let sess_from_db = query_as("SELECT * FROM sessions")
        .fetch_all(writer.db.executor())
        .await?;

    assert_eq!(vec![sess], sess_from_db);
    Ok(())
}

#[tokio::test]
async fn insert_usage() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    writer
        .find_or_insert_app(
            &AppIdentity::Win32 {
                path: "notepad.exe".to_string(),
            },
            Times::default(),
        )
        .await?;
    let mut sess = Session {
        id: Default::default(),
        app_id: Ref::new(1),
        title: "TITLE".to_string(),
        url: None,
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
        .fetch_all(writer.db.executor())
        .await?;
    assert_eq!(vec![usage], usage_from_db);
    Ok(())
}

#[tokio::test]
async fn update_usage_after_insert_usage() -> Result<()> {
    let db = test_db().await?;
    let mut writer = UsageWriter::new(db)?;
    writer
        .find_or_insert_app(
            &AppIdentity::Win32 {
                path: "notepad.exe".to_string(),
            },
            Times::default(),
        )
        .await?;
    let mut sess = Session {
        id: Default::default(),
        app_id: Ref::new(1),
        title: "TITLE".to_string(),
        url: None,
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
        .fetch_all(writer.db.executor())
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
        .find_or_insert_app(
            &AppIdentity::Win32 {
                path: "notepad.exe".to_string(),
            },
            Times::default(),
        )
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
        .fetch_all(writer.db.executor())
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
    writer
        .find_or_insert_app(
            &identity,
            Times {
                now: 400,
                ..Default::default()
            },
        )
        .await?;

    let mut updater = AppUpdater::new(writer.db)?;
    let icon = "icon1".to_string();
    let mut app = App {
        id: Ref::new(1),
        name: "name".to_string(),
        description: "desc".to_string(),
        company: "comp".to_string(),
        color: "red".to_string(),
        identity: identity.clone(), // ignored by query
        icon: Some(icon.clone()),
        tag_id: None,
        created_at: 400,
        updated_at: 400,
    };
    updater
        .update_app(
            &app,
            Times {
                now: 500,
                ..Default::default()
            },
        )
        .await?;

    #[derive(FromRow, PartialEq, Eq, Debug)]
    struct Res {
        #[sqlx(flatten)]
        app: App,
    }

    let res: Vec<Res> = query_as("SELECT * FROM apps")
        .fetch_all(updater.db.executor())
        .await?;

    app.updated_at = 500;
    assert_eq!(vec![Res { app }], res);
    Ok(())
}

async fn insert_app_raw(
    db: &mut Database,
    found: bool,
    name: &str,
    description: &str,
    company: &str,
    color: &str,
    tag_id: Option<i64>,
    identity_tag: u32,
    identity_text0: &str,
    icon: Option<String>,
    created_at: i64,
    initialized_at: Option<i64>,
    updated_at: i64,
) -> Result<Ref<App>> {
    let res = query("INSERT INTO apps VALUES (NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(found)
        .bind(name)
        .bind(description)
        .bind(company)
        .bind(color)
        .bind(icon)
        .bind(tag_id)
        .bind(identity_tag)
        .bind(identity_text0)
        .bind(created_at)
        .bind(initialized_at)
        .bind(updated_at)
        .execute(db.executor())
        .await?;
    Ok(Ref::new(res.last_insert_rowid()))
}

async fn insert_alert_raw(
    db: &mut Database,
    id: i64,
    a0: Option<i64>,
    a1: Option<i64>,
    a2: i64,
    a3: i64,
    a4: i64,
    a5: &str,
    a6: i64,
    a7: bool,
    a8: i64,
    a9: i64,
) -> Result<()> {
    query("INSERT INTO alerts VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(id)
        .bind(a0)
        .bind(a1)
        .bind(a2)
        .bind(a3)
        .bind(a4)
        .bind(a5)
        .bind(a6)
        .bind(a7)
        .bind(a8)
        .bind(a9)
        .execute(db.executor())
        .await?;
    Ok(())
}

async fn insert_reminder_raw(
    db: &mut Database,
    id: i64,
    a0: i64,
    a1: impl Into<Real>,
    a3: &str,
    a4: bool,
    a5: i64,
    a6: i64,
) -> Result<()> {
    query("INSERT INTO reminders VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(id)
        .bind(a0)
        .bind(a1.into())
        .bind(a3)
        .bind(a4)
        .bind(a5)
        .bind(a6)
        .execute(db.executor())
        .await?;
    Ok(())
}

async fn insert_tag_raw(
    db: &mut Database,
    a0: &str,
    a1: &str,
    a2: impl Into<Real>,
    a3: i64,
    a4: i64,
) -> Result<Ref<Tag>> {
    let res = query("INSERT INTO tags VALUES (NULL, ?, ?, ?, ?, ?)")
        .bind(a0)
        .bind(a1)
        .bind(a2.into())
        .bind(a3)
        .bind(a4)
        .execute(db.executor())
        .await?;
    Ok(Ref::new(res.last_insert_rowid()))
}

#[tokio::test]
async fn target_apps() -> Result<()> {
    let mut db = test_db().await?;
    {
        insert_tag_raw(&mut db, "tag_name1", "blue1", 0., 0, 0).await?;
        insert_tag_raw(&mut db, "tag_name2", "blue2", 0., 0, 0).await?;
        insert_tag_raw(&mut db, "tag_name3", "blue3", 0., 0, 0).await?;
        insert_tag_raw(&mut db, "tag_name4", "blue4", 0., 0, 0).await?;

        insert_app_raw(
            &mut db,
            true,
            "name1",
            "desc1",
            "comp1",
            "red1",
            None,
            1,
            "path1",
            None,
            0,
            Some(0),
            0,
        )
        .await?;
        insert_app_raw(
            &mut db,
            true,
            "name2",
            "desc2",
            "comp2",
            "red2",
            Some(2),
            0,
            "aumid2",
            None,
            0,
            Some(0),
            0,
        )
        .await?;
        insert_app_raw(
            &mut db,
            true,
            "name3",
            "desc3",
            "comp3",
            "red3",
            Some(3),
            1,
            "path3",
            None,
            0,
            Some(0),
            0,
        )
        .await?;
        insert_app_raw(
            &mut db,
            true,
            "name4",
            "desc4",
            "comp4",
            "red4",
            Some(2),
            1,
            "path4",
            None,
            0,
            Some(0),
            0,
        )
        .await?;
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
        tag_id: None,
        icon: None,
        created_at: 0,
        updated_at: 0,
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
        tag_id: Some(Ref::new(2)),
        icon: None,
        created_at: 0,
        updated_at: 0,
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
        tag_id: Some(Ref::new(3)),
        icon: None,
        created_at: 0,
        updated_at: 0,
    };
    let app4 = App {
        id: Ref::new(4),
        name: "name4".to_string(),
        description: "desc4".to_string(),
        company: "comp4".to_string(),
        color: "red4".to_string(),
        identity: AppIdentity::Win32 {
            path: "path4".to_string(),
        },
        tag_id: Some(Ref::new(2)),
        icon: None,
        created_at: 0,
        updated_at: 0,
    };

    assert_eq!(
        mgr.target_apps(&Target::App { id: (Ref::new(1)) }).await?,
        vec![app1.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::App { id: (Ref::new(2)) }).await?,
        vec![app2.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::App { id: (Ref::new(3)) }).await?,
        vec![app3.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::Tag { id: Ref::new(1) }).await?,
        vec![],
    );

    assert_eq!(
        mgr.target_apps(&Target::Tag { id: Ref::new(2) }).await?,
        vec![app2.id.clone(), app4.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::Tag { id: Ref::new(3) }).await?,
        vec![app3.id.clone()],
    );

    assert_eq!(
        mgr.target_apps(&Target::Tag { id: Ref::new(4) }).await?,
        vec![],
    );

    Ok(())
}

#[tokio::test]
async fn insert_alert_event() -> Result<()> {
    let mut db = test_db().await?;

    let id = 1;
    {
        insert_app_raw(
            &mut db,
            true,
            "name1",
            "desc1",
            "comp1",
            "red1",
            None,
            1,
            "path1",
            None,
            0,
            Some(0),
            0,
        )
        .await?;
        insert_alert_raw(&mut db, id, Some(1), None, 100, 1, 0, "", 1, true, 0, 0).await?;
    }

    let mut alert_event = AlertEvent {
        id: Default::default(),
        alert_id: Ref::new(1),
        timestamp: 1337,
        reason: Reason::Hit,
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
            &mut db,
            true,
            "name1",
            "desc1",
            "comp1",
            "red1",
            None,
            1,
            "path1",
            None,
            0,
            Some(0),
            0,
        )
        .await?;
        insert_alert_raw(
            &mut db,
            alert_id,
            Some(1),
            None,
            100,
            1,
            0,
            "",
            1,
            true,
            0,
            0,
        )
        .await?;
        insert_reminder_raw(&mut db, reminder_id, alert_id, 0.75, "hello", true, 0, 0).await?;
    }

    let mut reminder_event = ReminderEvent {
        id: Default::default(),
        reminder_id: Ref::new(reminder_id),
        timestamp: 1337,
        reason: Reason::Hit,
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

#[derive(Debug, Clone, Default)]
pub struct Times {
    pub now: i64,
    pub day_start: i64,
    pub week_start: i64,
    pub month_start: i64,
}

pub struct Tick(i64);

impl ToTicks for Tick {
    fn to_ticks(&self) -> i64 {
        self.0
    }
}

impl TimeSystem for Times {
    type Ticks = Tick;

    fn now(&self) -> Self::Ticks {
        Tick(self.now)
    }

    fn day_start(&self, _: bool) -> Self::Ticks {
        Tick(self.day_start)
    }

    fn week_start(&self, _: bool) -> Self::Ticks {
        Tick(self.week_start)
    }

    fn month_start(&self, _: bool) -> Self::Ticks {
        Tick(self.month_start)
    }
}

pub mod arrange {
    use super::*;
    use crate::entities::{TimeFrame, TriggerAction};

    pub async fn app(db: &mut Database, mut app: App) -> Result<App> {
        app.id = insert_app_raw(
            db,
            false,
            &app.name,
            &app.description,
            &app.company,
            &app.color,
            app.tag_id.as_ref().map(|id| id.0).clone(),
            if let AppIdentity::Win32 { .. } = app.identity {
                1
            } else {
                0
            },
            match &app.identity {
                AppIdentity::Win32 { path } => path,
                AppIdentity::Uwp { aumid } => aumid,
                AppIdentity::Website { base_url } => base_url,
            },
            app.icon.clone(),
            0,
            Some(0),
            0,
        )
        .await?;
        Ok(app)
    }

    pub async fn app_uninit(db: &mut Database, mut app: App) -> Result<App> {
        app.id = insert_app_raw(
            db,
            false,
            &app.name,
            &app.description,
            &app.company,
            &app.color,
            app.tag_id.as_ref().map(|id| id.0).clone(),
            if let AppIdentity::Win32 { .. } = app.identity {
                1
            } else {
                0
            },
            match &app.identity {
                AppIdentity::Win32 { path } => path,
                AppIdentity::Uwp { aumid } => aumid,
                AppIdentity::Website { base_url } => base_url,
            },
            app.icon.clone(),
            0,
            None,
            0,
        )
        .await?;
        Ok(app)
    }

    pub async fn session(db: &mut Database, mut session: Session) -> Result<Session> {
        let res = query("INSERT INTO sessions VALUES (NULL, ?, ?, ?)")
            .bind(session.app_id.clone())
            .bind(session.title.clone())
            .bind(session.url.clone())
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
        tag.id = insert_tag_raw(
            db,
            &tag.name,
            &tag.color,
            tag.score,
            tag.created_at,
            tag.updated_at,
        )
        .await?;
        Ok(tag)
    }

    pub async fn alert(db: &mut Database, alert: Alert) -> Result<Alert> {
        let time_frame = match alert.time_frame {
            TimeFrame::Daily => 0,
            TimeFrame::Weekly => 1,
            TimeFrame::Monthly => 2,
        };

        let (dim_duration, message_content, tag) = match &alert.trigger_action {
            TriggerAction::Kill => (None, None, 0),
            TriggerAction::Dim { duration } => (Some(*duration), None, 1),
            TriggerAction::Message { content } => (None, Some(content.to_string()), 2),
        };

        let (app_id, tag_id) = match alert.target.clone() {
            Target::App { id } => (Some(id.0), None),
            Target::Tag { id } => (None, Some(id.0)),
        };

        insert_alert_raw(
            db,
            alert.id.0,
            app_id,
            tag_id,
            alert.usage_limit,
            time_frame,
            dim_duration.unwrap_or(0),
            &message_content.unwrap_or("".to_string()),
            tag,
            alert.active,
            alert.created_at,
            alert.updated_at,
        )
        .await?;
        Ok(alert)
    }

    pub async fn reminder(db: &mut Database, reminder: Reminder) -> Result<Reminder> {
        insert_reminder_raw(
            db,
            reminder.id.0,
            reminder.alert_id.0,
            reminder.threshold,
            &reminder.message,
            reminder.active,
            reminder.created_at,
            reminder.updated_at,
        )
        .await?;
        Ok(reminder)
    }

    pub async fn alert_event(db: &mut Database, mut event: AlertEvent) -> Result<AlertEvent> {
        let res = query("INSERT INTO alert_events VALUES (NULL, ?, ?, ?)")
            .bind(event.alert_id.0)
            .bind(event.timestamp)
            .bind(&event.reason)
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
            .bind(event.reminder_id.0)
            .bind(event.timestamp)
            .bind(&event.reason)
            .execute(db.executor())
            .await?;
        event.id = Ref::new(res.last_insert_rowid());
        Ok(event)
    }
}

mod triggered_alerts {
    use super::*;
    use crate::entities::{TimeFrame, TriggerAction};

    #[tokio::test]
    async fn no_alerts() -> Result<()> {
        let db = test_db().await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 0,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let _alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 0,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let _session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
            },
        )
        .await?;

        let _alert = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 0,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 0,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let _alert2 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(2),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 51,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 100,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 10,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: false,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let alert2 = arrange::alert(
            &mut db,
            Alert {
                // alert1 upgraded
                id: Ref::new(2),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 100,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let _alert_event = arrange::alert_event(
            &mut db,
            AlertEvent {
                id: Ref::default(),
                alert_id: alert.id.clone().into(),
                timestamp: 75,
                reason: Reason::Hit,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 100,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let _alert_event = arrange::alert_event(
            &mut db,
            AlertEvent {
                id: Ref::default(),
                alert_id: alert.id.clone(),
                timestamp: 25,
                reason: Reason::Hit,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 100,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 50,
                time_frame: TimeFrame::Weekly,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 5000,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 50,
                time_frame: TimeFrame::Monthly,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 5000,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 5000,
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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 50,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 5000,
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

        let tag1 = arrange::tag(
            &mut db,
            Tag {
                id: Ref::default(),
                name: "tag_name1".to_string(),
                color: "blue1".to_string(),
                score: 5.0.into(),
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;
        let tag2 = arrange::tag(
            &mut db,
            Tag {
                id: Ref::default(),
                name: "tag_name2".to_string(),
                color: "blue2".to_string(),
                score: 0.0.into(),
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;
        let empty_tag = arrange::tag(
            &mut db,
            Tag {
                id: Ref::default(),
                name: "emptytag".to_string(),
                color: "e1".to_string(),
                score: (-5.0).into(),
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
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
                tag_id: Some(Ref::new(1)),
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let app3 = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name3".to_string(),
                description: "desc3".to_string(),
                company: "comp3".to_string(),
                color: "red3".to_string(),
                identity: AppIdentity::Uwp {
                    aumid: "aumid3".to_string(),
                },
                tag_id: Some(Ref::new(2)),
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let app4 = arrange::app(
            &mut db,
            App {
                id: Ref::default(),
                name: "name4".to_string(),
                description: "desc4".to_string(),
                company: "comp4".to_string(),
                color: "red4".to_string(),
                identity: AppIdentity::Uwp {
                    aumid: "aumid4".to_string(),
                },
                tag_id: Some(Ref::new(1)),
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        // emptytag = []
        // tag1 = [app2, app4]
        // tag2 = [app3]
        // all apps are 'used' from 0-100

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app_id: app1.id.clone(),
                title: "title".to_string(),
                url: None,
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
                url: None,
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
                app_id: app3.id.clone(),
                title: "title3".to_string(),
                url: None,
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
                app_id: app4.id.clone(),
                title: "title4".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::Tag {
                    id: empty_tag.id.clone(),
                },
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let _alert1 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(2),
                target: Target::Tag {
                    id: tag2.id.clone(),
                },
                usage_limit: 120,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let alert2 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(3),
                target: Target::Tag {
                    id: tag1.id.clone(),
                },
                usage_limit: 200,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let alert3 = arrange::alert(
            &mut db,
            Alert {
                id: Ref::new(4),
                target: Target::Tag {
                    id: tag2.id.clone(),
                },
                usage_limit: 100,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let alerts = mgr
            .triggered_alerts(&Times {
                now: 5000,
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
                    name: "tag_name1".to_string()
                },
                TriggeredAlert {
                    alert: alert3,
                    timestamp: None,
                    name: "tag_name2".to_string()
                }
            ]
        );
        Ok(())
    }
}

mod triggered_reminders {
    use super::*;
    use crate::entities::{TimeFrame, TriggerAction};

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
                tag_id: None,
                icon: None,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let session = arrange::session(
            db,
            Session {
                id: Ref::default(),
                app_id: app.id.clone(),
                title: "title".to_string(),
                url: None,
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
                id: Ref::new(1),
                target: Target::App {
                    id: (app.id.clone()),
                },
                usage_limit: 200,
                time_frame: TimeFrame::Daily,
                trigger_action: TriggerAction::Kill,
                active: true,
                created_at: 0,
                updated_at: 0,
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
                now: 5000,
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
                id: Ref::new(1),
                alert_id: alert.clone(),
                threshold: 0.50.into(),
                message: "hello".to_string(),
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let _not_hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(2),
                alert_id: alert.clone(),
                threshold: 0.51.into(),
                message: "hello".to_string(),
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                now: 5000,
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
                id: Ref::new(1),
                alert_id: alert.clone(),
                threshold: 0.50.into(),
                message: "hello".to_string(),
                active: false,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;
        let hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(2),
                alert_id: alert.clone(),
                threshold: 0.50.into(),
                message: "hello".to_string(),
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                now: 5000,
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
                id: Ref::new(1),
                alert_id: alert.clone().into(),
                threshold: 0.50.into(),
                message: "hello".to_string(),
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        arrange::reminder_event(
            &mut db,
            ReminderEvent {
                id: Default::default(),
                reminder_id: hit.id.clone(),
                timestamp: 25,
                reason: Reason::Hit,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                now: 5000,
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
                id: Ref::new(1),
                alert_id: alert.clone(),
                threshold: 0.50.into(),
                message: "hello".to_string(),
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        arrange::reminder_event(
            &mut db,
            ReminderEvent {
                id: Default::default(),
                reminder_id: hit.id.clone(),
                timestamp: 200,
                reason: Reason::Hit,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                now: 5000,
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
    pub async fn alert_event_after_start_matters_for_reminder_query() -> Result<()> {
        let mut db = test_db().await?;

        let alert = gen_alert(&mut db).await?;
        let _hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(1),
                alert_id: alert.clone(),
                threshold: 0.50.into(),
                message: "hello".to_string(),
                active: true,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await?;

        arrange::alert_event(
            &mut db,
            AlertEvent {
                id: Default::default(),
                alert_id: alert.clone(),
                timestamp: 200,
                reason: Reason::Ignored,
            },
        )
        .await?;

        let mut mgr = AlertManager::new(db)?;
        let reminders = mgr
            .triggered_reminders(&Times {
                now: 5000,
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
