use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};

use crate::i18n::{self, Language};

/// 设置菜单
pub async fn settings_menu() -> Result<()> {
    loop {
        let current_lang = i18n::current_language();
        let lang_display = match current_lang {
            Language::ZhCN => "中文 (简体)",
            Language::EnUS => "English (US)",
        };

        println!("\n{}", "========================================".bright_blue());
        println!("{}", format!("      {}      ", i18n::translate("menu.settings.title")).bright_blue().bold());
        println!("{}", "========================================".bright_blue());
        println!();
        println!("{}: {}", i18n::translate("menu.settings.current_lang").cyan(), lang_display.green().bold());
        println!();

        let items = vec![
            i18n::translate("menu.settings.language"),
            i18n::translate("menu.settings.back"),
        ];

        let selection = Select::new()
            .with_prompt("\n请选择操作")
            .items(&items)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                language_settings().await?;
            }
            1 => {
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// 语言设置
async fn language_settings() -> Result<()> {
    println!("\n{}", "========================================".bright_blue());
    println!("{}", format!("      {}      ", i18n::translate("menu.settings.language")).bright_blue().bold());
    println!("{}", "========================================".bright_blue());
    println!();

    let languages = vec![
        ("中文 (简体)", Language::ZhCN),
        ("English (US)", Language::EnUS),
    ];

    let items: Vec<&str> = languages.iter().map(|(name, _)| *name).collect();

    let current_lang = i18n::current_language();
    let default_index = languages.iter()
        .position(|(_, lang)| *lang == current_lang)
        .unwrap_or(0);

    let selection = Select::new()
        .with_prompt(i18n::translate("menu.settings.select_lang"))
        .items(&items)
        .default(default_index)
        .interact()?;

    let (lang_name, new_lang) = languages[selection];

    if new_lang != current_lang {
        i18n::set_language(new_lang);
        println!("\n{} {}", "✓".green(), i18n::translate("menu.settings.lang_changed").green());
        println!("{}: {}", i18n::translate("menu.settings.current_lang"), lang_name.green().bold());
    }

    let _ = Input::<String>::new()
        .with_prompt(format!("\n{}", i18n::translate("common.continue")))
        .allow_empty(true)
        .interact()?;

    Ok(())
}
