use serde::{Deserialize, Serialize};

fn default_plan_tier() -> String {
    "AUTO".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub email: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default = "default_plan_tier")]
    pub plan_tier: String,
    #[serde(default)]
    pub billing_error: Option<String>,
    #[serde(default)]
    pub available_credits: Option<f64>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub accounts: Vec<Account>,
}

pub struct AppState(pub std::sync::Mutex<AppConfig>);
