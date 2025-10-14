use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// 支持的语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    ZhCN,
    EnUS,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::ZhCN => "zh-CN",
            Language::EnUS => "en-US",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "zh-CN" | "zh" => Some(Language::ZhCN),
            "en-US" | "en" => Some(Language::EnUS),
            _ => None,
        }
    }
}

/// 全局当前语言
static CURRENT_LANG: Lazy<RwLock<Language>> = Lazy::new(|| {
    // 从环境变量读取语言设置，默认为中文
    let lang = std::env::var("LANG")
        .ok()
        .and_then(|l| {
            if l.starts_with("zh") {
                Some(Language::ZhCN)
            } else if l.starts_with("en") {
                Some(Language::EnUS)
            } else {
                None
            }
        })
        .unwrap_or(Language::ZhCN);

    RwLock::new(lang)
});

/// 获取当前语言
pub fn current_language() -> Language {
    *CURRENT_LANG.read().unwrap()
}

/// 设置当前语言
pub fn set_language(lang: Language) {
    *CURRENT_LANG.write().unwrap() = lang;
}

/// 翻译键
pub type TransKey = &'static str;

/// 翻译文本的宏
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::translate($key)
    };
}

/// 翻译文本
pub fn translate(key: TransKey) -> &'static str {
    let lang = current_language();
    TRANSLATIONS.get(&lang)
        .and_then(|map| map.get(key))
        .copied()
        .unwrap_or(key)
}

/// 所有翻译文本
static TRANSLATIONS: Lazy<HashMap<Language, HashMap<TransKey, &'static str>>> = Lazy::new(|| {
    let mut translations = HashMap::new();

    // 中文翻译
    let mut zh_cn = HashMap::new();

    // 通用
    zh_cn.insert("app.name", "Claude Code 配置管理器");
    zh_cn.insert("app.version", "v1.3.0");
    zh_cn.insert("app.cli_subtitle", "命令行版本");
    zh_cn.insert("app.exit_message", "感谢使用 Claude Code 配置管理器！");

    // 主菜单
    zh_cn.insert("menu.main.title", "请选择操作");
    zh_cn.insert("menu.main.account", "📋 账号管理");
    zh_cn.insert("menu.main.directory", "📁 目录管理");
    zh_cn.insert("menu.main.url", "🌐 URL 管理");
    zh_cn.insert("menu.main.switch", "⚡ 配置切换");
    zh_cn.insert("menu.main.webdav", "☁️  WebDAV 同步");
    zh_cn.insert("menu.main.logs", "📝 查看日志");
    zh_cn.insert("menu.main.remove_root", "🔓 删除限制代码");
    zh_cn.insert("menu.main.settings", "⚙️  设置");
    zh_cn.insert("menu.main.exit", "❌ 退出程序");

    // 设置菜单
    zh_cn.insert("menu.settings.title", "设置");
    zh_cn.insert("menu.settings.language", "🌐 语言设置");
    zh_cn.insert("menu.settings.back", "🔙 返回主菜单");
    zh_cn.insert("menu.settings.current_lang", "当前语言");
    zh_cn.insert("menu.settings.select_lang", "请选择语言");
    zh_cn.insert("menu.settings.lang_changed", "语言已切换");

    // 通用操作
    zh_cn.insert("common.success", "✓ 操作成功");
    zh_cn.insert("common.error", "✗ 操作失败");
    zh_cn.insert("common.cancel", "操作已取消");
    zh_cn.insert("common.back", "返回");
    zh_cn.insert("common.continue", "按 Enter 继续");
    zh_cn.insert("common.confirm", "是否继续？");
    zh_cn.insert("common.loading", "加载中...");

    // 数据库
    zh_cn.insert("db.init", "正在初始化数据库...");
    zh_cn.insert("db.init_success", "✓ 数据库初始化成功");
    zh_cn.insert("db.init_error", "✗ 数据库初始化失败");
    zh_cn.insert("db.fallback", "尝试使用默认配置创建数据库...");
    zh_cn.insert("db.fallback_success", "✓ 使用默认配置创建数据库成功");
    zh_cn.insert("db.fallback_error", "✗ 无法初始化数据库");

    translations.insert(Language::ZhCN, zh_cn);

    // 英文翻译
    let mut en_us = HashMap::new();

    // Common
    en_us.insert("app.name", "Claude Code Configuration Manager");
    en_us.insert("app.version", "v1.3.0");
    en_us.insert("app.cli_subtitle", "CLI Version");
    en_us.insert("app.exit_message", "Thank you for using Claude Code Configuration Manager!");

    // Main menu
    en_us.insert("menu.main.title", "Please select an operation");
    en_us.insert("menu.main.account", "📋 Account Management");
    en_us.insert("menu.main.directory", "📁 Directory Management");
    en_us.insert("menu.main.url", "🌐 URL Management");
    en_us.insert("menu.main.switch", "⚡ Configuration Switch");
    en_us.insert("menu.main.webdav", "☁️  WebDAV Sync");
    en_us.insert("menu.main.logs", "📝 View Logs");
    en_us.insert("menu.main.remove_root", "🔓 Remove Root Check");
    en_us.insert("menu.main.settings", "⚙️  Settings");
    en_us.insert("menu.main.exit", "❌ Exit");

    // Settings menu
    en_us.insert("menu.settings.title", "Settings");
    en_us.insert("menu.settings.language", "🌐 Language Settings");
    en_us.insert("menu.settings.back", "🔙 Back to Main Menu");
    en_us.insert("menu.settings.current_lang", "Current Language");
    en_us.insert("menu.settings.select_lang", "Please select a language");
    en_us.insert("menu.settings.lang_changed", "Language changed");

    // Common operations
    en_us.insert("common.success", "✓ Operation successful");
    en_us.insert("common.error", "✗ Operation failed");
    en_us.insert("common.cancel", "Operation cancelled");
    en_us.insert("common.back", "Back");
    en_us.insert("common.continue", "Press Enter to continue");
    en_us.insert("common.confirm", "Do you want to continue?");
    en_us.insert("common.loading", "Loading...");

    // Database
    en_us.insert("db.init", "Initializing database...");
    en_us.insert("db.init_success", "✓ Database initialized successfully");
    en_us.insert("db.init_error", "✗ Database initialization failed");
    en_us.insert("db.fallback", "Trying to create database with default configuration...");
    en_us.insert("db.fallback_success", "✓ Database created with default configuration successfully");
    en_us.insert("db.fallback_error", "✗ Cannot initialize database");

    translations.insert(Language::EnUS, en_us);

    translations
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert_eq!(Language::ZhCN.code(), "zh-CN");
        assert_eq!(Language::EnUS.code(), "en-US");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("zh-CN"), Some(Language::ZhCN));
        assert_eq!(Language::from_code("zh"), Some(Language::ZhCN));
        assert_eq!(Language::from_code("en-US"), Some(Language::EnUS));
        assert_eq!(Language::from_code("en"), Some(Language::EnUS));
        assert_eq!(Language::from_code("fr"), None);
    }

    #[test]
    fn test_translate() {
        set_language(Language::ZhCN);
        assert_eq!(translate("app.name"), "Claude Code 配置管理器");

        set_language(Language::EnUS);
        assert_eq!(translate("app.name"), "Claude Code Configuration Manager");
    }
}
