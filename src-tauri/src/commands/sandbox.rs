use std::fs;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::models::account::AppState;
use crate::models::storage::save_config;

fn parse_default_credentials() -> Result<(Option<String>, Option<String>), String> {
    let appdata = std::env::var("APPDATA")
        .map_err(|e| format!("[CRITICAL] 无法获取 APPDATA 环境变量: {}", e))?;
    let creds_path = std::path::PathBuf::from(&appdata).join("devin").join("credentials.toml");
    
    let mut token = None;
    if creds_path.exists() {
        let content = fs::read_to_string(&creds_path)
            .map_err(|e| format!("[CRITICAL] 读取 credentials.toml 失败: {}", e))?;
        for line in content.lines() {
            let clean = line.split('#').next().unwrap_or("").trim();
            if clean.starts_with("windsurf_api_key") {
                if let Some(eq_idx) = clean.find('=') {
                    let val = clean[eq_idx + 1..].trim().trim_matches('"').trim_matches('\'').trim().to_string();
                    if !val.is_empty() {
                        token = Some(val);
                    }
                }
            }
        }
    }

    let config_path = std::path::PathBuf::from(&appdata).join("devin").join("config.json");
    let mut org_id = None;
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("[CRITICAL] 读取 config.json 失败: {}", e))?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(devin_obj) = json.get("devin") {
                if let Some(org_val) = devin_obj.get("org_id") {
                    if let Some(org_str) = org_val.as_str() {
                        org_id = Some(org_str.to_string());
                    }
                }
            }
        }
    }

    Ok((token, org_id))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub token: Option<String>,
    pub org_id: Option<String>,
}

#[tauri::command]
pub fn import_current_credentials() -> Result<ImportResult, String> {
    let (token, org_id) = parse_default_credentials()?;
    if token.is_none() {
        return Err("[CRITICAL] 当前全局默认 IDE 处于未登录状态，无法导入凭据。请先在 Devin 客户端内完成登录。".to_string());
    }
    Ok(ImportResult { token, org_id })
}

fn copy_dir_all(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn apply_account_to_default_ide(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("[CRITICAL] Mutex lock failed: {}", e))?;
    let mut target_account = None;
    for acc in &config.accounts {
        if acc.id == id {
            target_account = Some(acc.clone());
            break;
        }
    }
    let target_account = target_account.ok_or_else(|| format!("[CRITICAL] 找不到 ID 为 {} 的账号", id))?;

    let app_config_dir = app.path().app_config_dir()
        .map_err(|e| format!("[CRITICAL] 无法获取 App Config 目录: {}", e))?;
    
    // 独立的本地 IDE 沙箱配置目录
    let profile_dir = app_config_dir.join("ide_profiles").join(&id);
    
    if !profile_dir.exists() {
        fs::create_dir_all(&profile_dir)
            .map_err(|e| format!("[CRITICAL] 无法创建本地 IDE 沙箱目录: {}", e))?;
            
        // 自动继承默认的全局配置（如主题、个人偏好设置与快捷键等），防止“新环境白板”
        if let Ok(appdata) = std::env::var("APPDATA") {
            let default_user_dir = std::path::PathBuf::from(appdata).join("devin").join("User");
            if default_user_dir.exists() {
                let dest_user_dir = profile_dir.join("User");
                if let Err(e) = copy_dir_all(&default_user_dir, &dest_user_dir) {
                    eprintln!("[WARNING] 继承全局默认 IDE 核心配置失败: {}", e);
                }
            }
        }
    }

    // 处理凭据沙箱隔离与双向同步
    let devin_dir = profile_dir.join("devin");
    if !devin_dir.exists() {
        fs::create_dir_all(&devin_dir)
            .map_err(|e| format!("[CRITICAL] 无法创建隔离凭据配置目录: {}", e))?;
    }
    let creds_path = devin_dir.join("credentials.toml");

    // 1. 尝试从本地沙箱 of credentials.toml 中反向同步 Token
    let mut synced_token = None;
    if creds_path.exists() {
        if let Ok(content) = fs::read_to_string(&creds_path) {
            for line in content.lines() {
                let clean = line.split('#').next().unwrap_or("").trim();
                if clean.starts_with("windsurf_api_key") {
                    if let Some(eq_idx) = clean.find('=') {
                        let val = clean[eq_idx + 1..].trim().trim_matches('"').trim_matches('\'').trim().to_string();
                        if !val.is_empty() {
                            synced_token = Some(val);
                        }
                    }
                }
            }
        }
    }

    let mut config_modified = false;
    let mut current_token = target_account.token.clone();

    if let Some(ref st) = synced_token {
        if target_account.token.as_ref() != Some(st) {
            // 沙箱中有新的或不同的 Token，优先反向同步到切号器数据库
            for acc in &mut config.accounts {
                if acc.id == id {
                    acc.token = Some(st.clone());
                    config_modified = true;
                    break;
                }
            }
            current_token = Some(st.clone());
        }
    }

    if config_modified {
        save_config(&app, &*config)?;
        let _ = app.emit("account-plan-updated", ());
    }

    // 2. 确保将最新的凭据写入沙箱中，如果当前切号器/最新同步的 Token 不为空
    if let Some(ref token) = current_token {
        if !token.is_empty() {
            let creds_content = format!(
                "windsurf_api_key = \"{}\"\napi_server_url = \"https://server.codeium.com\"\ndevin_webapp_host = \"app.devin.ai\"\ndevin_api_url = \"https://api.devin.ai\"\n",
                token
            );
            fs::write(&creds_path, creds_content)
                .map_err(|e| format!("[CRITICAL] 写入本地凭据 credentials.toml 失败: {}", e))?;
        }
    }

    let ide_path = std::path::PathBuf::from(r"E:\devin\Devin\Devin.exe");
    if !ide_path.exists() {
        return Err("[CRITICAL] 找不到本地 Devin 客户端 (E:\\devin\\Devin\\Devin.exe)。请确保该软件已正确安装。".to_string());
    }

    // 目录隔离拉起，并强制配置密码保存在隔离沙箱中，防止读写系统全局凭证管理器
    // 同时，修改 APPDATA 环境变量，使 Devin 插件的凭证文件 credentials.toml 被隔离存储在 profile_dir 下
    std::process::Command::new(ide_path)
        .env("APPDATA", &profile_dir)
        .arg(format!("--user-data-dir={}", profile_dir.to_string_lossy()))
        .arg("--password-store=basic")
        .spawn()
        .map_err(|e| format!("[CRITICAL] 启动 Devin.exe 客户端失败: {}", e))?;

    Ok(())
}
