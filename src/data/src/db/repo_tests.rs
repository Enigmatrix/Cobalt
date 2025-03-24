use std::collections::HashMap;
use std::sync::LazyLock;

use chrono::Local;
use repo::*;
use util::future as tokio;

use super::tests::*;
use super::*;
use crate::db::infused::{
    CreateAlert, CreateReminder, RefVec, ValuePerPeriod, WithDuration, WithGroupedDuration,
};
use crate::db::tests::arrange::*;
use crate::entities::{Reason, TimeFrame, TriggerAction};
use crate::table::Period;

const ONE_HOUR: i64 = 60 * 60 * 1000 * 10000;
const TEST_DATE: i64 = (1735776000 * 1000 + 62_135_596_800_000) * 10000;
static LOCAL_TEST_DATE: LazyLock<i64> = LazyLock::new(|| add_tz_shift(TEST_DATE));

fn add_tz_shift(ts: i64) -> i64 {
    // Get the current timezone offset in seconds
    let local_offset = Local::now().offset().local_minus_utc();
    // Convert seconds to 100-nanosecond intervals (same as the existing timestamp format)
    ts - (local_offset as i64) * 10_000_000
}

fn create_to_alert(c: CreateAlert, id: i64) -> Alert {
    Alert {
        id: Ref::new(id),
        target: c.target,
        usage_limit: c.usage_limit,
        time_frame: c.time_frame,
        trigger_action: c.trigger_action,
        active: true,
        created_at: 0,
        updated_at: 0,
    }
}

