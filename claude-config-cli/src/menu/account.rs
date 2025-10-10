use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select, Confirm};
use crate::{DbState, models::*};
use comfy_table::{Attribute, Cell, Color};

pub async fn account_menu(db: &DbState) -> Result<()> {
    loop {
        let items = vec![
            "📝 查看所有账号",
            "➕ 添加新账号",
            "✏️  编辑账号",
            "🗑️  删除账号",
            "🔙 返回主菜单",
        ];

        let selection = Select::new()
            .with_prompt("\n账号管理")
            .items(&items)
            .default(0)
            .interact()?;

        match selection {
            0 => list_accounts(db).await?,
            1 => add_account(db).await?,
            2 => edit_account(db).await?,
            3 => delete_account(db).await?,
            4 => break,
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

    let base_url: String = Input::new()
        .with_prompt("Base URL")
        .default("https://api.anthropic.com".to_string())
        .interact()?;

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

    let items: Vec<String> = response.accounts
        .iter()
        .map(|a| format!("{} - {}", a.name, a.base_url))
        .collect();

    let selection = Select::new()
        .with_prompt("选择要编辑的账号")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        let account = &response.accounts[idx];

        let name: String = Input::new()
            .with_prompt("账号名称")
            .default(account.name.clone())
            .interact()?;

        let token: String = Input::new()
            .with_prompt("API Token")
            .default(account.token.clone())
            .interact()?;

        let base_url: String = Input::new()
            .with_prompt("Base URL")
            .default(account.base_url.clone())
            .interact()?;

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

    let items: Vec<String> = response.accounts
        .iter()
        .map(|a| format!("{} - {}", a.name, a.base_url))
        .collect();

    let selection = Select::new()
        .with_prompt("选择要删除的账号")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
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
