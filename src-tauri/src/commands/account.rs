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

#[tauri::command]
pub async fn refresh_all_quotas(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let accounts_to_refresh = {
        let config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
        config.accounts.clone()
    };

    let client = reqwest::Client::new();
    let mut modified = false;

    for acc in accounts_to_refresh {
        if let Some(token) = &acc.token {
            if token.is_empty() { continue; }

            let session_req = client.get("https://app.devin.ai/api/auth/session")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await;

            if let Ok(res) = session_req {
                if res.status().is_success() {
                    if let Ok(json) = res.json::<serde_json::Value>().await {
                        let mut new_plan = None;
                        if let Some(p) = json.get("plan").and_then(|v| v.as_str()) {
                            new_plan = Some(p.to_uppercase());
                        } else if let Some(p) = json.pointer("/user/plan").and_then(|v| v.as_str()) {
                            new_plan = Some(p.to_uppercase());
                        } else if let Some(p) = json.get("tier").and_then(|v| v.as_str()) {
                            new_plan = Some(p.to_uppercase());
                        } else if let Some(p) = json.pointer("/user/tier").and_then(|v| v.as_str()) {
                            new_plan = Some(p.to_uppercase());
                        }

                        let mut org_id = None;
                        if let Some(o) = json.pointer("/user/org_id").and_then(|v| v.as_str()) {
                            org_id = Some(o.to_string());
                        } else if let Some(orgs) = json.pointer("/user/organizations").and_then(|v| v.as_array()) {
                            if !orgs.is_empty() {
                                if let Some(id) = orgs[0].get("id").and_then(|v| v.as_str()) {
                                    org_id = Some(id.to_string());
                                }
                            }
                        } else if let Some(orgs) = json.get("organizations").and_then(|v| v.as_array()) {
                            if !orgs.is_empty() {
                                if let Some(id) = orgs[0].get("id").and_then(|v| v.as_str()) {
                                    org_id = Some(id.to_string());
                                }
                            }
                        }

                        let mut new_billing_error = None;
                        let mut new_credits = None;

                        if let Some(oid) = org_id {
                            let billing_req = client.get(&format!("https://app.devin.ai/api/{}/billing/status", oid))
                                .header("Authorization", format!("Bearer {}", token))
                                .send()
                                .await;
                            
                            if let Ok(bres) = billing_req {
                                if bres.status().is_success() {
                                    if let Ok(bjson) = bres.json::<serde_json::Value>().await {
                                        if let Some(be) = bjson.get("billing_error").and_then(|v| v.as_str()) {
                                            new_billing_error = Some(be.to_string());
                                        }
                                        if let Some(ac) = bjson.get("available_credits").and_then(|v| v.as_f64()) {
                                            new_credits = Some(ac);
                                        }
                                    }
                                }
                            }
                        }

                        let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
                        for a in &mut config.accounts {
                            if a.id == acc.id {
                                if let Some(np) = new_plan {
                                    if a.plan_tier == "AUTO" || a.plan_tier != np {
                                        a.plan_tier = np;
                                        modified = true;
                                    }
                                }
                                
                                if a.billing_error != new_billing_error || a.available_credits != new_credits {
                                    a.billing_error = new_billing_error;
                                    a.available_credits = new_credits;
                                    modified = true;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    if modified {
        let config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
        save_config(&app, &*config)?;
        let _ = app.emit("account-quota-updated", ());
        let _ = app.emit("account-plan-updated", ());
    }

    Ok(())
}