fn infused_to_alert(c: infused::Alert) -> Alert {
    Alert {
        id: c.id,
        target: c.target,
        usage_limit: c.usage_limit,
        time_frame: c.time_frame,
        trigger_action: c.trigger_action,
        active: true,
        created_at: 0,
        updated_at: 0,
    }
}

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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(1),
            name: "tag1".to_string(),
            color: "red".to_string(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(2),
            name: "tag2".to_string(),
            color: "red".to_string(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(3),
            name: "tag3".to_string(),
            color: "red".to_string(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(4),
            name: "tag4".to_string(),
            color: "red".to_string(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let mut repo = Repository::new(db)?;
    // TODO test: durations for these
    let _ = repo
        .get_apps(Times {
            now: 5000,
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
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(2),
            name: "tag2".to_string(),
            color: "red".to_string(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(3),
            name: "tag3".to_string(),
            color: "red".to_string(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    arrange::tag(
        &mut db,
        Tag {
            id: Ref::new(4),
            name: "tag4".to_string(),
            color: "red".to_string(),
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let mut repo = Repository::new(db)?;
    // TODO test: durations for these
    let tags = repo
        .get_tags(Times {
            now: 5000,
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
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?
    .id;

    let alert11 = Alert {
        id: Ref::new(1),
        usage_limit: 100,
        target: Target::App { id: app1.clone() },
        time_frame: TimeFrame::Daily,
        trigger_action: TriggerAction::Kill,
        active: false,
        created_at: 0,
        updated_at: 0,
    };
    let alert12 = Alert {
        id: Ref::new(2),
        usage_limit: 1000,
        target: Target::App { id: app1.clone() },
        time_frame: TimeFrame::Daily,
        trigger_action: TriggerAction::Kill,
        active: true,
        created_at: 0,
        updated_at: 0,
    };
    let alert21 = Alert {
        id: Ref::new(3),
        usage_limit: 100,
        target: Target::App { id: app1.clone() },
        time_frame: TimeFrame::Weekly,
        trigger_action: TriggerAction::Dim { duration: 1 },
        created_at: 0,
        updated_at: 0,
        active: true,
    };
    let alert31 = Alert {
        id: Ref::new(4),
        usage_limit: 100,
        target: Target::App { id: app1.clone() },
        time_frame: TimeFrame::Monthly,
        trigger_action: TriggerAction::Message {
            content: "urmam".into(),
        },
        active: false,
        created_at: 0,
        updated_at: 0,
    };
    let alert32 = Alert {
        id: Ref::new(5),
        usage_limit: 100,
        target: Target::App { id: app1.clone() },
        time_frame: TimeFrame::Weekly,
        trigger_action: TriggerAction::Message {
            content: "urmam".into(),
        },
        active: false,
        created_at: 0,
        updated_at: 0,
    };
    let alert33 = Alert {
        id: Ref::new(6),
        usage_limit: 10,
        target: Target::App { id: app1.clone() },
        time_frame: TimeFrame::Monthly,
        trigger_action: TriggerAction::Message {
            content: "urmam".into(),
        },
        active: true,
        created_at: 0,
        updated_at: 0,
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
            now: 5000,
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })
        .await?;

    let from_db: HashMap<_, _> = alerts
        .values()
        .map(|alert| (alert.id.clone(), infused_to_alert(alert.clone())))
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
                alert_id: alert_id.clone(),
                timestamp,
                reason: Reason::Hit,
            },
        )
        .await?;
    }

    let alerts = repo
        .get_alerts(Times {
            now: 5000,
            day_start: 20,
            week_start: 10,
            month_start: 5,
        })
        .await?;

    let from_db: HashMap<_, _> = alerts
        .values()
        .map(|alert| (alert.id.clone(), alert.events.clone()))
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
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
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?
    .id;

    usages(
        &mut db,
        app1.clone(),
        vec![(
            *LOCAL_TEST_DATE + ONE_HOUR + 10,
            *LOCAL_TEST_DATE + 2 * ONE_HOUR + 110,
        )],
    )
    .await?;

    let mut repo = Repository::new(db)?;

    // test intersections + no usage found for app2

    // full range, no intersection
    let app_durations = repo
        .get_app_durations_per_period(
            *LOCAL_TEST_DATE,
            *LOCAL_TEST_DATE + 3 * ONE_HOUR,
            Period::Day,
        )
        .await?;
    assert_eq!(
        durmapperiod(vec![(
            app1.clone(),
            vec![(*LOCAL_TEST_DATE, ONE_HOUR + 100)]
        )]),
        app_durations
    );

    // intersect before
    let app_durations = repo
        .get_app_durations_per_period(
            *LOCAL_TEST_DATE + ONE_HOUR,
            *LOCAL_TEST_DATE + 2 * ONE_HOUR,
            Period::Day,
        )
        .await?;
    assert_eq!(
        durmapperiod(vec![(
            app1.clone(),
            vec![(*LOCAL_TEST_DATE, ONE_HOUR - 10)]
        )]),
        app_durations
    );

    // intersect after
    let app_durations = repo
        .get_app_durations_per_period(
            *LOCAL_TEST_DATE + 2 * ONE_HOUR,
            *LOCAL_TEST_DATE + 3 * ONE_HOUR,
            Period::Day,
        )
        .await?;
    assert_eq!(
        durmapperiod(vec![(app1.clone(), vec![(*LOCAL_TEST_DATE, 110)])]),
        app_durations
    );

    // exact
    let app_durations = repo
        .get_app_durations_per_period(
            *LOCAL_TEST_DATE + ONE_HOUR + 10,
            *LOCAL_TEST_DATE + 2 * ONE_HOUR + 110,
            Period::Day,
        )
        .await?;
    assert_eq!(
        durmapperiod(vec![(
            app1.clone(),
            vec![(*LOCAL_TEST_DATE, ONE_HOUR + 100)]
        )]),
        app_durations
    );

    // full range, no intersection, hour split
    let app_durations = repo
        .get_app_durations_per_period(
            *LOCAL_TEST_DATE,
            *LOCAL_TEST_DATE + 5 * ONE_HOUR,
            Period::Hour,
        )
        .await?;
    assert_eq!(
        durmapperiod(vec![(
            app1.clone(),
            vec![
                (*LOCAL_TEST_DATE + ONE_HOUR, ONE_HOUR - 10),
                (*LOCAL_TEST_DATE + 2 * ONE_HOUR, 110)
            ]
        )]),
        app_durations
    );

    usages(
        &mut repo.db,
        app2.clone(),
        vec![(
            *LOCAL_TEST_DATE + ONE_HOUR + 20,
            *LOCAL_TEST_DATE + 5 * ONE_HOUR + 120,
        )],
    )
    .await?;

    let app_durations = repo
        .get_app_durations_per_period(
            *LOCAL_TEST_DATE,
            *LOCAL_TEST_DATE + 5 * ONE_HOUR,
            Period::Hour,
        )
        .await?;
    assert_eq!(
        durmapperiod(vec![
            (
                app1.clone(),
                vec![
                    (*LOCAL_TEST_DATE + ONE_HOUR, ONE_HOUR - 10),
                    (*LOCAL_TEST_DATE + 2 * ONE_HOUR, 110)
                ]
            ),
            (
                app2.clone(),
                vec![
                    (*LOCAL_TEST_DATE + ONE_HOUR, ONE_HOUR - 20),
                    (*LOCAL_TEST_DATE + 2 * ONE_HOUR, ONE_HOUR),
                    (*LOCAL_TEST_DATE + 3 * ONE_HOUR, ONE_HOUR),
                    (*LOCAL_TEST_DATE + 4 * ONE_HOUR, ONE_HOUR),
                ]
            )
        ]),
        app_durations
    );

    usages(
        &mut repo.db,
        app1.clone(),
        vec![(
            *LOCAL_TEST_DATE + 5 * ONE_HOUR + 200,
            *LOCAL_TEST_DATE + 6 * ONE_HOUR + 200,
        )],
    )
    .await?;

    let app_durations = repo
        .get_app_durations_per_period(
            *LOCAL_TEST_DATE,
            *LOCAL_TEST_DATE + 7 * ONE_HOUR,
            Period::Hour,
        )
        .await?;
    assert_eq!(
        durmapperiod(vec![
            (
                app1.clone(),
                vec![
                    (*LOCAL_TEST_DATE + ONE_HOUR, ONE_HOUR - 10),
                    (*LOCAL_TEST_DATE + 2 * ONE_HOUR, 110),
                    (*LOCAL_TEST_DATE + 5 * ONE_HOUR, ONE_HOUR - 200),
                    (*LOCAL_TEST_DATE + 6 * ONE_HOUR, 200),
                ]
            ),
            (
                app2.clone(),
                vec![
                    (*LOCAL_TEST_DATE + ONE_HOUR, ONE_HOUR - 20),
                    (*LOCAL_TEST_DATE + 2 * ONE_HOUR, ONE_HOUR),
                    (*LOCAL_TEST_DATE + 3 * ONE_HOUR, ONE_HOUR),
                    (*LOCAL_TEST_DATE + 4 * ONE_HOUR, ONE_HOUR),
                    (*LOCAL_TEST_DATE + 5 * ONE_HOUR, 120),
                ]
            )
        ]),
        app_durations
    );

    Ok(())
}

#[tokio::test]
async fn create_alert() -> Result<()> {
    let mut db = test_db().await?;

    fn to_reminders(
        c: Vec<CreateReminder>,
        start_id: i64,
        alert_id: Ref<Alert>,
    ) -> Vec<infused::Reminder> {
        c.into_iter()
            .enumerate()
            .map(|(i, r)| infused::Reminder {
                id: Ref::new(start_id + i as i64),
                alert_id: alert_id.clone(),
                threshold: r.threshold,
                message: r.message,
                events: Default::default(),
                created_at: 0,
                updated_at: 0,
            })
            .collect()
    }
    fn reminders_eq(a: infused::Reminder, b: infused::Reminder) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.threshold, b.threshold);
        assert_eq!(a.message, b.message);
        assert_eq!(a.events, b.events);
    }

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
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?
    .id;

    let mut repo = Repository::new(db)?;

    // Test creating alert with no reminders
    let alert1 = CreateAlert {
        usage_limit: 100,
        target: Target::App { id: app1.clone() },
        time_frame: TimeFrame::Daily,
        trigger_action: TriggerAction::Kill,
        reminders: Vec::new(),
        ignore_trigger: false,
    };
    let ts = Times::default();
    let created1 = repo.create_alert(alert1.clone(), ts.clone()).await?;
    assert_eq!(
        infused_to_alert(created1.clone()),
        create_to_alert(alert1, 1)
    );
    assert!(created1.reminders.is_empty());

    // Test creating alert with one reminder
    let alert2 = CreateAlert {
        usage_limit: 200,
        target: Target::App { id: app1.clone() },
        time_frame: TimeFrame::Weekly,
        trigger_action: TriggerAction::Dim { duration: 500 },
        reminders: vec![CreateReminder {
            threshold: 0.5,
            message: "Half way there".into(),
            ignore_trigger: false,
        }],
        ignore_trigger: false,
    };
    let created2 = repo.create_alert(alert2.clone(), ts.clone()).await?;
    let created2id = Ref::new(2);
    assert_eq!(
        infused_to_alert(created2.clone()),
        create_to_alert(alert2.clone(), created2id.0)
    );
    created2
        .reminders
        .iter()
        .cloned()
        .zip(to_reminders(alert2.reminders, 1, created2id))
        .for_each(|(a, b)| reminders_eq(a, b));
    assert_eq!(created2.reminders.len(), 1);

    // Test creating alert with multiple reminders
    let alert3 = CreateAlert {
        usage_limit: 300,
        target: Target::App { id: app1.clone() },
        time_frame: TimeFrame::Monthly,
        trigger_action: TriggerAction::Message {
            content: "Time's up".into(),
        },
        reminders: vec![
            CreateReminder {
                threshold: 0.25,
                message: "Quarter way".into(),
                ignore_trigger: false,
            },
            CreateReminder {
                threshold: 0.5,
                message: "Half way there".into(),
                ignore_trigger: false,
            },
            CreateReminder {
                threshold: 0.75,
                message: "Almost there".into(),
                ignore_trigger: false,
            },
        ],
        ignore_trigger: false,
    };

    let created3 = repo.create_alert(alert3.clone(), ts.clone()).await?;
    let created3id = Ref::new(3);
    assert_eq!(
        infused_to_alert(created3.clone()),
        create_to_alert(alert3.clone(), created3id.0)
    );
    created3
        .reminders
        .iter()
        .cloned()
        .zip(to_reminders(alert3.reminders, 2, created3id))
        .for_each(|(a, b)| reminders_eq(a, b));
    assert_eq!(created3.reminders.len(), 3);

    Ok(())
}
