use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub token: String,
    pub base_url: String,
    pub model: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub token: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub token: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
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
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBaseUrlRequest {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBaseUrlRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AccountDirectory {
    pub id: i64,
    pub account_id: i64,
    pub directory_id: i64,
    pub created_at: DateTime<Utc>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWebDavConfigRequest {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub remote_path: Option<String>,
    pub auto_sync: Option<bool>,
    pub sync_interval: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWebDavConfigRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub remote_path: Option<String>,
    pub auto_sync: Option<bool>,
    pub sync_interval: Option<i64>,
    pub is_active: Option<bool>,
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