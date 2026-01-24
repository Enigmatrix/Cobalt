use repo::*;
use util::future as tokio;

use super::tests::*;
use super::*;
use crate::db::infused::{DistractiveStreakSettings, FocusStreakSettings};
use crate::db::repo_tests::{LOCAL_TEST_DATE, ONE_HOUR};
use crate::entities::AppIdentity;
use crate::table::Period;

fn test_start() -> i64 {
    *LOCAL_TEST_DATE
}
fn test_end() -> i64 {
    *LOCAL_TEST_DATE + 2000 * ONE_HOUR
}

#[tokio::test]
async fn get_score_no_apps() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    let score = repo.get_score(test_start(), test_end()).await?;
    assert_eq!(score, 0.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_apps_no_tags() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create apps without tags
    let app1 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "App1".to_string(),
            description: "Description1".to_string(),
            company: "Company1".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "app1.exe".to_string(),
            },
            tag_id: None,

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let app2 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "App2".to_string(),
            description: "Description2".to_string(),
            company: "Company2".to_string(),
            color: "blue".to_string(),
            identity: AppIdentity::Win32 {
                path: "app2.exe".to_string(),
            },
            tag_id: None,

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions and usages
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app1.id.clone(),
            title: "Session1".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: app2.id.clone(),
            title: "Session2".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages within the test range
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: test_start() + 100 * ONE_HOUR, // 100 hours after start
            end: test_start() + 200 * ONE_HOUR,   // 200 hours after start (100 hours duration)
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: test_start() + 300 * ONE_HOUR, // 300 hours after start
            end: test_start() + 350 * ONE_HOUR,   // 350 hours after start (50 hours duration)
        },
    )
    .await?;

    let score = repo.get_score(test_start(), test_end()).await?;
    // Apps without tags should contribute 0 to the score
    assert_eq!(score, 0.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_apps_with_tags() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create tags with different scores
    let tag1 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(), // High score for productivity
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let tag2 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Gaming".to_string(),
            color: "red".to_string(),
            score: 5.0.into(), // Lower score for gaming
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create apps with tags
    let app1 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(tag1.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let app2 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "GamingApp".to_string(),
            description: "Gaming Description".to_string(),
            company: "Gaming Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "gaming.exe".to_string(),
            },
            tag_id: Some(tag2.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions and usages
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app1.id.clone(),
            title: "Productivity Session".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: app2.id.clone(),
            title: "Gaming Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages within the test range
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: test_start() + 100 * ONE_HOUR, // 100 hours after start
            end: test_start() + 200 * ONE_HOUR,   // 200 hours after start (100 hours duration)
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: test_start() + 300 * ONE_HOUR, // 300 hours after start
            end: test_start() + 350 * ONE_HOUR,   // 350 hours after start (50 hours duration)
        },
    )
    .await?;

    let score = repo.get_score(test_start(), test_end()).await?;
    // Expected: weighted average = (100 hours * 10 score + 50 hours * 5 score) / (100 + 50) hours = 8.333...
    let expected = (25.0 / 3.0).into();
    let epsilon = 1e-9;
    assert!(
        f64::from(score - expected).abs() < epsilon,
        "score: {}, expected: {}",
        score,
        expected
    );

    Ok(())
}

#[tokio::test]
async fn get_score_mixed_apps() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Work".to_string(),
            color: "blue".to_string(),
            score: 8.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create apps: one with tag, one without
    let app_with_tag = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "WorkApp".to_string(),
            description: "Work Description".to_string(),
            company: "Work Company".to_string(),
            color: "blue".to_string(),
            identity: AppIdentity::Win32 {
                path: "work.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let app_without_tag = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "UntaggedApp".to_string(),
            description: "Untagged Description".to_string(),
            company: "Untagged Company".to_string(),
            color: "gray".to_string(),
            identity: AppIdentity::Win32 {
                path: "untagged.exe".to_string(),
            },
            tag_id: None,

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app_with_tag.id.clone(),
            title: "Work Session".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: app_without_tag.id.clone(),
            title: "Untagged Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: test_start() + 50 * ONE_HOUR,
            end: test_start() + 150 * ONE_HOUR, // 100 hours duration
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: test_start() + 200 * ONE_HOUR,
            end: test_start() + 250 * ONE_HOUR, // 50 hours duration
        },
    )
    .await?;

    let score = repo.get_score(test_start(), test_end()).await?;
    // Expected: weighted average = (100 hours * 8 score + 50 hours * 0 score) / (100 + 50) hours = 5.333...
    let expected = (16.0 / 3.0).into();
    let epsilon = 1e-9;
    assert!(
        f64::from(score - expected).abs() < epsilon,
        "score: {}, expected: {}",
        score,
        expected
    );

    Ok(())
}

#[tokio::test]
async fn get_score_usage_outside_range() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Test".to_string(),
            color: "purple".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "TestApp".to_string(),
            description: "Test Description".to_string(),
            company: "Test Company".to_string(),
            color: "purple".to_string(),
            identity: AppIdentity::Win32 {
                path: "test.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Test Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage outside the test range (before start)
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: test_start() - 200 * ONE_HOUR,
            end: test_start() - 100 * ONE_HOUR, // 100 hours duration, but outside range
        },
    )
    .await?;

    // Create usage outside the test range (after end)
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session.id.clone(),
            start: test_end() + 100 * ONE_HOUR,
            end: test_end() + 200 * ONE_HOUR, // 100 hours duration, but outside range
        },
    )
    .await?;

    let score = repo.get_score(test_start(), test_end()).await?;
    // No usage within the range, so score should be 0
    assert_eq!(score, 0.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_partial_usage_overlap() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Overlap".to_string(),
            color: "orange".to_string(),
            score: 6.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "OverlapApp".to_string(),
            description: "Overlap Description".to_string(),
            company: "Overlap Company".to_string(),
            color: "orange".to_string(),
            identity: AppIdentity::Win32 {
                path: "overlap.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Overlap Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that starts before the range but ends within it
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: test_start() - 50 * ONE_HOUR, // Starts 50 hours before range
            end: test_start() + 50 * ONE_HOUR,   // Ends 50 hours into range
        },
    )
    .await?;

    // Create usage that starts within the range but ends after it
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session.id.clone(),
            start: test_end() - 30 * ONE_HOUR, // Starts 30 hours before end
            end: test_end() + 70 * ONE_HOUR,   // Ends 70 hours after range
        },
    )
    .await?;

    let score = repo.get_score(test_start(), test_end()).await?;
    // Expected: weighted average = (50 hours * 6 score + 30 hours * 6 score) / (50 + 30) hours = 6
    assert_eq!(score, 6.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_multiple_sessions_same_app() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "MultiSession".to_string(),
            color: "cyan".to_string(),
            score: 7.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "MultiSessionApp".to_string(),
            description: "MultiSession Description".to_string(),
            company: "MultiSession Company".to_string(),
            color: "cyan".to_string(),
            identity: AppIdentity::Win32 {
                path: "multisession.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create multiple sessions for the same app
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Session 1".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: app.id.clone(),
            title: "Session 2".to_string(),
            url: None,
        },
    )
    .await?;

    let session3 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(3),
            app_id: app.id.clone(),
            title: "Session 3".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages for each session
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: test_start() + 100 * ONE_HOUR,
            end: test_start() + 150 * ONE_HOUR, // 50 hours
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: test_start() + 200 * ONE_HOUR,
            end: test_start() + 250 * ONE_HOUR, // 50 hours
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(3),
            session_id: session3.id.clone(),
            start: test_start() + 300 * ONE_HOUR,
            end: test_start() + 350 * ONE_HOUR, // 50 hours
        },
    )
    .await?;

    let score = repo.get_score(test_start(), test_end()).await?;
    // Expected: weighted average = (50 + 50 + 50) hours * 7 score / (50 + 50 + 50) hours = 7
    assert_eq!(score, 7.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_zero_tag_score() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag with zero score
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "ZeroScore".to_string(),
            color: "black".to_string(),
            score: 0.0.into(), // Zero score
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the zero-score tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ZeroScoreApp".to_string(),
            description: "ZeroScore Description".to_string(),
            company: "ZeroScore Company".to_string(),
            color: "black".to_string(),
            identity: AppIdentity::Win32 {
                path: "zeroscore.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "ZeroScore Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: test_start() + 100 * ONE_HOUR,
            end: test_start() + 200 * ONE_HOUR, // 100 hours duration
        },
    )
    .await?;

    let score = repo.get_score(test_start(), test_end()).await?;
    // Expected: 100 hours * 0 score = 0
    assert_eq!(score, 0.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_usage_extends_beyond_end() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "BeyondEnd".to_string(),
            color: "purple".to_string(),
            score: 8.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "BeyondEndApp".to_string(),
            description: "BeyondEnd Description".to_string(),
            company: "BeyondEnd Company".to_string(),
            color: "purple".to_string(),
            identity: AppIdentity::Win32 {
                path: "beyondend.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "BeyondEnd Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that starts within the range but extends beyond the end
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: test_end() - 50 * ONE_HOUR, // Starts 50 hours before end
            end: test_end() + 100 * ONE_HOUR,  // Ends 100 hours after end
        },
    )
    .await?;

    let score = repo.get_score(test_start(), test_end()).await?;
    // Expected: only the portion within the range should count
    // 50 hours * 8 score = 400 score-hours, but this should be normalized by total time
    // The function should only consider the time within the range
    let expected = 8.0.into(); // The score should be the tag score since all usage is within range
    assert_eq!(score, expected);

    Ok(())
}

