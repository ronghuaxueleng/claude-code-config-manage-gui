mod models;
mod database;
mod claude_config;
mod config_manager;
mod logger;
mod webdav;
mod menu;
mod i18n;

use anyhow::Result;
use colored::Colorize;
use console::Term;
use dialoguer::Select;
use std::sync::Arc;
use tokio::sync::Mutex;
use database::Database;

type DbState = Arc<Mutex<Database>>;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    if let Err(e) = logger::Logger::init() {
        eprintln!("日志系统初始化失败: {}", e);
    }

    // 清屏
    let term = Term::stdout();
    let _ = term.clear_screen();

    // 显示欢迎信息
    print_banner();

    // 初始化数据库
    println!("{}", i18n::translate("db.init").cyan());
    let db = match Database::new().await {
        Ok(database) => {
            println!("{}", i18n::translate("db.init_success").green());
            Arc::new(Mutex::new(database))
        }
        Err(e) => {
            eprintln!("{}", format!("{}: {}", i18n::translate("db.init_error"), e).red());
            println!("\n{}", i18n::translate("db.fallback"));
            match Database::create_with_fallback().await {
                Ok(database) => {
                    println!("{}", i18n::translate("db.fallback_success").green());
                    Arc::new(Mutex::new(database))
                }
                Err(e) => {
                    eprintln!("{}", format!("{}: {}", i18n::translate("db.fallback_error"), e).red());
                    return Err(e.into());
                }
            }
        }
    };

    println!();

    // 主菜单循环
    loop {
        let selection = show_main_menu()?;

        match selection {
            0 => {
                menu::account::account_menu(&db).await?;
            }
            1 => {
                menu::directory::directory_menu(&db).await?;
            }
            2 => {
                menu::base_url::base_url_menu(&db).await?;
            }
            3 => {
                menu::switch::switch_menu(&db).await?;
            }
            4 => {
                menu::webdav::webdav_menu(&db).await?;
            }
            5 => {
                menu::logs::logs_menu().await?;
            }
            6 => {
                remove_root_check()?;
            }
            7 => {
                menu::settings::settings_menu().await?;
            }
            8 => {
                println!("\n{}", i18n::translate("app.exit_message").green().bold());
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn print_banner() {
    println!("{}", "╔═══════════════════════════════════════════════════════════════╗".bright_blue());
    println!("{}", "║                                                               ║".bright_blue());
    println!("{}", format!("║        {} - {} {}            ║",
        i18n::translate("app.name"),
        i18n::translate("app.cli_subtitle"),
        i18n::translate("app.version")
    ).bright_blue().bold());
    println!("{}", "║        Claude Code Configuration Manager - CLI               ║".bright_blue());
    println!("{}", "║                                                               ║".bright_blue());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝".bright_blue());
    println!();
}

fn show_main_menu() -> Result<usize> {
    let items = vec![
        i18n::translate("menu.main.account"),
        i18n::translate("menu.main.directory"),
        i18n::translate("menu.main.url"),
        i18n::translate("menu.main.switch"),
        i18n::translate("menu.main.webdav"),
        i18n::translate("menu.main.logs"),
        i18n::translate("menu.main.remove_root"),
        i18n::translate("menu.main.settings"),
        i18n::translate("menu.main.exit"),
    ];

    let selection = Select::new()
        .with_prompt(format!("\n{}", i18n::translate("menu.main.title")))
        .items(&items)
        .default(0)
        .interact()?;

    Ok(selection)
}

fn remove_root_check() -> Result<()> {
    use dialoguer::{Input, Confirm};
    use std::process::Command;
    use std::io::Write;

    println!("\n{}", "========================================".bright_blue());
    println!("{}", "      删除 Claude Code Root Check      ".bright_blue().bold());
    println!("{}", "========================================".bright_blue());
    println!();

    // 将脚本内容嵌入到二进制文件中
    const SCRIPT_CONTENT: &str = include_str!("../resources/config/remove-root-check.sh");

    println!("{}", "此操作将执行以下步骤:".yellow());
    println!("  1. 查找 claude 命令位置");
    println!("  2. 创建包装脚本自动删除 root check 限制");
    println!("  3. 备份原始 claude 命令");
    println!("  4. 替换 claude 命令为包装脚本");
    println!();

    if !Confirm::new()
        .with_prompt("是否继续执行删除限制代码操作?")
        .default(false)
        .interact()?
    {
        println!("\n{}", "操作已取消".yellow());
        return Ok(());
    }

    println!("\n{}", "正在执行删除限制代码脚本...".cyan());
    println!();

    // 创建临时脚本文件
    let temp_dir = std::env::temp_dir();
    let temp_script = temp_dir.join("remove-root-check-temp.sh");

    // 将 Windows 换行符 (CRLF) 转换为 Unix 换行符 (LF)
    let script_content_unix = SCRIPT_CONTENT.replace("\r\n", "\n");

    // 写入脚本内容
    let mut file = std::fs::File::create(&temp_script)?;
    file.write_all(script_content_unix.as_bytes())?;
    drop(file);

    // 设置执行权限 (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_script)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_script, perms)?;
    }

    // 执行脚本
    let output = Command::new("bash")
        .arg(&temp_script)
        .output();

    // 清理临时文件
    let _ = std::fs::remove_file(&temp_script);

    match output {
        Ok(output) => {
            // 显示标准输出
            if !output.stdout.is_empty() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                print!("{}", stdout);
            }

            // 显示标准错误
            if !output.stderr.is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                print!("{}", stderr.red());
            }

            if output.status.success() {
                println!("\n{}", "✓ 删除限制代码完成".green().bold());
            } else {
                println!("\n{}", format!("✗ 脚本执行失败，退出代码: {}", output.status).red());
            }
        }
        Err(e) => {
            println!("{}", format!("✗ 执行脚本失败: {}", e).red());
        }
    }

    let _ = Input::<String>::new()
        .with_prompt("\n按 Enter 继续")
        .allow_empty(true)
        .interact()?;

    Ok(())
}
