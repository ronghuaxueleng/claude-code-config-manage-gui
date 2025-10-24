use crate::{models::*, t, DbState};
use anyhow::Result;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Color};
use dialoguer::{Confirm, Input, Select};

pub async fn base_url_menu(db: &DbState) -> Result<()> {
    let mut last_selection = 0;

    loop {
        let items = vec![
            t!("common.back"),
            t!("url.menu.list"),
            t!("url.menu.add"),
            t!("url.menu.edit"),
            t!("url.menu.delete"),
        ];

        let selection = match Select::new()
            .with_prompt(format!("\n{} (ESC {})", t!("url.menu.title"), t!("common.to_back")))
            .items(&items)
            .default(last_selection)
            .interact_opt()? {
                Some(sel) => sel,
                None => break, // 用户按了ESC，返回上一级
            };

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
        println!("\n{}", t!("url.list.no_records").yellow());
        return Ok(());
    }

    let mut table = super::create_table();
    table.set_header(vec![
        Cell::new(t!("url.list.header_id"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("url.list.header_name"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("url.list.header_url"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("url.list.header_description"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("url.list.header_api_key"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("url.list.header_default"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    for base_url in &base_urls {
        let is_default = if base_url.is_default { t!("url.list.default_yes") } else { "" };
        let description = base_url
            .description
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");
        table.add_row(vec![
            base_url.id.to_string(),
            base_url.name.clone(),
            base_url.url.clone(),
            description.to_string(),
            base_url.api_key.clone(),
            is_default.to_string(),
        ]);
    }

    println!("\n{}", table);
    println!("{}", t!("url.list.total").replace("{}", &base_urls.len().to_string()));

    let _ = Input::<String>::new()
        .with_prompt(t!("common.continue"))
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn add_base_url(db: &DbState) -> Result<()> {
    println!("\n{}", t!("url.add.title").green().bold());
    println!("{}", t!("common.input_cancel_hint").yellow());

    let name: String = Input::new()
        .with_prompt(t!("url.add.prompt_name"))
        .allow_empty(true)
        .interact_text()?;

    if name.trim().is_empty() || name.trim().eq_ignore_ascii_case("q") {
        println!("\n{}", t!("common.cancel").yellow());
        return Ok(());
    }

    let url: String = Input::new()
        .with_prompt(t!("url.add.prompt_url"))
        .default("https://api.anthropic.com".to_string())
        .allow_empty(true)
        .interact_text()?;

    if url.trim().is_empty() || url.trim().eq_ignore_ascii_case("q") {
        println!("\n{}", t!("common.cancel").yellow());
        return Ok(());
    }

    let description: String = Input::new()
        .with_prompt(t!("url.add.prompt_description"))
        .allow_empty(true)
        .interact_text()?;

    let api_key: String = Input::new()
        .with_prompt(t!("url.add.prompt_api_key"))
        .default("ANTHROPIC_API_KEY".to_string())
        .allow_empty(true)
        .interact_text()?;

    let api_key = if api_key.trim().is_empty() {
        "ANTHROPIC_API_KEY".to_string()
    } else {
        api_key
    };

    let is_default = Confirm::new()
        .with_prompt(t!("url.add.prompt_default"))
        .default(false)
        .interact()?;

    let db_lock = db.lock().await;
    let request = CreateBaseUrlRequest {
        name: name.clone(),
        url,
        description: if description.is_empty() {
            None
        } else {
            Some(description)
        },
        api_key: Some(api_key),
        is_default: Some(is_default),
    };

    match db_lock.create_base_url(request).await {
        Ok(_) => {
            println!("\n{}", t!("url.add.success").replace("{}", &name).green());
        }
        Err(e) => {
            println!("\n{}", t!("url.add.error").replace("{}", &e.to_string()).red());
        }
    }

    Ok(())
}

async fn edit_base_url(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let base_urls = db_lock.get_base_urls().await?;
    drop(db_lock);

    if base_urls.is_empty() {
        println!("\n{}", t!("url.list.no_records").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.cancel").to_string()];
    items.extend(base_urls.iter().map(|u| format!("{} - {}", u.name, u.url)));

    let selection = Select::new()
        .with_prompt(t!("url.edit.prompt"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let base_url = &base_urls[idx];

        println!("{}", t!("common.input_cancel_hint").yellow());

        let name: String = Input::new()
            .with_prompt(t!("url.add.prompt_name"))
            .default(base_url.name.clone())
            .allow_empty(true)
            .interact_text()?;

        let name = if name.trim().is_empty() {
            base_url.name.clone()
        } else {
            name
        };

        let url: String = Input::new()
            .with_prompt(t!("url.add.prompt_url"))
            .default(base_url.url.clone())
            .allow_empty(true)
            .interact_text()?;

        let url = if url.trim().is_empty() {
            base_url.url.clone()
        } else {
            url
        };

        let description: String = Input::new()
            .with_prompt(t!("url.add.prompt_description"))
            .default(base_url.description.clone().unwrap_or_default())
            .allow_empty(true)
            .interact_text()?;

        let api_key: String = Input::new()
            .with_prompt(t!("url.add.prompt_api_key"))
            .default(base_url.api_key.clone())
            .allow_empty(true)
            .interact_text()?;

        let api_key = if api_key.trim().is_empty() {
            "ANTHROPIC_API_KEY".to_string()
        } else {
            api_key
        };

        let is_default = Confirm::new()
            .with_prompt(t!("url.add.prompt_default"))
            .default(base_url.is_default)
            .interact()?;

        let db_lock = db.lock().await;
        let request = UpdateBaseUrlRequest {
            name: Some(name),
            url: Some(url),
            description: if description.is_empty() {
                None
            } else {
                Some(description)
            },
            api_key: Some(api_key),
            is_default: Some(is_default),
        };

        match db_lock.update_base_url(base_url.id, request).await {
            Ok(_) => {
                println!("\n{}", t!("url.edit.success").green());
            }
            Err(e) => {
                println!("\n{}", t!("url.edit.error").replace("{}", &e.to_string()).red());
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
        println!("\n{}", t!("url.list.no_records").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.cancel").to_string()];
    items.extend(base_urls.iter().map(|u| format!("{} - {}", u.name, u.url)));

    let selection = Select::new()
        .with_prompt(t!("url.delete.prompt"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let base_url = &base_urls[idx];

        if Confirm::new()
            .with_prompt(format!(
                "{} {}",
                t!("url.delete.confirm").replace("{}", &base_url.name),
                t!("url.delete.warning")
            ))
            .default(false)
            .interact()?
        {
            let db_lock = db.lock().await;
            match db_lock.delete_base_url(base_url.id).await {
                Ok(_) => {
                    println!("\n{}", t!("url.delete.success").green());
                }
                Err(e) => {
                    println!("\n{}", t!("url.delete.error").replace("{}", &e.to_string()).red());
                }
            }
        }
    }

    Ok(())
}