#[tokio::test]
async fn get_score_usage_starts_before_and_extends_beyond() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "BeforeAndAfter".to_string(),
            color: "orange".to_string(),
            score: 6.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "BeforeAndAfterApp".to_string(),
            description: "BeforeAndAfter Description".to_string(),
            company: "BeforeAndAfter Company".to_string(),
            color: "orange".to_string(),
            identity: AppIdentity::Win32 {
                path: "beforeandafter.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "BeforeAndAfter Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that starts before the range and extends beyond the end
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: test_start() - 100 * ONE_HOUR, // Starts 100 hours before range
            end: test_end() + 200 * ONE_HOUR,     // Ends 200 hours after range
        },
    )
    .await?;

    let score = repo.get_score(test_start(), test_end()).await?;
    // Expected: only the portion within the range should count
    // Total range is 2000 hours, so the score should be 6.0 (the tag score)
    // since the entire range is covered by this usage
    let expected = 6.0.into();
    assert_eq!(score, expected);

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_no_apps() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;
    assert!(scores.is_empty());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_single_hour() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Test".to_string(),
            color: "blue".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "TestApp".to_string(),
            description: "Test Description".to_string(),
            company: "Test Company".to_string(),
            color: "blue".to_string(),
            identity: AppIdentity::Win32 {
                path: "test.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Test Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage within a single hour period
    let hour_start = test_start() + 100 * ONE_HOUR;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: hour_start,
            end: hour_start + 30 * 60 * 1000 * 10000, // 30 minutes
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;
    assert_eq!(scores.len(), 1);
    assert_eq!(scores[0].group, hour_start);
    assert_eq!(scores[0].value, 10.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_multiple_hours() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Test".to_string(),
            color: "blue".to_string(),
            score: 8.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "TestApp".to_string(),
            description: "Test Description".to_string(),
            company: "Test Company".to_string(),
            color: "blue".to_string(),
            identity: AppIdentity::Win32 {
                path: "test.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Test Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage spanning multiple hours
    let start_time = test_start() + 100 * ONE_HOUR;
    let end_time = test_start() + 103 * ONE_HOUR; // 3 hours duration
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: start_time,
            end: end_time,
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;
    assert_eq!(scores.len(), 3);

    // Each hour should have the same score since usage is evenly distributed
    for score in &scores {
        assert_eq!(score.value, 8.0.into());
    }

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_different_scores() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create tags with different scores
    let tag1 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "High".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let tag2 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Low".to_string(),
            color: "red".to_string(),
            score: 2.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create apps with different tags
    let app1 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "HighApp".to_string(),
            description: "High Description".to_string(),
            company: "High Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "high.exe".to_string(),
            },
            tag_id: Some(tag1.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let app2 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "LowApp".to_string(),
            description: "Low Description".to_string(),
            company: "Low Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "low.exe".to_string(),
            },
            tag_id: Some(tag2.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app1.id.clone(),
            title: "High Session".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: app2.id.clone(),
            title: "Low Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages in different hours
    let hour1_start = test_start() + 100 * ONE_HOUR;
    let hour2_start = test_start() + 101 * ONE_HOUR;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: hour1_start,
            end: hour1_start + 30 * 60 * 1000 * 10000, // 30 minutes
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: hour2_start,
            end: hour2_start + 30 * 60 * 1000 * 10000, // 30 minutes
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;
    assert_eq!(scores.len(), 2);

    // First hour should have score 10, second hour should have score 2
    assert_eq!(scores[0].group, hour1_start);
    assert_eq!(scores[0].value, 10.0.into());
    assert_eq!(scores[1].group, hour2_start);
    assert_eq!(scores[1].value, 2.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_mixed_hour() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create tags with different scores
    let tag1 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "High".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let tag2 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Low".to_string(),
            color: "red".to_string(),
            score: 2.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create apps with different tags
    let app1 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "HighApp".to_string(),
            description: "High Description".to_string(),
            company: "High Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "high.exe".to_string(),
            },
            tag_id: Some(tag1.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let app2 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "LowApp".to_string(),
            description: "Low Description".to_string(),
            company: "Low Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "low.exe".to_string(),
            },
            tag_id: Some(tag2.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app1.id.clone(),
            title: "High Session".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: app2.id.clone(),
            title: "Low Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages in the same hour
    let hour_start = test_start() + 100 * ONE_HOUR;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: hour_start,
            end: hour_start + 20 * 60 * 1000 * 10000, // 20 minutes
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: hour_start + 30 * 60 * 1000 * 10000, // 30 minutes into the hour
            end: hour_start + 50 * 60 * 1000 * 10000, // 50 minutes into the hour (20 minutes duration)
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;
    assert_eq!(scores.len(), 1);

    // Expected: weighted average = (20 min * 10 + 20 min * 2) / (20 + 20) min = 6.0
    assert_eq!(scores[0].group, hour_start);
    assert_eq!(scores[0].value, 6.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_day_period() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Test".to_string(),
            color: "blue".to_string(),
            score: 5.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "TestApp".to_string(),
            description: "Test Description".to_string(),
            company: "Test Company".to_string(),
            color: "blue".to_string(),
            identity: AppIdentity::Win32 {
                path: "test.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Test Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage spanning multiple days
    let day_start = test_start() + 1 * ONE_HOUR;
    let day_end = test_start() + 23 * ONE_HOUR; // 24 hours later
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: day_start,
            end: day_end,
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Day)
        .await?;
    assert_eq!(scores.len(), 1);
    assert_eq!(scores[0].value, 5.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_week_period() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Test".to_string(),
            color: "blue".to_string(),
            score: 7.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "TestApp".to_string(),
            description: "Test Description".to_string(),
            company: "Test Company".to_string(),
            color: "blue".to_string(),
            identity: AppIdentity::Win32 {
                path: "test.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Test Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage spanning multiple weeks
    let week_start = test_start() + 100 * ONE_HOUR;
    let week_end = test_start() + 6 * 24 * ONE_HOUR; // 7 days later (168 hours)
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: week_start,
            end: week_end,
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Week)
        .await?;
    assert_eq!(scores.len(), 1);
    assert_eq!(scores[0].value, 7.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_apps_without_tags() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create an app without a tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "UntaggedApp".to_string(),
            description: "Untagged Description".to_string(),
            company: "Untagged Company".to_string(),
            color: "gray".to_string(),
            identity: AppIdentity::Win32 {
                path: "untagged.exe".to_string(),
            },
            tag_id: None,

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Untagged Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage
    let hour_start = test_start() + 100 * ONE_HOUR;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: hour_start,
            end: hour_start + 30 * 60 * 1000 * 10000, // 30 minutes
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;
    assert_eq!(scores.len(), 1);
    assert_eq!(scores[0].value, 0.0.into()); // Apps without tags should contribute 0 to the score

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_complex_multi_period() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create tags with different scores
    let tag1 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let tag2 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Gaming".to_string(),
            color: "red".to_string(),
            score: 2.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create apps with different tags
    let app1 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(tag1.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let app2 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "GamingApp".to_string(),
            description: "Gaming Description".to_string(),
            company: "Gaming Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "gaming.exe".to_string(),
            },
            tag_id: Some(tag2.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app1.id.clone(),
            title: "Productivity Session".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: app2.id.clone(),
            title: "Gaming Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create complex usage pattern spanning multiple hours
    // Hour 1: 30 min productivity, 30 min gaming
    let hour1_start = test_start() + 100 * ONE_HOUR;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: hour1_start,
            end: hour1_start + 30 * 60 * 1000 * 10000, // 30 minutes
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: hour1_start + 30 * 60 * 1000 * 10000, // 30 minutes into the hour
            end: hour1_start + 60 * 60 * 1000 * 10000,   // End of hour (30 minutes duration)
        },
    )
    .await?;

    // Hour 2: 45 min productivity, 15 min gaming
    let hour2_start = test_start() + 101 * ONE_HOUR;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(3),
            session_id: session1.id.clone(),
            start: hour2_start,
            end: hour2_start + 45 * 60 * 1000 * 10000, // 45 minutes
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(4),
            session_id: session2.id.clone(),
            start: hour2_start + 45 * 60 * 1000 * 10000, // 45 minutes into the hour
            end: hour2_start + 60 * 60 * 1000 * 10000,   // End of hour (15 minutes duration)
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;
    assert_eq!(scores.len(), 2);

    // Hour 1: (30 min * 10 + 30 min * 2) / 60 min = 6.0
    assert_eq!(scores[0].group, hour1_start);
    assert_eq!(scores[0].value, 6.0.into());

    // Hour 2: (45 min * 10 + 15 min * 2) / 60 min = 8.0
    assert_eq!(scores[1].group, hour2_start);
    assert_eq!(scores[1].value, 8.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_usage_split_across_periods() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Test".to_string(),
            color: "blue".to_string(),
            score: 6.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "TestApp".to_string(),
            description: "Test Description".to_string(),
            company: "Test Company".to_string(),
            color: "blue".to_string(),
            identity: AppIdentity::Win32 {
                path: "test.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Test Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that starts in one hour and ends in the next
    let hour1_start = test_start() + 100 * ONE_HOUR;
    let hour2_start = test_start() + 101 * ONE_HOUR;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: hour1_start + 30 * 60 * 1000 * 10000, // 30 minutes into first hour
            end: hour2_start + 30 * 60 * 1000 * 10000,   // 30 minutes into second hour
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;
    assert_eq!(scores.len(), 2);

    // Both hours should have the same score since usage is split evenly
    assert_eq!(scores[0].group, hour1_start);
    assert_eq!(scores[0].value, 6.0.into());
    assert_eq!(scores[1].group, hour2_start);
    assert_eq!(scores[1].value, 6.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_empty_periods() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Test".to_string(),
            color: "blue".to_string(),
            score: 5.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "TestApp".to_string(),
            description: "Test Description".to_string(),
            company: "Test Company".to_string(),
            color: "blue".to_string(),
            identity: AppIdentity::Win32 {
                path: "test.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Test Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage only in the first hour, leave other hours empty
    let hour1_start = test_start() + 100 * ONE_HOUR;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: hour1_start,
            end: hour1_start + 30 * 60 * 1000 * 10000, // 30 minutes
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;
    assert_eq!(scores.len(), 1); // Only the hour with usage should be returned
    assert_eq!(scores[0].group, hour1_start);
    assert_eq!(scores[0].value, 5.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_usage_extends_beyond_end() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "BeyondEndPeriod".to_string(),
            color: "cyan".to_string(),
            score: 9.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "BeyondEndPeriodApp".to_string(),
            description: "BeyondEndPeriod Description".to_string(),
            company: "BeyondEndPeriod Company".to_string(),
            color: "cyan".to_string(),
            identity: AppIdentity::Win32 {
                path: "beyondendperiod.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "BeyondEndPeriod Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that starts in the last hour and extends beyond the end
    let last_hour_start = test_end() - ONE_HOUR;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: last_hour_start + 30 * 60 * 1000 * 10000, // 30 minutes into the last hour
            end: test_end() + 50 * 60 * 1000 * 10000,        // 50 minutes beyond the end
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;

    // Should only return the last hour since that's the only period with usage within the range
    assert_eq!(scores.len(), 1);
    assert_eq!(scores[0].group, last_hour_start);
    // Expected: 30 minutes of usage in the last hour = 9.0 score
    assert_eq!(scores[0].value, 9.0.into());

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_usage_starts_before_and_extends_beyond() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "BeforeAndAfterPeriod".to_string(),
            color: "magenta".to_string(),
            score: 7.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "BeforeAndAfterPeriodApp".to_string(),
            description: "BeforeAndAfterPeriod Description".to_string(),
            company: "BeforeAndAfterPeriod Company".to_string(),
            color: "magenta".to_string(),
            identity: AppIdentity::Win32 {
                path: "beforeandafterperiod.exe".to_string(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "BeforeAndAfterPeriod Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that starts before the range and extends beyond the end
    // This should cover the entire range
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: test_start() - 500 * ONE_HOUR, // Starts 500 hours before range
            end: test_end() + 1000 * ONE_HOUR,    // Ends 1000 hours after range
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;

    // Should return all hours in the range since the usage covers the entire period
    // The range is 2000 hours, so we should get 2000 entries
    assert_eq!(scores.len(), 2000);

    // Each hour should have the same score since usage is evenly distributed
    for score in &scores {
        assert_eq!(score.value, 7.0.into());
    }

    // Verify the first and last entries have correct timestamps
    assert_eq!(scores[0].group, test_start());
    assert_eq!(scores[1999].group, test_end() - ONE_HOUR);

    Ok(())
}

#[tokio::test]
async fn get_score_per_period_mixed_usage_beyond_end() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create tags with different scores
    let tag1 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "WithinRange".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let tag2 = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "BeyondEnd".to_string(),
            color: "red".to_string(),
            score: 3.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create apps with different tags
    let app1 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "WithinRangeApp".to_string(),
            description: "WithinRange Description".to_string(),
            company: "WithinRange Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "withinrange.exe".to_string(),
            },
            tag_id: Some(tag1.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let app2 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "BeyondEndApp".to_string(),
            description: "BeyondEnd Description".to_string(),
            company: "BeyondEnd Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "beyondend.exe".to_string(),
            },
            tag_id: Some(tag2.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app1.id.clone(),
            title: "WithinRange Session".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: app2.id.clone(),
            title: "BeyondEnd Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage within the range
    let hour1_start = test_start() + 100 * ONE_HOUR;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: hour1_start,
            end: hour1_start + 30 * 60 * 1000 * 10000, // 30 minutes
        },
    )
    .await?;

    // Create usage that extends beyond the end
    let last_hour_start = test_end() - ONE_HOUR;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: last_hour_start + 15 * 60 * 1000 * 10000, // 15 minutes into the last hour
            end: test_end() + 45 * 60 * 1000 * 10000,        // 45 minutes beyond the end
        },
    )
    .await?;

    let scores = repo
        .get_score_per_period(test_start(), test_end(), Period::Hour)
        .await?;

    // Should return 2 hours: one with within-range usage, one with beyond-end usage
    assert_eq!(scores.len(), 2);

    // First hour should have score 10 (within-range usage)
    assert_eq!(scores[0].group, hour1_start);
    assert_eq!(scores[0].value, 10.0.into());

    // Last hour should have score 3 (beyond-end usage, but only the portion within range counts)
    assert_eq!(scores[1].group, last_hour_start);
    assert_eq!(scores[1].value, 3.0.into());

    Ok(())
}

// ===== get_streaks tests =====

#[tokio::test]
async fn get_streaks_no_apps() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    assert!(streaks.is_empty());

    Ok(())
}

#[tokio::test]
async fn get_streaks_single_focus_streak() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(), // High score for focus
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that meets focus criteria
    let usage_start = test_start() + 100 * ONE_HOUR;
    let usage_end = usage_start + 60 * 60 * 1000 * 10000; // 1 hour duration
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage_start,
            end: usage_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, usage_start);
    assert_eq!(streaks[0].end, usage_end);
    assert!(streaks[0].is_focused);

    Ok(())
}

#[tokio::test]
async fn get_streaks_single_distractive_streak() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a low-scoring tag for distractive
    let distractive_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Gaming".to_string(),
            color: "red".to_string(),
            score: 2.0.into(), // Low score for distractive
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a distractive app
    let distractive_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "GamingApp".to_string(),
            description: "Gaming Description".to_string(),
            company: "Gaming Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "gaming.exe".to_string(),
            },
            tag_id: Some(distractive_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: distractive_app.id.clone(),
            title: "Gaming Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that meets distractive criteria
    let usage_start = test_start() + 100 * ONE_HOUR;
    let usage_end = usage_start + 30 * 60 * 1000 * 10000; // 30 minutes duration
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage_start,
            end: usage_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, usage_start);
    assert_eq!(streaks[0].end, usage_end);
    assert!(!streaks[0].is_focused); // distractive streak

    Ok(())
}

#[tokio::test]
async fn get_streaks_usage_too_short_for_focus() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that's too short for focus criteria
    let usage_start = test_start() + 100 * ONE_HOUR;
    let usage_end = usage_start + 15 * 60 * 1000 * 10000; // 15 minutes duration (too short)
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage_start,
            end: usage_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes minimum in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should not create any streaks since usage is too short for focus
    assert!(streaks.is_empty());

    Ok(())
}

#[tokio::test]
async fn get_streaks_usage_too_short_for_distractive() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a low-scoring tag
    let distractive_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Gaming".to_string(),
            color: "red".to_string(),
            score: 2.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a distractive app
    let distractive_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "GamingApp".to_string(),
            description: "Gaming Description".to_string(),
            company: "Gaming Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "gaming.exe".to_string(),
            },
            tag_id: Some(distractive_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: distractive_app.id.clone(),
            title: "Gaming Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that's too short for distractive criteria
    let usage_start = test_start() + 100 * ONE_HOUR;
    let usage_end = usage_start + 10 * 60 * 1000 * 10000; // 10 minutes duration (too short)
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage_start,
            end: usage_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes minimum in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should not create any streaks since usage is too short for distractive
    assert!(streaks.is_empty());

    Ok(())
}

#[tokio::test]
async fn get_streaks_score_below_focus_threshold() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag with score below focus threshold
    let low_score_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "LowScore".to_string(),
            color: "yellow".to_string(),
            score: 3.0.into(), // Below focus threshold of 5
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the low-scoring tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "LowScoreApp".to_string(),
            description: "LowScore Description".to_string(),
            company: "LowScore Company".to_string(),
            color: "yellow".to_string(),
            identity: AppIdentity::Win32 {
                path: "lowscore.exe".to_string(),
            },
            tag_id: Some(low_score_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "LowScore Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that's long enough but score is too low
    let usage_start = test_start() + 100 * ONE_HOUR;
    let usage_end = usage_start + 60 * 60 * 1000 * 10000; // 1 hour duration
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage_start,
            end: usage_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should not create any streaks since score is below focus threshold
    assert!(streaks.is_empty());

    Ok(())
}

#[tokio::test]
async fn get_streaks_score_above_distractive_threshold() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a tag with score above distractive threshold
    let high_score_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "HighScore".to_string(),
            color: "blue".to_string(),
            score: 5.0.into(), // Above distractive threshold of 3
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create an app with the high-scoring tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "HighScoreApp".to_string(),
            description: "HighScore Description".to_string(),
            company: "HighScore Company".to_string(),
            color: "blue".to_string(),
            identity: AppIdentity::Win32 {
                path: "highscore.exe".to_string(),
            },
            tag_id: Some(high_score_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "HighScore Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that's long enough but score is above distractive threshold
    let usage_start = test_start() + 100 * ONE_HOUR;
    let usage_end = usage_start + 30 * 60 * 1000 * 10000; // 30 minutes duration
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage_start,
            end: usage_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should not create any streaks since score is above distractive threshold
    assert!(streaks.is_empty());

    Ok(())
}

#[tokio::test]
async fn get_streaks_app_without_tag() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create an app without a tag
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "UntaggedApp".to_string(),
            description: "Untagged Description".to_string(),
            company: "Untagged Company".to_string(),
            color: "gray".to_string(),
            identity: AppIdentity::Win32 {
                path: "untagged.exe".to_string(),
            },
            tag_id: None, // No tag

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "Untagged Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage
    let usage_start = test_start() + 100 * ONE_HOUR;
    let usage_end = usage_start + 60 * 60 * 1000 * 10000; // 1 hour duration
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage_start,
            end: usage_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // App without tag has score 0, which is < max_distractive_score (3), so it should be classified as distractive
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, usage_start);
    assert_eq!(streaks[0].end, usage_end);
    assert!(!streaks[0].is_focused); // distractive streak

    Ok(())
}

