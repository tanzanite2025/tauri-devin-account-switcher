mod models;
mod commands;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config = models::storage::load_config(app.handle()).unwrap_or_default();
            use tauri::Manager;
            app.manage(models::account::AppState(std::sync::Mutex::new(config)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::account::get_accounts,
            commands::account::add_account,
            commands::account::delete_account,
            commands::account::rename_account,
            commands::account::update_account_plan,
            commands::account::update_account_quota,
            commands::sandbox::import_current_credentials,
            commands::sandbox::apply_account_to_default_ide,
            commands::webview::open_account_window,
            commands::webview::capture_credentials,
            commands::webview::bind_captured_token,
            commands::webview::start_silent_login
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
