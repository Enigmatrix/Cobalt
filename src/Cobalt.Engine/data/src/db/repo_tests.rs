use std::collections::HashMap;

use repo::*;
use util::future as tokio;

use super::tests::*;
use super::*;

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
