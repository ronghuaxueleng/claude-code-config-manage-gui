use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select, Confirm};
use crate::{DbState, models::*};
use comfy_table::{Attribute, Cell, Color};

pub async fn directory_menu(db: &DbState) -> Result<()> {
    loop {
        let items = vec![
            "📝 查看所有目录",
            "➕ 添加新目录",
            "✏️  编辑目录",
            "🗑️  删除目录",
            "🔙 返回主菜单",
        ];

        let selection = Select::new()
            .with_prompt("\n目录管理")
            .items(&items)
            .default(0)
            .interact()?;

        match selection {
            0 => list_directories(db).await?,
            1 => add_directory(db).await?,
            2 => edit_directory(db).await?,
            3 => delete_directory(db).await?,
            4 => break,
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
        println!("\n{}", "暂无目录记录".yellow());
        return Ok(());
    }

    let mut table = super::create_table();
    table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("目录名称").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("路径").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("状态").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("存在性").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);

    for directory in &directories {
        let status = if directory.is_active { "🟢 活跃" } else { "⚪ 未活跃" };
        let exists = if std::path::Path::new(&directory.path).exists() {
            "✓ 存在"
        } else {
            "✗ 不存在"
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
    println!("共 {} 个目录", directories.len());

    let _ = Input::<String>::new()
        .with_prompt("按 Enter 继续")
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn add_directory(db: &DbState) -> Result<()> {
    println!("\n{}", "添加新目录".green().bold());

    let path: String = Input::new()
        .with_prompt("目录路径")
        .interact()?;

    // 检查路径是否存在
    if !std::path::Path::new(&path).exists() {
        println!("{}", "⚠️  警告: 该路径不存在".yellow());
        if !Confirm::new()
            .with_prompt("是否继续添加?")
            .default(false)
            .interact()?
        {
            return Ok(());
        }
    }

    let name: String = Input::new()
        .with_prompt("目录名称")
        .interact()?;

    let db_lock = db.lock().await;
    let request = CreateDirectoryRequest {
        path: path.clone(),
        name: name.clone(),
    };

    match db_lock.create_directory(request).await {
        Ok(_) => {
            println!("\n{}", format!("✓ 目录 '{}' 添加成功", name).green());
        }
        Err(e) => {
            println!("\n{}", format!("✗ 添加失败: {}", e).red());
        }
    }

    Ok(())
}

async fn edit_directory(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let directories = db_lock.get_directories().await?;
    drop(db_lock);

    if directories.is_empty() {
        println!("\n{}", "暂无目录记录".yellow());
        return Ok(());
    }

    let items: Vec<String> = directories
        .iter()
        .map(|d| format!("{} - {}", d.name, d.path))
        .collect();

    let selection = Select::new()
        .with_prompt("选择要编辑的目录")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        let directory = &directories[idx];

        let name: String = Input::new()
            .with_prompt("目录名称")
            .default(directory.name.clone())
            .interact()?;

        let path: String = Input::new()
            .with_prompt("目录路径")
            .default(directory.path.clone())
            .interact()?;

        let db_lock = db.lock().await;
        let request = UpdateDirectoryRequest {
            name: Some(name),
            path: Some(path),
        };

        match db_lock.update_directory(directory.id, request).await {
            Ok(_) => {
                println!("\n{}", "✓ 目录更新成功".green());
            }
            Err(e) => {
                println!("\n{}", format!("✗ 更新失败: {}", e).red());
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
        println!("\n{}", "暂无目录记录".yellow());
        return Ok(());
    }

    let items: Vec<String> = directories
        .iter()
        .map(|d| format!("{} - {}", d.name, d.path))
        .collect();

    let selection = Select::new()
        .with_prompt("选择要删除的目录")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        let directory = &directories[idx];

        if Confirm::new()
            .with_prompt(format!("确定要删除目录 '{}' 吗? (仅删除数据库记录，不删除实际文件)", directory.name))
            .default(false)
            .interact()?
        {
            let db_lock = db.lock().await;
            match db_lock.delete_directory(directory.id).await {
                Ok(_) => {
                    println!("\n{}", "✓ 目录删除成功".green());
                }
                Err(e) => {
                    println!("\n{}", format!("✗ 删除失败: {}", e).red());
                }
            }
        }
    }

    Ok(())
}
