mod models;
mod database;
mod claude_config;
mod config_manager;
mod logger;

use std::sync::Arc;
use tauri::{State, Manager, WindowEvent, tray::{TrayIconBuilder, TrayIconEvent}, menu::{Menu, MenuItem}};
use tokio::sync::Mutex;
use models::*;
use database::Database;
use claude_config::ClaudeConfigManager;

type DbState = Arc<Mutex<Database>>;

#[tauri::command]
async fn get_accounts(
    db: State<'_, DbState>,
    request: GetAccountsRequest,
) -> Result<AccountsResponse, String> {
    tracing::info!("获取账号列表, 请求参数: {:?}", request);
    
    let db = db.lock().await;
    
    match db.get_accounts(request).await {
        Ok(response) => {
            tracing::debug!("成功获取 {} 个账号", response.accounts.len());
            Ok(response)
        }
        Err(e) => {
            tracing::error!("获取账号列表失败: {}", e);
            Err(format!("获取账号列表失败: {}", e))
        }
    }
}

#[tauri::command]
#[allow(non_snake_case)]
async fn create_account(
    db: State<'_, DbState>,
    name: String,
    token: String,
    baseUrl: String,
    model: String,
) -> Result<Account, String> {
    tracing::info!("创建账号: name={}, baseUrl={}, model={}", name, baseUrl, model);
    
    let db = db.lock().await;
    let request = CreateAccountRequest {
        name,
        token,
        base_url: baseUrl,
        model,
    };
    
    match db.create_account(request).await {
        Ok(account) => {
            tracing::info!("成功创建账号: id={}, name={}", account.id, account.name);
            Ok(account)
        }
        Err(e) => {
            tracing::error!("创建账号失败: {}", e);
            Err(format!("创建账号失败: {}", e))
        }
    }
}

