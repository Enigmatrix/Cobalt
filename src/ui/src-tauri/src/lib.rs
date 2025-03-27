//! Tauri build script

mod error;
mod repo;
mod state;
mod tracing;

/// Tauri run entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(not(debug_assertions))]
            {
                use tauri::Manager;
                app.handle()
                    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                        let _ = app
                            .get_webview_window("main")
                            .expect("no main window")
                            .set_focus();
                    }));
            }

            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            state::init_state,
            repo::get_apps,
            repo::get_tags,
            repo::get_alerts,
            repo::get_app_durations,
            repo::get_app_durations_per_period,
            repo::get_tag_durations_per_period,
            repo::copy_from_seed_db,
            repo::copy_from_install_db,
            repo::update_usages_end,
            repo::update_app,
            repo::update_tag,
            repo::update_tag_apps,
            repo::create_tag,
            repo::remove_tag,
            repo::create_alert,
            repo::update_alert,
            repo::remove_alert,
            repo::create_alert_event_ignore,
            repo::get_app_session_usages,
            repo::get_interaction_periods,
            repo::get_system_events,
            tracing::log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
