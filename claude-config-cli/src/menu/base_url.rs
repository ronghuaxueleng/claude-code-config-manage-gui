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

        let selection = Select::new()
            .with_prompt(format!("\n{}", t!("url.menu.title")))
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

    let name: String = Input::new().with_prompt(t!("url.add.prompt_name")).interact()?;

    let url: String = Input::new()
        .with_prompt(t!("url.add.prompt_url"))
        .default("https://api.anthropic.com".to_string())
        .interact()?;

    let description: String = Input::new()
        .with_prompt(t!("url.add.prompt_description"))
        .allow_empty(true)
        .interact()?;

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

        let name: String = Input::new()
            .with_prompt(t!("url.add.prompt_name"))
            .default(base_url.name.clone())
            .interact()?;

        let url: String = Input::new()
            .with_prompt(t!("url.add.prompt_url"))
            .default(base_url.url.clone())
            .interact()?;

        let description: String = Input::new()
            .with_prompt(t!("url.add.prompt_description"))
            .default(base_url.description.clone().unwrap_or_default())
            .allow_empty(true)
            .interact()?;

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
