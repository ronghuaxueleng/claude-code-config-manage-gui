use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select, Confirm};
use crate::{DbState, models::*};
use comfy_table::{Attribute, Cell, Color};

pub async fn base_url_menu(db: &DbState) -> Result<()> {
    let mut last_selection = 0;

    loop {
        let items = vec![
            "🔙 返回主菜单",
            "📝 查看所有 URL",
            "➕ 添加新 URL",
            "✏️  编辑 URL",
            "🗑️  删除 URL",
        ];

        let selection = Select::new()
            .with_prompt("\nURL 管理")
            .items(&items)
            .default(last_selection)
            .interact()?;

        last_selection = selection;

        match selection {
            0 => break,
            1 => list_base_urls(db).await?,
            2 => add_base_url(db).await?,
            3 => edit_base_url(db).await?,
            4 => delete_base_url(db).await?,
            _ => unreachable!(),
        }
    }

    Ok(())
}

async fn list_base_urls(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let base_urls = db_lock.get_base_urls().await?;
    drop(db_lock);

    if base_urls.is_empty() {
        println!("\n{}", "暂无 URL 记录".yellow());
        return Ok(());
    }

    let mut table = super::create_table();
    table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("名称").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("URL").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("描述").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("默认").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);

    for base_url in &base_urls {
        let is_default = if base_url.is_default { "✓" } else { "" };
        let description = base_url.description.as_ref().map(|s| s.as_str()).unwrap_or("");
        table.add_row(vec![
            base_url.id.to_string(),
            base_url.name.clone(),
            base_url.url.clone(),
            description.to_string(),
            is_default.to_string(),
        ]);
    }

    println!("\n{}", table);
    println!("共 {} 个 URL", base_urls.len());

    let _ = Input::<String>::new()
        .with_prompt("按 Enter 继续")
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn add_base_url(db: &DbState) -> Result<()> {
    println!("\n{}", "添加新 URL".green().bold());

    let name: String = Input::new()
        .with_prompt("名称")
        .interact()?;

    let url: String = Input::new()
        .with_prompt("URL 地址")
        .default("https://api.anthropic.com".to_string())
        .interact()?;

    let description: String = Input::new()
        .with_prompt("描述（可选）")
        .allow_empty(true)
        .interact()?;

    let is_default = Confirm::new()
        .with_prompt("设为默认?")
        .default(false)
        .interact()?;

    let db_lock = db.lock().await;
    let request = CreateBaseUrlRequest {
        name: name.clone(),
        url,
        description: if description.is_empty() { None } else { Some(description) },
        is_default: Some(is_default),
    };

    match db_lock.create_base_url(request).await {
        Ok(_) => {
            println!("\n{}", format!("✓ URL '{}' 创建成功", name).green());
        }
        Err(e) => {
            println!("\n{}", format!("✗ 创建失败: {}", e).red());
        }
    }

    Ok(())
}

async fn edit_base_url(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let base_urls = db_lock.get_base_urls().await?;
    drop(db_lock);

    if base_urls.is_empty() {
        println!("\n{}", "暂无 URL 记录".yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec!["🔙 取消".to_string()];
    items.extend(
        base_urls
            .iter()
            .map(|u| format!("{} - {}", u.name, u.url))
    );

    let selection = Select::new()
        .with_prompt("选择要编辑的 URL")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let base_url = &base_urls[idx];

        let name: String = Input::new()
            .with_prompt("名称")
            .default(base_url.name.clone())
            .interact()?;

        let url: String = Input::new()
            .with_prompt("URL 地址")
            .default(base_url.url.clone())
            .interact()?;

        let description: String = Input::new()
            .with_prompt("描述（可选）")
            .default(base_url.description.clone().unwrap_or_default())
            .allow_empty(true)
            .interact()?;

        let is_default = Confirm::new()
            .with_prompt("设为默认?")
            .default(base_url.is_default)
            .interact()?;

        let db_lock = db.lock().await;
        let request = UpdateBaseUrlRequest {
            name: Some(name),
            url: Some(url),
            description: if description.is_empty() { None } else { Some(description) },
            is_default: Some(is_default),
        };

        match db_lock.update_base_url(base_url.id, request).await {
            Ok(_) => {
                println!("\n{}", "✓ URL 更新成功".green());
            }
            Err(e) => {
                println!("\n{}", format!("✗ 更新失败: {}", e).red());
            }
        }
    }

    Ok(())
}

async fn delete_base_url(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let base_urls = db_lock.get_base_urls().await?;
    drop(db_lock);

    if base_urls.is_empty() {
        println!("\n{}", "暂无 URL 记录".yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec!["🔙 取消".to_string()];
    items.extend(
        base_urls
            .iter()
            .map(|u| format!("{} - {}", u.name, u.url))
    );

    let selection = Select::new()
        .with_prompt("选择要删除的 URL")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let base_url = &base_urls[idx];

        if Confirm::new()
            .with_prompt(format!("确定要删除 URL '{}' 吗? (使用该 URL 的账号也将被删除)", base_url.name))
            .default(false)
            .interact()?
        {
            let db_lock = db.lock().await;
            match db_lock.delete_base_url(base_url.id).await {
                Ok(_) => {
                    println!("\n{}", "✓ URL 删除成功".green());
                }
                Err(e) => {
                    println!("\n{}", format!("✗ 删除失败: {}", e).red());
                }
            }
        }
    }

    Ok(())
}
