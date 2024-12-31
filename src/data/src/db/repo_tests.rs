use std::collections::HashMap;

use repo::*;
use util::future as tokio;

use super::tests::*;
use super::*;
use crate::db::tests::arrange::*;

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

    // app 1 and 3 share some tags
    // app 2 has no tags
    // app 2 has a tag but not shared with any other app
    arrange::app_tags(&mut db, Ref::new(1), Ref::new(1)).await?;
    arrange::app_tags(&mut db, Ref::new(1), Ref::new(2)).await?;
    arrange::app_tags(&mut db, Ref::new(1), Ref::new(3)).await?;

    arrange::app_tags(&mut db, Ref::new(3), Ref::new(1)).await?;
    arrange::app_tags(&mut db, Ref::new(3), Ref::new(3)).await?;

    arrange::app_tags(&mut db, Ref::new(4), Ref::new(4)).await?;

    arrange::app_tags(&mut db, Ref::new(5), Ref::new(4)).await?;

    let mut repo = Repository::new(db)?;
    // TODO test: durations for these
    let apps = repo
        .get_apps(Times {
            day_start: 0,
            week_start: 0,
            month_start: 0,
        })
        .await?;
    let from_db: HashMap<_, _> = apps
        .values()
        .map(|app| (app.inner.id.clone(), app.tags.clone()))
        .collect();
    assert_eq!(
        from_db,
        vec![
            (1, vec![1, 2, 3]),
            (2, vec![]),
            (3, vec![1, 3]),
            (4, vec![4]),
        ]
        .into_iter()
        .map(|(k, v)| (Ref::new(k), RefVec(v.into_iter().map(Ref::new).collect())))
        .collect()
    );
    Ok(())
}

#[tokio::test]
async fn get_tags() -> Result<()> {
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

    // tag 1 and 3 share some apps
    // tag 2 has no apps
    // tag 2 has a app but not shared with any other tag
    arrange::app_tags(&mut db, Ref::new(1), Ref::new(1)).await?;
    arrange::app_tags(&mut db, Ref::new(2), Ref::new(1)).await?;
    arrange::app_tags(&mut db, Ref::new(3), Ref::new(1)).await?;
    arrange::app_tags(&mut db, Ref::new(6), Ref::new(1)).await?;

    arrange::app_tags(&mut db, Ref::new(1), Ref::new(3)).await?;
    arrange::app_tags(&mut db, Ref::new(3), Ref::new(3)).await?;

    arrange::app_tags(&mut db, Ref::new(4), Ref::new(4)).await?;

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
        vec![
            (1, vec![1, 2, 3, 6]),
            (2, vec![]),
            (3, vec![1, 3]),
            (4, vec![4]),
        ]
        .into_iter()
        .map(|(k, v)| (Ref::new(k), RefVec(v.into_iter().map(Ref::new).collect())))
        .collect()
    );
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
