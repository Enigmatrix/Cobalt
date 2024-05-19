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
