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
            Ok(())
        })
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![state::init_state, repo::get_apps,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
