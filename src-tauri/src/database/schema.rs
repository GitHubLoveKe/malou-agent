/// 数据库表结构定义
pub const SCHEMA_SQL: &str = r#"
-- 会话表
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    model_id TEXT,
    total_tokens INTEGER DEFAULT 0,
    message_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 会话索引：按更新时间排序
CREATE INDEX IF NOT EXISTS idx_conversations_updated ON conversations(updated_at DESC);

-- 消息表
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    model_id TEXT,
    prompt_tokens INTEGER DEFAULT 0,
    completion_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

-- 消息索引：按会话和时间排序
CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id, created_at ASC);

-- Token 使用统计表
CREATE TABLE IF NOT EXISTS token_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    model_id TEXT NOT NULL,
    date TEXT NOT NULL,
    prompt_tokens INTEGER DEFAULT 0,
    completion_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    request_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL
);

-- Token 统计索引
CREATE INDEX IF NOT EXISTS idx_token_model_date ON token_usage(model_id, date DESC);

-- 模型配置表
CREATE TABLE IF NOT EXISTS model_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    config TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 模型配置索引
CREATE INDEX IF NOT EXISTS idx_model_configs_type ON model_configs(type, enabled);
"#;

/// 获取建表 SQL
pub fn get_schema_sql() -> &'static str {
    SCHEMA_SQL
}
