use anyhow::{Context, Result};
use reqwest_dav::{Auth, Client, ClientBuilder, Depth};
use serde_json::Value;
use sqlx::SqlitePool;
use tracing::{error, info, warn};

use crate::models::{CreateSyncLogRequest, WebDavConfig};

/// WebDAV 客户端管理器
pub struct WebDavManager {
    config: WebDavConfig,
    client: Client,
}

impl WebDavManager {
    /// 从数据库配置创建 WebDAV 管理器
    pub async fn from_config(config: WebDavConfig) -> Result<Self> {
        let auth = Auth::Basic(config.username.clone(), config.password.clone());

        let client = ClientBuilder::new()
            .set_host(config.url.clone())
            .set_auth(auth)
            .build()
            .context("Failed to build WebDAV client")?;

        Ok(Self { config, client })
    }

    /// 测试 WebDAV 连接
    pub async fn test_connection(&self) -> Result<bool> {
        match self.client.list("", Depth::Number(0)).await {
            Ok(_) => {
                info!("WebDAV connection test successful");
                Ok(true)
            }
            Err(e) => {
                error!("WebDAV connection test failed: {}", e);
                Err(anyhow::anyhow!("连接失败: {}", e))
            }
        }
    }

    /// 上传配置数据到 WebDAV
    pub async fn upload_config(&self, data: &Value, filename: &str) -> Result<()> {
        let remote_file = format!("{}/{}", self.config.remote_path, filename);
        let json_data = serde_json::to_string_pretty(data)?;

        info!("Uploading config to WebDAV: {}", remote_file);

        // 确保远程目录存在
        self.ensure_remote_dir().await?;

        // 上传文件
        self.client
            .put(&remote_file, json_data.as_bytes().to_vec())
            .await
            .context("Failed to upload config to WebDAV")?;

        info!("Config uploaded successfully");
        Ok(())
    }

    /// 从 WebDAV 下载配置数据
    pub async fn download_config(&self, filename: &str) -> Result<Value> {
        let remote_file = format!("{}/{}", self.config.remote_path, filename);

        info!("Downloading config from WebDAV: {}", remote_file);

        let response = self.client
            .get(&remote_file)
            .await
            .context("Failed to download config from WebDAV")?;

        let data = response.bytes().await
            .context("Failed to read response bytes")?;

        let json_str = String::from_utf8(data.to_vec())
            .context("Failed to parse downloaded data as UTF-8")?;

        let config: Value = serde_json::from_str(&json_str)
            .context("Failed to parse downloaded data as JSON")?;

        info!("Config downloaded successfully");
        Ok(config)
    }

    /// 列出远程目录中的文件
    pub async fn list_remote_files(&self) -> Result<Vec<String>> {
        info!("Listing files in remote directory: {}", self.config.remote_path);

        let list = self.client
            .list(&self.config.remote_path, Depth::Number(1))
            .await
            .context("Failed to list remote files")?;

        // reqwest_dav 的 ListEntity 通常包含 href 字段
        // 我们使用 format! 和 Debug 输出来获取信息
        let files: Vec<String> = list
            .iter()
            .filter_map(|item| {
                // 使用 Debug 输出查看结构
                let debug_str = format!("{:?}", item);
                info!("ListEntity debug: {}", debug_str);

                // ListEntity 通常是这样的结构: ListEntity { href: String, ... }
                // 尝试从 debug 字符串中提取 href
                // 这是临时方案，实际使用时应该根据具体的 ListEntity 定义来访问
                if let Some(start) = debug_str.find("href:") {
                    if let Some(href_start) = debug_str[start..].find('"') {
                        if let Some(href_end) = debug_str[start + href_start + 1..].find('"') {
                            let href = &debug_str[start + href_start + 1..start + href_start + 1 + href_end];
                            info!("Extracted href: {}", href);

                            // 从 href 中提取文件名（最后一个 / 之后的部分）
                            if let Some(last_slash) = href.rfind('/') {
                                let filename = &href[last_slash + 1..];
                                // 过滤掉空文件名和目录（以 / 结尾）
                                if !filename.is_empty() && !href.ends_with('/') {
                                    info!("Extracted filename: {}", filename);
                                    return Some(filename.to_string());
                                }
                            }
                        }
                    }
                }
                None
            })
            .collect();

        info!("Found {} files in remote directory", files.len());
        Ok(files)
    }

    /// 删除远程文件
    pub async fn delete_remote_file(&self, filename: &str) -> Result<()> {
        let remote_file = format!("{}/{}", self.config.remote_path, filename);

        info!("Deleting file from WebDAV: {}", remote_file);

        self.client
            .delete(&remote_file)
            .await
            .context("Failed to delete remote file")?;

        info!("File deleted successfully");
        Ok(())
    }

    /// 确保远程目录存在
    async fn ensure_remote_dir(&self) -> Result<()> {
        // 尝试创建目录,如果已存在会失败但不影响后续操作
        match self.client.mkcol(&self.config.remote_path).await {
            Ok(_) => {
                info!("Remote directory created: {}", self.config.remote_path);
            }
            Err(e) => {
                // 目录可能已存在,记录警告但不报错
                warn!("Failed to create remote directory (may already exist): {}", e);
            }
        }
        Ok(())
    }
}

