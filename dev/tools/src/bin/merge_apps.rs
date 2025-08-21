//! Merge apps in database (user can select)

use std::fmt::{self, Display};
use std::path::PathBuf;
use std::result::Result as StdResult;

use clap::Parser;
use data::db::repo::Repository;
use data::db::{Database, DatabasePool, infused};
use data::entities::{App, Ref};
use inquire::MultiSelect as InquireMultiSelect;
use platform::objects::Timestamp;
use tools::db::Source;
use util::error::{Context, ContextCompat, Result};
use util::tracing::{debug, error, info};
use util::{Target, config, future as tokio};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Database path
    #[arg()]
    db: Option<Source>,

    /// Target app ID to merge into
    #[arg(long)]
    target: Option<i64>,

    /// App IDs to merge (comma-separated)
    #[arg(long)]
    apps: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    util::set_target(Target::Tool {
        name: "merge_apps".to_string(),
    });
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;
    let db_path = args
        .db
        .as_ref()
        .unwrap_or(&Source::Current)
        .dir_path()?
        .join("main.db");
    debug!("db path: {}", db_path.display());

    let db_pool = DatabasePool::new(&db_path).await?;
    let ts = Timestamp::now();

    let db = db_pool.get_db().await?;
    let mut repo = Repository::new(db)?;
    let apps_map = repo.get_apps(ts).await?;
    let apps: Vec<infused::App> = apps_map.into_values().collect();

    let selected_apps = if let Some(apps_str) = args.apps {
        let app_ids = apps_str
            .split(',')
            .map(|s| s.trim().parse::<i64>())
            .collect::<StdResult<Vec<i64>, _>>()
            .context("Failed to parse app IDs")?;

        // Find apps by ID
        let selected_apps: Vec<infused::App> = apps
            .iter()
            .filter(|app| app_ids.contains(&app.inner.id.0))
            .cloned()
            .collect();

        if selected_apps.is_empty() {
            error!("No apps found with the provided IDs: {:?}", app_ids);
            return Ok(());
        }

        selected_apps
    } else {
        // Interactive mode (original behavior)
        let selected_apps = select_apps(apps.clone())?;

        if selected_apps.is_empty() {
            error!("No apps selected for merging. Exiting.");
            return Ok(());
        }

        selected_apps
    };

    let target_app = if let Some(target_id) = args.target {
        // Find target app

        selected_apps
            .iter()
            .find(|app| app.inner.id.0 == target_id)
            .map(|app| app.inner.id.clone())
            .context("Target app not found in the selected apps")?
    } else {
        // Select the app with the lowest id as the target

        selected_apps
            .iter()
            .map(|app| app.inner.id.clone())
            .min_by_key(|id| id.0)
            .unwrap()
            .clone()
    };

    merge_apps(
        &mut db_pool.get_db().await?,
        db_path,
        selected_apps.clone(),
        target_app.clone(),
    )
    .await?;

    let selected_app_ids = selected_apps
        .iter()
        .map(|app| app.inner.id.0.to_string())
        .collect::<Vec<_>>()
        .join(",");
    debug!(
        "cargo run --bin merge_apps -- --apps {} --target {}{}",
        selected_app_ids,
        target_app.0,
        if let Some(db) = args.db {
            db.to_string()
        } else {
            "".to_string()
        }
    );

    Ok(())
}

fn select_apps(apps: Vec<infused::App>) -> Result<Vec<infused::App>> {
    if apps.is_empty() {
        error!("No apps found in the database.");
        return Ok(Vec::new());
    }

    // Create display strings for current apps
    let app_display_strings: Vec<_> = apps
        .iter()
        .map(|app| {
            let identity_str = match &app.inner.identity {
                data::entities::AppIdentity::Uwp { aumid } => format!("UWP: {}", aumid.trim()),
                data::entities::AppIdentity::Win32 { path } => {
                    format!("Win32: {}", path.trim())
                }
                data::entities::AppIdentity::Website { base_url } => {
                    format!("Website: {}", base_url.trim())
                }
            };

            DisplayFirst {
                first: format!(
                    "{} - {} - {}",
                    app.inner.name.trim(),
                    app.inner.company.trim(),
                    identity_str,
                ),
                second: app.clone(),
            }
        })
        .collect();

    let selected_indices =
        InquireMultiSelect::new("Select apps to merge:", app_display_strings).prompt()?;

    // Convert selected indices to HashSet of Ref<App>
    let selected_apps: Vec<_> = selected_indices.into_iter().map(|app| app.second).collect();

    info!("Selected {} apps for merging.", selected_apps.len());
    Ok(selected_apps)
}

struct DisplayFirst<T1, T2> {
    pub first: T1,
    pub second: T2,
}

impl<T1: Display, T2> Display for DisplayFirst<T1, T2> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.first.fmt(f)
    }
}

async fn merge_apps(
    db: &mut Database,
    db_path: PathBuf,
    apps: Vec<infused::App>,
    target: Ref<App>,
) -> Result<()> {
    let mut tx = db.transaction().await?;

    // Remove the target app from the set
    let apps: Vec<_> = apps
        .into_iter()
        .filter(|app| app.inner.id != target)
        .collect();

    let app_ids: Vec<i64> = apps.iter().map(|app| app.inner.id.0).collect();

    if app_ids.is_empty() {
        tx.commit().await?;
        return Ok(());
    }

    // Build the placeholders for the IN clause
    let placeholders = std::iter::repeat_n("?", app_ids.len())
        .collect::<Vec<_>>()
        .join(",");

    // Update all sessions' app_id where it's equal to any app in {apps} with {target}
    let update_sessions_sql =
        format!("UPDATE sessions SET app_id = ? WHERE app_id IN ({placeholders})");

    let update_sessions_query = sqlx::query(&update_sessions_sql).bind(*target);
    let update_sessions_query = app_ids
        .iter()
        .fold(update_sessions_query, |q, id| q.bind(id));
    update_sessions_query.execute(&mut *tx).await?;

    // Update all alerts' app_id where it's equal to any app in {apps} with {target}
    let update_alerts_sql =
        format!("UPDATE alerts SET app_id = ? WHERE app_id IN ({placeholders})");
    let update_alerts_query = sqlx::query(&update_alerts_sql).bind(*target);
    let update_alerts_query = app_ids.iter().fold(update_alerts_query, |q, id| q.bind(id));
    update_alerts_query.execute(&mut *tx).await?;

    // Delete all apps in {apps} except {target}
    let delete_apps_sql = format!("DELETE FROM apps WHERE id IN ({placeholders})");
    let delete_apps_query = sqlx::query(&delete_apps_sql);
    let delete_apps_query = app_ids.iter().fold(delete_apps_query, |q, id| q.bind(id));
    delete_apps_query.execute(&mut *tx).await?;

    // Delete icons of the merged {apps} except {target}
    for app in apps {
        let Some(icon) = app.inner.icon else { continue };
        let path = db_path.join("..").join("icons").join(icon);
        std::fs::remove_file(&path).with_context(|| format!("remove icon {}", path.display()))?;
    }

    // Commit the transaction
    tx.commit().await?;

    Ok(())
}
