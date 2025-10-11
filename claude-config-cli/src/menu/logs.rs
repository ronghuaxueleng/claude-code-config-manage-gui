use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};
use crate::logger::Logger;

pub async fn logs_menu() -> Result<()> {
    let mut last_selection = 0;

    loop {
        let items = vec![
            "🔙 返回主菜单",
            "📝 查看最近日志",
            "📊 日志文件信息",
            "📂 打开日志目录",
        ];

        let selection = Select::new()
            .with_prompt("\n日志管理")
            .items(&items)
            .default(last_selection)
            .interact()?;

        last_selection = selection;

        match selection {
            0 => break,
            1 => view_recent_logs().await?,
            2 => show_log_info().await?,
            3 => open_log_directory().await?,
            _ => unreachable!(),
        }
    }

    Ok(())
}

async fn view_recent_logs() -> Result<()> {
    let lines: usize = Input::new()
        .with_prompt("显示最近多少行日志")
        .default(50)
        .interact()?;

    match Logger::get_recent_logs(Some(lines)) {
        Ok(logs) => {
            if logs.is_empty() {
                println!("\n{}", "暂无日志记录".yellow());
            } else {
                println!("\n{}", "最近的日志:".green().bold());
                for log in logs {
                    println!("{}", log);
                }
            }
        }
        Err(e) => {
            println!("{}", format!("✗ 读取日志失败: {}", e).red());
        }
    }

    let _ = Input::<String>::new()
        .with_prompt("按 Enter 继续")
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn show_log_info() -> Result<()> {
    match Logger::get_log_info() {
        Ok(info) => {
            println!("\n{}", "日志文件信息:".green().bold());
            if let Some(path) = info.get("log_file_path") {
                println!("  日志文件: {}", path);
            }
            if let Some(size) = info.get("log_file_size") {
                println!("  文件大小: {}", size);
            }
            if let Some(lines) = info.get("total_lines") {
                println!("  总行数: {}", lines);
            }
        }
        Err(e) => {
            println!("{}", format!("✗ 获取日志信息失败: {}", e).red());
        }
    }

    let _ = Input::<String>::new()
        .with_prompt("按 Enter 继续")
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn open_log_directory() -> Result<()> {
    match Logger::get_log_directory() {
        Ok(log_dir) => {
            println!("日志目录: {}", log_dir.display());

            // 在不同平台上打开目录
            #[cfg(target_os = "linux")]
            {
                match std::process::Command::new("xdg-open")
                    .arg(&log_dir)
                    .spawn()
                {
                    Ok(_) => println!("{}", "✓ 已打开日志目录".green()),
                    Err(e) => println!("{}", format!("✗ 打开目录失败: {}", e).red()),
                }
            }

            #[cfg(target_os = "windows")]
            {
                match std::process::Command::new("explorer")
                    .arg(&log_dir)
                    .spawn()
                {
                    Ok(_) => println!("{}", "✓ 已打开日志目录".green()),
                    Err(e) => println!("{}", format!("✗ 打开目录失败: {}", e).red()),
                }
            }

            #[cfg(target_os = "macos")]
            {
                match std::process::Command::new("open")
                    .arg(&log_dir)
                    .spawn()
                {
                    Ok(_) => println!("{}", "✓ 已打开日志目录".green()),
                    Err(e) => println!("{}", format!("✗ 打开目录失败: {}", e).red()),
                }
            }
        }
        Err(e) => {
            println!("{}", format!("✗ 获取日志目录失败: {}", e).red());
        }
    }

    let _ = Input::<String>::new()
        .with_prompt("按 Enter 继续")
        .allow_empty(true)
        .interact()?;

    Ok(())
}