/// 数据库操作 - WebDAV 配置
pub async fn get_webdav_configs(pool: &SqlitePool) -> Result<Vec<WebDavConfig>> {
    let configs = sqlx::query_as::<_, WebDavConfig>(
        "SELECT * FROM webdav_configs ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
    .context("Failed to get WebDAV configs from database")?;

    Ok(configs)
}

pub async fn get_webdav_config_by_id(pool: &SqlitePool, id: i64) -> Result<Option<WebDavConfig>> {
    let config = sqlx::query_as::<_, WebDavConfig>(
        "SELECT * FROM webdav_configs WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to get WebDAV config from database")?;

    Ok(config)
}

pub async fn get_active_webdav_config(pool: &SqlitePool) -> Result<Option<WebDavConfig>> {
    let config = sqlx::query_as::<_, WebDavConfig>(
        "SELECT * FROM webdav_configs WHERE is_active = 1 LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .context("Failed to get active WebDAV config from database")?;

    Ok(config)
}

pub async fn create_webdav_config(
    pool: &SqlitePool,
    name: &str,
    url: &str,
    username: &str,
    password: &str,
    remote_path: &str,
    auto_sync: bool,
    sync_interval: i64,
) -> Result<WebDavConfig> {
    let result = sqlx::query(
        "INSERT INTO webdav_configs (name, url, username, password, remote_path, auto_sync, sync_interval)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(name)
    .bind(url)
    .bind(username)
    .bind(password)
    .bind(remote_path)
    .bind(auto_sync)
    .bind(sync_interval)
    .execute(pool)
    .await
    .context("Failed to create WebDAV config")?;

    let config = get_webdav_config_by_id(pool, result.last_insert_rowid()).await?
        .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created WebDAV config"))?;

    Ok(config)
}

pub async fn update_webdav_config(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    url: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    remote_path: Option<&str>,
    auto_sync: Option<bool>,
    sync_interval: Option<i64>,
    is_active: Option<bool>,
) -> Result<WebDavConfig> {
    // 构建动态更新语句
    let mut updates = Vec::new();
    let mut values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> = Vec::new();

    if let Some(v) = name {
        updates.push("name = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = url {
        updates.push("url = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = username {
        updates.push("username = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = password {
        updates.push("password = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = remote_path {
        updates.push("remote_path = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = auto_sync {
        updates.push("auto_sync = ?");
        values.push(Box::new(v));
    }
    if let Some(v) = sync_interval {
        updates.push("sync_interval = ?");
        values.push(Box::new(v));
    }
    if let Some(v) = is_active {
        updates.push("is_active = ?");
        values.push(Box::new(v));

        // 如果设置为活跃,则取消其他配置的活跃状态
        if v {
            sqlx::query("UPDATE webdav_configs SET is_active = 0 WHERE id != ?")
                .bind(id)
                .execute(pool)
                .await
                .context("Failed to deactivate other WebDAV configs")?;
        }
    }

    if updates.is_empty() {
        return get_webdav_config_by_id(pool, id).await?
            .ok_or_else(|| anyhow::anyhow!("WebDAV config not found"));
    }

    updates.push("updated_at = CURRENT_TIMESTAMP");
    let update_sql = format!(
        "UPDATE webdav_configs SET {} WHERE id = ?",
        updates.join(", ")
    );

    // 使用原始参数构建查询
    let mut final_query = sqlx::query::<sqlx::Sqlite>(&update_sql);
    if let Some(v) = name { final_query = final_query.bind(v); }
    if let Some(v) = url { final_query = final_query.bind(v); }
    if let Some(v) = username { final_query = final_query.bind(v); }
    if let Some(v) = password { final_query = final_query.bind(v); }
    if let Some(v) = remote_path { final_query = final_query.bind(v); }
    if let Some(v) = auto_sync { final_query = final_query.bind(v); }
    if let Some(v) = sync_interval { final_query = final_query.bind(v); }
    if let Some(v) = is_active { final_query = final_query.bind(v); }
    final_query = final_query.bind(id);

    final_query.execute(pool).await
        .context("Failed to update WebDAV config")?;

    let config = get_webdav_config_by_id(pool, id).await?
        .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated WebDAV config"))?;

    Ok(config)
}

pub async fn delete_webdav_config(pool: &SqlitePool, id: i64) -> Result<()> {
    sqlx::query("DELETE FROM webdav_configs WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .context("Failed to delete WebDAV config")?;

    Ok(())
}

/// 记录同步日志
pub async fn create_sync_log(
    pool: &SqlitePool,
    log: CreateSyncLogRequest,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO sync_logs (webdav_config_id, sync_type, status, message)
         VALUES (?, ?, ?, ?)"
    )
    .bind(log.webdav_config_id)
    .bind(log.sync_type)
    .bind(log.status)
    .bind(log.message)
    .execute(pool)
    .await
    .context("Failed to create sync log")?;

    Ok(())
}

/// 获取同步日志
pub async fn get_sync_logs(pool: &SqlitePool, config_id: Option<i64>, limit: i64) -> Result<Vec<crate::models::SyncLog>> {
    let logs = if let Some(id) = config_id {
        sqlx::query_as::<_, crate::models::SyncLog>(
            "SELECT * FROM sync_logs WHERE webdav_config_id = ? ORDER BY synced_at DESC LIMIT ?"
        )
        .bind(id)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, crate::models::SyncLog>(
            "SELECT * FROM sync_logs ORDER BY synced_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }
    .context("Failed to get sync logs from database")?;

    Ok(logs)
}

/// 更新最后同步时间
pub async fn update_last_sync_time(pool: &SqlitePool, config_id: i64) -> Result<()> {
    sqlx::query("UPDATE webdav_configs SET last_sync_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(config_id)
        .execute(pool)
        .await
        .context("Failed to update last sync time")?;

    Ok(())
}
