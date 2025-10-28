use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub token: String,
    pub base_url: String,
    pub model: String,
    pub is_active: bool,
    pub custom_env_vars: String, // JSON 字符串存储自定义环境变量
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub token: String,
    pub base_url: String,
    pub model: String,
    pub custom_env_vars: Option<serde_json::Value>, // 自定义环境变量
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub token: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub custom_env_vars: Option<serde_json::Value>, // 自定义环境变量
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Directory {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDirectoryRequest {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDirectoryRequest {
    pub path: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct BaseUrl {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub api_key: String,
    pub is_default: bool,
    pub default_env_vars: String, // JSON 字符串存储默认环境变量
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBaseUrlRequest {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub api_key: Option<String>,
    pub is_default: Option<bool>,
    pub default_env_vars: Option<serde_json::Value>, // 默认环境变量
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBaseUrlRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub api_key: Option<String>,
    pub is_default: Option<bool>,
    pub default_env_vars: Option<serde_json::Value>, // 默认环境变量
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchAccountRequest {
    pub account_id: i64,
    pub directory_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub pages: i64,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_num: Option<i64>,
    pub next_num: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountsRequest {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub directory: Directory,
    pub env_config: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    #[allow(dead_code)]
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    #[allow(dead_code)]
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

// WebDAV 配置模型
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WebDavConfig {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub remote_path: String,
    pub auto_sync: bool,
    pub sync_interval: i64,
    pub is_active: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


// 同步日志模型
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SyncLog {
    pub id: i64,
    pub webdav_config_id: i64,
    pub sync_type: String,
    pub status: String,
    pub message: Option<String>,
    pub synced_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSyncLogRequest {
    pub webdav_config_id: i64,
    pub sync_type: String,
    pub status: String,
    pub message: Option<String>,
}

// 环境变量辅助方法
impl Account {
    /// 获取解析后的自定义环境变量
    /// 返回Option是为了区分"无环境变量"和"解析失败"
    pub fn get_custom_env_vars(&self) -> Option<HashMap<String, String>> {
        // 如果为空字符串，返回None
        if self.custom_env_vars.trim().is_empty() {
            return None;
        }
        // 尝试解析，失败时返回None
        match serde_json::from_str::<HashMap<String, String>>(&self.custom_env_vars) {
            Ok(map) if map.is_empty() => None,  // 空对象也返回None
            Ok(map) => Some(map),
            Err(_) => None,
        }
    }
}

impl BaseUrl {
    /// 获取解析后的默认环境变量
    /// 返回Option是为了区分"无环境变量"和"解析失败"
    pub fn get_default_env_vars(&self) -> Option<HashMap<String, String>> {
        // 如果为空字符串，返回None
        if self.default_env_vars.trim().is_empty() {
            return None;
        }
        // 尝试解析，失败时返回None
        match serde_json::from_str::<HashMap<String, String>>(&self.default_env_vars) {
            Ok(map) if map.is_empty() => None,  // 空对象也返回None
            Ok(map) => Some(map),
            Err(_) => None,
        }
    }
}

// 环境变量值类型推断
pub fn parse_env_value(value: &str) -> serde_json::Value {
    use serde_json::json;

    // 1. 尝试解析为布尔值
    if value.eq_ignore_ascii_case("true") {
        return json!(true);
    }
    if value.eq_ignore_ascii_case("false") {
        return json!(false);
    }

    // 2. 尝试解析为整数
    if let Ok(num) = value.parse::<i64>() {
        return json!(num);
    }

    // 3. 尝试解析为浮点数
    if let Ok(float) = value.parse::<f64>() {
        return json!(float);
    }

    // 4. 默认作为字符串
    json!(value)
}