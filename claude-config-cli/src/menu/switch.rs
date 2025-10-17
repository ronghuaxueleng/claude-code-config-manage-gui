use crate::{claude_config::ClaudeConfigManager, models::*, t, DbState};
use anyhow::Result;
use colored::Colorize;
use dialoguer::Select;
use std::fs;
use std::path::Path;

// 写入 Claude 配置到 .claude/settings.local.json
fn write_claude_settings(
    directory_path: &str,
    claude_settings_json: &str,
    account_token: &str,
    account_base_url: &str,
    account_model: &str,
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
        settings_obj.insert(
            "permissions".to_string(),
            serde_json::json!({
                "defaultMode": "bypassPermissions",
                "allow": ["*"]
            }),
        );
    } else {
        // 如果不跳过权限，使用默认的权限配置
        if !settings_obj.contains_key("permissions") {
            settings_obj.insert(
                "permissions".to_string(),
                serde_json::json!({
                    "defaultMode": "prompt",
                    "allow": []
                }),
            );
        }
    }

    // 确保 env 字段存在
    if !settings_obj.contains_key("env") {
        settings_obj.insert("env".to_string(), serde_json::json!({}));
    }

    let env_obj = settings_obj
        .get_mut("env")
        .unwrap()
        .as_object_mut()
        .unwrap();

    // 添加账号相关的环境变量
    env_obj.insert(
        "ANTHROPIC_API_KEY".to_string(),
        Value::String(account_token.to_string()),
    );
    env_obj.insert(
        "ANTHROPIC_AUTH_TOKEN".to_string(),
        Value::String(account_token.to_string()),
    );
    env_obj.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        Value::String(account_base_url.to_string()),
    );

    // 添加模型配置（如果账号设置了模型）
    if !account_model.is_empty() {
        env_obj.insert(
            "ANTHROPIC_MODEL".to_string(),
            Value::String(account_model.to_string()),
        );
    }

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
    println!("\n{}", t!("switch.title").green().bold());

    // 获取所有账号
    let db_lock = db.lock().await;
    let accounts_response = db_lock
        .get_accounts(GetAccountsRequest {
            page: Some(1),
            per_page: Some(100),
            search: None,
            base_url: None,
        })
        .await?;

    let directories = db_lock.get_directories().await?;
    drop(db_lock);

    if accounts_response.accounts.is_empty() {
        println!("\n{}", t!("switch.no_accounts").yellow());
        return Ok(());
    }

    if directories.is_empty() {
        println!("\n{}", t!("switch.no_directories").yellow());
        return Ok(());
    }

    // 选择账号
    let mut account_items: Vec<String> = vec![t!("common.back_cancel").to_string()];
    account_items.extend(
        accounts_response
            .accounts
            .iter()
            .map(|a| format!("{} - {}", a.name, a.base_url)),
    );

    let account_selection = Select::new()
        .with_prompt(t!("switch.select_account"))
        .items(&account_items)
        .interact_opt()?;

    if account_selection.is_none() || account_selection == Some(0) {
        return Ok(());
    }

    let account = &accounts_response.accounts[account_selection.unwrap() - 1];

    // 选择目录
    let mut directory_items: Vec<String> = vec![t!("common.back_cancel").to_string()];
    directory_items.extend(directories.iter().map(|d| {
        let exists = if std::path::Path::new(&d.path).exists() {
            "✓"
        } else {
            "✗"
        };
        format!("{} {} - {}", exists, d.name, d.path)
    }));

    let directory_selection = Select::new()
        .with_prompt(t!("switch.select_directory"))
        .items(&directory_items)
        .interact_opt()?;

    if directory_selection.is_none() || directory_selection == Some(0) {
        return Ok(());
    }

    let directory = &directories[directory_selection.unwrap() - 1];

    // 询问权限配置
    let skip_permissions = dialoguer::Confirm::new()
        .with_prompt(t!("switch.prompt_skip_permissions"))
        .default(true)
        .interact()?;

    // 沙盒模式默认开启
    let is_sandbox = true;

    // 执行切换
    println!("\n{}", t!("switch.switching").cyan());

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
                    println!(
                        "\n{}",
                        t!("switch.warn_claude_config").replace("{}", &e.to_string()).yellow()
                    );
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
                    }))
                    .unwrap()
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
                        &account.model,
                        skip_permissions,
                    ) {
                        Ok(_) => {
                            println!("\n{}", t!("switch.success").green().bold());
                            println!("{}", t!("switch.account").replace("{}", &account.name));
                            println!("{}", t!("switch.directory").replace("{}", &directory.name));
                            println!("{}", t!("switch.path").replace("{}", &directory.path));
                            println!("{}", t!("switch.sandbox"));
                            println!(
                                "{}",
                                t!("switch.permission").replace(
                                    "{}",
                                    if skip_permissions {
                                        t!("switch.permission_skipped")
                                    } else {
                                        t!("switch.permission_required")
                                    }
                                )
                            );
                        }
                        Err(e) => {
                            println!("\n{}", t!("switch.success_env").green().bold());
                            println!("{}", t!("switch.account").replace("{}", &account.name));
                            println!("{}", t!("switch.directory").replace("{}", &directory.name));
                            println!("{}", t!("switch.path").replace("{}", &directory.path));
                            println!("{}", t!("switch.sandbox"));
                            println!("\n{}", t!("switch.warn_write_fail").replace("{}", &e.to_string()).yellow());
                        }
                    }
                }
                Err(e) => {
                    println!("\n{}", t!("switch.error_update").replace("{}", &e.to_string()).red());
                }
            }
        }
        Err(e) => {
            println!("\n{}", t!("switch.error").replace("{}", &e.to_string()).red());
        }
    }

    let _ = dialoguer::Input::<String>::new()
        .with_prompt(t!("common.continue"))
        .allow_empty(true)
        .interact()?;

    Ok(())
}
