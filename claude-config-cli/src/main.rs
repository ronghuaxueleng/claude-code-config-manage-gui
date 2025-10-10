mod models;
mod database;
mod claude_config;
mod config_manager;
mod logger;
mod webdav;
mod menu;

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
    println!("{}", "正在初始化数据库...".cyan());
    let db = match Database::new().await {
        Ok(database) => {
            println!("{}", "✓ 数据库初始化成功".green());
            Arc::new(Mutex::new(database))
        }
        Err(e) => {
            eprintln!("{}", format!("✗ 数据库初始化失败: {}", e).red());
            println!("\n尝试使用默认配置创建数据库...");
            match Database::create_with_fallback().await {
                Ok(database) => {
                    println!("{}", "✓ 使用默认配置创建数据库成功".green());
                    Arc::new(Mutex::new(database))
                }
                Err(e) => {
                    eprintln!("{}", format!("✗ 无法初始化数据库: {}", e).red());
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
                menu::switch::switch_menu(&db).await?;
            }
            3 => {
                menu::webdav::webdav_menu(&db).await?;
            }
            4 => {
                menu::logs::logs_menu().await?;
            }
            5 => {
                println!("\n{}", "感谢使用 Claude Code 配置管理器！".green().bold());
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
    println!("{}", "║        Claude Code 配置管理器 - 命令行版本 v1.2.0            ║".bright_blue().bold());
    println!("{}", "║        Claude Code Configuration Manager - CLI               ║".bright_blue());
    println!("{}", "║                                                               ║".bright_blue());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝".bright_blue());
    println!();
}

fn show_main_menu() -> Result<usize> {
    let items = vec![
        "📋 账号管理",
        "📁 目录管理",
        "⚡ 配置切换",
        "☁️  WebDAV 同步",
        "📝 查看日志",
        "❌ 退出程序",
    ];

    let selection = Select::new()
        .with_prompt("\n请选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(selection)
}