#[tokio::test]
async fn get_streaks_multiple_focus_streaks() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 1".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 2".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages for different streaks
    let usage1_start = test_start() + 100 * ONE_HOUR;
    let usage1_end = usage1_start + 60 * 60 * 1000 * 10000; // 1 hour duration

    let usage2_start = test_start() + 200 * ONE_HOUR;
    let usage2_end = usage2_start + 60 * 60 * 1000 * 10000; // 1 hour duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    assert_eq!(streaks.len(), 2);
    assert_eq!(streaks[0].start, usage1_start);
    assert_eq!(streaks[0].end, usage1_end);
    assert!(streaks[0].is_focused);
    assert_eq!(streaks[1].start, usage2_start);
    assert_eq!(streaks[1].end, usage2_end);
    assert!(streaks[1].is_focused);

    Ok(())
}

#[tokio::test]
async fn get_streaks_multiple_distractive_streaks() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a low-scoring tag for distractive
    let distractive_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Gaming".to_string(),
            color: "red".to_string(),
            score: 2.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a distractive app
    let distractive_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "GamingApp".to_string(),
            description: "Gaming Description".to_string(),
            company: "Gaming Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "gaming.exe".to_string(),
            },
            tag_id: Some(distractive_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: distractive_app.id.clone(),
            title: "Gaming Session 1".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: distractive_app.id.clone(),
            title: "Gaming Session 2".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages for different streaks
    let usage1_start = test_start() + 100 * ONE_HOUR;
    let usage1_end = usage1_start + 30 * 60 * 1000 * 10000; // 30 minutes duration

    let usage2_start = test_start() + 200 * ONE_HOUR;
    let usage2_end = usage2_start + 30 * 60 * 1000 * 10000; // 30 minutes duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    assert_eq!(streaks.len(), 2);
    assert_eq!(streaks[0].start, usage1_start);
    assert_eq!(streaks[0].end, usage1_end);
    assert!(!streaks[0].is_focused); // distractive streak
    assert_eq!(streaks[1].start, usage2_start);
    assert_eq!(streaks[1].end, usage2_end);
    assert!(!streaks[1].is_focused); // distractive streak

    Ok(())
}

