use std::fs;
use tauri::{Emitter, Manager};

use crate::models::account::AppState;
use crate::models::storage::save_config;
use crate::utils::scripts::get_injection_script;

#[tauri::command]
pub fn open_account_window(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String, name: String) -> Result<(), String> {
    let label = format!("devin-profile-{}", id);
    
    if let Some(existing_window) = app.get_webview_window(&label) {
        existing_window.set_focus()
            .map_err(|e| format!("[CRITICAL] 无法激活已有窗口: {}", e))?;
        return Ok(());
    }
    
    let config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    let mut email_val = String::new();
    let mut password_val = String::new();
    
    if let Some(acc) = config.accounts.iter().find(|a| a.id == id) {
        if let Some(ref e) = acc.email {
            email_val = e.clone();
        }
        if let Some(ref p) = acc.password {
            password_val = p.clone();
        }
    }

    let config_dir = app.path().app_config_dir()
        .map_err(|e| format!("[CRITICAL] 无法获取 App Config 目录: {}", e))?;
    let data_dir = config_dir.join("profiles").join(&id);
    
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("[CRITICAL] 无法创建账号网页数据目录 {:?}: {}", data_dir, e))?;
    }
    
    let url = tauri::WebviewUrl::External("https://devin.ai/".parse().unwrap());
    
    let builder = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        url
    )
    .title(format!("Devin - {}", name))
    .data_directory(data_dir)
    .inner_size(1200.0, 800.0);

    let script = get_injection_script(&id, &email_val, &password_val);
    
    let builder = builder.initialization_script(&script);

    let _window = builder.build()
        .map_err(|e| format!("[CRITICAL] 无法创建独立的 Devin 账号窗口: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn bind_captured_token(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String, token: String) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    let mut found = false;
    let clean_token = token.trim().to_string();

    for acc in &mut config.accounts {
        if acc.id == id {
            if acc.token.as_ref() != Some(&clean_token) {
                acc.token = Some(clean_token.clone());
                found = true;
            }
            break;
        }
    }

    if found {
        save_config(&app, &*config)?;
        let _ = app.emit("account-plan-updated", ());
        let _ = app.emit("silent-login-success", id.clone());
    }

    // 尝试关闭后台静默登录的隐藏窗口
    let silent_label = format!("silent-login-{}", id);
    if let Some(w) = app.get_webview_window(&silent_label) {
        let _ = w.close();
    }

    Ok(())
}

#[tauri::command]
pub fn start_silent_login(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String, name: String) -> Result<(), String> {
    let label = format!("silent-login-{}", id);
    
    if let Some(existing_window) = app.get_webview_window(&label) {
        existing_window.set_focus()
            .map_err(|e| format!("[CRITICAL] 无法激活已有静默窗口: {}", e))?;
        return Ok(());
    }
    
    let config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    let mut email_val = String::new();
    let mut password_val = String::new();
    
    if let Some(acc) = config.accounts.iter().find(|a| a.id == id) {
        if let Some(ref e) = acc.email {
            email_val = e.clone();
        }
        if let Some(ref p) = acc.password {
            password_val = p.clone();
        }
    }

    if email_val.is_empty() || password_val.is_empty() {
        return Err("[CRITICAL] 静默登录需要录入邮箱和密码。请先在弹窗中录入该账号的邮箱与密码。".to_string());
    }

    let config_dir = app.path().app_config_dir()
        .map_err(|e| format!("[CRITICAL] 无法获取 App Config 目录: {}", e))?;
    let data_dir = config_dir.join("profiles").join(&id);
    
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("[CRITICAL] 无法创建账号网页数据目录 {:?}: {}", data_dir, e))?;
    }
    
    let url = tauri::WebviewUrl::External("https://devin.ai/".parse().unwrap());
    
    let builder = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        url
    )
    .title(format!("Devin Auto Login - {}", name))
    .data_directory(data_dir)
    .visible(false) // 隐藏拉起！
    .inner_size(1200.0, 800.0);

    let script = get_injection_script(&id, &email_val, &password_val);
    
    let builder = builder.initialization_script(&script);

    let _window = builder.build()
        .map_err(|e| format!("[CRITICAL] 无法创建独立的 Devin 账号窗口: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn capture_credentials(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String, email: Option<String>, password: Option<String>) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    let mut found = false;

    let clean_email = email.filter(|e| !e.trim().is_empty()).map(|e| e.trim().to_string());
    let clean_password = password.filter(|p| !p.trim().is_empty()).map(|p| p.trim().to_string());

    for acc in &mut config.accounts {
        if acc.id == id {
            if clean_email.is_some() {
                acc.email = clean_email.clone();
                found = true;
            }
            if clean_password.is_some() {
                acc.password = clean_password.clone();
                found = true;
            }
            break;
        }
    }

    if found {
        save_config(&app, &*config)?;
    }

    Ok(())
}
