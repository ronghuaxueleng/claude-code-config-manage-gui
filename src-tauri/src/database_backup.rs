use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPool, sqlite::SqlitePool, FromRow, Row};

// ================================
// Request/Response Types
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountsRequest {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub token: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub token: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Directory {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDirectoryRequest {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDirectoryRequest {
    pub path: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BaseUrl {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBaseUrlRequest {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBaseUrlRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchAccountRequest {
    pub account_id: i64,
    pub directory_id: i64,
}

// ================================
// Database Operations Trait
// ================================

#[async_trait]
pub trait DatabaseOperations {
    // Account operations
    async fn get_accounts(&self, request: GetAccountsRequest) -> Result<AccountsResponse>;
    async fn create_account(&self, request: CreateAccountRequest) -> Result<Account>;
    async fn update_account(&self, id: i64, request: UpdateAccountRequest) -> Result<Account>;
    async fn get_account(&self, id: i64) -> Result<Account>;
    async fn delete_account(&self, id: i64) -> Result<()>;
    async fn get_account_base_urls(&self) -> Result<Vec<String>>;

    // Directory operations
    async fn get_directories(&self) -> Result<Vec<Directory>>;
    async fn create_directory(&self, request: CreateDirectoryRequest) -> Result<Directory>;
    async fn update_directory(&self, id: i64, request: UpdateDirectoryRequest) -> Result<Directory>;
    async fn get_directory(&self, id: i64) -> Result<Directory>;
    async fn delete_directory(&self, id: i64) -> Result<()>;

    // BaseUrl operations
    async fn get_base_urls(&self) -> Result<Vec<BaseUrl>>;
    async fn create_base_url(&self, request: CreateBaseUrlRequest) -> Result<BaseUrl>;
    async fn update_base_url(&self, id: i64, request: UpdateBaseUrlRequest) -> Result<BaseUrl>;
    async fn get_base_url(&self, id: i64) -> Result<BaseUrl>;
    async fn delete_base_url(&self, id: i64) -> Result<()>;

    // Switch functionality
    async fn switch_account(&self, request: SwitchAccountRequest) -> Result<String>;

    // Utility operations
    async fn init_tables(&self) -> Result<()>;
    async fn initialize_default_base_urls(&self) -> Result<()>;
}

// ================================
// SQLite Implementation
// ================================

pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        use sqlx::sqlite::SqlitePoolOptions;
        
        let pool = SqlitePoolOptions::new()
            .max_connections(8)  
            .min_connections(2)   
            .acquire_timeout(std::time::Duration::from_secs(5))   // 更短的超时时间
            .idle_timeout(std::time::Duration::from_secs(180))    
            .max_lifetime(std::time::Duration::from_secs(600))    
            .test_before_acquire(false)  // 禁用测试，加快获取速度
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }
}

#[async_trait] 
impl DatabaseOperations for SqliteDatabase {
    async fn init_tables(&self) -> Result<()> {
        // Create accounts table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                token TEXT NOT NULL,
                base_url TEXT NOT NULL,
                model TEXT NOT NULL DEFAULT 'claude-sonnet-4-20250514',
                is_active BOOLEAN NOT NULL DEFAULT FALSE,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create directories table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS directories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT FALSE,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create base_urls table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS base_urls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                url TEXT NOT NULL UNIQUE,
                description TEXT,
                is_default BOOLEAN NOT NULL DEFAULT FALSE,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create account_directories table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account_directories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                directory_id INTEGER NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE,
                FOREIGN KEY (directory_id) REFERENCES directories (id) ON DELETE CASCADE,
                UNIQUE(account_id, directory_id)
            )
        "#)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn initialize_default_base_urls(&self) -> Result<()> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM base_urls")
            .fetch_one(&self.pool).await?;

        if count == 0 {
            let default_urls = vec![
                ("Anthropic官方", "https://api.anthropic.com", "Anthropic官方API地址", true),
                ("Claude.ai", "https://claude.ai", "Claude.ai网页版", false),
            ];

            for (name, url, description, is_default) in default_urls {
                sqlx::query(
                    "INSERT INTO base_urls (name, url, description, is_default, created_at, updated_at) 
                     VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(name)
                .bind(url)
                .bind(description)
                .bind(is_default)
                .bind(Utc::now())
                .bind(Utc::now())
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    async fn get_accounts(&self, request: GetAccountsRequest) -> Result<AccountsResponse> {
        let page = request.page.unwrap_or(1).max(1);
        let per_page = request.per_page.unwrap_or(10).max(1).min(100);
        let offset = (page - 1) * per_page;

        let mut query = "SELECT id, name, token, base_url, model, is_active, created_at, updated_at FROM accounts WHERE 1=1".to_string();
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
            q = q.bind(per_page).bind(offset);
            q.fetch_all(&self.pool).await?
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

    async fn create_account(&self, request: CreateAccountRequest) -> Result<Account> {
        let now = Utc::now();
        let result = sqlx::query(
            "INSERT INTO accounts (name, token, base_url, model, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.name)
        .bind(&request.token)
        .bind(&request.base_url)
        .bind(&request.model)
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

    async fn update_account(&self, id: i64, request: UpdateAccountRequest) -> Result<Account> {
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
        
        q = q.bind(now).bind(id);
        q.execute(&self.pool).await?;

        self.get_account(id).await
    }

    async fn get_account(&self, id: i64) -> Result<Account> {
        let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(account)
    }

    async fn delete_account(&self, id: i64) -> Result<()> {
        // SQLite版本需要显式启用外键约束
        if std::any::type_name::<Self>().contains("Sqlite") {
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&self.pool)
                .await?;
        }
        
        // 检查是否有关联的账号-目录记录
        let association_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM account_directories WHERE account_id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        
        if association_count > 0 {
            // 先删除关联记录
            tracing::info!("删除账号 {} 的关联记录，共 {} 条", id, association_count);
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
            return Err(anyhow::anyhow!("账号不存在或已被删除，ID: {}", id).into());
        }
        
        tracing::info!("成功删除账号，ID: {}", id);
        Ok(())
    }

    async fn get_account_base_urls(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT DISTINCT base_url FROM accounts WHERE base_url IS NOT NULL")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|(url,)| url).collect())
    }

    async fn get_directories(&self) -> Result<Vec<Directory>> {
        let directories = sqlx::query_as::<_, Directory>("SELECT * FROM directories ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(directories)
    }

    async fn create_directory(&self, request: CreateDirectoryRequest) -> Result<Directory> {
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

    async fn update_directory(&self, id: i64, request: UpdateDirectoryRequest) -> Result<Directory> {
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

    async fn get_directory(&self, id: i64) -> Result<Directory> {
        let directory = sqlx::query_as::<_, Directory>("SELECT * FROM directories WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(directory)
    }

    async fn delete_directory(&self, id: i64) -> Result<()> {
        // SQLite版本需要显式启用外键约束
        if std::any::type_name::<Self>().contains("Sqlite") {
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&self.pool)
                .await?;
        }
        
        // 先获取目录信息，检查文件系统中是否存在
        let directory = match sqlx::query_as::<_, crate::models::Directory>("SELECT * FROM directories WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await {
            Ok(dir) => dir,
            Err(_) => return Err(anyhow::anyhow!("目录记录不存在，ID: {}", id).into()),
        };
        
        // 检查目录在文件系统中是否存在
        let path_exists = std::path::Path::new(&directory.path).exists();
        
        if !path_exists {
            tracing::info!("目录 '{}' 在文件系统中不存在，将清理数据库记录", directory.path);
        } else {
            tracing::info!("目录 '{}' 在文件系统中存在，将进行正常删除", directory.path);
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
            tracing::info!("删除目录 {} 的关联记录，共 {} 条", id, association_count);
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
            return Err(anyhow::anyhow!("目录记录不存在或已被删除，ID: {}", id).into());
        }
        
        if path_exists {
            tracing::info!("成功删除目录记录，ID: {}，文件系统中的目录需要手动删除", id);
        } else {
            tracing::info!("成功清理不存在的目录记录，ID: {}", id);
        }
        
        Ok(())
    }

    async fn get_base_urls(&self) -> Result<Vec<BaseUrl>> {
        let base_urls = sqlx::query_as::<_, BaseUrl>("SELECT * FROM base_urls ORDER BY is_default DESC, created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(base_urls)
    }

    async fn create_base_url(&self, request: CreateBaseUrlRequest) -> Result<BaseUrl> {
        let now = Utc::now();
        let is_default = request.is_default.unwrap_or(false);

        // If setting as default, unset other defaults
        if is_default {
            sqlx::query("UPDATE base_urls SET is_default = FALSE")
                .execute(&self.pool)
                .await?;
        }

        let result = sqlx::query(
            "INSERT INTO base_urls (name, url, description, is_default, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.name)
        .bind(&request.url)
        .bind(&request.description)
        .bind(is_default)
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

    async fn update_base_url(&self, id: i64, request: UpdateBaseUrlRequest) -> Result<BaseUrl> {
        let now = Utc::now();
        
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
        if let Some(_is_default) = request.is_default {
            updates.push("is_default = ?");
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
        if let Some(is_default) = request.is_default {
            q = q.bind(is_default);
        }
        
        q = q.bind(now).bind(id);
        q.execute(&self.pool).await?;

        self.get_base_url(id).await
    }

    async fn get_base_url(&self, id: i64) -> Result<BaseUrl> {
        let base_url = sqlx::query_as::<_, BaseUrl>("SELECT * FROM base_urls WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(base_url)
    }

    async fn delete_base_url(&self, id: i64) -> Result<()> {
        // Base URL表通常没有外键引用，但仍添加验证
        let result = sqlx::query("DELETE FROM base_urls WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
            
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Base URL不存在或已被删除，ID: {}", id).into());
        }
        
        tracing::info!("成功删除Base URL，ID: {}", id);
        Ok(())
    }

    async fn switch_account(&self, request: SwitchAccountRequest) -> Result<String> {
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

        // Create or update association using INSERT OR REPLACE
        sqlx::query("INSERT OR REPLACE INTO account_directories (account_id, directory_id, created_at) VALUES (?, ?, ?)")
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
}

// ================================
// MySQL Implementation
// ================================

pub struct MySqlDatabase {
    pool: MySqlPool,
}

impl MySqlDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        use sqlx::mysql::MySqlPoolOptions;
        
        // 添加连接参数以提高性能
        let mut url_with_params = database_url.to_string();
        if !url_with_params.contains('?') {
            url_with_params.push('?');
        } else {
            url_with_params.push('&');
        }
        
        // 添加性能优化参数 - 禁用SSL并设置更短的超时
        url_with_params.push_str("sslmode=disabled&connect_timeout=2&read_timeout=5&write_timeout=5&tcp_keepalive=60");
        
        println!("MySQL连接URL: {}", url_with_params);
        
        let pool = MySqlPoolOptions::new()
            .max_connections(3)    // 大幅减少连接数，避免创建太多连接
            .min_connections(1)    // 保持1个最小连接
            .acquire_timeout(std::time::Duration::from_secs(2))   // 非常短的超时
            .idle_timeout(std::time::Duration::from_secs(60))     // 短空闲超时
            .max_lifetime(std::time::Duration::from_secs(180))    // 短生命周期
            .test_before_acquire(false)  
            .connect(&url_with_params)
            .await?;
            
        println!("MySQL连接池创建成功，连接数: {}", pool.size());
        Ok(Self { pool })
    }
}

#[async_trait]
impl DatabaseOperations for MySqlDatabase {
    async fn init_tables(&self) -> Result<()> {
        // 检查并确保外键约束已启用
        sqlx::query("SET foreign_key_checks = 1")
            .execute(&self.pool)
            .await?;
        
        tracing::info!("已确保MySQL外键约束启用");
        
        // Create accounts table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                name VARCHAR(255) NOT NULL UNIQUE,
                token TEXT NOT NULL,
                base_url TEXT NOT NULL,
                model VARCHAR(255) NOT NULL DEFAULT 'claude-sonnet-4-20250514',
                is_active BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create directories table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS directories (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                path VARCHAR(500) NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create base_urls table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS base_urls (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                name VARCHAR(255) NOT NULL UNIQUE,
                url VARCHAR(500) NOT NULL UNIQUE,
                description TEXT,
                is_default BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create account_directories table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS account_directories (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                account_id BIGINT NOT NULL,
                directory_id BIGINT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE,
                FOREIGN KEY (directory_id) REFERENCES directories (id) ON DELETE CASCADE,
                UNIQUE(account_id, directory_id)
            )
        "#)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn initialize_default_base_urls(&self) -> Result<()> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM base_urls")
            .fetch_one(&self.pool).await?;

        if count == 0 {
            let default_urls = vec![
                ("Anthropic官方", "https://api.anthropic.com", "Anthropic官方API地址", true),
                ("Claude.ai", "https://claude.ai", "Claude.ai网页版", false),
            ];

            for (name, url, description, is_default) in default_urls {
                sqlx::query(
                    "INSERT INTO base_urls (name, url, description, is_default, created_at, updated_at) 
                     VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(name)
                .bind(url)
                .bind(description)
                .bind(is_default)
                .bind(Utc::now())
                .bind(Utc::now())
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    async fn get_accounts(&self, request: GetAccountsRequest) -> Result<AccountsResponse> {
        let page = request.page.unwrap_or(1).max(1);
        let per_page = request.per_page.unwrap_or(10).max(1).min(100);
        let offset = (page - 1) * per_page;

        let mut query = "SELECT id, name, token, base_url, model, is_active, created_at, updated_at FROM accounts WHERE 1=1".to_string();
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
            q = q.bind(per_page).bind(offset);
            q.fetch_all(&self.pool).await?
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

    async fn create_account(&self, request: CreateAccountRequest) -> Result<Account> {
        let now = Utc::now();
        let result = sqlx::query(
            "INSERT INTO accounts (name, token, base_url, model, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.name)
        .bind(&request.token)
        .bind(&request.base_url)
        .bind(&request.model)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
            .bind(result.last_insert_id())
            .fetch_one(&self.pool)
            .await?;

        Ok(account)
    }

    async fn update_account(&self, id: i64, request: UpdateAccountRequest) -> Result<Account> {
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
        
        q = q.bind(now).bind(id);
        q.execute(&self.pool).await?;

        self.get_account(id).await
    }

    async fn get_account(&self, id: i64) -> Result<Account> {
        let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(account)
    }

    async fn delete_account(&self, id: i64) -> Result<()> {
        // SQLite版本需要显式启用外键约束
        if std::any::type_name::<Self>().contains("Sqlite") {
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&self.pool)
                .await?;
        }
        
        // 检查是否有关联的账号-目录记录
        let association_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM account_directories WHERE account_id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        
        if association_count > 0 {
            // 先删除关联记录
            tracing::info!("删除账号 {} 的关联记录，共 {} 条", id, association_count);
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
            return Err(anyhow::anyhow!("账号不存在或已被删除，ID: {}", id).into());
        }
        
        tracing::info!("成功删除账号，ID: {}", id);
        Ok(())
    }

    async fn get_account_base_urls(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT DISTINCT base_url FROM accounts WHERE base_url IS NOT NULL")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|(url,)| url).collect())
    }

    async fn get_directories(&self) -> Result<Vec<Directory>> {
        let directories = sqlx::query_as::<_, Directory>("SELECT * FROM directories ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(directories)
    }

    async fn create_directory(&self, request: CreateDirectoryRequest) -> Result<Directory> {
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
            .bind(result.last_insert_id())
            .fetch_one(&self.pool)
            .await?;

        Ok(directory)
    }

    async fn update_directory(&self, id: i64, request: UpdateDirectoryRequest) -> Result<Directory> {
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

    async fn get_directory(&self, id: i64) -> Result<Directory> {
        let directory = sqlx::query_as::<_, Directory>("SELECT * FROM directories WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(directory)
    }

    async fn delete_directory(&self, id: i64) -> Result<()> {
        // SQLite版本需要显式启用外键约束
        if std::any::type_name::<Self>().contains("Sqlite") {
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&self.pool)
                .await?;
        }
        
        // 先获取目录信息，检查文件系统中是否存在
        let directory = match sqlx::query_as::<_, crate::models::Directory>("SELECT * FROM directories WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await {
            Ok(dir) => dir,
            Err(_) => return Err(anyhow::anyhow!("目录记录不存在，ID: {}", id).into()),
        };
        
        // 检查目录在文件系统中是否存在
        let path_exists = std::path::Path::new(&directory.path).exists();
        
        if !path_exists {
            tracing::info!("目录 '{}' 在文件系统中不存在，将清理数据库记录", directory.path);
        } else {
            tracing::info!("目录 '{}' 在文件系统中存在，将进行正常删除", directory.path);
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
            tracing::info!("删除目录 {} 的关联记录，共 {} 条", id, association_count);
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
            return Err(anyhow::anyhow!("目录记录不存在或已被删除，ID: {}", id).into());
        }
        
        if path_exists {
            tracing::info!("成功删除目录记录，ID: {}，文件系统中的目录需要手动删除", id);
        } else {
            tracing::info!("成功清理不存在的目录记录，ID: {}", id);
        }
        
        Ok(())
    }

    async fn get_base_urls(&self) -> Result<Vec<BaseUrl>> {
        let base_urls = sqlx::query_as::<_, BaseUrl>("SELECT * FROM base_urls ORDER BY is_default DESC, created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(base_urls)
    }

    async fn create_base_url(&self, request: CreateBaseUrlRequest) -> Result<BaseUrl> {
        let now = Utc::now();
        let is_default = request.is_default.unwrap_or(false);

        // If setting as default, unset other defaults
        if is_default {
            sqlx::query("UPDATE base_urls SET is_default = FALSE")
                .execute(&self.pool)
                .await?;
        }

        let result = sqlx::query(
            "INSERT INTO base_urls (name, url, description, is_default, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.name)
        .bind(&request.url)
        .bind(&request.description)
        .bind(is_default)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let base_url = sqlx::query_as::<_, BaseUrl>("SELECT * FROM base_urls WHERE id = ?")
            .bind(result.last_insert_id())
            .fetch_one(&self.pool)
            .await?;

        Ok(base_url)
    }

    async fn update_base_url(&self, id: i64, request: UpdateBaseUrlRequest) -> Result<BaseUrl> {
        let now = Utc::now();
        
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
        if let Some(_is_default) = request.is_default {
            updates.push("is_default = ?");
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
        if let Some(is_default) = request.is_default {
            q = q.bind(is_default);
        }
        
        q = q.bind(now).bind(id);
        q.execute(&self.pool).await?;

        self.get_base_url(id).await
    }

    async fn get_base_url(&self, id: i64) -> Result<BaseUrl> {
        let base_url = sqlx::query_as::<_, BaseUrl>("SELECT * FROM base_urls WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(base_url)
    }

    async fn delete_base_url(&self, id: i64) -> Result<()> {
        // Base URL表通常没有外键引用，但仍添加验证
        let result = sqlx::query("DELETE FROM base_urls WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
            
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Base URL不存在或已被删除，ID: {}", id).into());
        }
        
        tracing::info!("成功删除Base URL，ID: {}", id);
        Ok(())
    }

    async fn switch_account(&self, request: SwitchAccountRequest) -> Result<String> {
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

        // Create or update association using REPLACE INTO (MySQL syntax)
        sqlx::query("REPLACE INTO account_directories (account_id, directory_id, created_at) VALUES (?, ?, ?)")
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
}

// ================================
// Retry Helper Function
// ================================

async fn retry_with_backoff<F, Fut, T>(f: F, max_retries: u32) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    // 更短的退避时间，快速重试
                    let delay = std::time::Duration::from_millis((50 + (attempt * 50)) as u64); 
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

// ================================
// Unified Database Interface
// ================================

pub enum Database {
    Sqlite(SqliteDatabase),
    MySql(MySqlDatabase),
}

impl Database {
    pub async fn new() -> Result<Self> {
        use crate::config_manager::ConfigManager;
        
        // 使用配置管理器获取数据库配置
        let config_manager = ConfigManager::new();
        let db_config = config_manager.get_default_database_config()
            .ok_or_else(|| anyhow::anyhow!("No database configuration found"))?;
        
        let mut database_url = db_config.url.clone();
        println!("初始化数据库连接: {}", database_url);
        
        // 处理MySQL URL格式转换
        if database_url.starts_with("mysql+pymysql://") {
            database_url = database_url.replace("mysql+pymysql://", "mysql://");
        }
        
        // 处理SQLite相对路径
        if database_url.starts_with("sqlite:///") && !database_url.starts_with("sqlite:////") {
            // 获取数据库文件名
            let db_filename = database_url.replace("sqlite:///", "");
            
            // 首先尝试从resources目录获取数据库文件，但要复制到可写位置
            if let Some(resource_db_path) = ConfigManager::get_resource_path(&db_filename) {
                // 将resources中的数据库复制到instance目录，确保可写
                let current_dir = std::env::current_dir()
                    .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
                
                let instance_dir = current_dir.join("instance");
                std::fs::create_dir_all(&instance_dir)
                    .map_err(|e| anyhow::anyhow!("Failed to create instance directory: {}", e))?;
                
                let writable_db_path = instance_dir.join(&db_filename);
                
                // 如果实例数据库不存在，从resources复制
                if !writable_db_path.exists() {
                    println!("从resources复制数据库到: {}", writable_db_path.display());
                    std::fs::copy(&resource_db_path, &writable_db_path)
                        .map_err(|e| anyhow::anyhow!("Failed to copy database from resources: {}", e))?;
                }
                
                database_url = format!("sqlite:{}", writable_db_path.display());
            } else {
                // 如果resources中没有，则使用原来的instance目录逻辑
                let current_dir = std::env::current_dir()
                    .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
                
                let instance_dir = current_dir.join("instance");
                std::fs::create_dir_all(&instance_dir)
                    .map_err(|e| anyhow::anyhow!("Failed to create instance directory: {}", e))?;
                
                let db_path = instance_dir.join(&db_filename);
                database_url = format!("sqlite:{}", db_path.display());
                
                // 确保父目录可写
                if let Some(parent) = db_path.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| anyhow::anyhow!("Failed to create parent directory: {}", e))?;
                    }
                }
            }
        }
        
        println!("连接到数据库: {}", database_url);
        
        let db = if database_url.starts_with("mysql") {
            println!("使用MySQL连接");
            Database::MySql(MySqlDatabase::new(&database_url).await?)
        } else {
            println!("使用SQLite连接");
            Database::Sqlite(SqliteDatabase::new(&database_url).await?)
        };
        
        db.init_tables().await?;
        db.initialize_default_base_urls().await?;
        Ok(db)
    }

    pub async fn init_tables(&self) -> Result<()> {
        match self {
            Database::Sqlite(db) => db.init_tables().await,
            Database::MySql(db) => db.init_tables().await,
        }
    }

    pub async fn initialize_default_base_urls(&self) -> Result<()> {
        match self {
            Database::Sqlite(db) => db.initialize_default_base_urls().await,
            Database::MySql(db) => db.initialize_default_base_urls().await,
        }
    }

    pub async fn get_accounts(&self, request: GetAccountsRequest) -> Result<AccountsResponse> {
        retry_with_backoff(|| async {
            match self {
                Database::Sqlite(db) => db.get_accounts(request.clone()).await,
                Database::MySql(db) => db.get_accounts(request.clone()).await,
            }
        }, 2).await
    }

    pub async fn create_account(&self, request: CreateAccountRequest) -> Result<Account> {
        match self {
            Database::Sqlite(db) => db.create_account(request).await,
            Database::MySql(db) => db.create_account(request).await,
        }
    }

    pub async fn update_account(&self, id: i64, request: UpdateAccountRequest) -> Result<Account> {
        match self {
            Database::Sqlite(db) => db.update_account(id, request).await,
            Database::MySql(db) => db.update_account(id, request).await,
        }
    }

    pub async fn get_account(&self, id: i64) -> Result<Account> {
        match self {
            Database::Sqlite(db) => db.get_account(id).await,
            Database::MySql(db) => db.get_account(id).await,
        }
    }

    pub async fn delete_account(&self, id: i64) -> Result<()> {
        match self {
            Database::Sqlite(db) => db.delete_account(id).await,
            Database::MySql(db) => db.delete_account(id).await,
        }
    }

    pub async fn get_account_base_urls(&self) -> Result<Vec<String>> {
        match self {
            Database::Sqlite(db) => db.get_account_base_urls().await,
            Database::MySql(db) => db.get_account_base_urls().await,
        }
    }

    pub async fn get_directories(&self) -> Result<Vec<Directory>> {
        retry_with_backoff(|| async {
            match self {
                Database::Sqlite(db) => db.get_directories().await,
                Database::MySql(db) => db.get_directories().await,
            }
        }, 2).await
    }

    pub async fn create_directory(&self, request: CreateDirectoryRequest) -> Result<Directory> {
        match self {
            Database::Sqlite(db) => db.create_directory(request).await,
            Database::MySql(db) => db.create_directory(request).await,
        }
    }

    pub async fn update_directory(&self, id: i64, request: UpdateDirectoryRequest) -> Result<Directory> {
        match self {
            Database::Sqlite(db) => db.update_directory(id, request).await,
            Database::MySql(db) => db.update_directory(id, request).await,
        }
    }

    pub async fn get_directory(&self, id: i64) -> Result<Directory> {
        match self {
            Database::Sqlite(db) => db.get_directory(id).await,
            Database::MySql(db) => db.get_directory(id).await,
        }
    }

    pub async fn delete_directory(&self, id: i64) -> Result<()> {
        match self {
            Database::Sqlite(db) => db.delete_directory(id).await,
            Database::MySql(db) => db.delete_directory(id).await,
        }
    }

    pub async fn test_connection_performance(&self) -> Result<String> {
        use std::time::Instant;
        
        let start = Instant::now();
        let result = match self {
            Database::Sqlite(db) => {
                let conn_start = Instant::now();
                let mut conn = db.pool.acquire().await?;
                let conn_time = conn_start.elapsed();
                
                let query_start = Instant::now();
                let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
                    .fetch_one(&mut *conn)
                    .await?;
                let query_time = query_start.elapsed();
                
                format!("SQLite - 获取连接: {}ms, 查询: {}ms, 总账号数: {}", 
                    conn_time.as_millis(), query_time.as_millis(), count.0)
            },
            Database::MySql(db) => {
                let conn_start = Instant::now();
                let mut conn = db.pool.acquire().await?;
                let conn_time = conn_start.elapsed();
                
                let query_start = Instant::now();
                let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
                    .fetch_one(&mut *conn)
                    .await?;
                let query_time = query_start.elapsed();
                
                format!("MySQL - 获取连接: {}ms, 查询: {}ms, 总账号数: {}", 
                    conn_time.as_millis(), query_time.as_millis(), count.0)
            }
        };
        
        let total_time = start.elapsed();
        Ok(format!("{}, 总耗时: {}ms", result, total_time.as_millis()))
    }

    pub async fn get_base_urls(&self) -> Result<Vec<BaseUrl>> {
        retry_with_backoff(|| async {
            match self {
                Database::Sqlite(db) => db.get_base_urls().await,
                Database::MySql(db) => db.get_base_urls().await,
            }
        }, 2).await
    }

    pub async fn create_base_url(&self, request: CreateBaseUrlRequest) -> Result<BaseUrl> {
        match self {
            Database::Sqlite(db) => db.create_base_url(request).await,
            Database::MySql(db) => db.create_base_url(request).await,
        }
    }

    pub async fn update_base_url(&self, id: i64, request: UpdateBaseUrlRequest) -> Result<BaseUrl> {
        match self {
            Database::Sqlite(db) => db.update_base_url(id, request).await,
            Database::MySql(db) => db.update_base_url(id, request).await,
        }
    }

    pub async fn get_base_url(&self, id: i64) -> Result<BaseUrl> {
        match self {
            Database::Sqlite(db) => db.get_base_url(id).await,
            Database::MySql(db) => db.get_base_url(id).await,
        }
    }

    pub async fn delete_base_url(&self, id: i64) -> Result<()> {
        match self {
            Database::Sqlite(db) => db.delete_base_url(id).await,
            Database::MySql(db) => db.delete_base_url(id).await,
        }
    }

    pub async fn switch_account(&self, request: SwitchAccountRequest) -> Result<String> {
        match self {
            Database::Sqlite(db) => db.switch_account(request).await,
            Database::MySql(db) => db.switch_account(request).await,
        }
    }

    pub async fn get_associations(&self) -> Result<Vec<std::collections::HashMap<String, serde_json::Value>>> {
        match self {
            Database::Sqlite(db) => {
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
                .fetch_all(&db.pool)
                .await?;

                let result = rows.into_iter().map(|row| {
                    let mut assoc = std::collections::HashMap::new();
                    if let Ok(val) = row.try_get::<i64, _>("id") { 
                        assoc.insert("id".to_string(), serde_json::Value::Number(serde_json::Number::from(val))); 
                    }
                    if let Ok(val) = row.try_get::<i64, _>("account_id") { 
                        assoc.insert("account_id".to_string(), serde_json::Value::Number(serde_json::Number::from(val))); 
                    }
                    if let Ok(val) = row.try_get::<i64, _>("directory_id") { 
                        assoc.insert("directory_id".to_string(), serde_json::Value::Number(serde_json::Number::from(val))); 
                    }
                    if let Ok(val) = row.try_get::<String, _>("account_name") { 
                        assoc.insert("account_name".to_string(), serde_json::Value::String(val)); 
                    }
                    if let Ok(val) = row.try_get::<String, _>("directory_name") { 
                        assoc.insert("directory_name".to_string(), serde_json::Value::String(val)); 
                    }
                    if let Ok(val) = row.try_get::<DateTime<Utc>, _>("created_at") { 
                        assoc.insert("created_at".to_string(), serde_json::Value::String(val.to_rfc3339())); 
                    }
                    assoc
                }).collect();

                Ok(result)
            }
            Database::MySql(db) => {
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
                .fetch_all(&db.pool)
                .await?;

                let result = rows.into_iter().map(|row| {
                    let mut assoc = std::collections::HashMap::new();
                    if let Ok(val) = row.try_get::<i64, _>("id") { 
                        assoc.insert("id".to_string(), serde_json::Value::Number(serde_json::Number::from(val))); 
                    }
                    if let Ok(val) = row.try_get::<i64, _>("account_id") { 
                        assoc.insert("account_id".to_string(), serde_json::Value::Number(serde_json::Number::from(val))); 
                    }
                    if let Ok(val) = row.try_get::<i64, _>("directory_id") { 
                        assoc.insert("directory_id".to_string(), serde_json::Value::Number(serde_json::Number::from(val))); 
                    }
                    if let Ok(val) = row.try_get::<String, _>("account_name") { 
                        assoc.insert("account_name".to_string(), serde_json::Value::String(val)); 
                    }
                    if let Ok(val) = row.try_get::<String, _>("directory_name") { 
                        assoc.insert("directory_name".to_string(), serde_json::Value::String(val)); 
                    }
                    if let Ok(val) = row.try_get::<DateTime<Utc>, _>("created_at") { 
                        assoc.insert("created_at".to_string(), serde_json::Value::String(val.to_rfc3339())); 
                    }
                    assoc
                }).collect();

                Ok(result)
            }
        }
    }
}