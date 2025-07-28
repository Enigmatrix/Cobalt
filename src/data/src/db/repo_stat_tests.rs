use repo::*;
use util::future as tokio;

use super::tests::*;
use super::*;
use crate::entities::AppIdentity;

const ONE_HOUR: i64 = 60 * 60 * 1000 * 10000; // 1 hour in ticks
const TEST_START: i64 = 1000 * ONE_HOUR; // Start time
const TEST_END: i64 = 2000 * ONE_HOUR; // End time

#[tokio::test]
async fn get_score_no_apps() -> Result<()> {
    let db = test_db().await?;
    let mut repo = Repository::new(db)?;

    let score = repo.get_score(TEST_START, TEST_END).await?;
    assert_eq!(score, 0.0);

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
            start: TEST_START + 100 * ONE_HOUR, // 100 hours after start
            end: TEST_START + 200 * ONE_HOUR,   // 200 hours after start (100 hours duration)
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: TEST_START + 300 * ONE_HOUR, // 300 hours after start
            end: TEST_START + 350 * ONE_HOUR,   // 350 hours after start (50 hours duration)
        },
    )
    .await?;

    let score = repo.get_score(TEST_START, TEST_END).await?;
    // Apps without tags should contribute 0 to the score
    assert_eq!(score, 0.0);

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
            score: 10, // High score for productivity
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
            score: 5, // Lower score for gaming
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
            start: TEST_START + 100 * ONE_HOUR, // 100 hours after start
            end: TEST_START + 200 * ONE_HOUR,   // 200 hours after start (100 hours duration)
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: TEST_START + 300 * ONE_HOUR, // 300 hours after start
            end: TEST_START + 350 * ONE_HOUR,   // 350 hours after start (50 hours duration)
        },
    )
    .await?;

    let score = repo.get_score(TEST_START, TEST_END).await?;
    // Expected: weighted average = (100 hours * 10 score + 50 hours * 5 score) / (100 + 50) hours = 8.333...
    let expected = 25.0 / 3.0;
    let epsilon = 1e-9;
    assert!(
        (score - expected).abs() < epsilon,
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
            score: 8,
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
            start: TEST_START + 50 * ONE_HOUR,
            end: TEST_START + 150 * ONE_HOUR, // 100 hours duration
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: TEST_START + 200 * ONE_HOUR,
            end: TEST_START + 250 * ONE_HOUR, // 50 hours duration
        },
    )
    .await?;

    let score = repo.get_score(TEST_START, TEST_END).await?;
    // Expected: weighted average = (100 hours * 8 score + 50 hours * 0 score) / (100 + 50) hours = 5.333...
    let expected = 16.0 / 3.0;
    let epsilon = 1e-9;
    assert!(
        (score - expected).abs() < epsilon,
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
            score: 10,
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
            start: TEST_START - 200 * ONE_HOUR,
            end: TEST_START - 100 * ONE_HOUR, // 100 hours duration, but outside range
        },
    )
    .await?;

    // Create usage outside the test range (after end)
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session.id.clone(),
            start: TEST_END + 100 * ONE_HOUR,
            end: TEST_END + 200 * ONE_HOUR, // 100 hours duration, but outside range
        },
    )
    .await?;

    let score = repo.get_score(TEST_START, TEST_END).await?;
    // No usage within the range, so score should be 0
    assert_eq!(score, 0.0);

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
            score: 6,
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
            start: TEST_START - 50 * ONE_HOUR, // Starts 50 hours before range
            end: TEST_START + 50 * ONE_HOUR,   // Ends 50 hours into range
        },
    )
    .await?;

    // Create usage that starts within the range but ends after it
    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session.id.clone(),
            start: TEST_END - 30 * ONE_HOUR, // Starts 30 hours before end
            end: TEST_END + 70 * ONE_HOUR,   // Ends 70 hours after range
        },
    )
    .await?;

    let score = repo.get_score(TEST_START, TEST_END).await?;
    // Expected: weighted average = (50 hours * 6 score + 30 hours * 6 score) / (50 + 30) hours = 6
    assert_eq!(score, 6.0);

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
            score: 7,
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
            start: TEST_START + 100 * ONE_HOUR,
            end: TEST_START + 150 * ONE_HOUR, // 50 hours
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(2),
            session_id: session2.id.clone(),
            start: TEST_START + 200 * ONE_HOUR,
            end: TEST_START + 250 * ONE_HOUR, // 50 hours
        },
    )
    .await?;

    arrange::usage(
        &mut repo.db,
        Usage {
            id: Ref::new(3),
            session_id: session3.id.clone(),
            start: TEST_START + 300 * ONE_HOUR,
            end: TEST_START + 350 * ONE_HOUR, // 50 hours
        },
    )
    .await?;

    let score = repo.get_score(TEST_START, TEST_END).await?;
    // Expected: weighted average = (50 + 50 + 50) hours * 7 score / (50 + 50 + 50) hours = 7
    assert_eq!(score, 7.0);

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
            score: 0, // Zero score
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
            start: TEST_START + 100 * ONE_HOUR,
            end: TEST_START + 200 * ONE_HOUR, // 100 hours duration
        },
    )
    .await?;

    let score = repo.get_score(TEST_START, TEST_END).await?;
    // Expected: 100 hours * 0 score = 0
    assert_eq!(score, 0.0);

    Ok(())
}
