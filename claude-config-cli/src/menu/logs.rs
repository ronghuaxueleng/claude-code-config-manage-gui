use crate::{logger::Logger, t};
use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};

pub async fn logs_menu() -> Result<()> {
    let mut last_selection = 0;

    loop {
        let items = vec![
            t!("logs.menu.back"),
            t!("logs.menu.view_recent"),
            t!("logs.menu.info"),
            t!("logs.menu.open_dir"),
        ];

        let selection = match Select::new()
            .with_prompt(format!("\n{} (ESC {})", t!("logs.menu.title"), t!("common.to_back")))
            .items(&items)
            .default(last_selection)
            .interact_opt()? {
                Some(sel) => sel,
                None => break, // 用户按了ESC，返回上一级
            };

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
        .with_prompt(t!("logs.prompt_lines"))
        .default(50)
        .interact()?;

    match Logger::get_recent_logs(Some(lines)) {
        Ok(logs) => {
            if logs.is_empty() {
                println!("\n{}", t!("logs.no_records").yellow());
            } else {
                println!("\n{}", t!("logs.title").green().bold());
                for log in logs {
                    println!("{}", log);
                }
            }
        }
        Err(e) => {
            println!("{}", t!("logs.read.error").replace("{}", &e.to_string()).red());
        }
    }

    let _ = Input::<String>::new()
        .with_prompt(t!("common.continue"))
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn show_log_info() -> Result<()> {
    match Logger::get_log_info() {
        Ok(info) => {
            println!("\n{}", t!("logs.info.title").green().bold());
            if let Some(path) = info.get("log_file_path") {
                println!("{}", t!("logs.file").replace("{}", &path.to_string()));
            }
            if let Some(size) = info.get("log_file_size") {
                println!("{}", t!("logs.size").replace("{}", &size.to_string()));
            }
            if let Some(lines) = info.get("total_lines") {
                println!("{}", t!("logs.lines").replace("{}", &lines.to_string()));
            }
        }
        Err(e) => {
            println!("{}", t!("logs.info.error").replace("{}", &e.to_string()).red());
        }
    }

    let _ = Input::<String>::new()
        .with_prompt(t!("common.continue"))
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn open_log_directory() -> Result<()> {
    match Logger::get_log_directory() {
        Ok(log_dir) => {
            println!("{}", t!("logs.directory").replace("{}", &log_dir.display().to_string()));

            // 在不同平台上打开目录
            #[cfg(target_os = "linux")]
            {
                match std::process::Command::new("xdg-open").arg(&log_dir).spawn() {
                    Ok(_) => println!("{}", t!("logs.directory_opened").green()),
                    Err(e) => println!("{}", t!("logs.open_dir.error").replace("{}", &e.to_string()).red()),
                }
            }

            #[cfg(target_os = "windows")]
            {
                match std::process::Command::new("explorer").arg(&log_dir).spawn() {
                    Ok(_) => println!("{}", t!("logs.directory_opened").green()),
                    Err(e) => println!("{}", t!("logs.open_dir.error").replace("{}", &e.to_string()).red()),
                }
            }

            #[cfg(target_os = "macos")]
            {
                match std::process::Command::new("open").arg(&log_dir).spawn() {
                    Ok(_) => println!("{}", t!("logs.directory_opened").green()),
                    Err(e) => println!("{}", t!("logs.open_dir.error").replace("{}", &e.to_string()).red()),
                }
            }
        }
        Err(e) => {
            println!("{}", t!("logs.directory.error").replace("{}", &e.to_string()).red());
        }
    }

    let _ = Input::<String>::new()
        .with_prompt(t!("common.continue"))
        .allow_empty(true)
        .interact()?;

    Ok(())
}
