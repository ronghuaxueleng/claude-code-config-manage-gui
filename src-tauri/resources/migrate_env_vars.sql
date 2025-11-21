-- 数据库迁移脚本：添加环境变量支持
-- 版本：1.6.0
-- 说明：为 base_urls 和 accounts 表添加环境变量字段

-- 1. 为 base_urls 表添加默认环境变量字段
-- 存储该 URL 端点的默认环境变量（JSON 格式）
ALTER TABLE base_urls ADD COLUMN default_env_vars TEXT DEFAULT '{}';

-- 2. 为 accounts 表添加自定义环境变量字段
-- 存储该账号的自定义环境变量（JSON 格式）
ALTER TABLE accounts ADD COLUMN custom_env_vars TEXT DEFAULT '{}';

-- 3. 更新现有数据的默认值（确保所有记录都有有效的 JSON）
UPDATE base_urls SET default_env_vars = '{}' WHERE default_env_vars IS NULL OR default_env_vars = '';
UPDATE accounts SET custom_env_vars = '{}' WHERE custom_env_vars IS NULL OR custom_env_vars = '';

-- 4. 为一些常用的 base_urls 设置默认环境变量示例
-- 官方 Anthropic API 的默认环境变量
UPDATE base_urls
SET default_env_vars = '{"DISABLE_AUTOUPDATER": "1", "ANTHROPIC_LOG_LEVEL": "info"}'
WHERE url = 'https://api.anthropic.com';

-- Claude API 的默认环境变量
UPDATE base_urls
SET default_env_vars = '{"DISABLE_AUTOUPDATER": "1", "CLAUDE_LOG_LEVEL": "debug"}'
WHERE url = 'https://api.claude.ai';

-- 本地开发环境的默认环境变量
UPDATE base_urls
SET default_env_vars = '{"DISABLE_AUTOUPDATER": "1", "DEBUG": "true", "LOG_LEVEL": "debug"}'
WHERE url = 'http://localhost:8000';