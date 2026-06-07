use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;
use uuid::Uuid;

use crate::models::account::{Account, AppState};
use crate::models::storage::save_config;

#[tauri::command]
pub fn get_accounts(state: tauri::State<'_, AppState>) -> Result<Vec<Account>, String> {
    let config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    Ok(config.accounts.clone())
}

#[tauri::command]
pub fn add_account(app: tauri::AppHandle, state: tauri::State<'_, AppState>, name: String, email: Option<String>, password: Option<String>, token: Option<String>, org_id: Option<String>, plan_tier: String) -> Result<Account, String> {
    if name.trim().is_empty() {
        return Err("[CRITICAL] 账号名称不能为空".to_string());
    }
    let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    
    let id = Uuid::new_v4().to_string();
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("[CRITICAL] 无法获取系统时间: {}", e))?
        .as_secs();
        
    let clean_email = email.filter(|e| !e.trim().is_empty()).map(|e| e.trim().to_string());
    let clean_password = password.filter(|p| !p.trim().is_empty()).map(|p| p.trim().to_string());
    let clean_token = token.filter(|t| !t.trim().is_empty()).map(|t| t.trim().to_string());
    let clean_org = org_id.filter(|o| !o.trim().is_empty()).map(|o| o.trim().to_string());
    let upper_plan = plan_tier.trim().to_uppercase();

    let new_account = Account {
        id: id.clone(),
        name: name.trim().to_string(),
        email: clean_email,
        password: clean_password,
        token: clean_token,
        org_id: clean_org,
        plan_tier: upper_plan,
        created_at,
    };
    
    config.accounts.push(new_account.clone());
    save_config(&app, &*config)?;
    
    Ok(new_account)
}

#[tauri::command]
pub fn delete_account(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    
    let original_len = config.accounts.len();
    config.accounts.retain(|acc| acc.id != id);
    
    if config.accounts.len() == original_len {
        return Err(format!("[CRITICAL] 未找到 ID 为 {} 的账号", id));
    }
    
    save_config(&app, &*config)?;
    
    let path = app.path().app_config_dir()
        .map_err(|e| format!("[CRITICAL] 无法获取系统 App Config 目录: {}", e))?;
    let data_dir = path.join("profiles").join(&id);
    if data_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&data_dir) {
            eprintln!("[WARNING] 清理账号网页数据目录失败 {:?}: {}", data_dir, e);
        }
    }

    let ide_data_dir = path.join("ide_profiles").join(&id);
    if ide_data_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&ide_data_dir) {
            eprintln!("[WARNING] 清理账号本地 IDE 数据目录失败 {:?}: {}", ide_data_dir, e);
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn rename_account(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String, new_name: String, email: Option<String>, password: Option<String>, token: Option<String>, org_id: Option<String>, plan_tier: String) -> Result<(), String> {
    if new_name.trim().is_empty() {
        return Err("[CRITICAL] 账号名称不能为空".to_string());
    }
    let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    
    let mut found = false;
    let clean_email = email.filter(|e| !e.trim().is_empty()).map(|e| e.trim().to_string());
    let clean_password = password.filter(|p| !p.trim().is_empty()).map(|p| p.trim().to_string());
    let clean_token = token.filter(|t| !t.trim().is_empty()).map(|t| t.trim().to_string());
    let clean_org = org_id.filter(|o| !o.trim().is_empty()).map(|o| o.trim().to_string());
    let upper_plan = plan_tier.trim().to_uppercase();

    for acc in &mut config.accounts {
        if acc.id == id {
            acc.name = new_name.trim().to_string();
            acc.email = clean_email.clone();
            acc.password = clean_password.clone();
            acc.token = clean_token.clone();
            acc.org_id = clean_org.clone();
            acc.plan_tier = upper_plan.clone();
            found = true;
            break;
        }
    }
    
    if !found {
        return Err(format!("[CRITICAL] 未找到 ID 为 {} 的账号", id));
    }
    
    save_config(&app, &*config)?;
    Ok(())
}

#[tauri::command]
pub fn update_account_plan(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String, plan: String) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    let mut found = false;
    let upper_plan = plan.trim().to_uppercase();

    for acc in &mut config.accounts {
        if acc.id == id {
            if acc.plan_tier == "AUTO" || acc.plan_tier != upper_plan {
                acc.plan_tier = upper_plan.clone();
                found = true;
            }
            break;
        }
    }

    if found {
        save_config(&app, &*config)?;
        let _ = app.emit("account-plan-updated", ());
    }

    Ok(())
}

#[tauri::command]
pub fn update_account_quota(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String, billing_error: Option<String>, available_credits: Option<f64>) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    let mut found = false;

    for acc in &mut config.accounts {
        if acc.id == id {
            if acc.billing_error != billing_error || acc.available_credits != available_credits {
                acc.billing_error = billing_error.clone();
                acc.available_credits = available_credits;
                found = true;
            }
            break;
        }
    }

    if found {
        save_config(&app, &*config)?;
        let _ = app.emit("account-quota-updated", ());
    }

    Ok(())
}
