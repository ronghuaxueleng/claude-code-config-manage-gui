use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: Option<i32>,
    pub max_overflow: Option<i32>,
    pub pool_timeout: Option<i32>,
    pub pool_recycle: Option<i32>,
    pub echo: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    pub port: Option<u16>,
    pub debug: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub connections: HashMap<String, DatabaseConfig>,
    pub current: String,
    pub app: Option<AppConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let mut connections = HashMap::new();

        connections.insert(
            "default".to_string(),
            DatabaseConfig {
                url: "sqlite:///claude_config.db".to_string(),
                pool_size: None,
                max_overflow: None,
                pool_timeout: None,
                pool_recycle: None,
                echo: None,
            },
        );

        Self {
            connections,
            current: "default".to_string(),
            app: Some(AppConfig {
                name: Some("Claude Configuration Manager".to_string()),
                version: Some("1.0.0".to_string()),
                port: Some(6666),
                debug: Some(false),
            }),
        }
    }
}

pub struct ConfigManager {
    pub config: Config,
    config_file: Option<PathBuf>,
}

impl ConfigManager {
    pub fn new() -> Self {
        let mut manager = Self {
            config: Config::default(),
            config_file: None,
        };

        // 尝试从resources目录加载config.json
        if let Some(resource_config_path) = Self::get_resource_path("config.json") {
            if let Ok(_) = manager.load_from_file(&resource_config_path) {
                println!(
                    "从resources目录加载配置文件: {}",
                    resource_config_path.display()
                );
                return manager;
            }
        }

        // 尝试从当前目录加载config.json
        if let Ok(current_dir) = std::env::current_dir() {
            let config_path = current_dir.join("config.json");
            if config_path.exists() {
                let _ = manager.load_from_file(&config_path);
                println!("从当前目录加载配置文件: {}", config_path.display());
            }
        }

        manager
    }

    /// 获取resources目录中文件的路径
    pub fn get_resource_path(filename: &str) -> Option<PathBuf> {
        // 尝试多个可能的resources路径
        let possible_paths = [
            // 开发环境：从src-tauri目录运行时
            PathBuf::from("src-tauri/resources").join(filename),
            PathBuf::from("resources").join(filename),
            // 构建后：相对于可执行文件
            std::env::current_exe()
                .ok()?
                .parent()?
                .join("resources")
                .join(filename),
            // Tauri打包后的路径
            std::env::current_exe()
                .ok()?
                .parent()?
                .parent()?
                .join("Resources")
                .join(filename),
            // Windows应用路径
            std::env::current_exe()
                .ok()?
                .parent()?
                .join("resources")
                .join(filename),
        ];

        for path in possible_paths {
            if path.exists() {
                println!("找到资源文件: {}", path.display());
                return Some(path);
            }
        }

        println!("未找到资源文件: {}", filename);
        None
    }

    /// 获取应用数据目录（用于存储用户数据，如数据库文件）
    /// Windows: %APPDATA%\claude-config-manager
    /// Linux/Mac: ~/.claude-config-manager
    #[allow(dead_code)]
    pub fn get_app_data_dir() -> Option<PathBuf> {
        // 跨平台的应用数据目录获取
        #[cfg(target_os = "windows")]
        {
            // Windows: 使用 APPDATA 目录，不加点前缀
            if let Ok(appdata) = std::env::var("APPDATA") {
                let app_data_dir = PathBuf::from(appdata).join("claude-config-manager");
                println!("Windows应用数据目录: {}", app_data_dir.display());
                return Some(app_data_dir);
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Linux/macOS: 使用 HOME 目录，加点前缀
            if let Ok(home) = std::env::var("HOME") {
                let app_data_dir = PathBuf::from(home).join(".claude-config-manager");
                println!("Unix应用数据目录: {}", app_data_dir.display());
                return Some(app_data_dir);
            }
        }

        // 回退方案：使用 USERPROFILE (Windows) 或 HOME
        if let Ok(user_dir) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            #[cfg(target_os = "windows")]
            let app_data_dir = PathBuf::from(user_dir).join("claude-config-manager");

            #[cfg(not(target_os = "windows"))]
            let app_data_dir = PathBuf::from(user_dir).join(".claude-config-manager");

            println!("回退应用数据目录: {}", app_data_dir.display());
            return Some(app_data_dir);
        }

        println!("无法确定应用数据目录");
        None
    }

    /// 获取resources目录的路径（用于存储数据库等数据文件）
    pub fn get_resource_dir() -> Option<PathBuf> {
        // 使用可执行文件同级的 resources 目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let resources_dir = exe_dir.join("resources");

                println!("resources目录路径: {}", resources_dir.display());
                println!("resources目录是否存在: {}", resources_dir.exists());

                // 返回 resources 目录路径（无论是否存在）
                // 调用者会负责创建目录
                return Some(resources_dir);
            }
        }

        println!("无法确定可执行文件路径");
        None
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, config_file: P) -> Result<()> {
        let content = fs::read_to_string(&config_file)?;
        let file_config: Config = serde_json::from_str(&content)?;

        // 合并配置（简单替换，可以后续优化为深度合并）
        self.config = file_config;
        self.config_file = Some(config_file.as_ref().to_path_buf());

        Ok(())
    }

    pub fn get_database_config(&self, connection_name: Option<&str>) -> Option<&DatabaseConfig> {
        let conn_name = connection_name.unwrap_or(&self.config.current);
        self.config.connections.get(conn_name)
    }

    pub fn get_default_database_config(&self) -> Option<&DatabaseConfig> {
        self.get_database_config(None)
    }
}