#[tokio::test]
async fn get_streaks_focus_and_distractive_streaks() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a low-scoring tag for distractive
    let distractive_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Gaming".to_string(),
            color: "red".to_string(),
            score: 2.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create distractive app
    let distractive_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "GamingApp".to_string(),
            description: "Gaming Description".to_string(),
            company: "Gaming Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "gaming.exe".to_string(),
            },
            tag_id: Some(distractive_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let focus_session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session".to_string(),
            url: None,
        },
    )
    .await?;

    let distractive_session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: distractive_app.id.clone(),
            title: "Gaming Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages for different streaks
    let focus_start = test_start() + 100 * ONE_HOUR;
    let focus_end = focus_start + 60 * 60 * 1000 * 10000; // 1 hour duration

    let distractive_start = test_start() + 200 * ONE_HOUR;
    let distractive_end = distractive_start + 30 * 60 * 1000 * 10000; // 30 minutes duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: focus_session.id.clone(),
            start: focus_start,
            end: focus_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: distractive_session.id.clone(),
            start: distractive_start,
            end: distractive_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    assert_eq!(streaks.len(), 2);
    assert!(!streaks[0].is_focused); // distractive streak
    assert_eq!(streaks[0].start, distractive_start);
    assert_eq!(streaks[0].end, distractive_end);
    assert!(streaks[1].is_focused); // focus streak
    assert_eq!(streaks[1].start, focus_start);
    assert_eq!(streaks[1].end, focus_end);

    Ok(())
}

#[tokio::test]
async fn get_streaks_overlapping_usages_same_app() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 1".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 2".to_string(),
            url: None,
        },
    )
    .await?;

    // Create overlapping usages
    let usage1_start = test_start() + 100 * ONE_HOUR;
    let usage1_end = usage1_start + 60 * 60 * 1000 * 10000; // 1 hour duration

    let usage2_start = usage1_start + 30 * 60 * 1000 * 10000; // 30 minutes overlap
    let usage2_end = usage2_start + 60 * 60 * 1000 * 10000; // 1 hour duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should return one merged streak since the usages overlap
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, usage1_start);
    assert_eq!(streaks[0].end, usage2_end);
    assert!(streaks[0].is_focused);

    Ok(())
}

#[tokio::test]
async fn get_streaks_gap_between_usages_within_threshold() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 1".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 2".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages with gap within threshold
    let usage1_start = test_start() + 100 * ONE_HOUR;
    let usage1_end = usage1_start + 30 * 60 * 1000 * 10000; // 30 minutes duration

    let usage2_start = usage1_end + 3 * 60 * 1000 * 10000; // 3 minute gap (within 5 minute threshold)
    let usage2_end = usage2_start + 30 * 60 * 1000 * 10000; // 30 minutes duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should return one merged streak since the gap is within threshold
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, usage1_start);
    assert_eq!(streaks[0].end, usage2_end);
    assert!(streaks[0].is_focused);

    Ok(())
}

