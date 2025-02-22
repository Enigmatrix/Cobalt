use std::collections::HashMap;

use repo::*;
use util::future as tokio;

use super::tests::*;
use super::*;
use crate::db::repo::infused::ValuePerPeriod;
use crate::db::tests::arrange::*;
use crate::entities::{TimeFrame, TriggerAction};
use crate::table::VersionedId;

#[tokio::test]
async fn get_apps() -> Result<()> {
    let mut db = test_db().await?;

    arrange::app(
        &mut db,
        App {
            id: Ref::new(1),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path1".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?;

    arrange::app(
        &mut db,
        App {
            id: Ref::new(2),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path2".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?;

    arrange::app(
        &mut db,
        App {
            id: Ref::new(3),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path3".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?;

    arrange::app(
        &mut db,
        App {
            id: Ref::new(4),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path4".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?;

    arrange::app_uninit(
        &mut db,
        App {
            id: Ref::new(5),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path5".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?;

    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(1),
            name: "tag1".to_string(),
            color: "red".to_string(),
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(2),
            name: "tag2".to_string(),
            color: "red".to_string(),
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(3),
            name: "tag3".to_string(),
            color: "red".to_string(),
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(4),
            name: "tag4".to_string(),
            color: "red".to_string(),
        },
    )
    .await?;

    let mut repo = Repository::new(db)?;
    // TODO test: durations for these
    let _ = repo
        .get_apps(Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })
        .await?;
    Ok(())
}

#[tokio::test]
async fn get_tags() -> Result<()> {
    let mut db = test_db().await?;

    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(1),
            name: "tag1".to_string(),
            color: "red".to_string(),
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(2),
            name: "tag2".to_string(),
            color: "red".to_string(),
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(3),
            name: "tag3".to_string(),
            color: "red".to_string(),
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(4),
            name: "tag4".to_string(),
            color: "red".to_string(),
        },
    )
    .await?;

    arrange::app(
        &mut db,
        App {
            id: Ref::new(1),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path1".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?;

    arrange::app(
        &mut db,
        App {
            id: Ref::new(2),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path2".to_string(),
            },
            tag_id: Some(Ref::new(1)),
            icon: None,
        },
    )
    .await?;

    arrange::app(
        &mut db,
        App {
            id: Ref::new(3),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path3".to_string(),
            },
            tag_id: Some(Ref::new(2)),
            icon: None,
        },
    )
    .await?;

    arrange::app(
        &mut db,
        App {
            id: Ref::new(4),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path4".to_string(),
            },
            tag_id: Some(Ref::new(1)),
            icon: None,
        },
    )
    .await?;

    arrange::app(
        &mut db,
        App {
            id: Ref::new(5),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path5".to_string(),
            },
            tag_id: Some(Ref::new(3)),
            icon: None,
        },
    )
    .await?;

    arrange::app_uninit(
        &mut db,
        App {
            id: Ref::new(6),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path6".to_string(),
            },
            tag_id: Some(Ref::new(3)),
            icon: None,
        },
    )
    .await?;

    let mut repo = Repository::new(db)?;
    // TODO test: durations for these
    let tags = repo
        .get_tags(Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })
        .await?;
    let from_db: HashMap<_, _> = tags
        .values()
        .map(|tag| (tag.inner.id.clone(), tag.apps.clone()))
        .collect();
    assert_eq!(
        from_db,
        vec![(1, vec![2, 4]), (2, vec![3]), (3, vec![5]), (4, vec![]),]
            .into_iter()
            .map(|(k, v)| (Ref::new(k), RefVec(v.into_iter().map(Ref::new).collect())))
            .collect()
    );
    Ok(())
}

#[tokio::test]
async fn get_alerts() -> Result<()> {
    let mut db = test_db().await?;

    let app1 = arrange::app(
        &mut db,
        App {
            id: Ref::new(1),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path1".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?
    .id;

    let alert11 = Alert {
        id: Ref::new(VersionedId { id: 1, version: 1 }),
        usage_limit: 100,
        target: Target::App(app1.clone()),
        time_frame: TimeFrame::Daily,
        trigger_action: TriggerAction::Kill,
    };
    let alert12 = Alert {
        id: Ref::new(VersionedId { id: 1, version: 2 }),
        usage_limit: 1000,
        target: Target::App(app1.clone()),
        time_frame: TimeFrame::Daily,
        trigger_action: TriggerAction::Kill,
    };
    let alert21 = Alert {
        id: Ref::new(VersionedId { id: 2, version: 1 }),
        usage_limit: 100,
        target: Target::App(app1.clone()),
        time_frame: TimeFrame::Weekly,
        trigger_action: TriggerAction::Dim(1),
    };
    let alert31 = Alert {
        id: Ref::new(VersionedId { id: 3, version: 1 }),
        usage_limit: 100,
        target: Target::App(app1.clone()),
        time_frame: TimeFrame::Monthly,
        trigger_action: TriggerAction::Message("urmam".into()),
    };
    let alert32 = Alert {
        id: Ref::new(VersionedId { id: 3, version: 2 }),
        usage_limit: 100,
        target: Target::App(app1.clone()),
        time_frame: TimeFrame::Weekly,
        trigger_action: TriggerAction::Message("urmam".into()),
    };
    let alert33 = Alert {
        id: Ref::new(VersionedId { id: 3, version: 3 }),
        usage_limit: 10,
        target: Target::App(app1.clone()),
        time_frame: TimeFrame::Monthly,
        trigger_action: TriggerAction::Message("urmam".into()),
    };

    let _alert11 = arrange::alert(&mut db, alert11.clone()).await?;
    let alert12 = arrange::alert(&mut db, alert12.clone()).await?;
    let alert21 = arrange::alert(&mut db, alert21.clone()).await?;
    let _alert31 = arrange::alert(&mut db, alert31.clone()).await?;
    let _alert32 = arrange::alert(&mut db, alert32.clone()).await?;
    let alert33 = arrange::alert(&mut db, alert33.clone()).await?;
    let mut repo = Repository::new(db)?;

    // Test that newer versions are returned even with events on older versions
    let alerts = repo
        .get_alerts(Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })
        .await?;

    let from_db: HashMap<_, _> = alerts
        .values()
        .map(|alert| (alert.inner.id.clone(), alert.inner.clone()))
        .collect();
    assert_eq!(
        from_db,
        vec![alert12.clone(), alert21.clone(), alert33.clone()]
            .into_iter()
            .map(|alert| (alert.id.clone(), alert))
            .collect()
    );

    let events = vec![
        (alert11.id.clone(), 10),
        (alert11.id.clone(), 15),
        (alert12.id.clone(), 16),
        (alert12.id.clone(), 20),
        (alert31.id.clone(), 1),
        (alert32.id.clone(), 2),
        (alert33.id.clone(), 6),
    ];
    for (alert_id, timestamp) in events {
        arrange::alert_event(
            &mut repo.db,
            AlertEvent {
                id: Ref::new(0),
                alert: alert_id.0.into(),
                timestamp,
            },
        )
        .await?;
    }

    let alerts = repo
        .get_alerts(Times {
            day_start: 20,
            week_start: 10,
            month_start: 5,
        })
        .await?;

    let from_db: HashMap<_, _> = alerts
        .values()
        .map(|alert| (alert.inner.id.clone(), alert.events.clone()))
        .collect();
    assert_eq!(
        from_db,
        vec![
            (
                alert12.id.clone(),
                ValuePerPeriod {
                    today: 1,
                    week: 2,
                    month: 2,
                }
            ),
            (
                alert21.id.clone(),
                ValuePerPeriod {
                    today: 0,
                    week: 0,
                    month: 0,
                }
            ),
            (
                alert33.id.clone(),
                ValuePerPeriod {
                    today: 0,
                    week: 0,
                    month: 1,
                }
            )
        ]
        .into_iter()
        .collect()
    );

    // TODO test reminders as well.

    Ok(())
}

// Inserts start-end usages into the database as single sessions
async fn usages(db: &mut Database, app_id: Ref<App>, usages: Vec<(i64, i64)>) -> Result<()> {
    for (start, end) in usages {
        let sid = session(
            db,
            Session {
                id: Ref::new(0),
                app_id: app_id.clone(),
                title: "".to_string(),
            },
        )
        .await?
        .id;

        usage(
            db,
            Usage {
                id: Ref::new(0),
                session_id: sid,
                start,
                end,
            },
        )
        .await?;
    }
    Ok(())
}

fn durmap(durs: Vec<(Ref<App>, i64)>) -> HashMap<Ref<App>, WithDuration<App>> {
    durs.into_iter()
        .map(|(id, duration)| (id.clone(), WithDuration { id, duration }))
        .collect()
}

fn durmapperiod(
    durs: Vec<(Ref<App>, Vec<(i64, i64)>)>,
) -> HashMap<Ref<App>, Vec<WithGroupedDuration<App>>> {
    durs.into_iter()
        .map(|(id, durs)| {
            (
                id.clone(),
                durs.into_iter()
                    .map(|(group, duration)| WithGroupedDuration {
                        id: id.clone(),
                        group,
                        duration,
                    })
                    .collect(),
            )
        })
        .collect()
}

#[tokio::test]
async fn get_app_durations() -> Result<()> {
    let mut db = test_db().await?;

    let app1 = arrange::app(
        &mut db,
        App {
            id: Ref::new(1),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path1".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?
    .id;

    let app2 = arrange::app(
        &mut db,
        App {
            id: Ref::new(2),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path2".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?
    .id;

    usages(&mut db, app1.clone(), vec![(10, 110)]).await?;

    let mut repo = Repository::new(db)?;

    // test intersections + no usage found for app

    let app_durations = repo.get_app_durations(0, 120).await?;
    assert_eq!(
        durmap(vec![(app1.clone(), 100), (app2.clone(), 0)]),
        app_durations
    );

    let app_durations = repo.get_app_durations(20, 120).await?;
    assert_eq!(
        durmap(vec![(app1.clone(), 90), (app2.clone(), 0)]),
        app_durations
    );

    let app_durations = repo.get_app_durations(0, 90).await?;
    assert_eq!(
        durmap(vec![(app1.clone(), 80), (app2.clone(), 0)]),
        app_durations
    );

    // test multiple

    usages(
        &mut repo.db,
        app1.clone(),
        vec![(110, 130), (130, 180), (220, 300)],
    )
    .await?;
    usages(&mut repo.db, app2.clone(), vec![(180, 220)]).await?;

    let app_durations = repo.get_app_durations(0, 300).await?;
    assert_eq!(
        durmap(vec![(app1.clone(), 250), (app2.clone(), 40)]),
        app_durations
    );

    let app_durations = repo.get_app_durations(10, 301).await?;
    assert_eq!(
        durmap(vec![(app1.clone(), 250), (app2.clone(), 40)]),
        app_durations
    );

    let app_durations = repo.get_app_durations(100, 200).await?;
    assert_eq!(
        durmap(vec![(app1.clone(), 80), (app2.clone(), 20)]),
        app_durations
    );

    let app_durations = repo.get_app_durations(100, 150).await?;
    assert_eq!(
        durmap(vec![(app1.clone(), 50), (app2.clone(), 0)]),
        app_durations
    );

    Ok(())
}

#[tokio::test]
async fn get_app_durations_per_period_singular_ts_test() -> Result<()> {
    let mut db = test_db().await?;

    let app1 = arrange::app(
        &mut db,
        App {
            id: Ref::new(1),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path1".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?
    .id;

    let app2 = arrange::app(
        &mut db,
        App {
            id: Ref::new(2),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path2".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?
    .id;

    usages(&mut db, app1.clone(), vec![(10, 110)]).await?;

    let mut repo = Repository::new(db)?;

    // test intersections + no usage found for app2

    let app_durations = repo.get_app_durations_per_period(0, 200, 150).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(0, 100)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(0, 100, 100).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(0, 90)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(0, 50, 100).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(0, 40)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(100, 200, 100).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(100, 10)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(50, 250, 100).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(50, 60)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(-250, 50, 200).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(-50, 40)])]),
        app_durations
    );

    // add usage for app2 which is out of range. rerun the same tests.
    usages(&mut repo.db, app2.clone(), vec![(400, 500)]).await?;

    let app_durations = repo.get_app_durations_per_period(0, 200, 150).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(0, 100)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(0, 100, 100).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(0, 90)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(0, 50, 100).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(0, 40)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(100, 200, 100).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(100, 10)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(50, 250, 100).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(50, 60)])]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(-250, 50, 200).await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(-50, 40)])]),
        app_durations
    );

    // test multiple groupings per usage period

    let app_durations = repo.get_app_durations_per_period(10, 50, 10).await?;
    assert_eq!(
        durmapperiod(vec![(
            app1.clone(),
            vec![(10, 10), (20, 10), (30, 10), (40, 10)]
        )]),
        app_durations
    );

    // misaligned end

    let app_durations = repo.get_app_durations_per_period(10, 49, 10).await?;
    assert_eq!(
        durmapperiod(vec![(
            app1.clone(),
            vec![(10, 10), (20, 10), (30, 10), (40, 9)]
        )]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(10, 51, 10).await?;
    assert_eq!(
        durmapperiod(vec![(
            app1.clone(),
            vec![(10, 10), (20, 10), (30, 10), (40, 10), (50, 1)]
        )]),
        app_durations
    );

    // misaligned start

    let app_durations = repo.get_app_durations_per_period(9, 50, 10).await?;
    assert_eq!(
        durmapperiod(vec![(
            app1.clone(),
            vec![(9, 9), (19, 10), (29, 10), (39, 10), (49, 1)]
        )]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(11, 50, 10).await?;
    assert_eq!(
        durmapperiod(vec![(
            app1.clone(),
            vec![(11, 10), (21, 10), (31, 10), (41, 9)]
        )]),
        app_durations
    );

    Ok(())
}

#[tokio::test]
async fn get_app_durations_per_period_multiple_ts_test() -> Result<()> {
    let mut db = test_db().await?;

    let app1 = arrange::app(
        &mut db,
        App {
            id: Ref::new(1),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path1".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?
    .id;

    let app2 = arrange::app(
        &mut db,
        App {
            id: Ref::new(2),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "path2".to_string(),
            },
            tag_id: None,
            icon: None,
        },
    )
    .await?
    .id;

    usages(
        &mut db,
        app1.clone(),
        vec![(10, 110), (130, 200), (220, 240), (240, 1000)],
    )
    .await?;
    usages(&mut db, app2.clone(), vec![(110, 130), (200, 220)]).await?;

    let mut repo = Repository::new(db)?;

    let app_durations = repo.get_app_durations_per_period(0, 300, 50).await?;
    assert_eq!(
        durmapperiod(vec![
            (
                app1.clone(),
                vec![
                    (0, 40),
                    (50, 50),
                    (100, 30),
                    (150, 50),
                    (200, 30),
                    (250, 50)
                ]
            ),
            (app2.clone(), vec![(100, 20), (200, 20)])
        ]),
        app_durations
    );

    let app_durations = repo.get_app_durations_per_period(10, 500, 100).await?;
    assert_eq!(
        durmapperiod(vec![
            (
                app1.clone(),
                vec![(10, 100), (110, 70), (210, 90), (310, 100), (410, 90)]
            ),
            (app2.clone(), vec![(110, 30), (210, 10)])
        ]),
        app_durations
    );

    Ok(())
}
