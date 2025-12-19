use crate::{models::*, t, DbState};
use anyhow::Result;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Color};
use dialoguer::{Confirm, Input, Select};

pub async fn account_menu(db: &DbState) -> Result<()> {
    let mut last_selection = 0;

    loop {
        let items = vec![
            t!("common.back"),
            t!("account.menu.list"),
            t!("account.menu.add"),
            t!("account.menu.edit"),
            t!("account.menu.delete"),
            t!("account.menu.import"),
            t!("account.menu.export"),
        ];

        let selection = match Select::new()
            .with_prompt(format!(
                "\n{} (ESC {})",
                t!("account.menu.title"),
                t!("common.to_back")
            ))
            .items(&items)
            .default(last_selection)
            .interact_opt()?
        {
            Some(sel) => sel,
            None => break, // 用户按了ESC，返回上一级
        };

        last_selection = selection;

        match selection {
            0 => break,
            1 => list_accounts(db).await?,
            2 => add_account(db).await?,
            3 => edit_account(db).await?,
            4 => delete_account(db).await?,
            5 => import_accounts(db).await?,
            6 => export_accounts(db).await?,
            _ => unreachable!(),
        }
    }

    Ok(())
}

async fn list_accounts(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let request = GetAccountsRequest {
        page: Some(1),
        per_page: Some(100),
        search: None,
        base_url: None,
    };

    let response = db_lock.get_accounts(request).await?;
    drop(db_lock);

    if response.accounts.is_empty() {
        println!("\n{}", t!("account.list.no_records").yellow());
        return Ok(());
    }

    let mut table = super::create_table();
    table.set_header(vec![
        Cell::new(t!("account.list.header_id"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("account.list.header_name"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("account.list.header_base_url"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("account.list.header_model"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("account.list.header_status"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    for account in &response.accounts {
        let status = if account.is_active {
            t!("account.list.status_active")
        } else {
            t!("account.list.status_inactive")
        };
        table.add_row(vec![
            account.id.to_string(),
            account.name.clone(),
            account.base_url.clone(),
            account.model.clone(),
            status.to_string(),
        ]);
    }

    println!("\n{}", table);
    println!(
        "{}",
        t!("account.list.total").replace("{}", &response.accounts.len().to_string())
    );

    let _ = Input::<String>::new()
        .with_prompt(t!("common.continue"))
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn add_account(db: &DbState) -> Result<()> {
    println!("\n{}", t!("account.add.title").green().bold());
    println!("{}", t!("common.input_cancel_hint").yellow());

    let name: String = Input::new()
        .with_prompt(t!("account.add.prompt_name"))
        .allow_empty(true)
        .interact_text()?;

    if name.trim().is_empty() {
        println!("\n{}", t!("common.cancel").yellow());
        return Ok(());
    }

    let token: String = Input::new()
        .with_prompt(t!("account.add.prompt_token"))
        .allow_empty(true)
        .interact_text()?;

    if token.trim().is_empty() {
        println!("\n{}", t!("common.cancel").yellow());
        return Ok(());
    }

    // 获取所有 Base URL
    let db_lock = db.lock().await;
    let base_urls = db_lock.get_base_urls().await?;
    drop(db_lock);

    let base_url: String = if base_urls.is_empty() {
        // 如果没有 Base URL，让用户手动输入
        println!("\n{}", t!("account.add.no_base_url").yellow());
        Input::new()
            .with_prompt(t!("account.add.prompt_base_url"))
            .default("https://api.anthropic.com".to_string())
            .interact()?
    } else {
        // 从列表选择 Base URL
        let items: Vec<String> = base_urls
            .iter()
            .map(|u| {
                if u.is_default {
                    format!("{} - {} {}", u.name, u.url, t!("account.default_indicator"))
                } else {
                    format!("{} - {}", u.name, u.url)
                }
            })
            .collect();

        let selection = Select::new()
            .with_prompt(t!("account.add.select_base_url"))
            .items(&items)
            .default(0)
            .interact()?;

        base_urls[selection].url.clone()
    };

    let model: String = Input::new()
        .with_prompt(t!("account.add.prompt_model"))
        .allow_empty(true)
        .interact_text()?;

    let db_lock = db.lock().await;
    let request = CreateAccountRequest {
        name: name.clone(),
        token,
        base_url,
        model,
        custom_env_vars: None,
    };

    match db_lock.create_account(request).await {
        Ok(_) => {
            println!(
                "\n{}",
                t!("account.add.success").replace("{}", &name).green()
            );
        }
        Err(e) => {
            println!(
                "\n{}",
                t!("account.add.error").replace("{}", &e.to_string()).red()
            );
        }
    }

    Ok(())
}

async fn edit_account(db: &DbState) -> Result<()> {
    // 先列出所有账号
    let db_lock = db.lock().await;
    let request = GetAccountsRequest {
        page: Some(1),
        per_page: Some(100),
        search: None,
        base_url: None,
    };
    let response = db_lock.get_accounts(request).await?;
    drop(db_lock);

    if response.accounts.is_empty() {
        println!("\n{}", t!("account.list.no_records").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.cancel").to_string()];
    items.extend(
        response
            .accounts
            .iter()
            .map(|a| format!("{} - {}", a.name, a.base_url)),
    );

    let selection = Select::new()
        .with_prompt(t!("account.edit.prompt"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let account = &response.accounts[idx];

        println!("{}", t!("common.input_cancel_hint").yellow());

        let name: String = Input::new()
            .with_prompt(t!("account.add.prompt_name"))
            .default(account.name.clone())
            .allow_empty(true)
            .interact_text()?;

        let name = if name.trim().is_empty() {
            account.name.clone()
        } else {
            name
        };

        let token: String = Input::new()
            .with_prompt(t!("account.add.prompt_token"))
            .default(account.token.clone())
            .allow_empty(true)
            .interact_text()?;

        let token = if token.trim().is_empty() {
            account.token.clone()
        } else {
            token
        };

        // 获取所有 Base URL
        let db_lock = db.lock().await;
        let base_urls = db_lock.get_base_urls().await?;
        drop(db_lock);

        let base_url: String = if base_urls.is_empty() {
            // 如果没有 Base URL，让用户手动输入
            println!("\n{}", t!("account.add.no_base_url").yellow());
            let input_url: String = Input::new()
                .with_prompt(t!("account.add.prompt_base_url"))
                .default(account.base_url.clone())
                .allow_empty(true)
                .interact_text()?;

            if input_url.trim().is_empty() {
                account.base_url.clone()
            } else {
                input_url
            }
        } else {
            // 从列表选择 Base URL
            let items: Vec<String> = base_urls
                .iter()
                .map(|u| {
                    if u.is_default {
                        format!("{} - {} {}", u.name, u.url, t!("account.default_indicator"))
                    } else {
                        format!("{} - {}", u.name, u.url)
                    }
                })
                .collect();

            // 查找当前账号使用的 Base URL 的索引
            let default_index = base_urls
                .iter()
                .position(|u| u.url == account.base_url)
                .unwrap_or(0);

            let selection = Select::new()
                .with_prompt(t!("account.add.select_base_url"))
                .items(&items)
                .default(default_index)
                .interact()?;

            base_urls[selection].url.clone()
        };

        let model: String = Input::new()
            .with_prompt(t!("account.add.prompt_model"))
            .default(account.model.clone())
            .allow_empty(true)
            .interact_text()?;

        let model = if model.trim().is_empty() {
            account.model.clone()
        } else {
            model
        };

        let db_lock = db.lock().await;
        let request = UpdateAccountRequest {
            name: Some(name),
            token: Some(token),
            base_url: Some(base_url),
            model: Some(model),
            custom_env_vars: None,
        };

        match db_lock.update_account(account.id, request).await {
            Ok(_) => {
                println!("\n{}", t!("account.edit.success").green());
            }
            Err(e) => {
                println!(
                    "\n{}",
                    t!("account.edit.error").replace("{}", &e.to_string()).red()
                );
            }
        }
    }

    Ok(())
}

async fn delete_account(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let request = GetAccountsRequest {
        page: Some(1),
        per_page: Some(100),
        search: None,
        base_url: None,
    };
    let response = db_lock.get_accounts(request).await?;
    drop(db_lock);

    if response.accounts.is_empty() {
        println!("\n{}", t!("account.list.no_records").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.cancel").to_string()];
    items.extend(
        response
            .accounts
            .iter()
            .map(|a| format!("{} - {}", a.name, a.base_url)),
    );

    let selection = Select::new()
        .with_prompt(t!("account.delete.prompt"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let account = &response.accounts[idx];

        if Confirm::new()
            .with_prompt(t!("account.delete.confirm").replace("{}", &account.name))
            .default(false)
            .interact()?
        {
            let db_lock = db.lock().await;
            match db_lock.delete_account(account.id).await {
                Ok(_) => {
                    println!("\n{}", t!("account.delete.success").green());
                }
                Err(e) => {
                    println!(
                        "\n{}",
                        t!("account.delete.error")
                            .replace("{}", &e.to_string())
                            .red()
                    );
                }
            }
        }
    }

    Ok(())
}

// 导出账号到JSON文件
async fn export_accounts(db: &DbState) -> Result<()> {
    use std::fs;

    println!("\n{}", t!("account.export.title").cyan().bold());

    // 获取所有账号
    let db_lock = db.lock().await;
    let request = GetAccountsRequest {
        page: Some(1),
        per_page: Some(10000),
        search: None,
        base_url: None,
    };
    let response = db_lock.get_accounts(request).await?;
    drop(db_lock);

    if response.accounts.is_empty() {
        println!("\n{}", t!("account.export.no_accounts").yellow());
        return Ok(());
    }

    // 构造导出数据（providers格式）
    let export_data = serde_json::json!({
        "providers": response.accounts.iter().map(|account| {
            serde_json::json!({
                "name": account.name,
                "url": account.base_url,
                "key": account.token,
                "is_enabled": account.is_active,
                "weight": 100,
                "priority": 1
            })
        }).collect::<Vec<_>>()
    });

    // 生成文件名
    let timestamp = chrono::Local::now().format("%Y-%m-%dT%H-%M-%S");
    let filename = format!("accounts_export_{}.json", timestamp);

    // 写入文件
    let json_str = serde_json::to_string_pretty(&export_data)?;
    fs::write(&filename, json_str)?;

    println!(
        "\n{}",
        t!("account.export.success")
            .replace("{}", &response.accounts.len().to_string())
            .replace("{file}", &filename)
            .green()
    );

    Ok(())
}

// 从JSON文件导入账号
async fn import_accounts(db: &DbState) -> Result<()> {
    use std::fs;

    println!("\n{}", t!("account.import.title").cyan().bold());

    // 输入文件路径
    let file_path: String = Input::new()
        .with_prompt(t!("account.import.prompt_file"))
        .interact_text()?;

    // 读取文件
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            println!(
                "\n{}",
                t!("account.import.error_read")
                    .replace("{}", &e.to_string())
                    .red()
            );
            return Ok(());
        }
    };

    // 解析JSON
    let data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            println!(
                "\n{}",
                t!("account.import.error_parse")
                    .replace("{}", &e.to_string())
                    .red()
            );
            return Ok(());
        }
    };

    // 验证格式
    let providers = match data.get("providers").and_then(|p| p.as_array()) {
        Some(p) => p,
        None => {
            println!("\n{}", t!("account.import.error_format").red());
            return Ok(());
        }
    };

    if providers.is_empty() {
        println!("\n{}", t!("account.import.no_accounts").yellow());
        return Ok(());
    }

    // 导入账号
    let mut imported_count = 0;
    let mut skipped_count = 0;

    println!("\n{}", t!("account.import.processing").cyan());

    // 先获取所有现有账号，避免每次都查询
    let db_lock = db.lock().await;
    let existing_accounts = db_lock
        .get_accounts(GetAccountsRequest {
            page: Some(1),
            per_page: Some(10000),
            search: None, // 不使用搜索，获取所有账号
            base_url: None,
        })
        .await?;
    drop(db_lock);

    for provider in providers {
        let name = provider.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let url = provider.get("url").and_then(|u| u.as_str()).unwrap_or("");
        let key = provider.get("key").and_then(|k| k.as_str()).unwrap_or("");

        if name.is_empty() || url.is_empty() || key.is_empty() {
            println!("  {} {}", "⚠".yellow(), t!("account.import.skip_invalid"));
            skipped_count += 1;
            continue;
        }

        // 检查是否已存在（精确匹配）
        let is_duplicate = existing_accounts
            .accounts
            .iter()
            .any(|acc| acc.name == name || acc.token == key);

        if is_duplicate {
            println!(
                "  {} {}: {}",
                "⊖".yellow(),
                t!("account.import.skip_exists"),
                name
            );
            skipped_count += 1;
            continue;
        }

        // 创建账号
        let db_lock = db.lock().await;
        match db_lock
            .create_account(CreateAccountRequest {
                name: name.to_string(),
                token: key.to_string(),
                base_url: url.to_string(),
                model: "".to_string(),
                custom_env_vars: None,
            })
            .await
        {
            Ok(_) => {
                println!(
                    "  {} {}: {}",
                    "✓".green(),
                    t!("account.import.imported"),
                    name
                );
                imported_count += 1;
            }
            Err(e) => {
                println!(
                    "  {} {}: {} ({})",
                    "✗".red(),
                    t!("account.import.failed"),
                    name,
                    e
                );
                skipped_count += 1;
            }
        }
        drop(db_lock);
    }

    // 显示结果
    println!("\n{}", "=".repeat(50).cyan());
    println!(
        "{}",
        t!("account.import.result_imported")
            .replace("{}", &imported_count.to_string())
            .green()
    );
    if skipped_count > 0 {
        println!(
            "{}",
            t!("account.import.result_skipped")
                .replace("{}", &skipped_count.to_string())
                .yellow()
        );
    }
    println!("{}", "=".repeat(50).cyan());

    Ok(())
}