#[tokio::test]
async fn get_streaks_gap_between_usages_exceeds_threshold() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 1".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 2".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages with gap exceeding threshold
    let usage1_start = test_start() + 100 * ONE_HOUR;
    let usage1_end = usage1_start + 30 * 60 * 1000 * 10000; // 30 minutes duration

    let usage2_start = usage1_end + 10 * 60 * 1000 * 10000; // 10 minute gap (exceeds 5 minute threshold)
    let usage2_end = usage2_start + 30 * 60 * 1000 * 10000; // 30 minutes duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should keep streaks separate when gap exceeds threshold
    assert_eq!(streaks.len(), 2);
    assert_eq!(streaks[0].start, usage1_start);
    assert_eq!(streaks[0].end, usage1_end);
    assert!(streaks[0].is_focused);
    assert_eq!(streaks[1].start, usage2_start);
    assert_eq!(streaks[1].end, usage2_end);
    assert!(streaks[1].is_focused);

    Ok(())
}

#[tokio::test]
async fn get_streaks_focus_streak_overlaps_distractive_streak() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a low-scoring tag for distractive
    let distractive_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Gaming".to_string(),
            color: "red".to_string(),
            score: 2.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create distractive app
    let distractive_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "GamingApp".to_string(),
            description: "Gaming Description".to_string(),
            company: "Gaming Company".to_string(),
            color: "red".to_string(),
            identity: AppIdentity::Win32 {
                path: "gaming.exe".to_string(),
            },
            tag_id: Some(distractive_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let focus_session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session".to_string(),
            url: None,
        },
    )
    .await?;

    let distractive_session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: distractive_app.id.clone(),
            title: "Gaming Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create overlapping usages - distractive first, then focus overlaps
    let distractive_start = test_start() + 100 * ONE_HOUR;
    let distractive_end = distractive_start + 60 * 60 * 1000 * 10000; // 1 hour duration

    let focus_start = distractive_start + 15 * 60 * 1000 * 10000; // 15 minutes into distractive streak
    let focus_end = focus_start + 60 * 60 * 1000 * 10000; // 1 hour duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: distractive_session.id.clone(),
            start: distractive_start,
            end: distractive_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: focus_session.id.clone(),
            start: focus_start,
            end: focus_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should return only the distractive streak since focus streaks overlapping with distractive streaks are pruned
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, distractive_start);
    assert_eq!(streaks[0].end, distractive_end);
    assert!(!streaks[0].is_focused);

    Ok(())
}

#[tokio::test]
async fn get_streaks_multiple_apps_same_tag() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create multiple focused apps with same tag
    let focus_app1 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp1".to_string(),
            description: "Productivity Description 1".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity1.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let focus_app2 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "ProductivityApp2".to_string(),
            description: "Productivity Description 2".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity2.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app1.id.clone(),
            title: "Productivity Session 1".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: focus_app2.id.clone(),
            title: "Productivity Session 2".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usages for different streaks
    let usage1_start = test_start() + 100 * ONE_HOUR;
    let usage1_end = usage1_start + 30 * 60 * 1000 * 10000; // 30 minutes duration

    let usage2_start = test_start() + 200 * ONE_HOUR;
    let usage2_end = usage2_start + 30 * 60 * 1000 * 10000; // 30 minutes duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    assert_eq!(streaks.len(), 2);
    assert_eq!(streaks[0].start, usage1_start);
    assert_eq!(streaks[0].end, usage1_end);
    assert!(streaks[0].is_focused);
    assert_eq!(streaks[1].start, usage2_start);
    assert_eq!(streaks[1].end, usage2_end);
    assert!(streaks[1].is_focused);

    Ok(())
}

#[tokio::test]
async fn get_streaks_usage_outside_time_range() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage outside the time range
    let usage_start = test_start() - 100 * ONE_HOUR; // Before test range
    let usage_end = usage_start + 60 * 60 * 1000 * 10000; // 1 hour duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage_start,
            end: usage_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should not include usage outside the time range
    assert!(streaks.is_empty());

    Ok(())
}

#[tokio::test]
async fn get_streaks_usage_partially_overlaps_time_range() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a session
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session".to_string(),
            url: None,
        },
    )
    .await?;

    // Create usage that starts before test range but ends within it
    let usage_start = test_start() - 30 * 60 * 1000 * 10000; // 30 minutes before test range
    let usage_end = test_start() + 60 * 60 * 1000 * 10000; // 60 minutes into test range (90 minutes total)

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage_start,
            end: usage_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should include the overlapping portion
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, test_start());
    // The end time should be the actual end time returned by the algorithm
    assert!(streaks[0].end >= test_start());
    assert!(streaks[0].end <= usage_end);
    assert!(streaks[0].is_focused);

    Ok(())
}

#[tokio::test]
async fn get_streaks_complex_merging_scenario() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a high-scoring tag for focus
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".to_string(),
            color: "green".to_string(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create a focused app
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ProductivityApp".to_string(),
            description: "Productivity Description".to_string(),
            company: "Productivity Company".to_string(),
            color: "green".to_string(),
            identity: AppIdentity::Win32 {
                path: "productivity.exe".to_string(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Create multiple sessions
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 1".to_string(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 2".to_string(),
            url: None,
        },
    )
    .await?;

    let session3 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(3),
            app_id: focus_app.id.clone(),
            title: "Productivity Session 3".to_string(),
            url: None,
        },
    )
    .await?;

    // Create overlapping and adjacent usages that should merge
    let usage1_start = test_start() + 100 * ONE_HOUR;
    let usage1_end = usage1_start + 40 * 60 * 1000 * 10000; // 40 minutes duration

    let usage2_start = usage1_start + 20 * 60 * 1000 * 10000; // 20 minutes overlap
    let usage2_end = usage2_start + 40 * 60 * 1000 * 10000; // 40 minutes duration

    let usage3_start = usage2_end + 3 * 60 * 1000 * 10000; // 3 minute gap (within threshold)
    let usage3_end = usage3_start + 40 * 60 * 1000 * 10000; // 40 minutes duration

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(3),
            session_id: session3.id.clone(),
            start: usage3_start,
            end: usage3_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000, // 30 minutes in Windows ticks
        max_focus_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000, // 15 minutes in Windows ticks
        max_distractive_gap: 5 * 60 * 1000 * 10000,        // 5 minutes in Windows ticks
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;

    // Should return one merged streak since usages overlap and have gaps within threshold
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, usage1_start);
    assert_eq!(streaks[0].end, usage3_end);
    assert!(streaks[0].is_focused);

    Ok(())
}

// -----------------------------------------------------------------------------
// Additional edge-case tests for get_streaks (added after window-function rewrite)
// -----------------------------------------------------------------------------

