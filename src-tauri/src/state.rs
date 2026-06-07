use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use tauri::{Manager, Emitter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default = "default_plan_tier")]
    pub plan_tier: String, // "AUTO", "FREE", "PRO", "MAX", "TEAMS", "ENTERPRISE"
    pub created_at: u64,
}

fn default_plan_tier() -> String {
    "AUTO".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub accounts: Vec<Account>,
}

fn get_config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
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
    if !config_path.exists() {
        let default_config = AppConfig::default();
        save_config(app, &default_config)?;
        return Ok(default_config);
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("[CRITICAL] 读取账户配置文件失败 {:?}: {}", config_path, e))?;
    
    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("[CRITICAL] 账户配置文件解析失败，数据可能损坏: {}", e))?;
        
    Ok(config)
}

pub fn save_config(app: &tauri::AppHandle, config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path(app)?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("[CRITICAL] 序列化账户配置失败: {}", e))?;
    
    fs::write(&config_path, content)
        .map_err(|e| format!("[CRITICAL] 写入账户配置文件失败 {:?}: {}", config_path, e))?;
        
    Ok(())
}

fn parse_default_credentials() -> Result<(Option<String>, Option<String>), String> {
    let appdata = std::env::var("APPDATA")
        .map_err(|e| format!("[CRITICAL] 无法获取 APPDATA 环境变量: {}", e))?;
    let creds_path = std::path::PathBuf::from(&appdata).join("devin").join("credentials.toml");
    
    let mut token = None;
    if creds_path.exists() {
        let content = fs::read_to_string(&creds_path)
            .map_err(|e| format!("[CRITICAL] 读取 credentials.toml 失败: {}", e))?;
        for line in content.lines() {
            let clean = line.trim();
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

#[tauri::command]
pub fn get_accounts(app: tauri::AppHandle) -> Result<Vec<Account>, String> {
    let config = load_config(&app)?;
    Ok(config.accounts)
}

#[tauri::command]
pub fn add_account(app: tauri::AppHandle, name: String, email: Option<String>, password: Option<String>, token: Option<String>, org_id: Option<String>, plan_tier: String) -> Result<Account, String> {
    if name.trim().is_empty() {
        return Err("[CRITICAL] 账号名称不能为空".to_string());
    }
    let mut config = load_config(&app)?;
    
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
        id,
        name: name.trim().to_string(),
        email: clean_email,
        password: clean_password,
        token: clean_token,
        org_id: clean_org,
        plan_tier: upper_plan,
        created_at,
    };
    
    config.accounts.push(new_account.clone());
    save_config(&app, &config)?;
    
    Ok(new_account)
}

#[tauri::command]
pub fn delete_account(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut config = load_config(&app)?;
    
    let original_len = config.accounts.len();
    config.accounts.retain(|acc| acc.id != id);
    
    if config.accounts.len() == original_len {
        return Err(format!("[CRITICAL] 未找到 ID 为 {} 的账号", id));
    }
    
    save_config(&app, &config)?;
    
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
pub fn rename_account(app: tauri::AppHandle, id: String, new_name: String, email: Option<String>, password: Option<String>, token: Option<String>, org_id: Option<String>, plan_tier: String) -> Result<(), String> {
    if new_name.trim().is_empty() {
        return Err("[CRITICAL] 账号名称不能为空".to_string());
    }
    let mut config = load_config(&app)?;
    
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
    
    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn update_account_plan(app: tauri::AppHandle, id: String, plan: String) -> Result<(), String> {
    let mut config = load_config(&app)?;
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
        save_config(&app, &config)?;
        let _ = app.emit("account-plan-updated", ());
    }

    Ok(())
}

#[tauri::command]
pub fn open_account_window(app: tauri::AppHandle, id: String, name: String) -> Result<(), String> {
    let label = format!("devin-profile-{}", id);
    
    if let Some(existing_window) = app.get_webview_window(&label) {
        existing_window.set_focus()
            .map_err(|e| format!("[CRITICAL] 无法激活已有窗口: {}", e))?;
        return Ok(());
    }
    
    let config = load_config(&app)?;
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

    let script = format!(
        r#"
        (function() {{
            const accountId = "{}";
            const email = "{}";
            const password = "{}";
            
            // 1. 代理拦截 window.fetch 获取 JWT Token
            const orgFetch = window.fetch;
            window.fetch = async function(...args) {{
                const url = args[0];
                const options = args[1] || {{}};
                
                if (options.headers) {{
                    let authHeader = "";
                    if (options.headers instanceof Headers) {{
                        authHeader = options.headers.get("Authorization") || "";
                    }} else if (typeof options.headers === "object") {{
                        authHeader = options.headers["Authorization"] || options.headers["authorization"] || "";
                    }}
                    if (authHeader && authHeader.includes("devin-session-token$")) {{
                        const tokenVal = authHeader.replace("Bearer ", "").trim();
                        if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {{
                            window.__TAURI_INTERNALS__.invoke("bind_captured_token", {{ id: accountId, token: tokenVal }})
                                .catch(err => console.error(err));
                        }}
                    }}
                }}
                
                const res = await orgFetch(...args);
                
                try {{
                    const clone = res.clone();
                    clone.json().then(data => {{
                        let foundToken = null;
                        
                        function search(obj) {{
                            if (!obj || foundToken) return;
                            if (typeof obj === "string") {{
                                if (obj.includes("devin-session-token$")) {{
                                    foundToken = obj;
                                }} else if (obj.startsWith("ey") && obj.length > 50) {{
                                    foundToken = "devin-session-token$" + obj;
                                }}
                                return;
                            }}
                            if (typeof obj === "object") {{
                                for (const k in obj) {{
                                    if (k === "token" || k === "accessToken" || k === "sessionToken" || k === "jwt") {{
                                        const val = obj[k];
                                        if (typeof val === "string") {{
                                            if (val.includes("devin-session-token$")) {{
                                                foundToken = val;
                                            }} else if (val.startsWith("ey") && val.length > 50) {{
                                                foundToken = "devin-session-token$" + val;
                                            }}
                                        }}
                                    }}
                                    search(obj[k]);
                                }}
                            }}
                        }}
                        
                        search(data);
                        if (foundToken) {{
                            if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {{
                                window.__TAURI_INTERNALS__.invoke("bind_captured_token", {{ id: accountId, token: foundToken }})
                                    .catch(err => console.error(err));
                            }}
                        }}
                    }}).catch((err) => {{ console.error("[CRITICAL] Failed to parse JSON or invoke bind_captured_token", err); }});
                }} catch(e) {{ console.error("[CRITICAL] Intercept fetch failed", e); }}
                
                return res;
            }};

            // 2. 自动填单与登录提交
            let submitted = false;
            function fillAndSubmit() {{
                if (submitted) return false;
                
                let emailFilled = false;
                let passFilled = false;
                
                const emailFields = document.querySelectorAll('input[type="email"], input[name="email"], input[name="username"], input[id*="email"], input[id*="username"]');
                const passFields = document.querySelectorAll('input[type="password"], input[name="password"], input[id*="password"]');
                
                if (emailFields.length > 0 && email) {{
                    for (const f of emailFields) {{
                        if (f.value !== email) {{
                            f.value = email;
                            f.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            f.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        }}
                    }}
                    emailFilled = true;
                }} else if (!email) {{
                    emailFilled = true;
                }}
                
                if (passFields.length > 0 && password) {{
                    for (const f of passFields) {{
                        if (f.value !== password) {{
                            f.value = password;
                            f.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            f.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        }}
                    }}
                    passFilled = true;
                }} else if (!password) {{
                    passFilled = true;
                }}
                
                if (emailFilled && passFilled) {{
                    const submitBtn = document.querySelector('button[type="submit"], button.login-btn, input[type="submit"], button[class*="login"], button[id*="login"]');
                    if (submitBtn) {{
                        submitted = true;
                        submitBtn.click();
                        clearInterval(timer);
                        return true;
                    }}
                }}
                return false;
            }}
            
            const timer = setInterval(fillAndSubmit, 500);
            setTimeout(() => clearInterval(timer), 20000);

            // 3. 保持 Plan 自动刷新
            async function checkPlan() {{
                try {{
                    const res = await fetch('/api/auth/session');
                    if (res.ok) {{
                        const data = await res.json();
                        let plan = null;
                        if (data) {{
                            if (data.plan) plan = data.plan;
                            else if (data.user && data.user.plan) plan = data.user.plan;
                            else if (data.tier) plan = data.tier;
                            else if (data.user && data.user.tier) plan = data.user.tier;
                        }}
                        
                        if (plan) {{
                            const upperPlan = plan.toUpperCase();
                            if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {{
                                await window.__TAURI_INTERNALS__.invoke("update_account_plan", {{ id: accountId, plan: upperPlan }});
                                return true;
                            }}
                        }}
                    }}
                }} catch (e) {{ console.error("[CRITICAL] fetch session plan failed", e); }}
                return false;
            }}

            let checkCount = 0;
            const planTimer = setInterval(async () => {{
                checkCount++;
                const success = await checkPlan();
                if (success || checkCount > 10) {{
                    clearInterval(planTimer);
                }}
            }}, 5000);
        }})();
        "#,
        id.replace('"', "\\\""),
        email_val.replace('"', "\\\""),
        password_val.replace('"', "\\\"")
    );
    
    let builder = builder.initialization_script(&script);

    let _window = builder.build()
        .map_err(|e| format!("[CRITICAL] 无法创建独立的 Devin 账号窗口: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn bind_captured_token(app: tauri::AppHandle, id: String, token: String) -> Result<(), String> {
    let mut config = load_config(&app)?;
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
        save_config(&app, &config)?;
        let _ = app.emit("account-plan-updated", ());
    }

    // 尝试关闭后台静默登录的隐藏窗口
    let silent_label = format!("silent-login-{}", id);
    if let Some(w) = app.get_webview_window(&silent_label) {
        let _ = w.close();
    }

    Ok(())
}

#[tauri::command]
pub fn start_silent_login(app: tauri::AppHandle, id: String, name: String) -> Result<(), String> {
    let label = format!("silent-login-{}", id);
    
    if let Some(existing_window) = app.get_webview_window(&label) {
        existing_window.set_focus()
            .map_err(|e| format!("[CRITICAL] 无法激活已有静默窗口: {}", e))?;
        return Ok(());
    }
    
    let config = load_config(&app)?;
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

    let script = format!(
        r#"
        (function() {{
            const accountId = "{}";
            const email = "{}";
            const password = "{}";
            
            // 1. 代理拦截 window.fetch 获取 JWT Token
            const orgFetch = window.fetch;
            window.fetch = async function(...args) {{
                const url = args[0];
                const options = args[1] || {{}};
                
                if (options.headers) {{
                    let authHeader = "";
                    if (options.headers instanceof Headers) {{
                        authHeader = options.headers.get("Authorization") || "";
                    }} else if (typeof options.headers === "object") {{
                        authHeader = options.headers["Authorization"] || options.headers["authorization"] || "";
                    }}
                    if (authHeader && authHeader.includes("devin-session-token$")) {{
                        const tokenVal = authHeader.replace("Bearer ", "").trim();
                        if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {{
                            window.__TAURI_INTERNALS__.invoke("bind_captured_token", {{ id: accountId, token: tokenVal }})
                                .catch(err => console.error(err));
                        }}
                    }}
                }}
                
                const res = await orgFetch(...args);
                
                try {{
                    const clone = res.clone();
                    clone.json().then(data => {{
                        let foundToken = null;
                        
                        function search(obj) {{
                            if (!obj || foundToken) return;
                            if (typeof obj === "string") {{
                                if (obj.includes("devin-session-token$")) {{
                                    foundToken = obj;
                                }} else if (obj.startsWith("ey") && obj.length > 50) {{
                                    foundToken = "devin-session-token$" + obj;
                                }}
                                return;
                            }}
                            if (typeof obj === "object") {{
                                for (const k in obj) {{
                                    if (k === "token" || k === "accessToken" || k === "sessionToken" || k === "jwt") {{
                                        const val = obj[k];
                                        if (typeof val === "string") {{
                                            if (val.includes("devin-session-token$")) {{
                                                foundToken = val;
                                            }} else if (val.startsWith("ey") && val.length > 50) {{
                                                foundToken = "devin-session-token$" + val;
                                            }}
                                        }}
                                    }}
                                    search(obj[k]);
                                }}
                            }}
                        }}
                        
                        search(data);
                        if (foundToken) {{
                            if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {{
                                window.__TAURI_INTERNALS__.invoke("bind_captured_token", {{ id: accountId, token: foundToken }})
                                    .catch(err => console.error(err));
                            }}
                        }}
                    }}).catch((err) => {{ console.error("[CRITICAL] Failed to parse JSON or invoke bind_captured_token", err); }});
                }} catch(e) {{ console.error("[CRITICAL] Intercept fetch failed", e); }}
                
                return res;
            }};

            // 2. 自动填单与登录提交
            let submitted = false;
            function fillAndSubmit() {{
                if (submitted) return false;
                
                let emailFilled = false;
                let passFilled = false;
                
                const emailFields = document.querySelectorAll('input[type="email"], input[name="email"], input[name="username"], input[id*="email"], input[id*="username"]');
                const passFields = document.querySelectorAll('input[type="password"], input[name="password"], input[id*="password"]');
                
                if (emailFields.length > 0 && email) {{
                    for (const f of emailFields) {{
                        if (f.value !== email) {{
                            f.value = email;
                            f.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            f.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        }}
                    }}
                    emailFilled = true;
                }} else if (!email) {{
                    emailFilled = true;
                }}
                
                if (passFields.length > 0 && password) {{
                    for (const f of passFields) {{
                        if (f.value !== password) {{
                            f.value = password;
                            f.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            f.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        }}
                    }}
                    passFilled = true;
                }} else if (!password) {{
                    passFilled = true;
                }}
                
                if (emailFilled && passFilled) {{
                    const submitBtn = document.querySelector('button[type="submit"], button.login-btn, input[type="submit"], button[class*="login"], button[id*="login"]');
                    if (submitBtn) {{
                        submitted = true;
                        submitBtn.click();
                        clearInterval(timer);
                        return true;
                    }}
                }}
                return false;
            }}
            
            const timer = setInterval(fillAndSubmit, 500);
            setTimeout(() => clearInterval(timer), 20000);

            // 3. 保持 Plan 自动刷新
            async function checkPlan() {{
                try {{
                    const res = await fetch('/api/auth/session');
                    if (res.ok) {{
                        const data = await res.json();
                        let plan = null;
                        if (data) {{
                            if (data.plan) plan = data.plan;
                            else if (data.user && data.user.plan) plan = data.user.plan;
                            else if (data.tier) plan = data.tier;
                            else if (data.user && data.user.tier) plan = data.user.tier;
                        }}
                        
                        if (plan) {{
                            const upperPlan = plan.toUpperCase();
                            if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {{
                                await window.__TAURI_INTERNALS__.invoke("update_account_plan", {{ id: accountId, plan: upperPlan }});
                                return true;
                            }}
                        }}
                    }}
                }} catch (e) {{ console.error("[CRITICAL] fetch session plan failed", e); }}
                return false;
            }}

            let checkCount = 0;
            const planTimer = setInterval(async () => {{
                checkCount++;
                const success = await checkPlan();
                if (success || checkCount > 10) {{
                    clearInterval(planTimer);
                }}
            }}, 5000);
        }})();
        "#,
        id.replace('"', "\\\""),
        email_val.replace('"', "\\\""),
        password_val.replace('"', "\\\"")
    );
    
    let builder = builder.initialization_script(&script);

    let _window = builder.build()
        .map_err(|e| format!("[CRITICAL] 无法启动后台自动登录沙箱: {}", e))?;
    
    Ok(())
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
pub fn apply_account_to_default_ide(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut config = load_config(&app)?;
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
                let clean = line.trim();
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
        save_config(&app, &config)?;
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

#[tauri::command]
pub fn capture_credentials(app: tauri::AppHandle, id: String, email: Option<String>, password: Option<String>) -> Result<(), String> {
    let mut config = load_config(&app)?;
    let mut found = false;

    let clean_email = email.filter(|e| !e.trim().is_empty()).map(|e| e.trim().to_string());
    let clean_password = password.filter(|p| !p.trim().is_empty()).map(|p| p.trim().to_string());

    for acc in &mut config.accounts {
        if acc.id == id {
            if clean_email.is_some() && acc.email != clean_email {
                acc.email = clean_email.clone();
                found = true;
            }
            if clean_password.is_some() && acc.password != clean_password {
                acc.password = clean_password.clone();
                found = true;
            }
            break;
        }
    }

    if found {
        save_config(&app, &config)?;
        let _ = app.emit("account-plan-updated", ());
    }

    Ok(())
}
