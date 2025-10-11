use anyhow::Result;
use colored::Colorize;
use dialoguer::Select;
use crate::{DbState, models::*, claude_config::ClaudeConfigManager};
use std::path::Path;
use std::fs;

// 写入 Claude 配置到 .claude/settings.local.json
fn write_claude_settings(
    directory_path: &str,
    claude_settings_json: &str,
    account_token: &str,
    account_base_url: &str,
    skip_permissions: bool,
) -> Result<()> {
    use serde_json::Value;

    // 解析 Claude 配置
    let mut claude_settings: Value = serde_json::from_str(claude_settings_json)?;

    // 确保是对象类型
    if !claude_settings.is_object() {
        claude_settings = serde_json::json!({});
    }

    let settings_obj = claude_settings.as_object_mut().unwrap();

    // 设置权限配置
    if skip_permissions {
        settings_obj.insert("permissions".to_string(), serde_json::json!({
            "defaultMode": "bypassPermissions",
            "allow": ["*"]
        }));
    } else {
        // 如果不跳过权限，使用默认的权限配置
        if !settings_obj.contains_key("permissions") {
            settings_obj.insert("permissions".to_string(), serde_json::json!({
                "defaultMode": "prompt",
                "allow": []
            }));
        }
    }

    // 确保 env 字段存在
    if !settings_obj.contains_key("env") {
        settings_obj.insert("env".to_string(), serde_json::json!({}));
    }

    let env_obj = settings_obj.get_mut("env").unwrap().as_object_mut().unwrap();

    // 添加账号相关的环境变量
    env_obj.insert("ANTHROPIC_API_KEY".to_string(), Value::String(account_token.to_string()));
    env_obj.insert("ANTHROPIC_AUTH_TOKEN".to_string(), Value::String(account_token.to_string()));
    env_obj.insert("ANTHROPIC_BASE_URL".to_string(), Value::String(account_base_url.to_string()));

    // 创建 .claude 目录
    let claude_dir = Path::new(directory_path).join(".claude");
    fs::create_dir_all(&claude_dir)?;

    // 写入 settings.local.json
    let settings_file = claude_dir.join("settings.local.json");
    let settings_json = serde_json::to_string_pretty(&claude_settings)?;
    fs::write(&settings_file, settings_json)?;

    Ok(())
}

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

    // 询问权限配置
    let skip_permissions = dialoguer::Confirm::new()
        .with_prompt("跳过权限检查? (推荐选择 Yes)")
        .default(true)
        .interact()?;

    // 沙盒模式默认开启
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
            // 获取 Claude 配置
            let claude_settings_json = match db_lock.get_claude_settings().await {
                Ok(json) => json,
                Err(e) => {
                    println!("\n{}", format!("警告: 获取Claude配置失败，使用默认配置: {}", e).yellow());
                    // 使用默认配置
                    serde_json::to_string(&serde_json::json!({
                        "permissions": {
                            "defaultMode": "bypassPermissions",
                            "allow": ["*"]
                        },
                        "env": {
                            "IS_SANDBOX": "1",
                            "DISABLE_AUTOUPDATER": 1
                        }
                    })).unwrap()
                }
            };

            drop(db_lock);

            // 更新环境配置文件
            let config_manager = ClaudeConfigManager::new(directory.path.clone());
            match config_manager.update_env_config_with_options(
                account.token.clone(),
                account.base_url.clone(),
                is_sandbox,
            ) {
                Ok(_) => {
                    // 写入 Claude 配置到 .claude/settings.local.json
                    match write_claude_settings(
                        &directory.path,
                        &claude_settings_json,
                        &account.token,
                        &account.base_url,
                        skip_permissions,
                    ) {
                        Ok(_) => {
                            println!("\n{}", "✓ 配置切换成功!".green().bold());
                            println!("  账号: {}", account.name);
                            println!("  目录: {}", directory.name);
                            println!("  路径: {}", directory.path);
                            println!("  沙盒模式: 已启用");
                            println!("  权限检查: {}", if skip_permissions { "已跳过" } else { "需要确认" });
                        }
                        Err(e) => {
                            println!("\n{}", "✓ 环境配置切换成功!".green().bold());
                            println!("  账号: {}", account.name);
                            println!("  目录: {}", directory.name);
                            println!("  路径: {}", directory.path);
                            println!("  沙盒模式: 已启用");
                            println!("\n{}", format!("警告: Claude配置写入失败: {}", e).yellow());
                        }
                    }
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
