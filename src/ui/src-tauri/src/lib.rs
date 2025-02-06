mod error;
mod repo;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            state::init_state,
            repo::get_apps,
            repo::get_tags,
            repo::get_app_durations,
            repo::get_app_durations_per_period,
            repo::copy_seed_db,
            repo::update_usages_end
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