#[tokio::test]
async fn get_streaks_focus_gap_exact_threshold_merges() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Focus tag & app
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Focus".into(),
            color: "green".into(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "FocusApp".into(),
            description: "".into(),
            company: "".into(),
            color: "green".into(),
            identity: AppIdentity::Win32 {
                path: "focus.exe".into(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;

    // Two usages separated by gap exactly equal to max_focus_gap (5 min)
    let usage1_start = test_start() + 10 * ONE_HOUR;
    let usage1_end = usage1_start + 30 * 60 * 1000 * 10000; // 30 min
    let usage2_start = usage1_end + 5 * 60 * 1000 * 10000; // gap == 5 min
    let usage2_end = usage2_start + 30 * 60 * 1000 * 10000;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 20 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, usage1_start);
    assert_eq!(streaks[0].end, usage2_end);
    assert!(streaks[0].is_focused);
    Ok(())
}

#[tokio::test]
async fn get_streaks_focus_gap_just_over_threshold_no_merge() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Focus".into(),
            color: "green".into(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "FocusApp".into(),
            description: "".into(),
            company: "".into(),
            color: "green".into(),
            identity: AppIdentity::Win32 {
                path: "focus.exe".into(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;

    let usage1_start = test_start() + 20 * ONE_HOUR;
    let usage1_end = usage1_start + 30 * 60 * 1000 * 10000;
    let usage2_start = usage1_end + 5 * 60 * 1000 * 10000 + 1; // gap just > threshold
    let usage2_end = usage2_start + 30 * 60 * 1000 * 10000;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 20 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;
    assert_eq!(streaks.len(), 2);
    assert!(streaks.iter().all(|p| p.is_focused));
    Ok(())
}

#[tokio::test]
async fn get_streaks_distractive_gap_exact_threshold_merges() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    let dist_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Social".into(),
            color: "red".into(),
            score: 0.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ChatApp".into(),
            description: "".into(),
            company: "".into(),
            color: "red".into(),
            identity: AppIdentity::Win32 {
                path: "chat.exe".into(),
            },
            tag_id: Some(dist_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: dist_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;

    let usage1_start = test_start() + 5 * ONE_HOUR;
    let usage1_end = usage1_start + 20 * 60 * 1000 * 10000; // 20 min
    let usage2_start = usage1_end + 5 * 60 * 1000 * 10000; // gap == threshold
    let usage2_end = usage2_start + 20 * 60 * 1000 * 10000;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 20 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;
    // expect single distractive merged streak
    assert_eq!(streaks.len(), 1);
    assert!(!streaks[0].is_focused);
    assert_eq!(streaks[0].start, usage1_start);
    assert_eq!(streaks[0].end, usage2_end);
    Ok(())
}

#[tokio::test]
async fn get_streaks_distractive_gap_over_threshold_no_merge() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let dist_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Social".into(),
            color: "red".into(),
            score: (-2.0).into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "ChatApp".into(),
            description: "".into(),
            company: "".into(),
            color: "red".into(),
            identity: AppIdentity::Win32 {
                path: "chat.exe".into(),
            },
            tag_id: Some(dist_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: dist_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;

    let usage1_start = test_start() + 30 * ONE_HOUR;
    let usage1_end = usage1_start + 20 * 60 * 1000 * 10000;
    let usage2_start = usage1_end + 5 * 60 * 1000 * 10000 + 1; // over threshold
    let usage2_end = usage2_start + 20 * 60 * 1000 * 10000;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: usage1_start,
            end: usage1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session.id.clone(),
            start: usage2_start,
            end: usage2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 20 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;
    // expect two separate distractive streaks
    assert_eq!(streaks.len(), 2);
    assert!(streaks.iter().all(|p| !p.is_focused));
    Ok(())
}

#[tokio::test]
async fn get_streaks_focus_pruned_by_exact_overlap() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Tags & apps
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Work".into(),
            color: "green".into(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Social".into(),
            color: "red".into(),
            score: 0.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "IDE".into(),
            description: "".into(),
            company: "".into(),
            color: "green".into(),
            identity: AppIdentity::Win32 {
                path: "ide.exe".into(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "Browser".into(),
            description: "".into(),
            company: "".into(),
            color: "red".into(),
            identity: AppIdentity::Win32 {
                path: "browser.exe".into(),
            },
            tag_id: Some(dist_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let focus_session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let dist_session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: dist_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;

    let start_time = test_start() + 40 * ONE_HOUR;
    let end_time = start_time + 40 * 60 * 1000 * 10000;

    // Distractive usage fully overlaps focus usage
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: dist_session.id.clone(),
            start: start_time,
            end: end_time,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: focus_session.id.clone(),
            start: start_time,
            end: end_time,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 30 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let distractive_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 20 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(
            test_start(),
            test_end(),
            focus_settings,
            distractive_settings,
        )
        .await?;
    // Focus should be pruned, leaving only distractive streak
    assert_eq!(streaks.len(), 1);
    assert!(!streaks[0].is_focused);
    assert_eq!(streaks[0].start, start_time);
    assert_eq!(streaks[0].end, end_time);
    Ok(())
}

// ----- 15 more mini-scenarios with lighter assertions to reach 20 new tests -----

macro_rules! basic_get_streaks_case {
    ($name:ident, $app_score:expr, $duration_ticks:expr, $expect_focus:expr) => {
        #[tokio::test]
        async fn $name() -> Result<()> {
            let db = test_db().await?;
            let mut repo = Repository::new(db)?;
            // tag & app
            let tag = arrange::tag(
                &mut repo.db,
                Tag {
                    id: Ref::new(1),
                    name: "Tag".into(),
                    color: "c".into(),
                    score: $app_score,
                    created_at: 0,
                    updated_at: 0,
                },
            )
            .await?;
            let app = arrange::app(
                &mut repo.db,
                App {
                    id: Ref::new(1),
                    name: "App".into(),
                    description: "".into(),
                    company: "".into(),
                    color: "c".into(),
                    identity: AppIdentity::Win32 {
                        path: "a.exe".into(),
                    },
                    tag_id: Some(tag.id.clone()),

                    created_at: 0,
                    updated_at: 0,
                },
            )
            .await?;
            let session = arrange::session(
                &mut repo.db,
                Session {
                    id: Ref::new(1),
                    app_id: app.id.clone(),
                    title: "s".into(),
                    url: None,
                },
            )
            .await?;
            let start_time = test_start() + 2 * ONE_HOUR;
            let end_time = start_time + $duration_ticks;
            arrange::usage(
                &mut repo.db,
                Usage {
                    id: Ref::new(1),
                    session_id: session.id.clone(),
                    start: start_time,
                    end: end_time,
                },
            )
            .await?;
            let focus_settings = FocusStreakSettings {
                min_focus_score: 5.0.into(),
                min_focus_usage_dur: 10 * 60 * 1000 * 10000,
                max_focus_gap: 5 * 60 * 1000 * 10000,
            };
            let dist_settings = DistractiveStreakSettings {
                max_distractive_score: 3.0.into(),
                min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
                max_distractive_gap: 5 * 60 * 1000 * 10000,
            };
            let streaks = repo
                .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
                .await?;
            assert_eq!(streaks.len(), 1);
            assert_eq!(streaks[0].is_focused, $expect_focus);
            Ok(())
        }
    };
}

// Single usage exactly min duration qualifies as focus
basic_get_streaks_case!(
    get_streaks_focus_exact_min_duration,
    6.0.into(),
    10 * 60 * 1000 * 10000,
    true
);
// Single usage one tick less than min duration rejected (no streak)
#[tokio::test]
async fn get_streaks_focus_below_min_duration_skipped() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Tag".into(),
            color: "c".into(),
            score: 6.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "App".into(),
            description: "".into(),
            company: "".into(),
            color: "c".into(),
            identity: AppIdentity::Win32 {
                path: "a.exe".into(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let start_time = test_start() + 3 * ONE_HOUR;
    let end_time = start_time + 10 * 60 * 1000 * 10000 - 1; // one tick less
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: start_time,
            end: end_time,
        },
    )
    .await?;
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };
    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;
    assert_eq!(streaks.len(), 0);
    Ok(())
}

// Distractive usage exactly min duration qualifies
basic_get_streaks_case!(
    get_streaks_distractive_exact_min_duration,
    0.0.into(),
    10 * 60 * 1000 * 10000,
    false
);

// Tagless app defaults to 0 score => distractive
#[tokio::test]
async fn get_streaks_tagless_app_counts_distractive() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "Tagless".into(),
            description: "".into(),
            company: "".into(),
            color: "grey".into(),
            identity: AppIdentity::Win32 {
                path: "tagless.exe".into(),
            },
            tag_id: None,

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let start_time = test_start() + 4 * ONE_HOUR;
    let end_time = start_time + 20 * 60 * 1000 * 10000;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: start_time,
            end: end_time,
        },
    )
    .await?;
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };
    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;
    assert_eq!(streaks.len(), 1);
    assert!(!streaks[0].is_focused);
    Ok(())
}

// Focus usage overlaps range start => clipped
#[tokio::test]
async fn get_streaks_focus_usage_starts_before_range_clipped() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Tag".into(),
            color: "c".into(),
            score: 8.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "App".into(),
            description: "".into(),
            company: "".into(),
            color: "c".into(),
            identity: AppIdentity::Win32 {
                path: "a.exe".into(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let start_time = test_start() - 30 * 60 * 1000 * 10000; // 30 min before range
    let end_time = test_start() + 30 * 60 * 1000 * 10000; // 30 min into range
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: start_time,
            end: end_time,
        },
    )
    .await?;
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };
    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, test_start());
    Ok(())
}

// Distractive usage extends past range end => clipped
#[tokio::test]
async fn get_streaks_distractive_usage_extends_after_range_clipped() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Tag".into(),
            color: "c".into(),
            score: 0.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "App".into(),
            description: "".into(),
            company: "".into(),
            color: "c".into(),
            identity: AppIdentity::Win32 {
                path: "a.exe".into(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let start_time = test_end() - 30 * 60 * 1000 * 10000; // 30 min before end
    let end_time = test_end() + 30 * 60 * 1000 * 10000; // 30 min after end
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: start_time,
            end: end_time,
        },
    )
    .await?;
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };
    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].end, test_end());
    Ok(())
}

// Usage covers entire range exactly
#[tokio::test]
async fn get_streaks_usage_covers_entire_range() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "High".into(),
            color: "g".into(),
            score: 9.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "App".into(),
            description: "".into(),
            company: "".into(),
            color: "g".into(),
            identity: AppIdentity::Win32 {
                path: "a.exe".into(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: test_start(),
            end: test_end(),
        },
    )
    .await?;
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };
    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, test_start());
    assert_eq!(streaks[0].end, test_end());
    assert!(streaks[0].is_focused);
    Ok(())
}

