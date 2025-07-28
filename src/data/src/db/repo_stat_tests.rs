use repo::*;
use util::future as tokio;

use super::tests::*;
use super::*;
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
            icon: None,
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
