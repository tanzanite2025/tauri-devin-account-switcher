use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use super::account::AppConfig;

pub fn get_config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let path = app.path().app_config_dir()
        .map_err(|e| format!("[CRITICAL] 无法获取系统 App Config 目录: {}", e))?;
    if !path.exists() {
        fs::create_dir_all(&path)
            .map_err(|e| format!("[CRITICAL] 无法创建 App Config 目录 {:?}: {}", path, e))?;
    }
    Ok(path.join("accounts.json"))
}

pub fn load_config(app: &tauri::AppHandle) -> Result<AppConfig, String> {
    let config_path = get_config_path(app)?;
    let backup_path = config_path.with_extension("json.bak");

    if !config_path.exists() {
        if backup_path.exists() {
            let _ = fs::copy(&backup_path, &config_path);
        } else {
            let default_config = AppConfig::default();
            save_config(app, &default_config)?;
            return Ok(default_config);
        }
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("[CRITICAL] 读取账户配置文件失败 {:?}: {}", config_path, e))?;
    
    match serde_json::from_str(&content) {
        Ok(config) => Ok(config),
        Err(e) => {
            eprintln!("[CRITICAL] 账户配置文件解析失败: {}。尝试从备份恢复...", e);
            if backup_path.exists() {
                let backup_content = fs::read_to_string(&backup_path)
                    .map_err(|err| format!("[CRITICAL] 读取备份文件失败: {}", err))?;
                match serde_json::from_str(&backup_content) {
                    Ok(config) => {
                        let _ = fs::copy(&backup_path, &config_path);
                        Ok(config)
                    },
                    Err(be) => Err(format!("[CRITICAL] 配置文件与备份文件均损坏。配置错误: {}, 备份错误: {}", e, be))
                }
            } else {
                Err(format!("[CRITICAL] 账户配置文件解析失败且无备份，数据可能损坏: {}", e))
            }
        }
    }
}

pub fn save_config(app: &tauri::AppHandle, config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path(app)?;
    let backup_path = config_path.with_extension("json.bak");
    let temp_path = config_path.with_extension("json.tmp");

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("[CRITICAL] 序列化账户配置失败: {}", e))?;
    
    fs::write(&temp_path, content)
        .map_err(|e| format!("[CRITICAL] 写入账户临时配置文件失败 {:?}: {}", temp_path, e))?;
        
    if config_path.exists() {
        let _ = fs::copy(&config_path, &backup_path);
    }
    
    fs::rename(&temp_path, &config_path)
        .map_err(|e| format!("[CRITICAL] 原子替换账户配置文件失败 {:?} -> {:?}: {}", temp_path, config_path, e))?;
        
    Ok(())
}