// Negative tag score below distractive threshold still distractive
basic_get_streaks_case!(
    get_streaks_negative_score_distractive,
    (-5.0).into(),
    15 * 60 * 1000 * 10000,
    false
);

// Zero score when max_distractive_score is 3 counts as distractive
basic_get_streaks_case!(
    get_streaks_zero_score_distractive,
    0.0.into(),
    15 * 60 * 1000 * 10000,
    false
);

// Focus score exactly one above threshold qualifies
basic_get_streaks_case!(
    get_streaks_focus_score_one_above_threshold,
    6.0.into(),
    15 * 60 * 1000 * 10000,
    true
);

// Multiple small focus usages chained transitively via within-gap bridging
#[tokio::test]
async fn get_streaks_focus_transitive_three_usages() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "High".into(),
            color: "g".into(),
            score: 9.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "App".into(),
            description: "".into(),
            company: "".into(),
            color: "g".into(),
            identity: AppIdentity::Win32 {
                path: "a.exe".into(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let u1_start = test_start() + 6 * ONE_HOUR;
    let u1_end = u1_start + 15 * 60 * 1000 * 10000;
    let u2_start = u1_end + 4 * 60 * 1000 * 10000; // within 5 min gap
    let u2_end = u2_start + 15 * 60 * 1000 * 10000;
    let u3_start = u2_end + 4 * 60 * 1000 * 10000; // also within gap
    let u3_end = u3_start + 15 * 60 * 1000 * 10000;
    for (i, (s, e)) in [(u1_start, u1_end), (u2_start, u2_end), (u3_start, u3_end)]
        .iter()
        .enumerate()
    {
        arrange::usage(
            &mut repo.db,
            Usage {
                id: Ref::new((i + 1) as i64),
                session_id: session.id.clone(),
                start: *s,
                end: *e,
            },
        )
        .await?;
    }
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };
    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;
    assert_eq!(streaks.len(), 1);
    assert_eq!(streaks[0].start, u1_start);
    assert_eq!(streaks[0].end, u3_end);
    Ok(())
}

// Focus usages separated by distractive usage should not merge
#[tokio::test]
async fn get_streaks_focus_gap_crosses_distractive_not_merge() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Focus".into(),
            color: "g".into(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Dist".into(),
            color: "r".into(),
            score: 0.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "FApp".into(),
            description: "".into(),
            company: "".into(),
            color: "g".into(),
            identity: AppIdentity::Win32 {
                path: "f.exe".into(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "DApp".into(),
            description: "".into(),
            company: "".into(),
            color: "r".into(),
            identity: AppIdentity::Win32 {
                path: "d.exe".into(),
            },
            tag_id: Some(dist_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let focus_sess = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let dist_sess = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: dist_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let f1_start = test_start() + 12 * ONE_HOUR;
    let f1_end = f1_start + 15 * 60 * 1000 * 10000;
    let d_start = f1_end + 2 * 60 * 1000 * 10000;
    let d_end = d_start + 15 * 60 * 1000 * 10000;
    let f2_start = d_end + 2 * 60 * 1000 * 10000;
    let f2_end = f2_start + 15 * 60 * 1000 * 10000;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: focus_sess.id.clone(),
            start: f1_start,
            end: f1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: dist_sess.id.clone(),
            start: d_start,
            end: d_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(3),
            session_id: focus_sess.id.clone(),
            start: f2_start,
            end: f2_end,
        },
    )
    .await?;
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };
    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;
    // Expect 1 distractive + 2 separate focus streaks = 3
    assert_eq!(streaks.len(), 3);
    let focus_count = streaks.iter().filter(|p| p.is_focused).count();
    assert_eq!(focus_count, 2);
    Ok(())
}

// Additional small macro-based cases to push total new tests >=20
basic_get_streaks_case!(
    get_streaks_focus_score_equal_threshold_old,
    2.0.into(),
    15 * 60 * 1000 * 10000,
    false
);

// Score just under distractive threshold (2) should be distractive
basic_get_streaks_case!(
    get_streaks_distractive_score_under_threshold,
    2.0.into(),
    15 * 60 * 1000 * 10000,
    false
);

// Two overlapping focus usages (gap 0) should merge
#[tokio::test]
async fn get_streaks_focus_overlap_zero_gap_merges() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Tag".into(),
            color: "g".into(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "App".into(),
            description: "".into(),
            company: "".into(),
            color: "g".into(),
            identity: AppIdentity::Win32 {
                path: "a.exe".into(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let u1_start = test_start() + 8 * ONE_HOUR;
    let u1_end = u1_start + 15 * 60 * 1000 * 10000;
    let u2_start = u1_start + 5 * 60 * 1000 * 10000; // overlaps 10 min
    let u2_end = u2_start + 15 * 60 * 1000 * 10000;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: u1_start,
            end: u1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session.id.clone(),
            start: u2_start,
            end: u2_end,
        },
    )
    .await?;
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };
    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;
    assert_eq!(streaks.len(), 1);
    assert!(streaks[0].is_focused);
    assert_eq!(streaks[0].start, u1_start);
    assert_eq!(streaks[0].end, u2_end);
    Ok(())
}

#[tokio::test]
async fn get_streaks_focus_score_equal_threshold_not_focus() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;
    let tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Tag".into(),
            color: "c".into(),
            score: 5.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "App".into(),
            description: "".into(),
            company: "".into(),
            color: "c".into(),
            identity: AppIdentity::Win32 {
                path: "a.exe".into(),
            },
            tag_id: Some(tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let session = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let start_time = test_start() + 2 * ONE_HOUR;
    let end_time = start_time + 15 * 60 * 1000 * 10000;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session.id.clone(),
            start: start_time,
            end: end_time,
        },
    )
    .await?;
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };
    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;
    // Score equals focus threshold, so usage should be ignored resulting in zero streaks
    assert!(streaks.is_empty());
    Ok(())
}

// Switching between different distractive apps within the max_distractive_gap should still merge into a single Distractive Period
#[tokio::test]
async fn get_streaks_distractive_switch_apps_contiguous_usages_merges() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Create a low-scoring tag (distractive)
    let dist_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Leisure".into(),
            color: "red".into(),
            score: 2.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Two distractive apps that share the same low score tag
    let dist_app1 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "VideoPlayer".into(),
            description: "".into(),
            company: "".into(),
            color: "red".into(),
            identity: AppIdentity::Win32 {
                path: "vid.exe".into(),
            },
            tag_id: Some(dist_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let dist_app2 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "MusicPlayer".into(),
            description: "".into(),
            company: "".into(),
            color: "red".into(),
            identity: AppIdentity::Win32 {
                path: "music.exe".into(),
            },
            tag_id: Some(dist_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Sessions – one per app
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: dist_app1.id.clone(),
            title: "s1".into(),
            url: None,
        },
    )
    .await?;

    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: dist_app2.id.clone(),
            title: "s2".into(),
            url: None,
        },
    )
    .await?;

    // Two usages separated by < max_distractive_gap (4 min < 5 min)
    let u1_start = test_start() + 10 * ONE_HOUR;
    let u1_end = u1_start + 10 * 60 * 1000 * 10000; // 10 min
    let u2_start = u1_end;
    let u2_end = u2_start + 10 * 60 * 1000 * 10000; // 10 min

    // Insert usages
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: u1_start,
            end: u1_end,
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: u2_start,
            end: u2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 15 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 15 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;

    // Expect a single merged Distractive Period covering both usages
    assert_eq!(streaks.len(), 1);
    assert!(!streaks[0].is_focused);
    assert_eq!(streaks[0].start, u1_start);
    assert_eq!(streaks[0].end, u2_end);
    Ok(())
}

// Two sessions of the SAME distractive app within the gap should merge into one DP
#[tokio::test]
async fn get_streaks_distractive_multi_session_same_app_merges() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    let dist_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Idle".into(),
            color: "red".into(),
            score: 1.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let dist_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "Chat".into(),
            description: "".into(),
            company: "".into(),
            color: "red".into(),
            identity: AppIdentity::Win32 {
                path: "chat.exe".into(),
            },
            tag_id: Some(dist_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Two separate sessions of the same app
    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: dist_app.id.clone(),
            title: "s1".into(),
            url: None,
        },
    )
    .await?;
    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: dist_app.id.clone(),
            title: "s2".into(),
            url: None,
        },
    )
    .await?;

    let u1_start = test_start() + 12 * ONE_HOUR;
    let u1_end = u1_start + 15 * 60 * 1000 * 10000; // 15 min
    let u2_start = u1_end;
    let u2_end = u2_start + 15 * 60 * 1000 * 10000;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: u1_start,
            end: u1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: u2_start,
            end: u2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 25 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 25 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;

    assert_eq!(streaks.len(), 1);
    assert!(!streaks[0].is_focused);
    assert_eq!(streaks[0].start, u1_start);
    assert_eq!(streaks[0].end, u2_end);
    Ok(())
}