#[tauri::command]
#[allow(non_snake_case)]
async fn update_account(
    db: State<'_, DbState>,
    id: i64,
    name: Option<String>,
    token: Option<String>,
    baseUrl: Option<String>,
    model: Option<String>,
) -> Result<Account, String> {
    let db = db.lock().await;
    let request = UpdateAccountRequest {
        name,
        token,
        base_url: baseUrl,
        model,
    };
    
    db.update_account(id, request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_account(db: State<'_, DbState>, id: i64) -> Result<String, String> {
    use tracing::info;
    
    info!("开始删除账号，ID: {}", id);
    
    let db = db.lock().await;
    
    // 先查询账号信息用于日志
    let account_info = match db.get_account(id).await {
        Ok(acc) => format!("账号: {} ({})", acc.name, acc.base_url),
        Err(_) => format!("未知账号 (ID: {})", id)
    };
    
    match db.delete_account(id).await {
        Ok(_) => {
            info!("成功删除 {}", account_info);
            Ok(format!("账号删除成功"))
        },
        Err(e) => {
            let error_msg = format!("删除 {} 失败: {}", account_info, e);
            tracing::error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
async fn get_account_base_urls(db: State<'_, DbState>) -> Result<Vec<String>, String> {
    let db = db.lock().await;
    db.get_account_base_urls()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_directories(db: State<'_, DbState>) -> Result<Vec<Directory>, String> {
    let db = db.lock().await;
    db.get_directories()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_directory(
    db: State<'_, DbState>,
    path: String,
    name: String,
) -> Result<Directory, String> {
    let db = db.lock().await;
    let request = CreateDirectoryRequest { path, name };
    
    db.create_directory(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_directory(
    db: State<'_, DbState>,
    id: i64,
    path: Option<String>,
    name: Option<String>,
) -> Result<Directory, String> {
    let db = db.lock().await;
    let request = UpdateDirectoryRequest { path, name };
    
    db.update_directory(id, request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_directory(db: State<'_, DbState>, id: i64) -> Result<String, String> {
    use tracing::info;
    
    info!("开始删除目录，ID: {}", id);
    
    let db = db.lock().await;
    
    // 先查询目录信息用于日志和文件系统检查
    let directory_info = match db.get_directory(id).await {
        Ok(dir) => {
            let path_exists = std::path::Path::new(&dir.path).exists();
            if path_exists {
                format!("目录: {} (路径: {})", dir.name, dir.path)
            } else {
                format!("目录: {} (路径: {} - 文件系统中不存在)", dir.name, dir.path)
            }
        },
        Err(_) => format!("未知目录 (ID: {})", id)
    };
    
    match db.delete_directory(id).await {
        Ok(_) => {
            info!("成功处理 {}", directory_info);
            // 检查是否是清理不存在的目录
            if directory_info.contains("文件系统中不存在") {
                Ok(format!("已清理不存在的目录记录"))
            } else {
                Ok(format!("目录删除成功"))
            }
        },
        Err(e) => {
            let error_msg = format!("删除 {} 失败: {}", directory_info, e);
            tracing::error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
async fn check_directory_exists(path: String) -> Result<bool, String> {
    Ok(std::path::Path::new(&path).exists())
}

#[tauri::command]
async fn get_base_urls(db: State<'_, DbState>) -> Result<Vec<BaseUrl>, String> {
    let db = db.lock().await;
    db.get_base_urls()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_base_url(
    db: State<'_, DbState>,
    name: String,
    url: String,
    description: Option<String>,
    is_default: Option<bool>,
) -> Result<BaseUrl, String> {
    let db = db.lock().await;
    let request = CreateBaseUrlRequest {
        name,
        url,
        description,
        is_default,
    };
    
    db.create_base_url(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_base_url(
    db: State<'_, DbState>,
    id: i64,
    name: Option<String>,
    url: Option<String>,
    description: Option<String>,
    is_default: Option<bool>,
) -> Result<BaseUrl, String> {
    let db = db.lock().await;
    let request = UpdateBaseUrlRequest {
        name,
        url,
        description,
        is_default,
    };
    
    db.update_base_url(id, request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_base_url(db: State<'_, DbState>, id: i64) -> Result<String, String> {
    use tracing::info;
    
    info!("开始删除Base URL，ID: {}", id);
    
    let db = db.lock().await;
    
    // 先查询Base URL信息用于日志
    let base_url_info = match db.get_base_url(id).await {
        Ok(url) => format!("Base URL: {} ({})", url.name, url.url),
        Err(_) => format!("未知Base URL (ID: {})", id)
    };
    
    match db.delete_base_url(id).await {
        Ok(_) => {
            info!("成功删除 {}", base_url_info);
            Ok(format!("Base URL删除成功"))
        },
        Err(e) => {
            let error_msg = format!("删除 {} 失败: {}", base_url_info, e);
            tracing::error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
#[allow(non_snake_case)]
async fn switch_account(
    db: State<'_, DbState>,
    accountId: i64,
    directoryId: i64,
    isSandbox: Option<bool>,
) -> Result<String, String> {
    tracing::info!("切换账号: accountId={}, directoryId={}, isSandbox={:?}", accountId, directoryId, isSandbox);
    let db_lock = db.lock().await;
    
    // Switch in database
    let request = SwitchAccountRequest {
        account_id: accountId,
        directory_id: directoryId,
    };
    let message = match db_lock.switch_account(request).await {
        Ok(msg) => {
            tracing::info!("数据库切换成功: {}", msg);
            msg
        }
        Err(e) => {
            tracing::error!("数据库切换失败: {}", e);
            return Err(format!("数据库切换失败: {}", e));
        }
    };
    
    // Get account and directory info
    let account = db_lock.get_account(accountId).await.map_err(|e| {
        tracing::error!("获取账号信息失败: {}", e);
        e.to_string()
    })?;
    let directory = db_lock.get_directory(directoryId).await.map_err(|e| {
        tracing::error!("获取目录信息失败: {}", e);
        e.to_string()
    })?;
    
    drop(db_lock); // Release the lock before doing file operations

    // Update Claude configuration file
    let config_manager = ClaudeConfigManager::new(directory.path.clone());
    config_manager
        .update_env_config_with_options(
            account.token,
            account.base_url,
            isSandbox.unwrap_or(true)
        )
        .map_err(|e| e.to_string())?;

    // Copy remove-root-check.sh to .claude directory
    let claude_dir = std::path::Path::new(&directory.path).join(".claude");

    // Ensure .claude directory exists
    if let Err(e) = std::fs::create_dir_all(&claude_dir) {
        tracing::warn!("创建.claude目录失败: {}，跳过复制脚本", e);
    } else {
        let script_content = include_str!("../resources/config/remove-root-check.sh");
        let script_file = claude_dir.join("remove-root-check.sh");

        match std::fs::write(&script_file, script_content) {
            Ok(_) => {
                tracing::info!("remove-root-check.sh 已复制到: {}", script_file.display());

                // Set executable permissions on Unix systems
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(&script_file) {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        if let Err(e) = std::fs::set_permissions(&script_file, perms) {
                            tracing::warn!("设置脚本执行权限失败: {}", e);
                        } else {
                            tracing::info!("已设置 remove-root-check.sh 为可执行");
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("复制 remove-root-check.sh 失败: {}，但不影响主要功能", e);
            }
        }
    }

    Ok(message)
}

#[tauri::command]
#[allow(non_snake_case)]
async fn get_current_config(
    db: State<'_, DbState>,
    directoryId: i64,
) -> Result<ConfigInfo, String> {
    let db = db.lock().await;
    let directory = db.get_directory(directoryId).await.map_err(|e| e.to_string())?;
    drop(db);
    
    let config_manager = ClaudeConfigManager::new(directory.path.clone());
    let env_config = config_manager.get_env_config().map_err(|e| e.to_string())?;
    
    Ok(ConfigInfo {
        directory,
        env_config,
    })
}

#[tauri::command]
async fn get_associations(db: State<'_, DbState>) -> Result<Vec<std::collections::HashMap<String, serde_json::Value>>, String> {
    let db = db.lock().await;
    db.get_associations()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_database_info() -> Result<std::collections::HashMap<String, String>, String> {
    let config_manager = config_manager::ConfigManager::new();
    
    // 获取当前默认连接名称
    let current_connection = config_manager.config.current.clone();
    
    // 获取当前连接的配置
    let db_config = config_manager.get_database_config(Some(&current_connection))
        .ok_or_else(|| format!("Database configuration '{}' not found", current_connection))?;
    
    let mut info = std::collections::HashMap::new();
    
    // 隐藏密码信息
    let mut safe_url = db_config.url.clone();
    if safe_url.contains("://") && safe_url.contains("@") {
        // 对于包含认证信息的URL，隐藏密码部分
        if let Some(at_pos) = safe_url.rfind('@') {
            if let Some(scheme_end) = safe_url.find("://") {
                let scheme_part = &safe_url[..scheme_end + 3];
                let host_part = &safe_url[at_pos..];
                if let Some(colon_pos) = safe_url[scheme_end + 3..at_pos].rfind(':') {
                    let user_part = &safe_url[scheme_end + 3..scheme_end + 3 + colon_pos];
                    safe_url = format!("{}{}:***{}", scheme_part, user_part, host_part);
                }
            }
        }
    }
    
    info.insert("name".to_string(), current_connection.clone());
    info.insert("url".to_string(), safe_url);
    info.insert("pool_size".to_string(), db_config.pool_size.unwrap_or(10).to_string());
    info.insert("max_overflow".to_string(), db_config.max_overflow.unwrap_or(20).to_string());
    info.insert("pool_timeout".to_string(), db_config.pool_timeout.unwrap_or(30).to_string());
    info.insert("pool_recycle".to_string(), db_config.pool_recycle.unwrap_or(3600).to_string());
    info.insert("echo".to_string(), db_config.echo.unwrap_or(false).to_string());
    
    // 对于SQLite，设置一些固定值
    if current_connection == "default" {
        info.insert("checked_in".to_string(), "1".to_string());
        info.insert("checked_out".to_string(), "0".to_string());
        info.insert("overflow".to_string(), "0".to_string());
    } else {
        // 对于其他数据库，由于当前版本不支持，显示未连接状态
        info.insert("checked_in".to_string(), "0".to_string());
        info.insert("checked_out".to_string(), "0".to_string());
        info.insert("overflow".to_string(), "0".to_string());
    }
    
    Ok(info)
}

#[tauri::command]
async fn get_database_connections() -> Result<serde_json::Value, String> {
    let config_manager = config_manager::ConfigManager::new();
    
    let mut connections = serde_json::Map::new();
    
    // 获取所有连接配置
    for (name, db_config) in &config_manager.config.connections {
        let mut safe_config = serde_json::Map::new();
        let mut safe_url = db_config.url.clone();
        
        // 隐藏密码信息
        if safe_url.contains("://") && safe_url.contains("@") {
            if let Some(at_pos) = safe_url.rfind('@') {
                if let Some(scheme_end) = safe_url.find("://") {
                    let scheme_part = &safe_url[..scheme_end + 3];
                    let host_part = &safe_url[at_pos..];
                    if let Some(colon_pos) = safe_url[scheme_end + 3..at_pos].rfind(':') {
                        let user_part = &safe_url[scheme_end + 3..scheme_end + 3 + colon_pos];
                        safe_url = format!("{}{}:***{}", scheme_part, user_part, host_part);
                    }
                }
            }
        }
        
        safe_config.insert("url".to_string(), serde_json::Value::String(safe_url));
        safe_config.insert("pool_size".to_string(), serde_json::Value::Number(
            serde_json::Number::from(db_config.pool_size.unwrap_or(10))
        ));
        connections.insert(name.clone(), serde_json::Value::Object(safe_config));
    }
    
    let mut result = serde_json::Map::new();
    result.insert("connections".to_string(), serde_json::Value::Object(connections));
    // 从配置中读取当前默认连接
    result.insert("current".to_string(), serde_json::Value::String(config_manager.config.current.clone()));
    
    Ok(serde_json::Value::Object(result))
}

#[tauri::command]
async fn switch_database(connection_name: String) -> Result<String, String> {
    let mut config_manager = config_manager::ConfigManager::new();
    
    // 检查连接是否存在
    if config_manager.get_database_config(Some(&connection_name)).is_none() {
        return Err(format!("数据库连接 '{}' 不存在", connection_name));
    }
    
    // 对于非SQLite连接，给出警告但仍然允许切换配置
    let warning = if connection_name != "default" {
        "注意：当前版本仅支持SQLite数据库实际连接，其他数据库仅作配置切换。"
    } else {
        ""
    };
    
    // 切换默认连接
    match config_manager.set_default_connection(&connection_name) {
        Ok(_) => {
            let message = if warning.is_empty() {
                format!("已切换到数据库连接: {}", connection_name)
            } else {
                format!("已切换到数据库连接: {}。{}", connection_name, warning)
            };
            Ok(message)
        }
        Err(e) => Err(format!("切换数据库连接失败: {}", e))
    }
}

#[tauri::command]
async fn test_database(connection_name: String) -> Result<String, String> {
    let config_manager = config_manager::ConfigManager::new();
    
    // 检查连接配置是否存在
    let db_config = config_manager.get_database_config(Some(&connection_name))
        .ok_or_else(|| format!("数据库连接 '{}' 不存在", connection_name))?;
    
    // 验证URL格式
    if db_config.url.is_empty() {
        return Err("数据库URL不能为空".to_string());
    }
    
    match connection_name.as_str() {
        "default" => {
            // 对于默认SQLite，尝试实际连接
            match crate::database::Database::new().await {
                Ok(_) => Ok("SQLite数据库连接测试成功".to_string()),
                Err(e) => Err(format!("SQLite数据库连接测试失败: {}", e)),
            }
        }
        "mysql" => {
            // 对于MySQL，尝试实际连接
            match test_mysql_connection(&db_config.url).await {
                Ok(_) => Ok("MySQL数据库连接测试成功".to_string()),
                Err(e) => Err(format!("MySQL数据库连接测试失败: {}", e)),
            }
        }
        _ => {
            Err(format!("不支持的数据库类型: {}", connection_name))
        }
    }
}

async fn test_mysql_connection(url: &str) -> Result<(), sqlx::Error> {
    use sqlx::MySqlPool;
    use std::time::Duration;
    
    // 转换Python格式的URL为Rust MySQL格式
    let mysql_url = if url.starts_with("mysql+pymysql://") {
        url.replace("mysql+pymysql://", "mysql://")
    } else {
        url.to_string()
    };
    
    // 创建连接池，设置连接超时
    let pool = tokio::time::timeout(
        Duration::from_secs(10),  // 10秒连接超时
        MySqlPool::connect(&mysql_url)
    ).await
    .map_err(|_| sqlx::Error::Io(std::io::Error::new(
        std::io::ErrorKind::TimedOut, 
        "连接超时"
    )))??;
    
    // 执行一个简单的查询来测试连接，设置查询超时
    tokio::time::timeout(
        Duration::from_secs(5),  // 5秒查询超时
        sqlx::query("SELECT 1").fetch_one(&pool)
    ).await
    .map_err(|_| sqlx::Error::Io(std::io::Error::new(
        std::io::ErrorKind::TimedOut, 
        "查询超时"
    )))??;
    
    // 关闭连接池
    pool.close().await;
    
    Ok(())
}

// 日志相关命令
#[tauri::command]
async fn get_log_info() -> Result<serde_json::Value, String> {
    crate::logger::Logger::get_log_info()
        .map_err(|e| format!("获取日志信息失败: {}", e))
}

#[tauri::command]
async fn get_recent_logs(lines: Option<usize>) -> Result<Vec<String>, String> {
    crate::logger::Logger::get_recent_logs(lines)
        .map_err(|e| format!("读取日志失败: {}", e))
}

#[tauri::command]
async fn log_test_message(level: String, message: String) -> Result<String, String> {
    match level.to_lowercase().as_str() {
        "info" => {
            tracing::info!("测试日志消息: {}", message);
            Ok("INFO日志已记录".to_string())
        }
        "warn" => {
            tracing::warn!("测试警告消息: {}", message);
            Ok("WARN日志已记录".to_string())
        }
        "error" => {
            tracing::error!("测试错误消息: {}", message);
            Ok("ERROR日志已记录".to_string())
        }
        "debug" => {
            tracing::debug!("测试调试消息: {}", message);
            Ok("DEBUG日志已记录".to_string())
        }
        _ => {
            Err("不支持的日志级别，支持: info, warn, error, debug".to_string())
        }
    }
}

#[tauri::command]
async fn get_log_file_path() -> Result<String, String> {
    match crate::logger::Logger::get_log_directory() {
        Ok(log_dir) => {
            let log_file = log_dir.join("claude-config-manager.log");
            Ok(log_file.display().to_string())
        }
        Err(e) => Err(format!("获取日志路径失败: {}", e))
    }
}

#[tauri::command]
async fn open_log_directory() -> Result<String, String> {
    match crate::logger::Logger::get_log_directory() {
        Ok(log_dir) => {
            // 确保目录存在
            std::fs::create_dir_all(&log_dir)
                .map_err(|e| format!("创建日志目录失败: {}", e))?;
            
            // 在不同平台上打开目录
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("explorer")
                    .arg(&log_dir)
                    .spawn()
                    .map_err(|e| format!("打开目录失败: {}", e))?;
            }
            
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(&log_dir)
                    .spawn()
                    .map_err(|e| format!("打开目录失败: {}", e))?;
            }
            
            #[cfg(target_os = "linux")]
            {
                std::process::Command::new("xdg-open")
                    .arg(&log_dir)
                    .spawn()
                    .map_err(|e| format!("打开目录失败: {}", e))?;
            }
            
            Ok(format!("已打开日志目录: {}", log_dir.display()))
        }
        Err(e) => Err(format!("获取日志目录失败: {}", e))
    }
}

// Claude Settings commands
#[tauri::command]
async fn get_claude_settings_from_db(db: State<'_, DbState>) -> Result<serde_json::Value, String> {
    tracing::info!("从数据库获取Claude设置");
    
    let db = db.lock().await;
    let settings_json = db.get_claude_settings()
        .await
        .map_err(|e| format!("获取Claude设置失败: {}", e))?;
    
    let settings: serde_json::Value = serde_json::from_str(&settings_json)
        .map_err(|e| format!("解析Claude设置失败: {}", e))?;
    
    Ok(settings)
}

#[tauri::command]
async fn save_claude_settings_to_db(
    db: State<'_, DbState>,
    settings_json: String
) -> Result<String, String> {
    tracing::info!("保存Claude设置到数据库: {}", settings_json);
    
    // Validate JSON format
    serde_json::from_str::<serde_json::Value>(&settings_json)
        .map_err(|e| format!("JSON格式错误: {}", e))?;
    
    let db = db.lock().await;
    db.save_claude_settings(&settings_json)
        .await
        .map_err(|e| format!("保存Claude设置失败: {}", e))?;
    
    tracing::info!("Claude设置保存成功");
    Ok("Claude设置保存成功".to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
async fn switch_account_with_claude_settings(
    db: State<'_, DbState>,
    accountId: i64,
    directoryId: i64,
    isSandbox: Option<bool>,
    claudeSettings: serde_json::Value,
) -> Result<String, String> {
    tracing::info!("切换账号并写入Claude设置: accountId={}, directoryId={}, isSandbox={:?}", accountId, directoryId, isSandbox);
    let db_lock = db.lock().await;
    
    // Switch in database
    let request = SwitchAccountRequest {
        account_id: accountId,
        directory_id: directoryId,
    };
    let message = match db_lock.switch_account(request).await {
        Ok(msg) => {
            tracing::info!("数据库切换成功: {}", msg);
            msg
        }
        Err(e) => {
            tracing::error!("数据库切换失败: {}", e);
            return Err(format!("数据库切换失败: {}", e));
        }
    };
    
    // Get account and directory info
    let account = db_lock.get_account(accountId).await.map_err(|e| {
        tracing::error!("获取账号信息失败: {}", e);
        e.to_string()
    })?;
    let directory = db_lock.get_directory(directoryId).await.map_err(|e| {
        tracing::error!("获取目录信息失败: {}", e);
        e.to_string()
    })?;
    
    drop(db_lock); // Release the lock before doing file operations
    
    // Clone account information before using it
    let account_token = account.token.clone();
    let account_base_url = account.base_url.clone();
    
    // Update Claude configuration file with environment setup
    let config_manager = ClaudeConfigManager::new(directory.path.clone());
    config_manager
        .update_env_config_with_options(
            account.token, 
            account.base_url,
            isSandbox.unwrap_or(true)
        )
        .map_err(|e| e.to_string())?;
    
    // Write Claude settings to .claude/settings.local.json with merged environment variables
    let claude_dir = std::path::Path::new(&directory.path).join(".claude");
    std::fs::create_dir_all(&claude_dir)
        .map_err(|e| format!("创建.claude目录失败: {}", e))?;
    
    // Merge Claude settings with account environment variables
    let mut merged_settings = claudeSettings.clone();
    
    // Ensure env section exists
    if !merged_settings.is_object() {
        merged_settings = serde_json::json!({});
    }
    let settings_obj = merged_settings.as_object_mut().unwrap();
    
    if !settings_obj.contains_key("env") {
        settings_obj.insert("env".to_string(), serde_json::json!({}));
    }
    
    let env_obj = settings_obj.get_mut("env").unwrap().as_object_mut().unwrap();
    
    // Add account-specific environment variables using cloned values
    env_obj.insert("ANTHROPIC_API_KEY".to_string(), serde_json::Value::String(account_token.clone()));
    env_obj.insert("ANTHROPIC_AUTH_TOKEN".to_string(), serde_json::Value::String(account_token));
    env_obj.insert("ANTHROPIC_BASE_URL".to_string(), serde_json::Value::String(account_base_url));
    
    let settings_file = claude_dir.join("settings.local.json");
    let settings_json = serde_json::to_string_pretty(&merged_settings)
        .map_err(|e| format!("序列化Claude设置失败: {}", e))?;
    
    std::fs::write(&settings_file, settings_json)
        .map_err(|e| format!("写入Claude设置文件失败: {}", e))?;

    tracing::info!("Claude设置已写入: {}", settings_file.display());
    tracing::info!("账号环境变量已合并: ANTHROPIC_API_KEY, ANTHROPIC_AUTH_TOKEN, ANTHROPIC_BASE_URL");

    // Copy remove-root-check.sh to .claude directory
    let script_content = include_str!("../resources/config/remove-root-check.sh");
    let script_file = claude_dir.join("remove-root-check.sh");

    match std::fs::write(&script_file, script_content) {
        Ok(_) => {
            tracing::info!("remove-root-check.sh 已复制到: {}", script_file.display());

            // Set executable permissions on Unix systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&script_file)
                    .map_err(|e| format!("获取脚本文件权限失败: {}", e))?
                    .permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&script_file, perms)
                    .map_err(|e| format!("设置脚本执行权限失败: {}", e))?;
                tracing::info!("已设置 remove-root-check.sh 为可执行");
            }
        }
        Err(e) => {
            tracing::warn!("复制 remove-root-check.sh 失败: {}，但不影响主要功能", e);
        }
    }

    let final_message = format!("{} Claude配置和账号环境变量已写入 .claude/settings.local.json", message);
    Ok(final_message)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    if let Err(e) = crate::logger::Logger::init() {
        eprintln!("日志系统初始化失败: {}", e);
    }
    
    tracing::info!("Claude Configuration Manager 启动中...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 当尝试启动第二个实例时的处理逻辑
            tracing::info!("检测到重复启动，显示现有窗口");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                // 阻止默认的关闭行为，改为隐藏到托盘
                api.prevent_close();
                window.hide().unwrap();
            }
            _ => {}
        })
        .setup(|app| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let db = runtime.block_on(async {
                // 尝试多次初始化数据库，包括自动修复
                let mut retry_count = 0;
                const MAX_RETRIES: u32 = 3;
                
                loop {
                    match Database::new().await {
                        Ok(database) => {
                            tracing::info!("数据库初始化成功");
                            if retry_count > 0 {
                                println!("数据库初始化成功（重试 {} 次后）", retry_count);
                            }
                            break database;
                        }
                        Err(e) => {
                            retry_count += 1;
                            tracing::error!("数据库初始化失败 (尝试 {}/{}): {}", retry_count, MAX_RETRIES, e);
                            
                            if retry_count >= MAX_RETRIES {
                                // 最后尝试：创建默认数据库配置
                                eprintln!("数据库初始化失败，正在尝试创建默认配置...");
                                
                                match Database::create_with_fallback().await {
                                    Ok(database) => {
                                        tracing::warn!("使用默认配置创建数据库成功");
                                        println!("已创建默认数据库配置");
                                        break database;
                                    }
                                    Err(fallback_error) => {
                                        tracing::error!("创建默认数据库配置也失败: {}", fallback_error);
                                        eprintln!("数据库初始化完全失败:");
                                        eprintln!("  - 原始错误: {}", e);
                                        eprintln!("  - 默认配置错误: {}", fallback_error);
                                        eprintln!("请检查:");
                                        eprintln!("  1. 应用目录权限");
                                        eprintln!("  2. 磁盘空间");
                                        eprintln!("  3. 杀毒软件是否阻止文件创建");
                                        eprintln!("");
                                        eprintln!("应用将退出，请解决上述问题后重新启动。");
                                        
                                        std::process::exit(1);
                                    }
                                }
                            } else {
                                // 短暂延迟后重试
                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            }
                        }
                    }
                }
            });
            
            app.manage(Arc::new(Mutex::new(db)));

            // 创建托盘菜单
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>).unwrap();
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();
            let menu = Menu::with_items(app, &[&show_item, &quit_item]).unwrap();

            // 创建托盘图标
            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Claude Configuration Manager")
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, button_state, .. } = event {
                        if button == tauri::tray::MouseButton::Left && button_state == tauri::tray::MouseButtonState::Up {
                            let app = tray.app_handle();
                            let window = app.get_webview_window("main").unwrap();
                            if window.is_visible().unwrap() {
                                window.hide().unwrap();
                            } else {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            create_account,
            update_account,
            delete_account,
            get_account_base_urls,
            get_directories,
            create_directory,
            update_directory,
            delete_directory,
            check_directory_exists,
            get_base_urls,
            create_base_url,
            update_base_url,
            delete_base_url,
            switch_account,
            switch_account_with_claude_settings,
            get_current_config,
            get_associations,
            get_database_info,
            get_database_connections,
            switch_database,
            test_database,
            get_log_info,
            get_recent_logs,
            log_test_message,
            get_log_file_path,
            open_log_directory,
            get_claude_settings_from_db,
            save_claude_settings_to_db
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
