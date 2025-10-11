use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select, Confirm};
use crate::{DbState, models::*};
use comfy_table::{Attribute, Cell, Color};

pub async fn account_menu(db: &DbState) -> Result<()> {
    let mut last_selection = 0;

    loop {
        let items = vec![
            "🔙 返回主菜单",
            "📝 查看所有账号",
            "➕ 添加新账号",
            "✏️  编辑账号",
            "🗑️  删除账号",
        ];

        let selection = Select::new()
            .with_prompt("\n账号管理")
            .items(&items)
            .default(last_selection)
            .interact()?;

        last_selection = selection;

        match selection {
            0 => break,
            1 => list_accounts(db).await?,
            2 => add_account(db).await?,
            3 => edit_account(db).await?,
            4 => delete_account(db).await?,
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
        println!("\n{}", "暂无账号记录".yellow());
        return Ok(());
    }

    let mut table = super::create_table();
    table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("账号名称").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Base URL").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("模型").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("状态").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);

    for account in &response.accounts {
        let status = if account.is_active { "🟢 活跃" } else { "⚪ 未活跃" };
        table.add_row(vec![
            account.id.to_string(),
            account.name.clone(),
            account.base_url.clone(),
            account.model.clone(),
            status.to_string(),
        ]);
    }

    println!("\n{}", table);
    println!("共 {} 个账号", response.accounts.len());

    let _ = Input::<String>::new()
        .with_prompt("按 Enter 继续")
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn add_account(db: &DbState) -> Result<()> {
    println!("\n{}", "添加新账号".green().bold());

    let name: String = Input::new()
        .with_prompt("账号名称")
        .interact()?;

    let token: String = Input::new()
        .with_prompt("API Token")
        .interact()?;

    // 获取所有 Base URL
    let db_lock = db.lock().await;
    let base_urls = db_lock.get_base_urls().await?;
    drop(db_lock);

    let base_url: String = if base_urls.is_empty() {
        // 如果没有 Base URL，让用户手动输入
        println!("\n{}", "暂无可用的 Base URL，请手动输入".yellow());
        Input::new()
            .with_prompt("Base URL")
            .default("https://api.anthropic.com".to_string())
            .interact()?
    } else {
        // 从列表选择 Base URL
        let items: Vec<String> = base_urls
            .iter()
            .map(|u| {
                if u.is_default {
                    format!("{} - {} (默认)", u.name, u.url)
                } else {
                    format!("{} - {}", u.name, u.url)
                }
            })
            .collect();

        let selection = Select::new()
            .with_prompt("选择 Base URL")
            .items(&items)
            .default(0)
            .interact()?;

        base_urls[selection].url.clone()
    };

    let model: String = Input::new()
        .with_prompt("模型")
        .default("claude-sonnet-4-20250514".to_string())
        .interact()?;

    let db_lock = db.lock().await;
    let request = CreateAccountRequest {
        name: name.clone(),
        token,
        base_url,
        model,
    };

    match db_lock.create_account(request).await {
        Ok(_) => {
            println!("\n{}", format!("✓ 账号 '{}' 创建成功", name).green());
        }
        Err(e) => {
            println!("\n{}", format!("✗ 创建失败: {}", e).red());
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
        println!("\n{}", "暂无账号记录".yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec!["🔙 取消".to_string()];
    items.extend(
        response.accounts
            .iter()
            .map(|a| format!("{} - {}", a.name, a.base_url))
    );

    let selection = Select::new()
        .with_prompt("选择要编辑的账号")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let account = &response.accounts[idx];

        let name: String = Input::new()
            .with_prompt("账号名称")
            .default(account.name.clone())
            .interact()?;

        let token: String = Input::new()
            .with_prompt("API Token")
            .default(account.token.clone())
            .interact()?;

        // 获取所有 Base URL
        let db_lock = db.lock().await;
        let base_urls = db_lock.get_base_urls().await?;
        drop(db_lock);

        let base_url: String = if base_urls.is_empty() {
            // 如果没有 Base URL，让用户手动输入
            println!("\n{}", "暂无可用的 Base URL，请手动输入".yellow());
            Input::new()
                .with_prompt("Base URL")
                .default(account.base_url.clone())
                .interact()?
        } else {
            // 从列表选择 Base URL
            let items: Vec<String> = base_urls
                .iter()
                .map(|u| {
                    if u.is_default {
                        format!("{} - {} (默认)", u.name, u.url)
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
                .with_prompt("选择 Base URL")
                .items(&items)
                .default(default_index)
                .interact()?;

            base_urls[selection].url.clone()
        };

        let model: String = Input::new()
            .with_prompt("模型")
            .default(account.model.clone())
            .interact()?;

        let db_lock = db.lock().await;
        let request = UpdateAccountRequest {
            name: Some(name),
            token: Some(token),
            base_url: Some(base_url),
            model: Some(model),
        };

        match db_lock.update_account(account.id, request).await {
            Ok(_) => {
                println!("\n{}", "✓ 账号更新成功".green());
            }
            Err(e) => {
                println!("\n{}", format!("✗ 更新失败: {}", e).red());
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
        println!("\n{}", "暂无账号记录".yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec!["🔙 取消".to_string()];
    items.extend(
        response.accounts
            .iter()
            .map(|a| format!("{} - {}", a.name, a.base_url))
    );

    let selection = Select::new()
        .with_prompt("选择要删除的账号")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let account = &response.accounts[idx];

        if Confirm::new()
            .with_prompt(format!("确定要删除账号 '{}' 吗?", account.name))
            .default(false)
            .interact()?
        {
            let db_lock = db.lock().await;
            match db_lock.delete_account(account.id).await {
                Ok(_) => {
                    println!("\n{}", "✓ 账号删除成功".green());
                }
                Err(e) => {
                    println!("\n{}", format!("✗ 删除失败: {}", e).red());
                }
            }
        }
    }

    Ok(())
}
