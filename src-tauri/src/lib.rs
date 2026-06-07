mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            state::get_accounts,
            state::add_account,
            state::delete_account,
            state::rename_account,
            state::open_account_window,
            state::update_account_plan,
            state::import_current_credentials,
            state::apply_account_to_default_ide,
            state::capture_credentials,
            state::bind_captured_token,
            state::start_silent_login
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
