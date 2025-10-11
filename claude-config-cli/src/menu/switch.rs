use anyhow::Result;
use colored::Colorize;
use dialoguer::Select;
use crate::{DbState, models::*, claude_config::ClaudeConfigManager};

pub async fn switch_menu(db: &DbState) -> Result<()> {
    println!("\n{}", "配置切换".green().bold());

    // 获取所有账号
    let db_lock = db.lock().await;
    let accounts_response = db_lock.get_accounts(GetAccountsRequest {
        page: Some(1),
        per_page: Some(100),
        search: None,
        base_url: None,
    }).await?;

    let directories = db_lock.get_directories().await?;
    drop(db_lock);

    if accounts_response.accounts.is_empty() {
        println!("\n{}", "暂无账号记录，请先添加账号".yellow());
        return Ok(());
    }

    if directories.is_empty() {
        println!("\n{}", "暂无目录记录，请先添加目录".yellow());
        return Ok(());
    }

    // 选择账号
    let mut account_items: Vec<String> = vec!["🔙 取消".to_string()];
    account_items.extend(
        accounts_response.accounts
            .iter()
            .map(|a| format!("{} - {}", a.name, a.base_url))
    );

    let account_selection = Select::new()
        .with_prompt("选择账号")
        .items(&account_items)
        .interact_opt()?;

    if account_selection.is_none() || account_selection == Some(0) {
        return Ok(());
    }

    let account = &accounts_response.accounts[account_selection.unwrap() - 1];

    // 选择目录
    let mut directory_items: Vec<String> = vec!["🔙 取消".to_string()];
    directory_items.extend(
        directories
            .iter()
            .map(|d| {
                let exists = if std::path::Path::new(&d.path).exists() {
                    "✓"
                } else {
                    "✗"
                };
                format!("{} {} - {}", exists, d.name, d.path)
            })
    );

    let directory_selection = Select::new()
        .with_prompt("选择目录")
        .items(&directory_items)
        .interact_opt()?;

    if directory_selection.is_none() || directory_selection == Some(0) {
        return Ok(());
    }

    let directory = &directories[directory_selection.unwrap() - 1];

    // 默认启用沙盒模式
    let is_sandbox = true;

    // 执行切换
    println!("\n{}", "正在切换配置...".cyan());

    let db_lock = db.lock().await;
    let request = SwitchAccountRequest {
        account_id: account.id,
        directory_id: directory.id,
    };

    match db_lock.switch_account(request).await {
        Ok(_) => {
            drop(db_lock);

            // 更新配置文件
            let config_manager = ClaudeConfigManager::new(directory.path.clone());
            match config_manager.update_env_config_with_options(
                account.token.clone(),
                account.base_url.clone(),
                is_sandbox,
            ) {
                Ok(_) => {
                    println!("\n{}", "✓ 配置切换成功!".green().bold());
                    println!("  账号: {}", account.name);
                    println!("  目录: {}", directory.name);
                    println!("  路径: {}", directory.path);
                    println!("  沙盒: {}", if is_sandbox { "启用" } else { "禁用" });
                }
                Err(e) => {
                    println!("\n{}", format!("✗ 配置文件更新失败: {}", e).red());
                }
            }
        }
        Err(e) => {
            println!("\n{}", format!("✗ 切换失败: {}", e).red());
        }
    }

    let _ = dialoguer::Input::<String>::new()
        .with_prompt("按 Enter 继续")
        .allow_empty(true)
        .interact()?;

    Ok(())
}
