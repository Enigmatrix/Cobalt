//! Merge apps in database (user can select)

use std::collections::HashSet;
use std::fmt::{self, Display};
use std::path::PathBuf;

use clap::Parser;
use data::db::repo::Repository;
use data::db::{Database, DatabasePool, infused};
use data::entities::{App, Ref};
use inquire::MultiSelect as InquireMultiSelect;
use platform::objects::Timestamp;
use util::error::{ContextCompat, Result};
use util::tracing::debug;
use util::{Target, config, future as tokio};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Database path
    #[arg()]
    db_path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    util::set_target(Target::Tool {
        name: "merge_apps".to_string(),
    });
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let args = Args::parse();
    let db_path = args
        .db_path
        .map(PathBuf::from)
        .context("db path not provided")
        .or_else(|_| config.connection_string())?;
    debug!("db path: {}", db_path.display());

    let db_pool = DatabasePool::new(db_path).await?;
    let ts = Timestamp::now();

    let db = db_pool.get_db().await?;
    let mut repo = Repository::new(db)?;
    let apps_map = repo.get_apps(ts).await?;
    let apps: Vec<infused::App> = apps_map.into_values().collect();

    let selected_apps = select_apps(apps.clone())?;

    if selected_apps.is_empty() {
        println!("No apps selected for merging. Exiting.");
        return Ok(());
    }

    // Select the app with the lowest id as the target
    let target_app = selected_apps
        .iter()
        .min_by_key(|app| app.0)
        .unwrap()
        .clone();

    merge_apps(&mut db_pool.get_db().await?, selected_apps, target_app).await?;

    Ok(())
}

fn select_apps(apps: Vec<infused::App>) -> Result<HashSet<Ref<App>>> {
    if apps.is_empty() {
        println!("No apps found in the database.");
        return Ok(HashSet::new());
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
    let selected_apps: HashSet<Ref<App>> = selected_indices
        .into_iter()
        .map(|app| app.second.inner.id)
        .collect();

    println!("Selected {} apps for merging.", selected_apps.len());
    return Ok(selected_apps);
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
    mut apps: HashSet<Ref<App>>,
    target: Ref<App>,
) -> Result<()> {
    let mut tx = db.transaction().await?;

    // Remove the target app from the set
    apps.remove(&target);

    let app_ids: Vec<i64> = apps.iter().map(|app| **app).collect();

    if app_ids.is_empty() {
        tx.commit().await?;
        return Ok(());
    }

    // Build the placeholders for the IN clause
    let placeholders = std::iter::repeat("?")
        .take(app_ids.len())
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

    // Commit the transaction
    tx.commit().await?;

    Ok(())
}
