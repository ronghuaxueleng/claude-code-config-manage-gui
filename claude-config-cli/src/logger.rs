use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct Logger;

impl Logger {
    /// 初始化日志系统
    pub fn init() -> Result<()> {
        // 获取可执行文件目录
        let exe_dir = get_exe_dir()?;

        // 创建logs目录
        let logs_dir = exe_dir.join("logs");
        fs::create_dir_all(&logs_dir)?;

        // 创建日志文件appender（每天滚动）
        let file_appender = rolling::daily(&logs_dir, "claude-config-manager.log");
        let (non_blocking_file, _guard) = non_blocking(file_appender);

        // 设置日志级别，默认为INFO
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        // 构建订阅器 - 只输出到文件，不输出到控制台
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::Layer::new()
                    .with_writer(non_blocking_file)
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_timer(fmt::time::ChronoUtc::rfc_3339()),
            )
            .init();

        // 防止guard被释放
        std::mem::forget(_guard);

        tracing::info!("Logger initialized, logs directory: {}", logs_dir.display());

        Ok(())
    }

    /// 获取日志目录路径
    pub fn get_log_directory() -> Result<PathBuf> {
        let exe_dir = get_exe_dir()?;
        Ok(exe_dir.join("logs"))
    }

    /// 获取日志信息
    pub fn get_log_info() -> Result<serde_json::Value> {
        let logs_dir = Self::get_log_directory()?;

        let mut info = serde_json::Map::new();
        info.insert(
            "log_directory".to_string(),
            serde_json::Value::String(logs_dir.display().to_string()),
        );
        info.insert(
            "log_file".to_string(),
            serde_json::Value::String("claude-config-manager.log".to_string()),
        );

        // 检查日志文件是否存在
        let log_files = fs::read_dir(&logs_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                    Some(path.file_name()?.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        info.insert(
            "log_files".to_string(),
            serde_json::Value::Array(
                log_files
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect(),
            ),
        );

        Ok(serde_json::Value::Object(info))
    }

    /// 读取最近的日志行
    pub fn get_recent_logs(lines: Option<usize>) -> Result<Vec<String>> {
        let logs_dir = Self::get_log_directory()?;
        let log_file = logs_dir.join("claude-config-manager.log");

        if !log_file.exists() {
            return Ok(vec!["日志文件不存在".to_string()]);
        }

        let content = fs::read_to_string(&log_file)?;
        let all_lines: Vec<&str> = content.lines().collect();

        let line_count = lines.unwrap_or(50).min(1000); // 最多返回1000行
        let start_index = if all_lines.len() > line_count {
            all_lines.len() - line_count
        } else {
            0
        };

        let recent_lines: Vec<String> = all_lines[start_index..]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Ok(recent_lines)
    }
}

/// 获取可执行文件所在目录
fn get_exe_dir() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?;
    Ok(exe_dir.to_path_buf())
}

/// 测试日志功能
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_exe_dir() {
        let dir = get_exe_dir().unwrap();
        println!("Executable directory: {}", dir.display());
        assert!(dir.exists());
    }

    #[tokio::test]
    async fn test_logger_init() {
        Logger::init().unwrap();

        tracing::info!("Test info log");
        tracing::warn!("Test warning log");
        tracing::error!("Test error log");

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let log_info = Logger::get_log_info().unwrap();
        println!(
            "Log info: {}",
            serde_json::to_string_pretty(&log_info).unwrap()
        );

        let recent_logs = Logger::get_recent_logs(Some(10)).unwrap();
        println!("Recent logs: {:#?}", recent_logs);
    }
}
