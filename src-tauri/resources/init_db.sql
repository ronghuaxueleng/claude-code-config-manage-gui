-- SQLite database initialization script for Claude Config Manager
-- This creates all necessary tables with default data

-- Create accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    token TEXT NOT NULL,
    base_url TEXT NOT NULL,
    model TEXT NOT NULL DEFAULT 'claude-sonnet-4-20250514',
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
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
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
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

-- Insert default base URLs
INSERT OR IGNORE INTO base_urls (name, url, description, is_default) VALUES
    ('Anthropic Official', 'https://api.anthropic.com', 'Official Anthropic API endpoint', 1),
    ('Claude API', 'https://api.claude.ai', 'Claude API endpoint', 0),
    ('Local Development', 'http://localhost:8000', 'Local development server', 0);

-- Insert sample data (optional - commented out)
-- INSERT OR IGNORE INTO accounts (name, token, base_url) VALUES
--     ('Sample Account', 'sk-ant-api03-sample-token-here', 'https://api.anthropic.com');

-- INSERT OR IGNORE INTO directories (name, path) VALUES
--     ('Sample Project', '/path/to/sample/project');