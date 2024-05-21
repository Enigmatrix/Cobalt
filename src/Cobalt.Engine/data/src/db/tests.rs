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
    let (init, path): (bool, String) = db.conn.query_row("SELECT * FROM apps", params![], |f| {
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
    let ip_from_db = db
        .conn
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

#[test]
fn insert_alert_event() -> Result<()> {
    let mut db = test_db()?;

    let id = 1;
    {
        let mut insert_app = insert_stmt!(db.conn, App)?;
        insert_app.execute(params![
            true, true, "name1", "desc1", "comp1", "red1", 1, "path1", false
        ])?;
        let mut insert_alert = prepare_stmt!(
            db.conn,
            "INSERT INTO alerts VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )?;
        insert_alert.execute(params![id, 1, 1, Option::<Ref<Tag>>::None, 100, 1, 0, 0, 1])?;
    }

    let mut alert_event = AlertEvent {
        id: Default::default(),
        alert: Ref::new(VersionedId { id: id, version: 1 }),
        timestamp: 1337,
    };

    {
        let mut mgr = AlertManager::new(&mut db)?;

        mgr.insert_alert_event(&alert_event)?;
    }

    let alert_event_from_db = db
        .conn
        .query_row("SELECT * FROM alert_events", params![], |f| {
            Ok(AlertEvent {
                id: f.get(0)?,
                alert: Ref::new(VersionedId {
                    id: f.get(1)?,
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

    let alert_id = 1;
    let reminder_id = 1;
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
            alert_id,
            1,
            1,
            Option::<Ref<Tag>>::None,
            100,
            1,
            0,
            0,
            1
        ])?;
        insert_reminder.execute(params![reminder_id, 1, alert_id, 1, 0.75, "hello",])?;
    }

    let mut reminder_event = ReminderEvent {
        id: Default::default(),
        reminder: Ref::new(VersionedId {
            id: reminder_id,
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
                        id: f.get(1)?,
                        version: f.get(2)?,
                    }),
                    timestamp: f.get(3)?,
                })
            })?;
    reminder_event.id = Ref::new(1);

    assert_eq!(reminder_event, reminder_event_from_db);

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

    pub fn app(db: &mut Database, mut app: App) -> Result<App> {
        db.conn.execute(
            "INSERT INTO apps VALUES (NULL, ?, ?, ?, ?, ?, ?, ?, ?, NULL)",
            params![
                false,
                false,
                app.name,
                app.description,
                app.company,
                app.color,
                if let AppIdentity::Win32 { .. } = app.identity {
                    1
                } else {
                    0
                },
                match &app.identity {
                    AppIdentity::Win32 { path } => path,
                    AppIdentity::Uwp { aumid } => aumid,
                },
            ],
        )?;
        app.id = Ref::new(db.conn.last_insert_rowid() as u64);
        Ok(app)
    }

    pub fn session(db: &mut Database, mut session: Session) -> Result<Session> {
        db.conn.execute(
            "INSERT INTO sessions VALUES (NULL, ?, ?)",
            params![session.app, session.title],
        )?;
        session.id = Ref::new(db.conn.last_insert_rowid() as u64);
        Ok(session)
    }

    pub fn usage(db: &mut Database, mut usage: Usage) -> Result<Usage> {
        db.conn.execute(
            "INSERT INTO usages VALUES (NULL, ?, ?, ?)",
            params![usage.session, usage.start, usage.end],
        )?;
        usage.id = Ref::new(db.conn.last_insert_rowid() as u64);
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

    pub fn tag(db: &mut Database, mut tag: Tag) -> Result<Tag> {
        db.conn.execute(
            "INSERT INTO tags VALUES (NULL, ?, ?)",
            params![tag.name, tag.color],
        )?;
        tag.id = Ref::new(db.conn.last_insert_rowid() as u64);
        Ok(tag)
    }

    pub fn app_tags(db: &mut Database, app_id: Ref<App>, tag_id: Ref<Tag>) -> Result<()> {
        db.conn.execute(
            "INSERT INTO _app_tags VALUES (?, ?)",
            params![app_id, tag_id],
        )?;
        Ok(())
    }

    pub fn alert(db: &mut Database, alert: Alert) -> Result<Alert> {
        let time_frame = match alert.time_frame {
            TimeFrame::Daily => 0,
            TimeFrame::Weekly => 1,
            TimeFrame::Monthly => 2,
        };

        let (dim_duration, message_content, tag) = match &alert.trigger_action {
            TriggerAction::Kill => (None, None, 0),
            TriggerAction::Dim(dur) => (Some(dur), None, 1),
            TriggerAction::Message(content) => (None, Some(content), 2),
        };

        let (app_id, tag_id) = match alert.target.clone() {
            Target::App(app) => (Some(app), None),
            Target::Tag(tag) => (None, Some(tag)),
        };

        db.conn.execute(
            "INSERT INTO alerts VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                alert.id.id,
                alert.id.version,
                app_id,
                tag_id,
                alert.usage_limit,
                time_frame,
                dim_duration,
                message_content,
                tag
            ],
        )?;
        Ok(alert)
    }

    pub fn reminder(db: &mut Database, reminder: Reminder) -> Result<Reminder> {
        db.conn.execute(
            "INSERT INTO reminders VALUES (?, ?, ?, ?, ?, ?)",
            params![
                reminder.id.id,
                reminder.id.version,
                reminder.alert.id,
                reminder.alert.version,
                reminder.threshold,
                reminder.message,
            ],
        )?;
        Ok(reminder)
    }

    pub fn alert_event(db: &mut Database, mut event: AlertEvent) -> Result<AlertEvent> {
        db.conn.execute(
            "INSERT INTO alert_events VALUES (NULL, ?, ?, ?)",
            params![event.alert.id, event.alert.version, event.timestamp,],
        )?;
        event.id = Ref::new(db.conn.last_insert_rowid() as u64);
        Ok(event)
    }

    pub fn reminder_event(db: &mut Database, mut event: ReminderEvent) -> Result<ReminderEvent> {
        db.conn.execute(
            "INSERT INTO reminder_events VALUES (NULL, ?, ?, ?)",
            params![event.reminder.id, event.reminder.version, event.timestamp,],
        )?;
        event.id = Ref::new(db.conn.last_insert_rowid() as u64);
        Ok(event)
    }

    pub fn to_id(n: u64) -> u64 {
        n
    }
}

mod triggered_alerts {
    use super::*;

    #[test]
    fn no_alerts() -> Result<()> {
        let mut db = test_db()?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })?;
        assert_eq!(alerts, vec![]);
        Ok(())
    }

    #[test]
    fn one_alert_one_app_no_sessions() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })?;
        assert_eq!(alerts, vec![]);
        Ok(())
    }

    #[test]
    fn one_alert_one_app_one_session_no_usages() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let _session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })?;
        assert_eq!(alerts, vec![]);
        Ok(())
    }

    #[test]
    fn triggered_correctly() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 0,
                end: 100,
            },
        )?;

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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })?;
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

    #[test]
    fn halfway_usage_does_not_trigger() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 0,
                end: 100,
            },
        )?;

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
        )?;

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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 50,
            week_start: 50,
            month_start: 50,
        })?;
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

    #[test]
    fn exactly_one_alert_from_one_usage_despite_lower_version_existing() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 0,
                end: 100,
            },
        )?;

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
        )?;

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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })?;
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

    #[test]
    fn alert_event_after_start() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 0,
                end: 100,
            },
        )?;

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
        )?;

        let _alert_event = arrange::alert_event(
            &mut db,
            AlertEvent {
                id: Ref::default(),
                alert: alert.id.clone(),
                timestamp: 75,
            },
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 50,
            week_start: 0,
            month_start: 0,
        })?;
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

    #[test]
    fn alert_event_before_start() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 0,
                end: 100,
            },
        )?;

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
        )?;

        let _alert_event = arrange::alert_event(
            &mut db,
            AlertEvent {
                id: Ref::default(),
                alert: alert.id.clone(),
                timestamp: 25,
            },
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 50,
            week_start: 0,
            month_start: 0,
        })?;
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

    #[test]
    fn weekly_event_triggered() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 0,
                end: 100,
            },
        )?;

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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 1000,
            week_start: 50,
            month_start: 1000,
        })?;
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

    #[test]
    fn monthly_event_triggered() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        let _usage = arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 0,
                end: 100,
            },
        )?;

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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 1000,
            week_start: 1000,
            month_start: 50,
        })?;
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

    #[test]
    fn multiple_usages() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 0,
                end: 10,
            },
        )?;

        // 25 units
        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 10,
                end: 75,
            },
        )?;

        // 25 units
        for i in 0..25 {
            arrange::usage(
                &mut db,
                Usage {
                    id: Ref::default(),
                    session: session.id.clone(),
                    start: 200 + i,
                    end: 200 + i + 1,
                },
            )?;
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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 50,
            week_start: 0,
            month_start: 0,
        })?;
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

    #[test]
    fn multiple_usages_less_than_usage_limit() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 0,
                end: 10,
            },
        )?;

        // 10 units
        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 65,
                end: 75,
            },
        )?;

        // 25 units
        for i in 0..25 {
            arrange::usage(
                &mut db,
                Usage {
                    id: Ref::default(),
                    session: session.id.clone(),
                    start: 200 + i,
                    end: 200 + i + 1,
                },
            )?;
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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 50,
            week_start: 0,
            month_start: 0,
        })?;
        assert_eq!(alerts, vec![]);
        Ok(())
    }

    #[test]
    fn using_tags() -> Result<()> {
        let mut db = test_db()?;

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
            },
        )?;

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
            },
        )?;

        let empty_tag = arrange::tag(
            &mut db,
            Tag {
                id: Ref::default(),
                name: "emptytag".to_string(),
                color: "e1".to_string(),
            },
        )?;
        let tag1 = arrange::tag(
            &mut db,
            Tag {
                id: Ref::default(),
                name: "tag_name1".to_string(),
                color: "blue1".to_string(),
            },
        )?;
        arrange::app_tags(&mut db, app1.id.clone(), tag1.id.clone())?;
        let tag2 = arrange::tag(
            &mut db,
            Tag {
                id: Ref::default(),
                name: "tag_name2".to_string(),
                color: "blue2".to_string(),
            },
        )?;
        arrange::app_tags(&mut db, app1.id.clone(), tag2.id.clone())?;
        arrange::app_tags(&mut db, app2.id.clone(), tag2.id.clone())?;

        // emptytag = []
        // tag1 = [app1]
        // tag2 = [app1, app2]

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app1.id.clone(),
                title: "title".to_string(),
            },
        )?
        .id;

        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session,
                start: 0,
                end: 100,
            },
        )?;

        let session = arrange::session(
            &mut db,
            Session {
                id: Ref::default(),
                app: app2.id.clone(),
                title: "title2".to_string(),
            },
        )?
        .id;

        arrange::usage(
            &mut db,
            Usage {
                id: Ref::default(),
                session,
                start: 0,
                end: 100,
            },
        )?;

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
        )?;

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
        )?;

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
        )?;

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
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let alerts = mgr.triggered_alerts(&Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })?;
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

    // generate an alert which will not fire but the reminder will.
    fn gen_alert(db: &mut Database) -> Result<Ref<Alert>> {
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
            },
        )?;

        let session = arrange::session(
            db,
            Session {
                id: Ref::default(),
                app: app.id.clone(),
                title: "title".to_string(),
            },
        )?;

        let _usage = arrange::usage(
            db,
            Usage {
                id: Ref::default(),
                session: session.id.clone(),
                start: 50,
                end: 150,
            },
        )?;

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
        )?;
        Ok(alert.id)
    }

    #[test]
    pub fn no_reminders() -> Result<()> {
        let mut db = test_db()?;

        let _alert = gen_alert(&mut db)?;

        let mut mgr = AlertManager::new(&mut db)?;
        let reminders = mgr.triggered_reminders(&Times {
            day_start: 50,
            week_start: 0,
            month_start: 0,
        })?;
        assert_eq!(
            reminders
                .into_iter()
                .map(|r| (r.reminder.id, r.name))
                .collect::<Vec<_>>(),
            vec![]
        );
        Ok(())
    }

    #[test]
    pub fn triggered_correctly() -> Result<()> {
        let mut db = test_db()?;

        let alert = gen_alert(&mut db)?;
        let hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                alert: alert.clone(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )?;

        let _not_hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(2),
                    version: 1,
                }),
                alert: alert.clone(),
                threshold: 0.51,
                message: "hello".to_string(),
            },
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let reminders = mgr.triggered_reminders(&Times {
            day_start: 50,
            week_start: 0,
            month_start: 0,
        })?;
        assert_eq!(
            reminders
                .into_iter()
                .map(|r| (r.reminder.id, r.name))
                .collect::<Vec<_>>(),
            vec![(hit.id, "name".to_string())]
        );
        Ok(())
    }

    #[test]
    pub fn only_max_reminder() -> Result<()> {
        let mut db = test_db()?;

        let alert = gen_alert(&mut db)?;

        let _not_hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                alert: alert.clone(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )?;
        let hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 2,
                }),
                alert: alert.clone(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let reminders = mgr.triggered_reminders(&Times {
            day_start: 50,
            week_start: 0,
            month_start: 0,
        })?;
        assert_eq!(
            reminders
                .into_iter()
                .map(|r| (r.reminder.id, r.name))
                .collect::<Vec<_>>(),
            vec![(hit.id, "name".to_string())]
        );
        Ok(())
    }

    #[test]
    pub fn reminder_event_before_start_does_not_matter() -> Result<()> {
        let mut db = test_db()?;

        let alert = gen_alert(&mut db)?;
        let hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                alert: alert.clone(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )?;

        arrange::reminder_event(
            &mut db,
            ReminderEvent {
                id: Default::default(),
                reminder: hit.id.clone(),
                timestamp: 25,
            },
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let reminders = mgr.triggered_reminders(&Times {
            day_start: 50,
            week_start: 0,
            month_start: 0,
        })?;
        assert_eq!(
            reminders
                .into_iter()
                .map(|r| (r.reminder.id, r.name))
                .collect::<Vec<_>>(),
            vec![(hit.id, "name".to_string())]
        );
        Ok(())
    }

    #[test]
    pub fn reminder_event_after_start_matters() -> Result<()> {
        let mut db = test_db()?;

        let alert = gen_alert(&mut db)?;
        let hit = arrange::reminder(
            &mut db,
            Reminder {
                id: Ref::new(VersionedId {
                    id: arrange::to_id(1),
                    version: 1,
                }),
                alert: alert.clone(),
                threshold: 0.50,
                message: "hello".to_string(),
            },
        )?;

        arrange::reminder_event(
            &mut db,
            ReminderEvent {
                id: Default::default(),
                reminder: hit.id.clone(),
                timestamp: 200,
            },
        )?;

        let mut mgr = AlertManager::new(&mut db)?;
        let reminders = mgr.triggered_reminders(&Times {
            day_start: 50,
            week_start: 0,
            month_start: 0,
        })?;
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
