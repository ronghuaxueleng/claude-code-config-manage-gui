use crate::{models::*, t, webdav, DbState};
use anyhow::Result;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Color};
use dialoguer::{Confirm, Input, Select};

pub async fn webdav_menu(db: &DbState) -> Result<()> {
    let mut last_selection = 0;

    loop {
        let items = vec![
            t!("webdav.menu.back"),
            t!("webdav.menu.list"),
            t!("webdav.menu.add"),
            t!("webdav.menu.test_connection"),
            t!("webdav.menu.upload_config"),
            t!("webdav.menu.download_config"),
            t!("webdav.menu.list_remote"),
            t!("webdav.menu.delete_config"),
        ];

        let selection = Select::new()
            .with_prompt(format!("\n{}", t!("webdav.menu.title")))
            .items(&items)
            .default(last_selection)
            .interact()?;

        last_selection = selection;

        match selection {
            0 => break,
            1 => list_configs(db).await?,
            2 => add_config(db).await?,
            3 => test_connection(db).await?,
            4 => upload_config(db).await?,
            5 => download_config(db).await?,
            6 => list_remote_files(db).await?,
            7 => delete_config(db).await?,
            _ => unreachable!(),
        }
    }

    Ok(())
}

async fn list_configs(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let pool = db_lock.get_pool();
    let configs = webdav::get_webdav_configs(pool).await?;
    drop(db_lock);

    if configs.is_empty() {
        println!("\n{}", t!("webdav.list.no_config").yellow());
        return Ok(());
    }

    let mut table = super::create_table();
    table.set_header(vec![
        Cell::new(t!("webdav.list.header_id"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("webdav.list.header_name"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("webdav.list.header_url"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("webdav.list.header_username"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("webdav.list.header_remote_path"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("webdav.list.header_auto_sync"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(t!("webdav.list.header_status"))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    for config in &configs {
        let auto_sync = if config.auto_sync { t!("webdav.list.auto_sync_yes") } else { t!("webdav.list.auto_sync_no") };
        let status = if config.is_active {
            t!("webdav.list.status_active")
        } else {
            t!("webdav.list.status_inactive")
        };

        table.add_row(vec![
            config.id.to_string(),
            config.name.clone(),
            config.url.clone(),
            config.username.clone(),
            config.remote_path.clone(),
            auto_sync.to_string(),
            status.to_string(),
        ]);
    }

    println!("\n{}", table);
    println!("{}", t!("webdav.list.total").replace("{}", &configs.len().to_string()));

    let _ = Input::<String>::new()
        .with_prompt(t!("common.continue"))
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn add_config(db: &DbState) -> Result<()> {
    println!("\n{}", t!("webdav.add.title").green().bold());

    let name: String = Input::new().with_prompt(t!("webdav.add.prompt_name")).interact()?;

    let url: String = Input::new().with_prompt(t!("webdav.add.prompt_url")).interact()?;

    let username: String = Input::new().with_prompt(t!("webdav.add.prompt_username")).interact()?;

    let password: String = Input::new().with_prompt(t!("webdav.add.prompt_password")).interact()?;

    // 使用固定的默认值，不再询问用户
    let remote_path = "/claude-config";
    let auto_sync = false;
    let sync_interval: i64 = 3600;

    let db_lock = db.lock().await;
    let pool = db_lock.get_pool();

    match webdav::create_webdav_config(
        pool,
        &name,
        &url,
        &username,
        &password,
        remote_path,
        auto_sync,
        sync_interval,
    )
    .await
    {
        Ok(_) => {
            println!("\n{}", t!("webdav.add.success").replace("{}", &name).green());
        }
        Err(e) => {
            println!("\n{}", t!("webdav.add.error").replace("{}", &e.to_string()).red());
        }
    }

    Ok(())
}

async fn test_connection(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let pool = db_lock.get_pool();
    let configs = webdav::get_webdav_configs(pool).await?;
    drop(db_lock);

    if configs.is_empty() {
        println!("\n{}", t!("webdav.list.no_config").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.back_cancel").to_string()];
    items.extend(configs.iter().map(|c| format!("{} - {}", c.name, c.url)));

    let selection = Select::new()
        .with_prompt(t!("webdav.test.select_config"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let config = &configs[idx];

        println!("\n{}", t!("webdav.test.testing").cyan());

        let manager = webdav::WebDavManager::from_config(config.clone()).await?;

        match manager.test_connection().await {
            Ok(_) => {
                println!("{}", t!("webdav.test.success").green());
            }
            Err(e) => {
                println!("{}", t!("webdav.test.error").replace("{}", &e.to_string()).red());
            }
        }

        let _ = Input::<String>::new()
            .with_prompt(t!("common.continue"))
            .allow_empty(true)
            .interact()?;
    }

    Ok(())
}

async fn upload_config(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let pool = db_lock.get_pool();
    let configs = webdav::get_webdav_configs(pool).await?;

    if configs.is_empty() {
        drop(db_lock);
        println!("\n{}", t!("webdav.list.no_config").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.back_cancel").to_string()];
    items.extend(configs.iter().map(|c| format!("{} - {}", c.name, c.url)));

    drop(db_lock);

    let selection = Select::new()
        .with_prompt(t!("webdav.upload.select_config"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let config = &configs[idx];

        let filename: String = Input::new()
            .with_prompt(t!("webdav.upload.prompt_filename"))
            .default(format!(
                "claude-config-{}.json",
                chrono::Local::now().format("%Y%m%d-%H%M%S")
            ))
            .interact()?;

        println!("\n{}", t!("webdav.upload.uploading").cyan());

        // 导出数据库配置
        let db_lock = db.lock().await;
        let accounts = db_lock
            .get_accounts(GetAccountsRequest {
                page: Some(1),
                per_page: Some(1000),
                search: None,
                base_url: None,
            })
            .await?;

        let base_urls = db_lock.get_base_urls().await?;
        let claude_settings_json = db_lock.get_claude_settings().await?;
        let claude_settings: serde_json::Value = serde_json::from_str(&claude_settings_json)?;

        drop(db_lock);

        let data = serde_json::json!({
            "accounts": accounts.accounts,
            "base_urls": base_urls,
            "claude_settings": claude_settings,
            "exported_at": chrono::Utc::now().to_rfc3339(),
        });

        let manager = webdav::WebDavManager::from_config(config.clone()).await?;

        match manager.upload_config(&data, &filename).await {
            Ok(_) => {
                println!(
                    "{}",
                    t!("webdav.upload.success").replace("{}", &filename).green()
                );

                // 记录同步日志
                let db_lock = db.lock().await;
                let pool = db_lock.get_pool();
                let _ = webdav::create_sync_log(
                    pool,
                    CreateSyncLogRequest {
                        webdav_config_id: config.id,
                        sync_type: "upload".to_string(),
                        status: "success".to_string(),
                        message: Some(t!("webdav.upload.success_log").replace("{}", &filename).to_string()),
                    },
                )
                .await;

                let _ = webdav::update_last_sync_time(pool, config.id).await;
            }
            Err(e) => {
                println!("{}", t!("webdav.upload.error").replace("{}", &e.to_string()).red());
            }
        }

        let _ = Input::<String>::new()
            .with_prompt(t!("common.continue"))
            .allow_empty(true)
            .interact()?;
    }

    Ok(())
}

async fn download_config(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let pool = db_lock.get_pool();
    let configs = webdav::get_webdav_configs(pool).await?;
    drop(db_lock);

    if configs.is_empty() {
        println!("\n{}", t!("webdav.list.no_config").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.back_cancel").to_string()];
    items.extend(configs.iter().map(|c| format!("{} - {}", c.name, c.url)));

    let selection = Select::new()
        .with_prompt(t!("webdav.upload.select_config"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let config = &configs[idx];

        // 列出远程文件
        println!("\n{}", t!("webdav.download.getting_files").cyan());
        let manager = webdav::WebDavManager::from_config(config.clone()).await?;
        let files = manager.list_remote_files().await?;

        if files.is_empty() {
            println!("{}", t!("webdav.download.no_files").yellow());
            return Ok(());
        }

        let mut file_items: Vec<String> = vec![t!("common.back_cancel").to_string()];
        file_items.extend(files.clone());

        let file_selection = Select::new()
            .with_prompt(t!("webdav.download.select_file"))
            .items(&file_items)
            .interact_opt()?;

        if let Some(file_idx) = file_selection {
            if file_idx == 0 {
                return Ok(());
            }
            let file_idx = file_idx - 1;
            let filename = &files[file_idx];

            println!("\n{}", t!("webdav.download.downloading").cyan());

            match manager.download_config(filename).await {
                Ok(data) => {
                    // 导入配置到数据库
                    let db_lock = db.lock().await;
                    let pool = db_lock.get_pool();

                    // 先删除所有现有账号和 Base URLs,实现完全覆盖
                    println!("\n{}", t!("webdav.upload.clearing").yellow());

                    let _ = sqlx::query("DELETE FROM accounts").execute(pool).await;

                    let _ = sqlx::query("DELETE FROM base_urls").execute(pool).await;

                    println!("{}", t!("webdav.upload.cleared").green());

                    // 解析账号数据
                    if let Some(accounts_array) = data.get("accounts").and_then(|v| v.as_array()) {
                        println!("\n{}", t!("webdav.upload.importing_accounts").cyan());
                        let mut success_count = 0;

                        for account_data in accounts_array {
                            if let (Some(name), Some(token), Some(base_url)) = (
                                account_data.get("name").and_then(|v| v.as_str()),
                                account_data.get("token").and_then(|v| v.as_str()),
                                account_data.get("base_url").and_then(|v| v.as_str()),
                            ) {
                                let model = account_data
                                    .get("model")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("claude-sonnet-4-20250514");

                                let request = CreateAccountRequest {
                                    name: name.to_string(),
                                    token: token.to_string(),
                                    base_url: base_url.to_string(),
                                    model: model.to_string(),
                                };

                                if let Ok(_) = db_lock.create_account(request).await {
                                    success_count += 1;
                                }
                            }
                        }

                        println!("{}", t!("webdav.upload.imported_accounts").replace("{}", &success_count.to_string()).green());
                    }

                    // 解析 Base URLs 数据
                    if let Some(base_urls_array) = data.get("base_urls").and_then(|v| v.as_array())
                    {
                        println!("\n{}", t!("webdav.upload.importing_urls").cyan());
                        let mut success_count = 0;

                        for base_url_data in base_urls_array {
                            if let (Some(name), Some(url)) = (
                                base_url_data.get("name").and_then(|v| v.as_str()),
                                base_url_data.get("url").and_then(|v| v.as_str()),
                            ) {
                                let description = base_url_data
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                let is_default =
                                    base_url_data.get("is_default").and_then(|v| v.as_bool());

                                let request = CreateBaseUrlRequest {
                                    name: name.to_string(),
                                    url: url.to_string(),
                                    description,
                                    is_default,
                                };

                                if let Ok(_) = db_lock.create_base_url(request).await {
                                    success_count += 1;
                                }
                            }
                        }

                        println!(
                            "{}",
                            t!("webdav.upload.imported_urls").replace("{}", &success_count.to_string()).green()
                        );
                    }

                    // 解析 Claude 设置数据
                    if let Some(claude_settings) = data.get("claude_settings") {
                        let settings_json = serde_json::to_string(claude_settings)?;
                        let _ = db_lock.save_claude_settings(&settings_json).await;
                    }

                    println!(
                        "{}",
                        t!("webdav.download.success").replace("{}", filename).green()
                    );

                    // 记录同步日志
                    let pool = db_lock.get_pool();
                    let _ = webdav::create_sync_log(
                        pool,
                        CreateSyncLogRequest {
                            webdav_config_id: config.id,
                            sync_type: "download".to_string(),
                            status: "success".to_string(),
                            message: Some(t!("webdav.download.success_log").replace("{}", filename).to_string()),
                        },
                    )
                    .await;

                    let _ = webdav::update_last_sync_time(pool, config.id).await;
                }
                Err(e) => {
                    println!("{}", t!("webdav.download.error").replace("{}", &e.to_string()).red());
                }
            }

            let _ = Input::<String>::new()
                .with_prompt(t!("common.continue"))
                .allow_empty(true)
                .interact()?;
        }
    }

    Ok(())
}

async fn list_remote_files(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let pool = db_lock.get_pool();
    let configs = webdav::get_webdav_configs(pool).await?;
    drop(db_lock);

    if configs.is_empty() {
        println!("\n{}", t!("webdav.list.no_config").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.back_cancel").to_string()];
    items.extend(configs.iter().map(|c| format!("{} - {}", c.name, c.url)));

    let selection = Select::new()
        .with_prompt(t!("webdav.upload.select_config"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let config = &configs[idx];

        println!("\n{}", t!("webdav.download.getting_files").cyan());

        let manager = webdav::WebDavManager::from_config(config.clone()).await?;

        match manager.list_remote_files().await {
            Ok(files) => {
                if files.is_empty() {
                    println!("{}", t!("webdav.download.no_files").yellow());
                } else {
                    println!("\n{}", t!("webdav.list.title").green().bold());
                    for (i, file) in files.iter().enumerate() {
                        println!("  {}. {}", i + 1, file);
                    }
                }
            }
            Err(e) => {
                println!("{}", t!("webdav.list.error").replace("{}", &e.to_string()).red());
            }
        }

        let _ = Input::<String>::new()
            .with_prompt(t!("common.continue"))
            .allow_empty(true)
            .interact()?;
    }

    Ok(())
}

async fn delete_config(db: &DbState) -> Result<()> {
    let db_lock = db.lock().await;
    let pool = db_lock.get_pool();
    let configs = webdav::get_webdav_configs(pool).await?;
    drop(db_lock);

    if configs.is_empty() {
        println!("\n{}", t!("webdav.list.no_config").yellow());
        return Ok(());
    }

    let mut items: Vec<String> = vec![t!("common.back_cancel").to_string()];
    items.extend(configs.iter().map(|c| format!("{} - {}", c.name, c.url)));

    let selection = Select::new()
        .with_prompt(t!("webdav.delete.select_config"))
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        if idx == 0 {
            return Ok(());
        }
        let idx = idx - 1;
        let config = &configs[idx];

        if Confirm::new()
            .with_prompt(t!("webdav.delete.confirm").replace("{}", &config.name))
            .default(false)
            .interact()?
        {
            let db_lock = db.lock().await;
            let pool = db_lock.get_pool();

            match webdav::delete_webdav_config(pool, config.id).await {
                Ok(_) => {
                    println!("\n{}", t!("webdav.delete.success").green());
                }
                Err(e) => {
                    println!("\n{}", t!("webdav.delete.error").replace("{}", &e.to_string()).red());
                }
            }
        }
    }

    Ok(())
}
