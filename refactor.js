import fs from 'fs';
import path from 'path';

const statePath = path.join(process.cwd(), 'src-tauri/src/state.rs');
let code = fs.readFileSync(statePath, 'utf-8');

// 1. Add AppState definition
code = code.replace(
  'pub struct AppConfig {\n    pub accounts: Vec<Account>,\n}',
  'pub struct AppConfig {\n    pub accounts: Vec<Account>,\n}\n\npub struct AppState(pub std::sync::Mutex<AppConfig>);'
);

// 2. Replace get_accounts
code = code.replace(
  'pub fn get_accounts(app: tauri::AppHandle) -> Result<Vec<Account>, String> {\n    let config = load_config(&app)?;\n    Ok(config.accounts)',
  'pub fn get_accounts(state: tauri::State<\'_, AppState>) -> Result<Vec<Account>, String> {\n    let config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;\n    Ok(config.accounts.clone())'
);

// 3. Helper to replace other commands
function replaceCommand(cmdName, argsBefore, replacementCode) {
  const regex = new RegExp(`pub fn ${cmdName}\\(app: tauri::AppHandle([\\s\\S]*?)\\{[\\s\\S]*?let mut config = load_config\\(&app\\)\\?;`, 'm');
  code = code.replace(regex, `pub fn ${cmdName}(app: tauri::AppHandle, state: tauri::State<'_, AppState>$1{
    let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;`);
}

replaceCommand('add_account', ', name: String, email: Option<String>, password: Option<String>, token: Option<String>, org_id: Option<String>, plan_tier: String\\) -> Result<Account, String> ');
replaceCommand('delete_account', ', id: String\\) -> Result<\\(\\), String> ');
replaceCommand('rename_account', ', id: String, new_name: String, email: Option<String>, password: Option<String>, token: Option<String>, org_id: Option<String>, plan_tier: String\\) -> Result<\\(\\), String> ');
replaceCommand('update_account_plan', ', id: String, plan: String\\) -> Result<\\(\\), String> ');
replaceCommand('bind_captured_token', ', id: String, token: String\\) -> Result<\\(\\), String> ');
replaceCommand('capture_credentials', ', id: String, email: Option<String>, password: Option<String>\\) -> Result<\\(\\), String> ');

// 4. Special commands where config is read but not modified (or modified carefully)
code = code.replace(
  `pub fn open_account_window(app: tauri::AppHandle, id: String, name: String) -> Result<(), String> {`,
  `pub fn open_account_window(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String, name: String) -> Result<(), String> {`
);
code = code.replace(
  `pub fn start_silent_login(app: tauri::AppHandle, id: String, name: String) -> Result<(), String> {`,
  `pub fn start_silent_login(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String, name: String) -> Result<(), String> {`
);
code = code.replace(
  `pub fn apply_account_to_default_ide(app: tauri::AppHandle, id: String) -> Result<(), String> {`,
  `pub fn apply_account_to_default_ide(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {`
);

// Fix load_config calls in the above 3 commands
code = code.replace(
  /let config = load_config\(&app\)\?;/g,
  `let config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;`
);
code = code.replace(
  /let mut config = load_config\(&app\)\?;/g,
  `let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;`
);

// 5. Replace save_config(&app, &config) with save_config(&app, &*config)
code = code.replace(/save_config\(&app, &config\)\?/g, 'save_config(&app, &*config)?');

// 6. Replace __TAURI_INTERNALS__.invoke with window.__TAURI__.core.invoke
code = code.replace(/window.__TAURI_INTERNALS__.invoke/g, 'window.__TAURI__.core.invoke');
code = code.replace(/window.__TAURI_INTERNALS__/g, 'window.__TAURI__');

fs.writeFileSync(statePath, code);
console.log('Refactoring state.rs completed.');

// 7. Refactor lib.rs
const libPath = path.join(process.cwd(), 'src-tauri/src/lib.rs');
let libCode = fs.readFileSync(libPath, 'utf-8');

const setupCode = `
        .setup(|app| {
            let config = state::load_config(app.handle()).unwrap_or_default();
            app.manage(state::AppState(std::sync::Mutex::new(config)));
            Ok(())
        })
        .invoke_handler`;

libCode = libCode.replace('.invoke_handler', setupCode);
fs.writeFileSync(libPath, libCode);
console.log('Refactoring lib.rs completed.');
