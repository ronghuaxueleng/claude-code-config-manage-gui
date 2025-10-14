use crate::{models::*, t, DbState};
use anyhow::Result;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Color};
use dialoguer::{Confirm, Input, Select};

pub async fn directory_menu(db: &DbState) -> Result<()> {
    let mut last_selection = 0;

    loop {
        let items = vec![
            t!("common.back"),
            t!("directory.menu.list"),
            t!("directory.menu.add"),
            t!("directory.menu.edit"),
            t!("directory.menu.delete"),
        ];

        let selection = Select::new()
            .with_prompt(format!("\n{}", t!("directory.menu.title")))
            .items(&items)
            .default(last_selection)
            .interact()?;

        last_selection = selection;

        match selection {
            0 => break,
            1 => list_directories(db).await?,
            2 => add_directory(db).await?,
            3 => edit_directory(db).await?,
            4 => delete_directory(db).await?,
            _ => unreachable!(),
        }
    }

    Ok(())
}

async fn list_directories(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let directories = db_lock.get_directories().await?;
    drop(db_lock);

    if directories.is_empty() {
        println!("\n{}", t!("directory.list.no_records").yellow());
        return Ok(());
    }

    let mut table = super::create_table();
    table.set_header(vec![
        Cell::new(t!("directory.list.header_id"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("directory.list.header_name"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("directory.list.header_path"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("account.list.header_status"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("directory.list.header_exists"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    for directory in &directories {
        let status = if directory.is_active {
            t!("account.list.status_active")
        } else {
            t!("account.list.status_inactive")
        };
        let exists = if std::path::Path::new(&directory.path).exists() {
            t!("directory.list.exists")
        } else {
            t!("directory.list.not_exists")
        };

        table.add_row(vec![
            directory.id.to_string(),
            directory.name.clone(),
            directory.path.clone(),
            status.to_string(),
            exists.to_string(),
        ]);
    }

    println!("\n{}", table);
    println!("{}", t!("directory.list.total").replace("{}", &directories.len().to_string()));

    let _ = Input::<String>::new()
        .with_prompt(t!("common.continue"))
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn add_directory(db: &DbState) -> Result<()> {
    println!("\n{}", t!("directory.add.title").green().bold());

    let path: String = Input::new().with_prompt(t!("directory.add.prompt_path")).interact()?;

    // 检查路径是否存在
    if !std::path::Path::new(&path).exists() {
        println!("{}", t!("directory.add.warn_path_not_exists").yellow());
        if !Confirm::new()
            .with_prompt(t!("common.confirm"))
            .default(false)
            .interact()?
        {
            return Ok(());
        }
    }

    let name: String = Input::new().with_prompt(t!("directory.add.prompt_name")).interact()?;

    let db_lock = db.lock().await;
    let request = CreateDirectoryRequest {
        path: path.clone(),
        name: name.clone(),
    };

    match db_lock.create_directory(request).await {
        Ok(_) => {
            println!("\n{}", t!("directory.add.success").replace("{}", &name).green());
        }
        Err(e) => {
            println!("\n{}", t!("directory.add.error").replace("{}", &e.to_string()).red());
        }
    }

    Ok(())
}

async fn edit_directory(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let directories = db_lock.get_directories().await?;
    drop(db_lock);

    if directories.is_empty() {
        println!("\n{}", t!("directory.list.no_records").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.cancel").to_string()];
    items.extend(
        directories
            .iter()
            .map(|d| format!("{} - {}", d.name, d.path)),
    );

    let selection = Select::new()
        .with_prompt(t!("directory.edit.prompt"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let directory = &directories[idx];

        let name: String = Input::new()
            .with_prompt(t!("directory.add.prompt_name"))
            .default(directory.name.clone())
            .interact()?;

        let path: String = Input::new()
            .with_prompt(t!("directory.add.prompt_path"))
            .default(directory.path.clone())
            .interact()?;

        let db_lock = db.lock().await;
        let request = UpdateDirectoryRequest {
            name: Some(name),
            path: Some(path),
        };

        match db_lock.update_directory(directory.id, request).await {
            Ok(_) => {
                println!("\n{}", t!("directory.edit.success").green());
            }
            Err(e) => {
                println!("\n{}", t!("directory.edit.error").replace("{}", &e.to_string()).red());
            }
        }
    }

    Ok(())
}

async fn delete_directory(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let directories = db_lock.get_directories().await?;
    drop(db_lock);

    if directories.is_empty() {
        println!("\n{}", t!("directory.list.no_records").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.cancel").to_string()];
    items.extend(
        directories
            .iter()
            .map(|d| format!("{} - {}", d.name, d.path)),
    );

    let selection = Select::new()
        .with_prompt(t!("directory.delete.prompt"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let directory = &directories[idx];

        if Confirm::new()
            .with_prompt(format!(
                "{} {}",
                t!("directory.delete.confirm").replace("{}", &directory.name),
                t!("directory.delete.warning")
            ))
            .default(false)
            .interact()?
        {
            let db_lock = db.lock().await;
            match db_lock.delete_directory(directory.id).await {
                Ok(_) => {
                    println!("\n{}", t!("directory.delete.success").green());
                }
                Err(e) => {
                    println!("\n{}", t!("directory.delete.error").replace("{}", &e.to_string()).red());
                }
            }
        }
    }

    Ok(())
}
