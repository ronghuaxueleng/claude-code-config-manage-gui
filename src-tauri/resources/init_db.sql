-- SQLite database initialization script for Claude Config Manager
-- This creates all necessary tables with default data

-- Create accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    token TEXT NOT NULL,
    base_url TEXT NOT NULL,
    model TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    custom_env_vars TEXT NOT NULL DEFAULT '{}',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create directories table
CREATE TABLE IF NOT EXISTS directories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create base_urls table
CREATE TABLE IF NOT EXISTS base_urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL UNIQUE,
    description TEXT,
    api_key TEXT NOT NULL DEFAULT 'ANTHROPIC_API_KEY',
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    default_env_vars TEXT NOT NULL DEFAULT '{}',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create account_directories table
CREATE TABLE IF NOT EXISTS account_directories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    directory_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE,
    FOREIGN KEY (directory_id) REFERENCES directories (id) ON DELETE CASCADE,
    UNIQUE(account_id, directory_id)
);

-- Insert default base URLs with default environment variables
INSERT OR IGNORE INTO base_urls (name, url, description, is_default, default_env_vars) VALUES
    ('Anthropic Official', 'https://api.anthropic.com', 'Official Anthropic API endpoint', 1, '{"DISABLE_AUTOUPDATER": "1", "ANTHROPIC_LOG_LEVEL": "info"}'),
    ('Claude API', 'https://api.claude.ai', 'Claude API endpoint', 0, '{"DISABLE_AUTOUPDATER": "1", "CLAUDE_LOG_LEVEL": "debug"}'),
    ('Local Development', 'http://localhost:8000', 'Local development server', 0, '{"DISABLE_AUTOUPDATER": "1", "DEBUG": "true", "LOG_LEVEL": "debug"}');

-- Insert sample data (optional - commented out)
-- INSERT OR IGNORE INTO accounts (name, token, base_url) VALUES
--     ('Sample Account', 'sk-ant-api03-sample-token-here', 'https://api.anthropic.com');

-- INSERT OR IGNORE INTO directories (name, path) VALUES
--     ('Sample Project', '/path/to/sample/project');

-- Create webdav_configs table for WebDAV synchronization
CREATE TABLE IF NOT EXISTS webdav_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    remote_path TEXT NOT NULL DEFAULT '/claude-config',
    auto_sync BOOLEAN NOT NULL DEFAULT FALSE,
    sync_interval INTEGER NOT NULL DEFAULT 3600,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    last_sync_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create sync_logs table for tracking synchronization history
CREATE TABLE IF NOT EXISTS sync_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    webdav_config_id INTEGER NOT NULL,
    sync_type TEXT NOT NULL CHECK(sync_type IN ('upload', 'download', 'auto')),
    status TEXT NOT NULL CHECK(status IN ('success', 'failed', 'pending')),
    message TEXT,
    synced_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (webdav_config_id) REFERENCES webdav_configs (id) ON DELETE CASCADE
);