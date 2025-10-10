use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select, Confirm};
use crate::{DbState, models::*, webdav};
use comfy_table::{Attribute, Cell, Color};

pub async fn webdav_menu(db: &DbState) -> Result<()> {
    loop {
        let items = vec![
            "🔙 返回主菜单",
            "📝 查看 WebDAV 配置",
            "➕ 添加 WebDAV 配置",
            "🧪 测试连接",
            "⬆️  上传配置到云端",
            "⬇️  从云端下载配置",
            "📂 查看远程文件",
            "🗑️  删除配置",
        ];

        let selection = Select::new()
            .with_prompt("\nWebDAV 同步管理")
            .items(&items)
            .default(0)
            .interact()?;

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
        println!("\n{}", "暂无 WebDAV 配置".yellow());
        return Ok(());
    }

    let mut table = super::create_table();
    table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("名称").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("URL").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("用户名").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("远程路径").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("自动同步").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("状态").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);

    for config in &configs {
        let auto_sync = if config.auto_sync { "✓" } else { "✗" };
        let status = if config.is_active { "🟢 活跃" } else { "⚪ 未活跃" };

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
    println!("共 {} 个配置", configs.len());

    let _ = Input::<String>::new()
        .with_prompt("按 Enter 继续")
        .allow_empty(true)
        .interact()?;

    Ok(())
}