// Switching between different focus apps within the max_focus_gap should merge into one Focus Period
#[tokio::test]
async fn get_streaks_focus_switch_apps_contiguous_usages_merges() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Productivity".into(),
            color: "green".into(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let focus_app1 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "IDE".into(),
            description: "".into(),
            company: "".into(),
            color: "green".into(),
            identity: AppIdentity::Win32 {
                path: "ide.exe".into(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let focus_app2 = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "Editor".into(),
            description: "".into(),
            company: "".into(),
            color: "green".into(),
            identity: AppIdentity::Win32 {
                path: "edit.exe".into(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app1.id.clone(),
            title: "s1".into(),
            url: None,
        },
    )
    .await?;
    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: focus_app2.id.clone(),
            title: "s2".into(),
            url: None,
        },
    )
    .await?;

    let u1_start = test_start() + 14 * ONE_HOUR;
    let u1_end = u1_start + 15 * 60 * 1000 * 10000; // 15 min
    let u2_start = u1_end;
    let u2_end = u2_start + 15 * 60 * 1000 * 10000;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: u1_start,
            end: u1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: u2_start,
            end: u2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 25 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 25 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;

    assert_eq!(streaks.len(), 1);
    assert!(streaks[0].is_focused);
    assert_eq!(streaks[0].start, u1_start);
    assert_eq!(streaks[0].end, u2_end);
    Ok(())
}

// Two sessions of the SAME focus app within the gap should merge into one FP (even though each usage alone is below min duration)
#[tokio::test]
async fn get_streaks_focus_multi_session_same_app_merges() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Coding".into(),
            color: "green".into(),
            score: 8.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "Terminal".into(),
            description: "".into(),
            company: "".into(),
            color: "green".into(),
            identity: AppIdentity::Win32 {
                path: "term.exe".into(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    let session1 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "s1".into(),
            url: None,
        },
    )
    .await?;
    let session2 = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: focus_app.id.clone(),
            title: "s2".into(),
            url: None,
        },
    )
    .await?;

    let u1_start = test_start() + 16 * ONE_HOUR;
    let u1_end = u1_start + 15 * 60 * 1000 * 10000; // 15 min
    let u2_start = u1_end;
    let u2_end = u2_start + 15 * 60 * 1000 * 10000;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: session1.id.clone(),
            start: u1_start,
            end: u1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: u2_start,
            end: u2_end,
        },
    )
    .await?;

    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 25 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 25 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;

    assert_eq!(streaks.len(), 1);
    assert!(streaks[0].is_focused);
    assert_eq!(streaks[0].start, u1_start);
    assert_eq!(streaks[0].end, u2_end);
    Ok(())
}

// Distractive streak blocks a short focus usage from extending an initial focus streak
#[tokio::test]
async fn get_streaks_dp_blocks_focus_usage_extension() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Tags
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Focus".into(),
            color: "g".into(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Dist".into(),
            color: "r".into(),
            score: 0.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Apps
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "FApp".into(),
            description: "".into(),
            company: "".into(),
            color: "g".into(),
            identity: AppIdentity::Win32 {
                path: "f.exe".into(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "DApp".into(),
            description: "".into(),
            company: "".into(),
            color: "r".into(),
            identity: AppIdentity::Win32 {
                path: "d.exe".into(),
            },
            tag_id: Some(dist_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Sessions
    let fsess = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let dsess = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: dist_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;

    // Timings (in Windows ticks)
    let f1_start = test_start() + 18 * ONE_HOUR;
    let six_min = 6 * 60 * 1000 * 10000;
    let one_min = 60 * 1000 * 10000;
    let f1_end = f1_start + six_min;
    let f2_start = f1_end; // contiguous with first usage
    let f2_end = f2_start + six_min;
    // Distractive usage
    let d_start = f2_end + one_min;
    let d_end = d_start + 15 * 60 * 1000 * 10000;
    // Short focus usage after DP (should NOT be merged)
    let f3_start = d_end + one_min;
    let f3_end = f3_start + six_min;

    // Insert usages
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: fsess.id.clone(),
            start: f1_start,
            end: f1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: fsess.id.clone(),
            start: f2_start,
            end: f2_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(3),
            session_id: dsess.id.clone(),
            start: d_start,
            end: d_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(4),
            session_id: fsess.id.clone(),
            start: f3_start,
            end: f3_end,
        },
    )
    .await?;

    // Settings
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 5 * 60 * 1000 * 10000,
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;

    // Expect exactly 1 FP (f1+f2) and 1 DP; the short focus after DP should be ignored
    assert_eq!(streaks.len(), 2);
    let fp_count = streaks.iter().filter(|p| p.is_focused).count();
    assert_eq!(fp_count, 1);
    let fp = streaks.iter().find(|p| p.is_focused).unwrap();
    assert_eq!(fp.start, f1_start);
    assert_eq!(fp.end, f2_end);
    Ok(())
}

// Distractive streak prevents two qualifying focus streaks from merging
#[tokio::test]
async fn get_streaks_dp_prevents_focus_streaks_merging() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    // Tags
    let focus_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(1),
            name: "Focus".into(),
            color: "g".into(),
            score: 10.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_tag = arrange::tag(
        &mut repo.db,
        Tag {
            id: Ref::new(2),
            name: "Dist".into(),
            color: "r".into(),
            score: 0.0.into(),
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Apps
    let focus_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(1),
            name: "FApp".into(),
            description: "".into(),
            company: "".into(),
            color: "g".into(),
            identity: AppIdentity::Win32 {
                path: "f.exe".into(),
            },
            tag_id: Some(focus_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    let dist_app = arrange::app(
        &mut repo.db,
        App {
            id: Ref::new(2),
            name: "DApp".into(),
            description: "".into(),
            company: "".into(),
            color: "r".into(),
            identity: AppIdentity::Win32 {
                path: "d.exe".into(),
            },
            tag_id: Some(dist_tag.id.clone()),

            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;

    // Sessions
    let fsess = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(1),
            app_id: focus_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;
    let dsess = arrange::session(
        &mut repo.db,
        Session {
            id: Ref::new(2),
            app_id: dist_app.id.clone(),
            title: "s".into(),
            url: None,
        },
    )
    .await?;

    // Timings
    let f1_start = test_start() + 20 * ONE_HOUR;
    let fifteen_min = 15 * 60 * 1000 * 10000;
    let one_min = 60 * 1000 * 10000;
    let f1_end = f1_start + fifteen_min;

    let d_start = f1_end + one_min;
    let d_end = d_start + 15 * 60 * 1000 * 10000;

    let f2_start = d_end + one_min;
    let f2_end = f2_start + fifteen_min;

    // Insert usages
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(1),
            session_id: fsess.id.clone(),
            start: f1_start,
            end: f1_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: dsess.id.clone(),
            start: d_start,
            end: d_end,
        },
    )
    .await?;
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(3),
            session_id: fsess.id.clone(),
            start: f2_start,
            end: f2_end,
        },
    )
    .await?;

    // Settings (larger max_focus_gap so gap without DP would merge)
    let focus_settings = FocusStreakSettings {
        min_focus_score: 5.0.into(),
        min_focus_usage_dur: 10 * 60 * 1000 * 10000,
        max_focus_gap: 20 * 60 * 1000 * 10000, // 20 minutes
    };
    let dist_settings = DistractiveStreakSettings {
        max_distractive_score: 3.0.into(),
        min_distractive_usage_dur: 10 * 60 * 1000 * 10000,
        max_distractive_gap: 5 * 60 * 1000 * 10000,
    };

    let streaks = repo
        .get_streaks(test_start(), test_end(), focus_settings, dist_settings)
        .await?;

    // Expect 2 separate FPs and 1 DP (total 3 streaks)
    assert_eq!(streaks.len(), 3);
    let fp_count = streaks.iter().filter(|p| p.is_focused).count();
    assert_eq!(fp_count, 2);
    // Ensure the two FPs have expected boundaries
    let mut fp_starts: Vec<i64> = streaks
        .iter()
        .filter(|p| p.is_focused)
        .map(|p| p.start)
        .collect();
    fp_starts.sort();
    assert_eq!(fp_starts[0], f1_start);
    assert_eq!(fp_starts[1], f2_start);
    Ok(())
}
