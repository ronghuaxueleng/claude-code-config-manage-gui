use sqlx::{sqlite::SqlitePool, Row, Error as SqlxError};
use chrono::{Utc, DateTime};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::models::*;
use crate::config_manager::ConfigManager;
use tracing::{info, error, warn};

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    /// 获取数据库连接池引用
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
    /// 创建带有回退策略的数据库连接
    /// 当正常初始化失败时，尝试在用户主目录创建数据库
    pub async fn create_with_fallback() -> Result<Self, SqlxError> {
        info!("尝试使用回退策略初始化数据库");
        println!("正在尝试回退策略，将在用户主目录创建数据库...");
        
        // 获取用户主目录
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| {
                error!("无法获取用户主目录");
                SqlxError::Configuration("无法获取用户主目录，请检查环境变量 HOME 或 USERPROFILE".into())
            })?;
        
        // 创建应用数据目录
        let app_data_dir = std::path::PathBuf::from(&home_dir).join(".claude-config-manager");
        println!("创建应用数据目录: {}", app_data_dir.display());
        
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| {
                error!("无法创建应用数据目录 {}: {}", app_data_dir.display(), e);
                SqlxError::Configuration(format!("无法创建应用数据目录 {}: {}", app_data_dir.display(), e).into())
            })?;
        
        // 使用固定的数据库文件名
        let db_path = app_data_dir.join("claude_config.db");
        
        // 修复：使用正确的 SQLite URL 格式
        #[cfg(windows)]
        let database_url = {
            let normalized_path = db_path.display().to_string().replace('\\', "/");
            format!("sqlite:///{}?mode=rwc", normalized_path)
        };
        #[cfg(not(windows))]
        let database_url = format!("sqlite:///{}?mode=rwc", db_path.display());
        
        info!("回退数据库路径: {}", database_url);
        println!("数据库将创建在: {}", db_path.display());
        
        // 确保父目录可写
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| SqlxError::Configuration(format!("无法创建数据库目录: {}", e).into()))?;
            }
        }
        
        // 连接数据库
        let pool = SqlitePool::connect(&database_url).await
            .map_err(|e| {
                error!("回退策略连接数据库失败: {}", e);
                println!("回退策略数据库连接失败: {}", e);
                e
            })?;
        
        info!("回退策略数据库连接成功");
        println!("数据库连接成功！");
        
        let db = Self { pool };

        // 初始化数据库结构（包括迁移）
        println!("正在初始化数据库结构...");
        db.migrate().await
            .map_err(|e| {
                error!("回退策略数据库迁移和初始化失败: {}", e);
                println!("数据库迁移和初始化失败: {}", e);
                e
            })?;

        info!("回退策略数据库迁移和初始化完成");
        println!("数据库初始化完成！应用现在应该可以正常工作了。");
        Ok(db)
    }

    pub async fn new() -> Result<Self, SqlxError> {
        info!("开始初始化数据库");

        // 使用配置管理器获取数据库配置
        let config_manager = ConfigManager::new();
        let db_config = config_manager.get_default_database_config()
            .ok_or_else(|| SqlxError::Configuration("No database configuration found".into()))?;

        let mut database_url = db_config.url.clone();
        info!("原始数据库URL: {}", database_url);

        // 处理SQLite相对路径，将数据库放在用户数据目录而不是resources目录
        if database_url.starts_with("sqlite:///") && !database_url.starts_with("sqlite:////") {
            // 获取数据库文件名
            let db_filename = database_url.replace("sqlite:///", "");
            info!("提取的数据库文件名: {}", db_filename);

            // 使用用户数据目录存储数据库（确保重装应用后数据不丢失）
            let final_db_path = if let Some(user_data_dir) = ConfigManager::get_app_data_dir() {
                info!("使用用户数据目录作为数据库位置: {}", user_data_dir.display());

                // 确保用户数据目录存在
                if !user_data_dir.exists() {
                    info!("创建用户数据目录: {}", user_data_dir.display());
                    std::fs::create_dir_all(&user_data_dir)
                        .map_err(|e| SqlxError::Configuration(format!("创建用户数据目录失败: {}", e).into()))?;
                }

                user_data_dir.join(&db_filename)
            } else {
                // 如果无法获取用户数据目录，使用当前目录
                warn!("无法获取用户数据目录，使用当前目录");
                let current_dir = std::env::current_dir()
                    .map_err(|e| SqlxError::Configuration(format!("获取当前目录失败: {}", e).into()))?;
                current_dir.join(&db_filename)
            };

            // 数据迁移：检查旧的数据库位置
            if !final_db_path.exists() {
                let mut migrated = false;

                // 1. 检查应用内resources目录（旧版本的错误位置）
                if let Some(old_resources_dir) = ConfigManager::get_resource_dir() {
                    let old_db_path = old_resources_dir.join(&db_filename);
                    if old_db_path.exists() {
                        info!("发现旧版本数据库位置（应用内），开始迁移: {} -> {}", old_db_path.display(), final_db_path.display());
                        match std::fs::copy(&old_db_path, &final_db_path) {
                            Ok(_) => {
                                info!("数据库迁移成功！");
                                let _ = std::fs::remove_file(&old_db_path);
                                info!("已清理旧数据库文件");
                                migrated = true;
                            }
                            Err(e) => {
                                warn!("数据库迁移失败: {}", e);
                            }
                        }
                    }
                }

                // 2. 检查Windows平台可能的错误路径
                if !migrated {
                    #[cfg(target_os = "windows")]
                    {
                        // Windows可能的错误路径列表
                        let mut possible_old_paths = Vec::new();

                        // 错误路径1: %APPDATA%\.claude-config-manager (带点前缀的错误实现)
                        if let Ok(appdata) = std::env::var("APPDATA") {
                            possible_old_paths.push((
                                PathBuf::from(appdata).join(".claude-config-manager").join(&db_filename),
                                "APPDATA错误路径"
                            ));
                        }

                        // 错误路径2: %USERPROFILE%\.claude-config-manager (回退逻辑可能导致)
                        if let Ok(userprofile) = std::env::var("USERPROFILE") {
                            possible_old_paths.push((
                                PathBuf::from(userprofile).join(".claude-config-manager").join(&db_filename),
                                "USERPROFILE错误路径"
                            ));
                        }

                        // 错误路径3: %USERPROFILE%\claude-config-manager (可能的其他变体)
                        if let Ok(userprofile) = std::env::var("USERPROFILE") {
                            possible_old_paths.push((
                                PathBuf::from(userprofile).join("claude-config-manager").join(&db_filename),
                                "USERPROFILE变体路径"
                            ));
                        }

                        // 尝试从这些路径迁移数据（安全迁移）
                        for (old_path, path_type) in possible_old_paths {
                            if old_path.exists() && old_path != final_db_path {
                                // 检查文件大小，确保不是空文件
                                if let Ok(metadata) = std::fs::metadata(&old_path) {
                                    if metadata.len() == 0 {
                                        warn!("跳过空的数据库文件: {}", old_path.display());
                                        continue;
                                    }

                                    info!("发现Windows {}数据库 ({}字节)，开始迁移: {} -> {}",
                                          path_type, metadata.len(), old_path.display(), final_db_path.display());

                                    // 安全迁移：先复制，再验证，最后删除
                                    match std::fs::copy(&old_path, &final_db_path) {
                                        Ok(bytes_copied) => {
                                            // 验证复制的完整性
                                            if bytes_copied == metadata.len() {
                                                // 再次验证目标文件存在且大小正确
                                                if let Ok(new_metadata) = std::fs::metadata(&final_db_path) {
                                                    if new_metadata.len() == metadata.len() {
                                                        info!("Windows {}数据库迁移成功！({} 字节)", path_type, bytes_copied);

                                                        // 安全地删除旧文件（只有在新文件验证通过后）
                                                        match std::fs::remove_file(&old_path) {
                                                            Ok(_) => {
                                                                info!("已安全删除旧数据库文件: {}", old_path.display());

                                                                // 尝试删除空目录（如果完全为空的话）
                                                                if let Some(parent) = old_path.parent() {
                                                                    if let Ok(entries) = std::fs::read_dir(parent) {
                                                                        let entry_count = entries.count();
                                                                        if entry_count == 0 {
                                                                            if let Ok(_) = std::fs::remove_dir(parent) {
                                                                                info!("已清理空的旧目录: {}", parent.display());
                                                                            }
                                                                        } else {
                                                                            info!("旧目录不为空 ({} 项)，保留: {}", entry_count, parent.display());
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                warn!("删除旧数据库文件失败（但迁移成功）: {} - {}", old_path.display(), e);
                                                            }
                                                        }

                                                        break; // 找到并迁移了一个，就停止
                                                    } else {
                                                        error!("迁移后文件大小不匹配！原:{} 新:{}", metadata.len(), new_metadata.len());
                                                        // 删除可能损坏的文件
                                                        let _ = std::fs::remove_file(&final_db_path);
                                                    }
                                                } else {
                                                    error!("迁移后无法验证目标文件");
                                                    let _ = std::fs::remove_file(&final_db_path);
                                                }
                                            } else {
                                                error!("复制的字节数不匹配！期望:{} 实际:{}", metadata.len(), bytes_copied);
                                                let _ = std::fs::remove_file(&final_db_path);
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Windows {}数据库迁移失败: {}", path_type, e);
                                        }
                                    }
                                } else {
                                    warn!("无法读取旧数据库文件信息: {}", old_path.display());
                                }
                            }
                        }
                    }

                    #[cfg(not(target_os = "windows"))]
                    {
                        // Unix: 检查是否存在其他可能的错误路径（如果有的话）
                        // 当前Unix路径应该是正确的，但为了完整性保留此代码块
                    }
                }
            }

            // 检查数据库文件状态
            match std::fs::metadata(&final_db_path) {
                Ok(metadata) => {
                    info!("数据库文件已存在: {}, 大小: {} bytes", final_db_path.display(), metadata.len());
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        info!("数据库文件不存在，SQLite 将在连接时自动创建: {}", final_db_path.display());
                    } else {
                        warn!("检查数据库文件时出现问题 {}: {}", final_db_path.display(), e);
                    }
                }
            }
            
            // 修复：使用正确的 SQLite URL 格式
            #[cfg(windows)]
            {
                // Windows 路径处理：将反斜杠转换为正斜杠，并使用正确的 SQLite URL 格式
                let normalized_path = final_db_path.display().to_string().replace('\\', "/");
                database_url = format!("sqlite:///{}?mode=rwc", normalized_path);
            }
            #[cfg(not(windows))]
            {
                database_url = format!("sqlite:///{}?mode=rwc", final_db_path.display());
            }
            
            info!("最终数据库URL: {}", database_url);
            
            // 确保数据库所在目录存在且可写
            if let Some(parent) = final_db_path.parent() {
                if !parent.exists() {
                    info!("创建数据库目录: {}", parent.display());
                    std::fs::create_dir_all(parent)
                        .map_err(|e| SqlxError::Configuration(format!("Failed to create database directory {}: {}", parent.display(), e).into()))?;
                } else {
                    info!("数据库目录已存在: {}", parent.display());
                }
            }
            
            // 检查数据库文件是否可访问（仅记录状态，不进行测试创建）
            match std::fs::metadata(&final_db_path) {
                Ok(metadata) => {
                    info!("数据库文件已存在: {}, 大小: {} bytes", final_db_path.display(), metadata.len());
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        info!("数据库文件不存在，SQLite 将在连接时自动创建: {}", final_db_path.display());
                    } else {
                        warn!("检查数据库文件时出现问题 {}: {}", final_db_path.display(), e);
                    }
                }
            }
        }
        
        info!("尝试连接数据库: {}", database_url);
        
        let pool = match SqlitePool::connect(&database_url).await {
            Ok(pool) => {
                info!("数据库连接成功");
                pool
            },
            Err(e) => {
                error!("数据库连接失败，URL: {}, 错误: {}", database_url, e);
                
                // 如果是相对路径，打印绝对路径信息
                if database_url.starts_with("sqlite:") {
                    let db_path = database_url.replace("sqlite:", "");
                    let absolute_path = std::path::Path::new(&db_path).canonicalize()
                        .unwrap_or_else(|_| PathBuf::from(&db_path));
                    error!("数据库文件绝对路径: {}", absolute_path.display());
                    
                    // 检查目录权限
                    if let Some(parent) = std::path::Path::new(&db_path).parent() {
                        match std::fs::metadata(parent) {
                            Ok(metadata) => {
                                error!("父目录 {} 存在，权限: {:?}", parent.display(), metadata.permissions());
                            },
                            Err(e) => {
                                error!("父目录 {} 不可访问: {}", parent.display(), e);
                            }
                        }
                    }
                    
                    // 提供诊断建议
                    warn!("数据库连接失败，可能的原因:");
                    warn!("1. 路径权限问题");
                    warn!("2. SQLite 版本不兼容"); 
                    warn!("3. 文件被其他进程占用");
                }
                
                return Err(e);
            }
        };
        
        let db = Self { pool };

        info!("开始数据库迁移和初始化");
        match db.migrate().await {
            Ok(_) => info!("数据库迁移和初始化完成"),
            Err(e) => {
                error!("数据库迁移和初始化失败: {}", e);
                return Err(e);
            }
        }

        Ok(db)
    }

    async fn initialize(&self) -> Result<(), SqlxError> {
        // 启用外键约束
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.pool)
            .await?;
        
        info!("已启用SQLite外键约束");
        
        // Create accounts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                token TEXT NOT NULL,
                base_url TEXT NOT NULL,
                model TEXT NOT NULL DEFAULT '',
                is_active BOOLEAN NOT NULL DEFAULT FALSE,
                custom_env_vars TEXT DEFAULT '{}',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create directories table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS directories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT FALSE,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create base_urls table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS base_urls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                url TEXT NOT NULL UNIQUE,
                description TEXT,
                api_key TEXT NOT NULL DEFAULT 'ANTHROPIC_API_KEY',
                is_default BOOLEAN NOT NULL DEFAULT FALSE,
                default_env_vars TEXT DEFAULT '{}',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create account_directories table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS account_directories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                directory_id INTEGER NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE,
                FOREIGN KEY (directory_id) REFERENCES directories (id) ON DELETE CASCADE,
                UNIQUE(account_id, directory_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create claude_settings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS claude_settings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                settings_json TEXT NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create webdav_configs table for WebDAV synchronization
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS webdav_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                url TEXT NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                remote_path TEXT NOT NULL DEFAULT '/claude-config',
                auto_sync BOOLEAN NOT NULL DEFAULT FALSE,
                sync_interval INTEGER NOT NULL DEFAULT 3600,
                is_active BOOLEAN NOT NULL DEFAULT FALSE,
                last_sync_at DATETIME,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create sync_logs table for tracking synchronization history
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sync_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                webdav_config_id INTEGER NOT NULL,
                sync_type TEXT NOT NULL CHECK(sync_type IN ('upload', 'download', 'auto')),
                status TEXT NOT NULL CHECK(status IN ('success', 'failed', 'pending')),
                message TEXT,
                synced_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (webdav_config_id) REFERENCES webdav_configs (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Initialize only essential default data
        self.initialize_default_base_urls().await?;
        // 不再初始化示例账号和目录数据

        // 输出初始化完成信息
        let base_url_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM base_urls")
            .fetch_one(&self.pool).await?;

        println!("数据库初始化完成 - 默认 API 端点: {} 个", base_url_count);
        println!("数据库已就绪，请在界面中添加您的账号和项目目录");

        Ok(())
    }

    /// 迁移数据库，确保所有表都存在
    pub async fn migrate(&self) -> Result<(), SqlxError> {
        info!("开始数据库迁移和初始化");

        // 首先运行初始化，确保所有表都存在（使用 IF NOT EXISTS，不会影响现有表）
        self.initialize().await?;

        // 然后检查并添加可能缺失的字段（针对已有数据库的升级）

        // 检查 accounts 表是否存在 model 字段
        let has_model_field_result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM pragma_table_info('accounts') WHERE name = 'model'"
        )
        .fetch_one(&self.pool)
        .await;

        match has_model_field_result {
            Ok(count) => {
                if count == 0 {
                    // 添加 model 字段
                    info!("检测到 accounts 表缺少 model 字段，开始添加...");
                    sqlx::query("ALTER TABLE accounts ADD COLUMN model TEXT NOT NULL DEFAULT ''")
                        .execute(&self.pool)
                        .await?;
                    info!("已成功添加 model 字段到 accounts 表");
                } else {
                    info!("accounts 表已包含 model 字段，无需添加");
                }
            }
            Err(e) => {
                warn!("检查 accounts 表 model 字段时出错，表可能不存在: {}", e);
            }
        }

        // 检查 base_urls 表是否存在 api_key 字段
        let has_api_key_field_result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM pragma_table_info('base_urls') WHERE name = 'api_key'"
        )
        .fetch_one(&self.pool)
        .await;

        match has_api_key_field_result {
            Ok(count) => {
                if count == 0 {
                    // 添加 api_key 字段
                    info!("检测到 base_urls 表缺少 api_key 字段，开始添加...");
                    sqlx::query("ALTER TABLE base_urls ADD COLUMN api_key TEXT NOT NULL DEFAULT 'ANTHROPIC_API_KEY'")
                        .execute(&self.pool)
                        .await?;
                    info!("已成功添加 api_key 字段到 base_urls 表");
                } else {
                    info!("base_urls 表已包含 api_key 字段，无需添加");
                }
            }
            Err(e) => {
                warn!("检查 base_urls 表 api_key 字段时出错，表可能不存在: {}", e);
            }
        }

        // 检查 accounts 表是否存在 custom_env_vars 字段
        let has_custom_env_vars_field_result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM pragma_table_info('accounts') WHERE name = 'custom_env_vars'"
        )
        .fetch_one(&self.pool)
        .await;

        match has_custom_env_vars_field_result {
            Ok(count) => {
                if count == 0 {
                    // 添加 custom_env_vars 字段
                    info!("检测到 accounts 表缺少 custom_env_vars 字段，开始添加...");
                    sqlx::query("ALTER TABLE accounts ADD COLUMN custom_env_vars TEXT DEFAULT '{}'")
                        .execute(&self.pool)
                        .await?;
                    info!("已成功添加 custom_env_vars 字段到 accounts 表");
                } else {
                    info!("accounts 表已包含 custom_env_vars 字段，无需添加");
                }
            }
            Err(e) => {
                warn!("检查 accounts 表 custom_env_vars 字段时出错，表可能不存在: {}", e);
            }
        }

        // 检查 base_urls 表是否存在 default_env_vars 字段
        let has_default_env_vars_field_result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM pragma_table_info('base_urls') WHERE name = 'default_env_vars'"
        )
        .fetch_one(&self.pool)
        .await;

        match has_default_env_vars_field_result {
            Ok(count) => {
                if count == 0 {
                    // 添加 default_env_vars 字段
                    info!("检测到 base_urls 表缺少 default_env_vars 字段，开始添加...");
                    sqlx::query("ALTER TABLE base_urls ADD COLUMN default_env_vars TEXT DEFAULT '{}'")
                        .execute(&self.pool)
                        .await?;
                    info!("已成功添加 default_env_vars 字段到 base_urls 表");
                } else {
                    info!("base_urls 表已包含 default_env_vars 字段，无需添加");
                }
            }
            Err(e) => {
                warn!("检查 base_urls 表 default_env_vars 字段时出错，表可能不存在: {}", e);
            }
        }

        info!("数据库迁移完成");
        Ok(())
    }

    async fn initialize_default_base_urls(&self) -> Result<(), SqlxError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM base_urls")
            .fetch_one(&self.pool)
            .await?;

        if count == 0 {
            let default_urls = vec![
                ("Anthropic官方", "https://api.anthropic.com", "Anthropic官方API地址", "ANTHROPIC_API_KEY", true),
                // 只保留官方API端点，移除网页版
            ];

            for (name, url, description, api_key, is_default) in default_urls {
                sqlx::query(
                    "INSERT INTO base_urls (name, url, description, api_key, is_default, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(name)
                .bind(url)
                .bind(description)
                .bind(api_key)
                .bind(is_default)
                .bind(Utc::now())
                .bind(Utc::now())
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }


    // Account methods
    pub async fn get_accounts(&self, request: GetAccountsRequest) -> Result<AccountsResponse, SqlxError> {
        let page = request.page.unwrap_or(1).max(1);
        let per_page = request.per_page.unwrap_or(10).max(1).min(100);
        let offset = (page - 1) * per_page;

        let mut query = "SELECT * FROM accounts WHERE 1=1".to_string();
        let mut count_query = "SELECT COUNT(*) FROM accounts WHERE 1=1".to_string();
        let mut params = Vec::new();

        if let Some(search) = &request.search {
            if !search.is_empty() {
                query.push_str(" AND (name LIKE ? OR token LIKE ?)");
                count_query.push_str(" AND (name LIKE ? OR token LIKE ?)");
                let search_pattern = format!("%{}%", search);
                params.push(search_pattern.clone());
                params.push(search_pattern);
            }
        }

        if let Some(base_url) = &request.base_url {
            if !base_url.is_empty() {
                query.push_str(" AND base_url = ?");
                count_query.push_str(" AND base_url = ?");
                params.push(base_url.clone());
            }
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        

        let total: i64 = {
            let mut q = sqlx::query_scalar(&count_query);
            for param in &params {
                q = q.bind(param);
            }
            q.fetch_one(&self.pool).await?
        };

        let accounts: Vec<Account> = {
            let mut q = sqlx::query_as(&query);
            for param in &params {
                q = q.bind(param);
            }
            q.bind(per_page).bind(offset).fetch_all(&self.pool).await?
        };

        let pages = (total + per_page - 1) / per_page;
        let has_prev = page > 1;
        let has_next = page < pages;
        let prev_num = if has_prev { Some(page - 1) } else { None };
        let next_num = if has_next { Some(page + 1) } else { None };

        Ok(AccountsResponse {
            accounts,
            pagination: PaginationInfo {
                page,
                per_page,
                total,
                pages,
                has_prev,
                has_next,
                prev_num,
                next_num,
            },
        })
    }

    pub async fn create_account(&self, request: CreateAccountRequest) -> Result<Account, SqlxError> {
        let now = Utc::now();

        // 处理自定义环境变量
        let custom_env_vars_json = if let Some(env_vars) = request.custom_env_vars {
            serde_json::to_string(&env_vars).unwrap_or_else(|_| "{}".to_string())
        } else {
            "{}".to_string()
        };

        let result = sqlx::query(
            "INSERT INTO accounts (name, token, base_url, model, custom_env_vars, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.name)
        .bind(&request.token)
        .bind(&request.base_url)
        .bind(&request.model)
        .bind(&custom_env_vars_json)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
            .bind(result.last_insert_rowid())
            .fetch_one(&self.pool)
            .await?;

        Ok(account)
    }

    pub async fn update_account(&self, id: i64, request: UpdateAccountRequest) -> Result<Account, SqlxError> {
        let now = Utc::now();
        let mut updates = Vec::new();

        if let Some(_name) = &request.name {
            updates.push("name = ?");
        }
        if let Some(_token) = &request.token {
            updates.push("token = ?");
        }
        if let Some(_base_url) = &request.base_url {
            updates.push("base_url = ?");
        }
        if let Some(_model) = &request.model {
            updates.push("model = ?");
        }
        // 检查自定义环境变量，如果为空对象则跳过更新
        if let Some(custom_env_vars) = &request.custom_env_vars {
            // 将 serde_json::Value 转换为字符串以检查是否为空
            if let Ok(json_str) = serde_json::to_string(custom_env_vars) {
                // 只有非空对象才添加更新
                if json_str.trim() != "{}" {
                    updates.push("custom_env_vars = ?");
                }
            } else {
                // JSON序列化失败，跳过更新
                tracing::warn!("自定义环境变量序列化失败，跳过更新");
            }
        }

        if updates.is_empty() {
            return self.get_account(id).await;
        }

        updates.push("updated_at = ?");
        let query = format!("UPDATE accounts SET {} WHERE id = ?", updates.join(", "));

        let mut q = sqlx::query(&query);

        if let Some(name) = &request.name {
            q = q.bind(name);
        }
        if let Some(token) = &request.token {
            q = q.bind(token);
        }
        if let Some(base_url) = &request.base_url {
            q = q.bind(base_url);
        }
        if let Some(model) = &request.model {
            q = q.bind(model);
        }
        // 只有非空环境变量才绑定参数
        if let Some(custom_env_vars) = &request.custom_env_vars {
            if let Ok(json_str) = serde_json::to_string(custom_env_vars) {
                if json_str.trim() != "{}" {
                    let custom_env_vars_json = serde_json::to_string(custom_env_vars)
                        .unwrap_or_else(|_| "{}".to_string());
                    q = q.bind(custom_env_vars_json);
                }
            }
        }

        q = q.bind(now).bind(id);
        q.execute(&self.pool).await?;

        self.get_account(id).await
    }

    pub async fn get_account(&self, id: i64) -> Result<Account, SqlxError> {
        sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn delete_account(&self, id: i64) -> Result<(), SqlxError> {
        // 启用外键约束
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.pool)
            .await?;
        
        // 检查是否有关联的账号-目录记录
        let association_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM account_directories WHERE account_id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        
        if association_count > 0 {
            // 先删除关联记录
            info!("删除账号 {} 的关联记录，共 {} 条", id, association_count);
            sqlx::query("DELETE FROM account_directories WHERE account_id = ?")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        
        // 删除账号记录
        let result = sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
            
        if result.rows_affected() == 0 {
            return Err(SqlxError::RowNotFound);
        }
        
        info!("成功删除账号，ID: {}", id);
        Ok(())
    }

    pub async fn get_account_base_urls(&self) -> Result<Vec<String>, SqlxError> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT DISTINCT base_url FROM accounts WHERE base_url IS NOT NULL")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|(url,)| url).collect())
    }

    // Directory methods
    pub async fn get_directories(&self) -> Result<Vec<Directory>, SqlxError> {
        sqlx::query_as::<_, Directory>("SELECT * FROM directories ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create_directory(&self, request: CreateDirectoryRequest) -> Result<Directory, SqlxError> {
        let now = Utc::now();
        let result = sqlx::query(
            "INSERT INTO directories (path, name, created_at, updated_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&request.path)
        .bind(&request.name)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let directory = sqlx::query_as::<_, Directory>("SELECT * FROM directories WHERE id = ?")
            .bind(result.last_insert_rowid())
            .fetch_one(&self.pool)
            .await?;

        Ok(directory)
    }

    pub async fn update_directory(&self, id: i64, request: UpdateDirectoryRequest) -> Result<Directory, SqlxError> {
        let now = Utc::now();
        let mut updates = Vec::new();

        if let Some(_path) = &request.path {
            updates.push("path = ?");
        }
        if let Some(_name) = &request.name {
            updates.push("name = ?");
        }

        if updates.is_empty() {
            return self.get_directory(id).await;
        }

        updates.push("updated_at = ?");
        let query = format!("UPDATE directories SET {} WHERE id = ?", updates.join(", "));

        let mut q = sqlx::query(&query);
        
        if let Some(path) = &request.path {
            q = q.bind(path);
        }
        if let Some(name) = &request.name {
            q = q.bind(name);
        }
        
        q = q.bind(now).bind(id);
        q.execute(&self.pool).await?;

        self.get_directory(id).await
    }

    pub async fn get_directory(&self, id: i64) -> Result<Directory, SqlxError> {
        sqlx::query_as::<_, Directory>("SELECT * FROM directories WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn delete_directory(&self, id: i64) -> Result<(), SqlxError> {
        // 启用外键约束
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.pool)
            .await?;
        
        // 先获取目录信息，检查文件系统中是否存在
        let directory = match sqlx::query_as::<_, crate::models::Directory>("SELECT * FROM directories WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await {
            Ok(dir) => dir,
            Err(_) => return Err(SqlxError::RowNotFound),
        };
        
        // 检查目录在文件系统中是否存在
        let path_exists = std::path::Path::new(&directory.path).exists();
        
        if !path_exists {
            info!("目录 '{}' 在文件系统中不存在，将清理数据库记录", directory.path);
        } else {
            info!("目录 '{}' 在文件系统中存在，将进行正常删除", directory.path);
        }
        
        // 检查是否有关联的账号-目录记录
        let association_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM account_directories WHERE directory_id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        
        if association_count > 0 {
            // 先删除关联记录
            info!("删除目录 {} 的关联记录，共 {} 条", id, association_count);
            sqlx::query("DELETE FROM account_directories WHERE directory_id = ?")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        
        // 删除目录记录
        let result = sqlx::query("DELETE FROM directories WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
            
        if result.rows_affected() == 0 {
            return Err(SqlxError::RowNotFound);
        }
        
        if path_exists {
            info!("成功删除目录记录，ID: {}，文件系统中的目录需要手动删除", id);
        } else {
            info!("成功清理不存在的目录记录，ID: {}", id);
        }
        
        Ok(())
    }

    // BaseUrl methods
    pub async fn get_base_urls(&self) -> Result<Vec<BaseUrl>, SqlxError> {
        sqlx::query_as::<_, BaseUrl>("SELECT * FROM base_urls ORDER BY is_default DESC, created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create_base_url(&self, request: CreateBaseUrlRequest) -> Result<BaseUrl, SqlxError> {
        let now = Utc::now();
        let is_default = request.is_default.unwrap_or(false);
        let api_key = request.api_key.unwrap_or_else(|| "ANTHROPIC_API_KEY".to_string());

        // 处理默认环境变量
        let default_env_vars_json = if let Some(env_vars) = request.default_env_vars {
            serde_json::to_string(&env_vars).unwrap_or_else(|_| "{}".to_string())
        } else {
            "{}".to_string()
        };

        // If setting as default, unset other defaults
        if is_default {
            sqlx::query("UPDATE base_urls SET is_default = FALSE")
                .execute(&self.pool)
                .await?;
        }

        let result = sqlx::query(
            "INSERT INTO base_urls (name, url, description, api_key, is_default, default_env_vars, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.name)
        .bind(&request.url)
        .bind(&request.description)
        .bind(&api_key)
        .bind(is_default)
        .bind(&default_env_vars_json)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let base_url = sqlx::query_as::<_, BaseUrl>("SELECT * FROM base_urls WHERE id = ?")
            .bind(result.last_insert_rowid())
            .fetch_one(&self.pool)
            .await?;

        Ok(base_url)
    }

    pub async fn update_base_url(&self, id: i64, request: UpdateBaseUrlRequest) -> Result<BaseUrl, SqlxError> {
        let now = Utc::now();

        // 获取旧的 base_url 信息，用于级联更新账号
        let old_base_url = self.get_base_url(id).await?;
        let old_url = old_base_url.url.clone();

        // If setting as default, unset other defaults
        if let Some(true) = request.is_default {
            sqlx::query("UPDATE base_urls SET is_default = FALSE")
                .execute(&self.pool)
                .await?;
        }

        let mut updates = Vec::new();
        if let Some(_name) = &request.name {
            updates.push("name = ?");
        }
        if let Some(_url) = &request.url {
            updates.push("url = ?");
        }
        if let Some(_description) = &request.description {
            updates.push("description = ?");
        }
        if let Some(_api_key) = &request.api_key {
            updates.push("api_key = ?");
        }
        if let Some(_is_default) = request.is_default {
            updates.push("is_default = ?");
        }
        // 检查默认环境变量，如果为空对象则跳过更新
        if let Some(default_env_vars) = &request.default_env_vars {
            // 将 serde_json::Value 转换为字符串以检查是否为空
            if let Ok(json_str) = serde_json::to_string(default_env_vars) {
                // 只有非空对象才添加更新
                if json_str.trim() != "{}" {
                    updates.push("default_env_vars = ?");
                }
            } else {
                // JSON序列化失败，跳过更新
                tracing::warn!("默认环境变量序列化失败，跳过更新");
            }
        }

        if updates.is_empty() {
            return self.get_base_url(id).await;
        }

        updates.push("updated_at = ?");
        let query = format!("UPDATE base_urls SET {} WHERE id = ?", updates.join(", "));

        let mut q = sqlx::query(&query);

        if let Some(name) = &request.name {
            q = q.bind(name);
        }
        if let Some(url) = &request.url {
            q = q.bind(url);
        }
        if let Some(description) = &request.description {
            q = q.bind(description);
        }
        if let Some(api_key) = &request.api_key {
            q = q.bind(api_key);
        }
        if let Some(is_default) = request.is_default {
            q = q.bind(is_default);
        }
        // 只有非空环境变量才绑定参数
        if let Some(default_env_vars) = &request.default_env_vars {
            if let Ok(json_str) = serde_json::to_string(default_env_vars) {
                if json_str.trim() != "{}" {
                    let default_env_vars_json = serde_json::to_string(default_env_vars)
                        .unwrap_or_else(|_| "{}".to_string());
                    q = q.bind(default_env_vars_json);
                }
            }
        }

        q = q.bind(now).bind(id);
        q.execute(&self.pool).await?;

        // 如果 URL 发生了变化，级联更新所有使用该 URL 的账号
        if let Some(new_url) = &request.url {
            if new_url != &old_url {
                let result = sqlx::query(
                    "UPDATE accounts SET base_url = ?, updated_at = ? WHERE base_url = ?"
                )
                .bind(new_url)
                .bind(now)
                .bind(&old_url)
                .execute(&self.pool)
                .await?;

                let affected_rows = result.rows_affected();
                if affected_rows > 0 {
                    info!("更新 Base URL '{}' 时，级联更新了 {} 个账号的 base_url", old_base_url.name, affected_rows);
                }
            }
        }

        self.get_base_url(id).await
    }

    pub async fn get_base_url(&self, id: i64) -> Result<BaseUrl, SqlxError> {
        sqlx::query_as::<_, BaseUrl>("SELECT * FROM base_urls WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn delete_base_url(&self, id: i64) -> Result<(), SqlxError> {
        // 先获取要删除的 base_url 信息
        let base_url = self.get_base_url(id).await?;

        // 查找使用这个 base_url 的所有账号
        let affected_accounts: Vec<(i64, String)> = sqlx::query_as(
            "SELECT id, name FROM accounts WHERE base_url = ?"
        )
        .bind(&base_url.url)
        .fetch_all(&self.pool)
        .await?;

        if !affected_accounts.is_empty() {
            info!("删除 Base URL '{}' 时，同时删除 {} 个关联的账号", base_url.name, affected_accounts.len());

            // 删除所有使用该 base_url 的账号
            for (account_id, account_name) in affected_accounts {
                info!("删除账号: {} (ID: {})，因为其使用的 Base URL 被删除", account_name, account_id);

                // 删除账号（会自动级联删除关联记录）
                self.delete_account(account_id).await?;
            }
        }

        // 删除 Base URL 记录
        let result = sqlx::query("DELETE FROM base_urls WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(SqlxError::RowNotFound);
        }

        info!("成功删除Base URL '{}' (ID: {})", base_url.name, id);
        Ok(())
    }

    // Switch account functionality
    pub async fn switch_account(&self, request: SwitchAccountRequest) -> Result<String, SqlxError> {
        // Reset all active states
        sqlx::query("UPDATE accounts SET is_active = FALSE")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE directories SET is_active = FALSE")
            .execute(&self.pool)
            .await?;

        // Set new active states
        sqlx::query("UPDATE accounts SET is_active = TRUE WHERE id = ?")
            .bind(request.account_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE directories SET is_active = TRUE WHERE id = ?")
            .bind(request.directory_id)
            .execute(&self.pool)
            .await?;

        // Create or update association
        sqlx::query(
            "INSERT OR IGNORE INTO account_directories (account_id, directory_id, created_at) 
             VALUES (?, ?, ?)"
        )
        .bind(request.account_id)
        .bind(request.directory_id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        // Get account and directory info for response
        let account = self.get_account(request.account_id).await?;
        let directory = self.get_directory(request.directory_id).await?;

        Ok(format!(
            "已切换到账号 {}，目录 {}",
            account.name, directory.name
        ))
    }

    // Association methods
    pub async fn get_associations(&self) -> Result<Vec<HashMap<String, serde_json::Value>>, SqlxError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                ad.id,
                ad.account_id,
                ad.directory_id,
                a.name as account_name,
                d.name as directory_name,
                ad.created_at
            FROM account_directories ad
            JOIN accounts a ON ad.account_id = a.id
            JOIN directories d ON ad.directory_id = d.id
            ORDER BY ad.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut associations = Vec::new();
        for row in rows {
            let mut assoc = HashMap::new();
            assoc.insert("id".to_string(), serde_json::Value::Number(row.get::<i64, _>("id").into()));
            assoc.insert("account_id".to_string(), serde_json::Value::Number(row.get::<i64, _>("account_id").into()));
            assoc.insert("directory_id".to_string(), serde_json::Value::Number(row.get::<i64, _>("directory_id").into()));
            assoc.insert("account_name".to_string(), serde_json::Value::String(row.get("account_name")));
            assoc.insert("directory_name".to_string(), serde_json::Value::String(row.get("directory_name")));
            assoc.insert("created_at".to_string(), serde_json::Value::String(row.get::<DateTime<Utc>, _>("created_at").to_rfc3339()));
            associations.push(assoc);
        }

        Ok(associations)
    }

    // Claude Settings methods
    pub async fn save_claude_settings(&self, settings_json: &str) -> Result<(), SqlxError> {
        // First try to update existing settings
        let result = sqlx::query(
            r#"
            UPDATE claude_settings 
            SET settings_json = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = (SELECT MIN(id) FROM claude_settings)
            "#
        )
        .bind(settings_json)
        .execute(&self.pool)
        .await?;

        // If no rows were affected, insert a new record
        if result.rows_affected() == 0 {
            sqlx::query(
                r#"
                INSERT INTO claude_settings (settings_json)
                VALUES (?)
                "#
            )
            .bind(settings_json)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_claude_settings(&self) -> Result<String, SqlxError> {
        let row = sqlx::query(
            r#"
            SELECT settings_json FROM claude_settings 
            ORDER BY updated_at DESC 
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(row.get("settings_json")),
            None => {
                // Return default settings if no settings exist
                let default_settings = r#"{
                    "permissions": {
                        "defaultMode": "bypassPermissions",
                        "allow": ["*"],
                        "deny": []
                    },
                    "env": {
                        "IS_SANDBOX": "1",
                        "DISABLE_AUTOUPDATER": 1
                    }
                }"#;
                Ok(default_settings.to_string())
            }
        }
    }
}