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
    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            Language::ZhCN => "zh-CN",
            Language::EnUS => "en-US",
        }
    }

    #[allow(dead_code)]
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
    TRANSLATIONS
        .get(&lang)
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
    zh_cn.insert("app.version", "v1.6.0");
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
    zh_cn.insert("menu.main.language", "🌐 English");
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
    zh_cn.insert("common.back_cancel", "🔙 取消");
    zh_cn.insert("common.continue", "按 Enter 继续");
    zh_cn.insert("common.confirm", "是否继续？");
    zh_cn.insert("common.loading", "加载中...");
    zh_cn.insert("common.select_operation", "请选择操作");
    zh_cn.insert("common.to_exit", "按ESC退出");
    zh_cn.insert("common.to_back", "按ESC返回");
    zh_cn.insert(
        "common.input_cancel_hint",
        "提示: 直接按Enter（不输入任何内容）可取消",
    );

    // 数据库
    zh_cn.insert("db.init", "正在初始化数据库...");
    zh_cn.insert("db.init_success", "✓ 数据库初始化成功");
    zh_cn.insert("db.init_error", "✗ 数据库初始化失败");
    zh_cn.insert("db.fallback", "尝试使用默认配置创建数据库...");
    zh_cn.insert("db.fallback_success", "✓ 使用默认配置创建数据库成功");
    zh_cn.insert("db.fallback_error", "✗ 无法初始化数据库");

    // 账号管理
    zh_cn.insert("account.menu.title", "账号管理");
    zh_cn.insert("account.menu.list", "📝 查看所有账号");
    zh_cn.insert("account.menu.add", "➕ 添加新账号");
    zh_cn.insert("account.menu.edit", "✏️  编辑账号");
    zh_cn.insert("account.menu.delete", "🗑️  删除账号");
    zh_cn.insert("account.menu.import", "📥 批量导入");
    zh_cn.insert("account.menu.export", "📤 批量导出");
    zh_cn.insert("account.list.no_records", "暂无账号记录");
    zh_cn.insert("account.list.header_id", "ID");
    zh_cn.insert("account.list.header_name", "账号名称");
    zh_cn.insert("account.list.header_base_url", "Base URL");
    zh_cn.insert("account.list.header_model", "模型");
    zh_cn.insert("account.list.header_status", "状态");
    zh_cn.insert("account.list.status_active", "🟢 活跃");
    zh_cn.insert("account.list.status_inactive", "⚪ 未活跃");
    zh_cn.insert("account.list.total", "共 {} 个账号");
    zh_cn.insert("account.add.title", "添加新账号");
    zh_cn.insert("account.add.prompt_name", "账号名称");
    zh_cn.insert("account.add.prompt_token", "API Token");
    zh_cn.insert("account.add.prompt_base_url", "Base URL");
    zh_cn.insert("account.add.prompt_model", "模型");
    zh_cn.insert("account.add.no_base_url", "暂无可用的 Base URL，请手动输入");
    zh_cn.insert("account.add.select_base_url", "选择 Base URL");
    zh_cn.insert("account.add.success", "✓ 账号 '{}' 创建成功");
    zh_cn.insert("account.add.error", "✗ 创建失败: {}");
    zh_cn.insert("account.edit.prompt", "选择要编辑的账号");
    zh_cn.insert("account.edit.success", "✓ 账号更新成功");
    zh_cn.insert("account.edit.error", "✗ 更新失败: {}");
    zh_cn.insert("account.delete.prompt", "选择要删除的账号");
    zh_cn.insert("account.delete.confirm", "确定要删除账号 '{}' 吗?");
    zh_cn.insert("account.delete.success", "✓ 账号删除成功");
    zh_cn.insert("account.delete.error", "✗ 删除失败: {}");
    zh_cn.insert("account.export.title", "批量导出账号");
    zh_cn.insert("account.export.no_accounts", "暂无账号可导出");
    zh_cn.insert(
        "account.export.success",
        "✓ 成功导出 {} 个账号到文件: {file}",
    );
    zh_cn.insert("account.import.title", "批量导入账号");
    zh_cn.insert("account.import.prompt_file", "请输入JSON文件路径");
    zh_cn.insert("account.import.error_read", "✗ 读取文件失败: {}");
    zh_cn.insert("account.import.error_parse", "✗ 解析JSON失败: {}");
    zh_cn.insert(
        "account.import.error_format",
        "✗ 文件格式无效，缺少 providers 数组",
    );
    zh_cn.insert("account.import.no_accounts", "文件中没有账号数据");
    zh_cn.insert("account.import.processing", "正在处理导入...");
    zh_cn.insert("account.import.skip_invalid", "跳过无效数据");
    zh_cn.insert("account.import.skip_exists", "跳过已存在");
    zh_cn.insert("account.import.imported", "已导入");
    zh_cn.insert("account.import.failed", "导入失败");
    zh_cn.insert("account.import.result_imported", "✓ 成功导入 {} 个账号");
    zh_cn.insert("account.import.result_skipped", "⊖ 跳过 {} 个账号");
    zh_cn.insert("account.default_indicator", "(默认)");

    // 目录管理
    zh_cn.insert("directory.menu.title", "目录管理");
    zh_cn.insert("directory.menu.list", "📝 查看所有目录");
    zh_cn.insert("directory.menu.add", "➕ 添加新目录");
    zh_cn.insert("directory.menu.edit", "✏️  编辑目录");
    zh_cn.insert("directory.menu.delete", "🗑️  删除目录");
    zh_cn.insert("directory.list.no_records", "暂无目录记录");
    zh_cn.insert("directory.list.header_id", "ID");
    zh_cn.insert("directory.list.header_name", "目录名称");
    zh_cn.insert("directory.list.header_path", "路径");
    zh_cn.insert("directory.list.header_exists", "存在性");
    zh_cn.insert("directory.list.exists", "✓ 存在");
    zh_cn.insert("directory.list.not_exists", "✗ 不存在");
    zh_cn.insert("directory.list.total", "共 {} 个目录");
    zh_cn.insert("directory.add.title", "添加新目录");
    zh_cn.insert("directory.add.prompt_name", "目录名称");
    zh_cn.insert("directory.add.prompt_path", "路径");
    zh_cn.insert(
        "directory.add.warn_path_not_exists",
        "⚠️  警告: 该路径不存在",
    );
    zh_cn.insert("directory.add.success", "✓ 目录 '{}' 添加成功");
    zh_cn.insert("directory.add.error", "✗ 添加失败: {}");
    zh_cn.insert("directory.edit.prompt", "选择要编辑的目录");
    zh_cn.insert("directory.edit.success", "✓ 目录更新成功");
    zh_cn.insert("directory.edit.error", "✗ 更新失败: {}");
    zh_cn.insert("directory.delete.prompt", "选择要删除的目录");
    zh_cn.insert("directory.delete.confirm", "确定要删除目录 '{}' 吗?");
    zh_cn.insert(
        "directory.delete.warning",
        "(仅删除数据库记录，不删除实际文件)",
    );
    zh_cn.insert("directory.delete.success", "✓ 目录删除成功");
    zh_cn.insert("directory.delete.error", "✗ 删除失败: {}");

    // URL管理
    zh_cn.insert("url.menu.title", "URL 管理");
    zh_cn.insert("url.menu.list", "📝 查看所有 URL");
    zh_cn.insert("url.menu.add", "➕ 添加新 URL");
    zh_cn.insert("url.menu.edit", "✏️  编辑 URL");
    zh_cn.insert("url.menu.delete", "🗑️  删除 URL");
    zh_cn.insert("url.list.no_records", "暂无 URL 记录");
    zh_cn.insert("url.list.header_id", "ID");
    zh_cn.insert("url.list.header_name", "名称");
    zh_cn.insert("url.list.header_url", "URL");
    zh_cn.insert("url.list.header_description", "描述");
    zh_cn.insert("url.list.header_api_key", "API Key 环境变量");
    zh_cn.insert("url.list.header_default", "默认");
    zh_cn.insert("url.list.default_yes", "是");
    zh_cn.insert("url.list.default_no", "否");
    zh_cn.insert("url.list.total", "共 {} 个 URL");
    zh_cn.insert("url.add.title", "添加新 URL");
    zh_cn.insert("url.add.prompt_name", "名称");
    zh_cn.insert("url.add.prompt_url", "URL");
    zh_cn.insert("url.add.prompt_description", "描述（可选）");
    zh_cn.insert(
        "url.add.prompt_api_key",
        "API Key 环境变量名（默认: ANTHROPIC_API_KEY）",
    );
    zh_cn.insert("url.add.prompt_default", "设为默认?");
    zh_cn.insert("url.add.success", "✓ URL '{}' 创建成功");
    zh_cn.insert("url.add.error", "✗ 创建失败: {}");
    zh_cn.insert("url.edit.prompt", "选择要编辑的 URL");
    zh_cn.insert("url.edit.success", "✓ URL 更新成功");
    zh_cn.insert("url.edit.error", "✗ 更新失败: {}");
    zh_cn.insert("url.delete.prompt", "选择要删除的 URL");
    zh_cn.insert("url.delete.confirm", "确定要删除 URL '{}' 吗?");
    zh_cn.insert("url.delete.warning", "(使用该 URL 的账号也将被删除)");
    zh_cn.insert("url.delete.success", "✓ URL 删除成功");
    zh_cn.insert("url.delete.error", "✗ 删除失败: {}");

    // 配置切换
    zh_cn.insert("switch.title", "配置切换");
    zh_cn.insert("switch.no_accounts", "暂无账号记录，请先添加账号");
    zh_cn.insert("switch.no_directories", "暂无目录记录，请先添加目录");
    zh_cn.insert("switch.select_account", "选择账号");
    zh_cn.insert("switch.select_directory", "选择目录");
    zh_cn.insert(
        "switch.prompt_skip_permissions",
        "跳过权限检查? (推荐选择 Yes)",
    );
    zh_cn.insert(
        "switch.prompt_use_proxy",
        "使用代理? (从 Claude 配置中加载代理设置)",
    );
    zh_cn.insert("switch.switching", "正在切换配置...");
    zh_cn.insert("switch.success", "✓ 配置切换成功!");
    zh_cn.insert("switch.success_env", "✓ 环境配置切换成功!");
    zh_cn.insert("switch.account", "  账号: {}");
    zh_cn.insert("switch.directory", "  目录: {}");
    zh_cn.insert("switch.path", "  路径: {}");
    zh_cn.insert("switch.sandbox", "  沙盒模式: 已启用");
    zh_cn.insert("switch.permission", "  权限检查: {}");
    zh_cn.insert("switch.permission_skipped", "已跳过");
    zh_cn.insert("switch.permission_required", "需要确认");
    zh_cn.insert("switch.proxy", "  代理: {}");
    zh_cn.insert("switch.proxy_enabled", "已启用");
    zh_cn.insert("switch.proxy_disabled", "未启用");
    zh_cn.insert(
        "switch.warn_claude_config",
        "警告: 获取Claude配置失败，使用默认配置: {}",
    );
    zh_cn.insert("switch.warn_write_fail", "警告: Claude配置写入失败: {}");
    zh_cn.insert("switch.error_update", "✗ 配置文件更新失败: {}");
    zh_cn.insert("switch.error", "✗ 切换失败: {}");
    zh_cn.insert(
        "switch.claude_local_md_found",
        "⚠️  目标目录中已存在 CLAUDE.local.md 文件",
    );
    zh_cn.insert(
        "switch.keep_claude_local_md",
        "是否保留现有的 CLAUDE.local.md 文件? (选择 No 将覆盖)",
    );

    // WebDAV 同步
    zh_cn.insert("webdav.menu.title", "WebDAV 同步管理");
    zh_cn.insert("webdav.menu.back", "🔙 返回主菜单");
    zh_cn.insert("webdav.menu.list", "📝 查看 WebDAV 配置");
    zh_cn.insert("webdav.menu.add", "➕ 添加 WebDAV 配置");
    zh_cn.insert("webdav.menu.test_connection", "🧪 测试连接");
    zh_cn.insert("webdav.menu.upload_config", "⬆️  上传配置到云端");
    zh_cn.insert("webdav.menu.download_config", "⬇️  从云端下载配置");
    zh_cn.insert("webdav.menu.list_remote", "📂 查看远程文件");
    zh_cn.insert("webdav.menu.delete_config", "🗑️  删除配置");
    zh_cn.insert("webdav.list.no_config", "暂无 WebDAV 配置");
    zh_cn.insert("webdav.list.header_id", "ID");
    zh_cn.insert("webdav.list.header_name", "名称");
    zh_cn.insert("webdav.list.header_url", "URL");
    zh_cn.insert("webdav.list.header_username", "用户名");
    zh_cn.insert("webdav.list.header_remote_path", "远程路径");
    zh_cn.insert("webdav.list.header_auto_sync", "自动同步");
    zh_cn.insert("webdav.list.header_status", "状态");
    zh_cn.insert("webdav.list.auto_sync_yes", "✓");
    zh_cn.insert("webdav.list.auto_sync_no", "✗");
    zh_cn.insert("webdav.list.status_active", "🟢 活跃");
    zh_cn.insert("webdav.list.status_inactive", "⚪ 未活跃");
    zh_cn.insert("webdav.list.total", "共 {} 个配置");
    zh_cn.insert("webdav.add.title", "添加 WebDAV 配置");
    zh_cn.insert("webdav.add.prompt_name", "配置名称");
    zh_cn.insert("webdav.add.prompt_url", "WebDAV URL");
    zh_cn.insert("webdav.add.prompt_username", "用户名");
    zh_cn.insert("webdav.add.prompt_password", "密码");
    zh_cn.insert("webdav.add.success", "✓ WebDAV 配置 '{}' 创建成功");
    zh_cn.insert("webdav.add.error", "✗ 创建失败: {}");
    zh_cn.insert("webdav.test.select_config", "选择要测试的配置");
    zh_cn.insert("webdav.test.testing", "正在测试连接...");
    zh_cn.insert("webdav.test.success", "✓ WebDAV 连接测试成功");
    zh_cn.insert("webdav.test.error", "✗ 连接测试失败: {}");
    zh_cn.insert("webdav.upload.select_config", "选择 WebDAV 配置");
    zh_cn.insert("webdav.upload.prompt_filename", "文件名");
    zh_cn.insert("webdav.upload.uploading", "正在上传配置到云端...");
    zh_cn.insert("webdav.upload.clearing", "正在清空现有配置...");
    zh_cn.insert("webdav.upload.cleared", "✓ 已清空现有账号和 Base URLs");
    zh_cn.insert("webdav.upload.importing_accounts", "正在导入账号...");
    zh_cn.insert("webdav.upload.imported_accounts", "✓ 成功导入 {} 个账号");
    zh_cn.insert("webdav.upload.importing_urls", "正在导入 Base URLs...");
    zh_cn.insert("webdav.upload.imported_urls", "✓ 成功导入 {} 个 Base URL");
    zh_cn.insert("webdav.upload.success", "✓ 配置已成功上传到 WebDAV: {}");
    zh_cn.insert("webdav.upload.success_log", "成功上传配置文件: {}");
    zh_cn.insert("webdav.upload.error", "✗ 上传失败: {}");
    zh_cn.insert("webdav.download.getting_files", "正在获取远程文件列表...");
    zh_cn.insert("webdav.download.no_files", "远程没有配置文件");
    zh_cn.insert("webdav.download.select_file", "选择要下载的文件");
    zh_cn.insert("webdav.download.downloading", "正在从云端下载配置...");
    zh_cn.insert(
        "webdav.download.success",
        "✓ 配置已成功从 WebDAV 下载并导入: {}",
    );
    zh_cn.insert("webdav.download.success_log", "成功下载并导入配置文件: {}");
    zh_cn.insert("webdav.download.error", "✗ 下载失败: {}");
    zh_cn.insert("webdav.list.title", "远程文件列表:");
    zh_cn.insert("webdav.list.error", "✗ 获取文件列表失败: {}");
    zh_cn.insert("webdav.delete.select_config", "选择要删除的配置");
    zh_cn.insert("webdav.delete.confirm", "确定要删除配置 '{}' 吗?");
    zh_cn.insert("webdav.delete.success", "✓ 配置删除成功");
    zh_cn.insert("webdav.delete.error", "✗ 删除失败: {}");

    // 日志查看
    zh_cn.insert("logs.menu.title", "日志管理");
    zh_cn.insert("logs.menu.back", "🔙 返回主菜单");
    zh_cn.insert("logs.menu.view_recent", "📝 查看最近日志");
    zh_cn.insert("logs.menu.info", "📊 日志文件信息");
    zh_cn.insert("logs.menu.open_dir", "📂 打开日志目录");
    zh_cn.insert("logs.prompt_lines", "显示最近多少行日志");
    zh_cn.insert("logs.title", "最近的日志:");
    zh_cn.insert("logs.no_records", "暂无日志记录");
    zh_cn.insert("logs.info.title", "日志文件信息:");
    zh_cn.insert("logs.file", "  日志文件: {}");
    zh_cn.insert("logs.size", "  文件大小: {}");
    zh_cn.insert("logs.lines", "  总行数: {}");
    zh_cn.insert("logs.info.error", "✗ 获取日志信息失败: {}");
    zh_cn.insert("logs.directory", "日志目录: {}");
    zh_cn.insert("logs.directory_opened", "✓ 已打开日志目录");
    zh_cn.insert("logs.directory.error", "✗ 获取日志目录失败: {}");
    zh_cn.insert("logs.open_dir.error", "✗ 打开目录失败: {}");
    zh_cn.insert("logs.read.error", "✗ 读取日志失败: {}");

    // 删除限制代码
    zh_cn.insert("remove_root.title", "删除 Claude Code Root Check");
    zh_cn.insert("remove_root.steps_intro", "此操作将执行以下步骤:");
    zh_cn.insert("remove_root.step1", "  1. 查找 claude 命令位置");
    zh_cn.insert(
        "remove_root.step2",
        "  2. 创建包装脚本自动删除 root check 限制",
    );
    zh_cn.insert("remove_root.step3", "  3. 备份原始 claude 命令");
    zh_cn.insert("remove_root.step4", "  4. 替换 claude 命令为包装脚本");
    zh_cn.insert("remove_root.confirm", "是否继续执行删除限制代码操作?");
    zh_cn.insert("remove_root.executing", "正在执行删除限制代码脚本...");
    zh_cn.insert("remove_root.success", "✓ 删除限制代码完成");
    zh_cn.insert("remove_root.error_exit", "✗ 脚本执行失败，退出代码: {}");
    zh_cn.insert("remove_root.error_execute", "✗ 执行脚本失败: {}");
    zh_cn.insert("remove_root.error_stderr", "错误输出:\n{}");
    zh_cn.insert("remove_root.error", "✗ 删除限制代码脚本不存在: {}");

    translations.insert(Language::ZhCN, zh_cn);

    // 英文翻译
    let mut en_us = HashMap::new();

    // Common
    en_us.insert("app.name", "Claude Code Configuration Manager");
    en_us.insert("app.version", "v1.6.0");
    en_us.insert("app.cli_subtitle", "CLI Version");
    en_us.insert(
        "app.exit_message",
        "Thank you for using Claude Code Configuration Manager!",
    );

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
    en_us.insert("menu.main.language", "🌐 中文");
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
    en_us.insert("common.back_cancel", "🔙 Cancel");
    en_us.insert("common.continue", "Press Enter to continue");
    en_us.insert("common.confirm", "Do you want to continue?");
    en_us.insert("common.loading", "Loading...");
    en_us.insert("common.select_operation", "Please select an operation");
    en_us.insert("common.to_exit", "press ESC to exit");
    en_us.insert("common.to_back", "press ESC to go back");
    en_us.insert(
        "common.input_cancel_hint",
        "Hint: Press Enter without typing anything to cancel",
    );

    // Database
    en_us.insert("db.init", "Initializing database...");
    en_us.insert("db.init_success", "✓ Database initialized successfully");
    en_us.insert("db.init_error", "✗ Database initialization failed");
    en_us.insert(
        "db.fallback",
        "Trying to create database with default configuration...",
    );
    en_us.insert(
        "db.fallback_success",
        "✓ Database created with default configuration successfully",
    );
    en_us.insert("db.fallback_error", "✗ Cannot initialize database");

    // Account Management
    en_us.insert("account.menu.title", "Account Management");
    en_us.insert("account.menu.list", "📝 View All Accounts");
    en_us.insert("account.menu.add", "➕ Add New Account");
    en_us.insert("account.menu.edit", "✏️  Edit Account");
    en_us.insert("account.menu.delete", "🗑️  Delete Account");
    en_us.insert("account.menu.import", "📥 Batch Import");
    en_us.insert("account.menu.export", "📤 Batch Export");
    en_us.insert("account.list.no_records", "No account records");
    en_us.insert("account.list.header_id", "ID");
    en_us.insert("account.list.header_name", "Account Name");
    en_us.insert("account.list.header_base_url", "Base URL");
    en_us.insert("account.list.header_model", "Model");
    en_us.insert("account.list.header_status", "Status");
    en_us.insert("account.list.status_active", "🟢 Active");
    en_us.insert("account.list.status_inactive", "⚪ Inactive");
    en_us.insert("account.list.total", "Total {} accounts");
    en_us.insert("account.add.title", "Add New Account");
    en_us.insert("account.add.prompt_name", "Account Name");
    en_us.insert("account.add.prompt_token", "API Token");
    en_us.insert("account.add.prompt_base_url", "Base URL");
    en_us.insert("account.add.prompt_model", "Model");
    en_us.insert(
        "account.add.no_base_url",
        "No available Base URL, please enter manually",
    );
    en_us.insert("account.add.select_base_url", "Select Base URL");
    en_us.insert("account.add.success", "✓ Account '{}' created successfully");
    en_us.insert("account.add.error", "✗ Creation failed: {}");
    en_us.insert("account.edit.prompt", "Select account to edit");
    en_us.insert("account.edit.success", "✓ Account updated successfully");
    en_us.insert("account.edit.error", "✗ Update failed: {}");
    en_us.insert("account.delete.prompt", "Select account to delete");
    en_us.insert(
        "account.delete.confirm",
        "Are you sure you want to delete account '{}'?",
    );
    en_us.insert("account.delete.success", "✓ Account deleted successfully");
    en_us.insert("account.delete.error", "✗ Deletion failed: {}");
    en_us.insert("account.export.title", "Batch Export Accounts");
    en_us.insert("account.export.no_accounts", "No accounts to export");
    en_us.insert(
        "account.export.success",
        "✓ Successfully exported {} account(s) to file: {file}",
    );
    en_us.insert("account.import.title", "Batch Import Accounts");
    en_us.insert("account.import.prompt_file", "Enter JSON file path");
    en_us.insert("account.import.error_read", "✗ Failed to read file: {}");
    en_us.insert("account.import.error_parse", "✗ Failed to parse JSON: {}");
    en_us.insert(
        "account.import.error_format",
        "✗ Invalid file format, missing providers array",
    );
    en_us.insert("account.import.no_accounts", "No account data in file");
    en_us.insert("account.import.processing", "Processing import...");
    en_us.insert("account.import.skip_invalid", "Skip invalid data");
    en_us.insert("account.import.skip_exists", "Skip existing");
    en_us.insert("account.import.imported", "Imported");
    en_us.insert("account.import.failed", "Import failed");
    en_us.insert(
        "account.import.result_imported",
        "✓ Successfully imported {} account(s)",
    );
    en_us.insert("account.import.result_skipped", "⊖ Skipped {} account(s)");
    en_us.insert("account.default_indicator", "(default)");

    // Directory Management
    en_us.insert("directory.menu.title", "Directory Management");
    en_us.insert("directory.menu.list", "📝 View All Directories");
    en_us.insert("directory.menu.add", "➕ Add New Directory");
    en_us.insert("directory.menu.edit", "✏️  Edit Directory");
    en_us.insert("directory.menu.delete", "🗑️  Delete Directory");
    en_us.insert("directory.list.no_records", "No directory records");
    en_us.insert("directory.list.header_id", "ID");
    en_us.insert("directory.list.header_name", "Directory Name");
    en_us.insert("directory.list.header_path", "Path");
    en_us.insert("directory.list.header_exists", "Exists");
    en_us.insert("directory.list.exists", "✓ Exists");
    en_us.insert("directory.list.not_exists", "✗ Not Exists");
    en_us.insert("directory.list.total", "Total {} directories");
    en_us.insert("directory.add.title", "Add New Directory");
    en_us.insert("directory.add.prompt_name", "Directory Name");
    en_us.insert("directory.add.prompt_path", "Path");
    en_us.insert(
        "directory.add.warn_path_not_exists",
        "⚠️  Warning: Path does not exist",
    );
    en_us.insert(
        "directory.add.success",
        "✓ Directory '{}' added successfully",
    );
    en_us.insert("directory.add.error", "✗ Addition failed: {}");
    en_us.insert("directory.edit.prompt", "Select directory to edit");
    en_us.insert("directory.edit.success", "✓ Directory updated successfully");
    en_us.insert("directory.edit.error", "✗ Update failed: {}");
    en_us.insert("directory.delete.prompt", "Select directory to delete");
    en_us.insert(
        "directory.delete.confirm",
        "Are you sure you want to delete directory '{}'?",
    );
    en_us.insert(
        "directory.delete.warning",
        "(Only deletes database record, not actual files)",
    );
    en_us.insert(
        "directory.delete.success",
        "✓ Directory deleted successfully",
    );
    en_us.insert("directory.delete.error", "✗ Deletion failed: {}");

    // URL Management
    en_us.insert("url.menu.title", "URL Management");
    en_us.insert("url.menu.list", "📝 View All URLs");
    en_us.insert("url.menu.add", "➕ Add New URL");
    en_us.insert("url.menu.edit", "✏️  Edit URL");
    en_us.insert("url.menu.delete", "🗑️  Delete URL");
    en_us.insert("url.list.no_records", "No URL records");
    en_us.insert("url.list.header_id", "ID");
    en_us.insert("url.list.header_name", "Name");
    en_us.insert("url.list.header_url", "URL");
    en_us.insert("url.list.header_description", "Description");
    en_us.insert("url.list.header_api_key", "API Key Env Var");
    en_us.insert("url.list.header_default", "Default");
    en_us.insert("url.list.default_yes", "Yes");
    en_us.insert("url.list.default_no", "No");
    en_us.insert("url.list.total", "Total {} URLs");
    en_us.insert("url.add.title", "Add New URL");
    en_us.insert("url.add.prompt_name", "Name");
    en_us.insert("url.add.prompt_url", "URL");
    en_us.insert("url.add.prompt_description", "Description (Optional)");
    en_us.insert(
        "url.add.prompt_api_key",
        "API Key Environment Variable (Default: ANTHROPIC_API_KEY)",
    );
    en_us.insert("url.add.prompt_default", "Set as default?");
    en_us.insert("url.add.success", "✓ URL '{}' created successfully");
    en_us.insert("url.add.error", "✗ Creation failed: {}");
    en_us.insert("url.edit.prompt", "Select URL to edit");
    en_us.insert("url.edit.success", "✓ URL updated successfully");
    en_us.insert("url.edit.error", "✗ Update failed: {}");
    en_us.insert("url.delete.prompt", "Select URL to delete");
    en_us.insert(
        "url.delete.confirm",
        "Are you sure you want to delete URL '{}'?",
    );
    en_us.insert(
        "url.delete.warning",
        "(Accounts using this URL will also be deleted)",
    );
    en_us.insert("url.delete.success", "✓ URL deleted successfully");
    en_us.insert("url.delete.error", "✗ Deletion failed: {}");

    // Configuration Switch
    en_us.insert("switch.title", "Configuration Switch");
    en_us.insert(
        "switch.no_accounts",
        "No account records, please add an account first",
    );
    en_us.insert(
        "switch.no_directories",
        "No directory records, please add a directory first",
    );
    en_us.insert("switch.select_account", "Select Account");
    en_us.insert("switch.select_directory", "Select Directory");
    en_us.insert(
        "switch.prompt_skip_permissions",
        "Skip permission check? (Recommended: Yes)",
    );
    en_us.insert(
        "switch.prompt_use_proxy",
        "Use proxy? (Load proxy settings from Claude config)",
    );
    en_us.insert("switch.switching", "Switching configuration...");
    en_us.insert("switch.success", "✓ Configuration switched successfully!");
    en_us.insert(
        "switch.success_env",
        "✓ Environment configuration switched successfully!",
    );
    en_us.insert("switch.account", "  Account: {}");
    en_us.insert("switch.directory", "  Directory: {}");
    en_us.insert("switch.path", "  Path: {}");
    en_us.insert("switch.sandbox", "  Sandbox Mode: Enabled");
    en_us.insert("switch.permission", "  Permission Check: {}");
    en_us.insert("switch.permission_skipped", "Skipped");
    en_us.insert("switch.permission_required", "Required");
    en_us.insert("switch.proxy", "  Proxy: {}");
    en_us.insert("switch.proxy_enabled", "Enabled");
    en_us.insert("switch.proxy_disabled", "Disabled");
    en_us.insert(
        "switch.warn_claude_config",
        "Warning: Failed to get Claude config, using default: {}",
    );
    en_us.insert(
        "switch.warn_write_fail",
        "Warning: Failed to write Claude config: {}",
    );
    en_us.insert(
        "switch.error_update",
        "✗ Configuration file update failed: {}",
    );
    en_us.insert("switch.error", "✗ Switch failed: {}");
    en_us.insert(
        "switch.claude_local_md_found",
        "⚠️  CLAUDE.local.md file already exists in the target directory",
    );
    en_us.insert(
        "switch.keep_claude_local_md",
        "Keep existing CLAUDE.local.md file? (Select No to overwrite)",
    );

    // WebDAV Sync
    en_us.insert("webdav.menu.title", "WebDAV Sync");
    en_us.insert("webdav.menu.config", "⚙️  Configure WebDAV");
    en_us.insert("webdav.menu.test", "🔌 Test Connection");
    en_us.insert("webdav.menu.upload", "⬆️  Upload Configuration");
    en_us.insert("webdav.menu.download", "⬇️  Download Configuration");
    en_us.insert("webdav.menu.list", "📝 View Remote Files");
    en_us.insert("webdav.menu.delete", "🗑️  Delete Configuration");
    en_us.insert("webdav.test.success", "✓ WebDAV connection test successful");
    en_us.insert("webdav.test.error", "✗ Connection test failed: {}");
    en_us.insert(
        "webdav.upload.clearing",
        "Clearing existing configuration...",
    );
    en_us.insert(
        "webdav.upload.cleared",
        "✓ Cleared existing accounts and Base URLs",
    );
    en_us.insert("webdav.upload.importing_accounts", "Importing accounts...");
    en_us.insert(
        "webdav.upload.imported_accounts",
        "✓ Successfully imported {} accounts",
    );
    en_us.insert("webdav.upload.importing_urls", "Importing Base URLs...");
    en_us.insert(
        "webdav.upload.imported_urls",
        "✓ Successfully imported {} Base URLs",
    );
    en_us.insert(
        "webdav.upload.success",
        "✓ Configuration successfully uploaded to WebDAV: {}",
    );
    en_us.insert(
        "webdav.upload.success_log",
        "Successfully uploaded configuration file: {}",
    );
    en_us.insert("webdav.upload.error", "✗ Upload failed: {}");
    en_us.insert(
        "webdav.download.success",
        "✓ Configuration successfully downloaded from WebDAV and imported: {}",
    );
    en_us.insert(
        "webdav.download.success_log",
        "Successfully downloaded and imported configuration file: {}",
    );
    en_us.insert("webdav.download.error", "✗ Download failed: {}");
    en_us.insert("webdav.list.title", "Remote File List:");
    en_us.insert("webdav.list.error", "✗ Failed to get file list: {}");
    en_us.insert(
        "webdav.delete.success",
        "✓ Configuration deleted successfully",
    );
    en_us.insert("webdav.delete.error", "✗ Deletion failed: {}");

    // Logs
    en_us.insert("logs.menu.title", "Log Management");
    en_us.insert("logs.menu.back", "🔙 Back to Main Menu");
    en_us.insert("logs.menu.view_recent", "📝 View Recent Logs");
    en_us.insert("logs.menu.info", "📊 Log File Information");
    en_us.insert("logs.menu.open_dir", "📂 Open Log Directory");
    en_us.insert("logs.prompt_lines", "How many recent lines to display");
    en_us.insert("logs.title", "Recent Logs:");
    en_us.insert("logs.no_records", "No log records");
    en_us.insert("logs.info.title", "Log File Information:");
    en_us.insert("logs.file", "  Log File: {}");
    en_us.insert("logs.size", "  File Size: {}");
    en_us.insert("logs.lines", "  Total Lines: {}");
    en_us.insert("logs.info.error", "✗ Failed to get log information: {}");
    en_us.insert("logs.directory", "Log Directory: {}");
    en_us.insert("logs.directory_opened", "✓ Log directory opened");
    en_us.insert("logs.directory.error", "✗ Failed to get log directory: {}");
    en_us.insert("logs.open_dir.error", "✗ Failed to open directory: {}");
    en_us.insert("logs.read.error", "✗ Failed to read logs: {}");

    // Remove Root Check
    en_us.insert("remove_root.title", "Remove Claude Code Root Check");
    en_us.insert(
        "remove_root.steps_intro",
        "This operation will perform the following steps:",
    );
    en_us.insert("remove_root.step1", "  1. Locate claude command");
    en_us.insert(
        "remove_root.step2",
        "  2. Create wrapper script to remove root check",
    );
    en_us.insert("remove_root.step3", "  3. Backup original claude command");
    en_us.insert(
        "remove_root.step4",
        "  4. Replace claude command with wrapper script",
    );
    en_us.insert("remove_root.confirm", "Continue with root check removal?");
    en_us.insert(
        "remove_root.executing",
        "Executing root check removal script...",
    );
    en_us.insert("remove_root.success", "✓ Root check removal completed");
    en_us.insert(
        "remove_root.error_exit",
        "✗ Script execution failed, exit code: {}",
    );
    en_us.insert("remove_root.error_execute", "✗ Script execution failed: {}");
    en_us.insert("remove_root.error_stderr", "Error output:\n{}");
    en_us.insert(
        "remove_root.error",
        "✗ Root check removal script not found: {}",
    );

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