async fn add_config(db: &DbState) -> Result<()> {
    println!("\n{}", "添加 WebDAV 配置".green().bold());

    let name: String = Input::new()
        .with_prompt("配置名称")
        .interact()?;

    let url: String = Input::new()
        .with_prompt("WebDAV URL")
        .interact()?;

    let username: String = Input::new()
        .with_prompt("用户名")
        .interact()?;

    let password: String = Input::new()
        .with_prompt("密码")
        .interact()?;

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
            println!("\n{}", format!("✓ WebDAV 配置 '{}' 创建成功", name).green());
        }
        Err(e) => {
            println!("\n{}", format!("✗ 创建失败: {}", e).red());
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
        println!("\n{}", "暂无 WebDAV 配置".yellow());
        return Ok(());
    }

    let items: Vec<String> = configs
        .iter()
        .map(|c| format!("{} - {}", c.name, c.url))
        .collect();

    let selection = Select::new()
        .with_prompt("选择要测试的配置")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        let config = &configs[idx];

        println!("\n{}", "正在测试连接...".cyan());

        let manager = webdav::WebDavManager::from_config(config.clone()).await?;

        match manager.test_connection().await {
            Ok(_) => {
                println!("{}", "✓ WebDAV 连接测试成功".green());
            }
            Err(e) => {
                println!("{}", format!("✗ 连接测试失败: {}", e).red());
            }
        }

        let _ = Input::<String>::new()
            .with_prompt("按 Enter 继续")
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
        println!("\n{}", "暂无 WebDAV 配置".yellow());
        return Ok(());
    }

    let items: Vec<String> = configs
        .iter()
        .map(|c| format!("{} - {}", c.name, c.url))
        .collect();

    drop(db_lock);

    let selection = Select::new()
        .with_prompt("选择 WebDAV 配置")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        let config = &configs[idx];

        let filename: String = Input::new()
            .with_prompt("文件名")
            .default(format!(
                "claude-config-{}.json",
                chrono::Local::now().format("%Y%m%d-%H%M%S")
            ))
            .interact()?;

        println!("\n{}", "正在上传配置到云端...".cyan());

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
        let claude_settings: serde_json::Value =
            serde_json::from_str(&claude_settings_json)?;

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
                println!("{}", format!("✓ 配置已成功上传到 WebDAV: {}", filename).green());

                // 记录同步日志
                let db_lock = db.lock().await;
                let pool = db_lock.get_pool();
                let _ = webdav::create_sync_log(
                    pool,
                    CreateSyncLogRequest {
                        webdav_config_id: config.id,
                        sync_type: "upload".to_string(),
                        status: "success".to_string(),
                        message: Some(format!("成功上传配置文件: {}", filename)),
                    },
                )
                .await;

                let _ = webdav::update_last_sync_time(pool, config.id).await;
            }
            Err(e) => {
                println!("{}", format!("✗ 上传失败: {}", e).red());
            }
        }

        let _ = Input::<String>::new()
            .with_prompt("按 Enter 继续")
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
        println!("\n{}", "暂无 WebDAV 配置".yellow());
        return Ok(());
    }

    let items: Vec<String> = configs
        .iter()
        .map(|c| format!("{} - {}", c.name, c.url))
        .collect();

    let selection = Select::new()
        .with_prompt("选择 WebDAV 配置")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        let config = &configs[idx];

        // 列出远程文件
        println!("\n{}", "正在获取远程文件列表...".cyan());
        let manager = webdav::WebDavManager::from_config(config.clone()).await?;
        let files = manager.list_remote_files().await?;

        if files.is_empty() {
            println!("{}", "远程没有配置文件".yellow());
            return Ok(());
        }

        let file_selection = Select::new()
            .with_prompt("选择要下载的文件")
            .items(&files)
            .interact_opt()?;

        if let Some(file_idx) = file_selection {
            let filename = &files[file_idx];

            println!("\n{}", "正在从云端下载配置...".cyan());

            match manager.download_config(filename).await {
                Ok(data) => {
                    // 导入配置到数据库
                    let db_lock = db.lock().await;
                    let pool = db_lock.get_pool();

                    // 先删除所有现有账号和 Base URLs,实现完全覆盖
                    println!("\n{}", "正在清空现有配置...".yellow());

                    let _ = sqlx::query("DELETE FROM accounts")
                        .execute(pool)
                        .await;

                    let _ = sqlx::query("DELETE FROM base_urls")
                        .execute(pool)
                        .await;

                    println!("{}", "✓ 已清空现有账号和 Base URLs".green());

                    // 解析账号数据
                    if let Some(accounts_array) = data.get("accounts").and_then(|v| v.as_array())
                    {
                        println!("\n{}", "正在导入账号...".cyan());
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

                        println!("{}", format!("✓ 成功导入 {} 个账号", success_count).green());
                    }

                    // 解析 Base URLs 数据
                    if let Some(base_urls_array) =
                        data.get("base_urls").and_then(|v| v.as_array())
                    {
                        println!("\n{}", "正在导入 Base URLs...".cyan());
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

                        println!("{}", format!("✓ 成功导入 {} 个 Base URL", success_count).green());
                    }

                    // 解析 Claude 设置数据
                    if let Some(claude_settings) = data.get("claude_settings") {
                        let settings_json = serde_json::to_string(claude_settings)?;
                        let _ = db_lock.save_claude_settings(&settings_json).await;
                    }

                    println!(
                        "{}",
                        format!("✓ 配置已成功从 WebDAV 下载并导入: {}", filename).green()
                    );

                    // 记录同步日志
                    let pool = db_lock.get_pool();
                    let _ = webdav::create_sync_log(
                        pool,
                        CreateSyncLogRequest {
                            webdav_config_id: config.id,
                            sync_type: "download".to_string(),
                            status: "success".to_string(),
                            message: Some(format!("成功下载并导入配置文件: {}", filename)),
                        },
                    )
                    .await;

                    let _ = webdav::update_last_sync_time(pool, config.id).await;
                }
                Err(e) => {
                    println!("{}", format!("✗ 下载失败: {}", e).red());
                }
            }

            let _ = Input::<String>::new()
                .with_prompt("按 Enter 继续")
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
        println!("\n{}", "暂无 WebDAV 配置".yellow());
        return Ok(());
    }

    let items: Vec<String> = configs
        .iter()
        .map(|c| format!("{} - {}", c.name, c.url))
        .collect();

    let selection = Select::new()
        .with_prompt("选择 WebDAV 配置")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        let config = &configs[idx];

        println!("\n{}", "正在获取远程文件列表...".cyan());

        let manager = webdav::WebDavManager::from_config(config.clone()).await?;

        match manager.list_remote_files().await {
            Ok(files) => {
                if files.is_empty() {
                    println!("{}", "远程没有配置文件".yellow());
                } else {
                    println!("\n{}", "远程文件列表:".green().bold());
                    for (i, file) in files.iter().enumerate() {
                        println!("  {}. {}", i + 1, file);
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("✗ 获取文件列表失败: {}", e).red());
            }
        }

        let _ = Input::<String>::new()
            .with_prompt("按 Enter 继续")
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
        println!("\n{}", "暂无 WebDAV 配置".yellow());
        return Ok(());
    }

    let items: Vec<String> = configs
        .iter()
        .map(|c| format!("{} - {}", c.name, c.url))
        .collect();

    let selection = Select::new()
        .with_prompt("选择要删除的配置")
        .items(&items)
        .interact_opt()?;

    if let Some(idx) = selection {
        let config = &configs[idx];

        if Confirm::new()
            .with_prompt(format!("确定要删除配置 '{}' 吗?", config.name))
            .default(false)
            .interact()?
        {
            let db_lock = db.lock().await;
            let pool = db_lock.get_pool();

            match webdav::delete_webdav_config(pool, config.id).await {
                Ok(_) => {
                    println!("\n{}", "✓ 配置删除成功".green());
                }
                Err(e) => {
                    println!("\n{}", format!("✗ 删除失败: {}", e).red());
                }
            }
        }
    }

    Ok(())
}
