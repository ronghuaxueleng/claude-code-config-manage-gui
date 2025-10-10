use crate::models::AppSettings;
use std::path::PathBuf;
use tokio::fs;

pub struct SettingsManager {
    settings_file: PathBuf,
}

impl SettingsManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let settings_dir = Self::get_settings_directory()?;
        
        // 确保配置目录存在
        if !settings_dir.exists() {
            std::fs::create_dir_all(&settings_dir)?;
        }
        
        let settings_file = settings_dir.join("settings.json");
        
        Ok(Self {
            settings_file,
        })
    }
    
    fn get_settings_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| "APPDATA environment variable not found")?;
            Ok(PathBuf::from(appdata).join("ClaudeConfigManager"))
        }
        
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "HOME environment variable not found")?;
            Ok(PathBuf::from(home).join("Library/Application Support/ClaudeConfigManager"))
        }
        
        #[cfg(target_os = "linux")]
        {
            // 按照XDG规范
            let config_home = std::env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    format!("{}/.config", home)
                });
            Ok(PathBuf::from(config_home).join("claude-config-manager"))
        }
    }
    
    pub async fn load_settings(&self) -> Result<AppSettings, Box<dyn std::error::Error>> {
        if !self.settings_file.exists() {
            // 如果设置文件不存在，返回默认设置并创建文件
            let default_settings = AppSettings::default();
            self.save_settings(&default_settings).await?;
            return Ok(default_settings);
        }
        
        let content = fs::read_to_string(&self.settings_file).await?;
        let settings: AppSettings = serde_json::from_str(&content)
            .unwrap_or_else(|_| AppSettings::default());
        
        Ok(settings)
    }
    
    pub async fn save_settings(&self, settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(settings)?;
        fs::write(&self.settings_file, json_content).await?;
        Ok(())
    }
    
    pub fn get_default_allowed_tools() -> Vec<String> {
        vec![
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
            "MultiEdit".to_string(),
            "Bash".to_string(),
            "Glob".to_string(),
            "Grep".to_string(),
            "WebSearch".to_string(),
            "WebFetch".to_string(),
            "Task".to_string(),
            "TodoWrite".to_string(),
            "BashOutput".to_string(),
            "KillBash".to_string(),
            "NotebookEdit".to_string(),
        ]
    }
    
    pub async fn check_for_updates(&self) -> Result<String, Box<dyn std::error::Error>> {
        // 这里是一个模拟的更新检查功能
        // 在实际应用中，这应该连接到实际的更新服务器
        
        // 模拟网络请求延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        // 模拟检查结果
        let current_version = env!("CARGO_PKG_VERSION");
        
        // 这里可以实现真实的版本检查逻辑
        // 比如从GitHub API或者自己的更新服务器获取最新版本信息
        
        Ok(format!("当前版本: {}。暂无可用更新。", current_version))
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize SettingsManager")
    }
}