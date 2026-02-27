use rusqlite::{params, Result as SqliteResult};
use crate::database::{Database, Message, CreateMessage};

/// 消息数据访问层
pub struct MessageRepo {
    db: Database,
}

impl MessageRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 创建新消息
    pub fn create(&self, req: CreateMessage) -> SqliteResult<Message> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let prompt_tokens = req.prompt_tokens.unwrap_or(0);
        let completion_tokens = req.completion_tokens.unwrap_or(0);
        let total_tokens = req.total_tokens.unwrap_or(prompt_tokens + completion_tokens);

        self.db.execute(|conn| {
            conn.execute(
                "INSERT INTO messages (id, conversation_id, role, content, model_id, prompt_tokens, completion_tokens, total_tokens, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    id,
                    req.conversation_id,
                    req.role,
                    req.content,
                    req.model_id,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                    now
                ],
            )?;
            Ok(())
        })?;

        Ok(Message {
            id,
            conversation_id: req.conversation_id,
            role: req.role,
            content: req.content,
            model_id: req.model_id,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            created_at: now,
        })
    }

    /// 获取会话的消息列表
    pub fn list_by_conversation(&self, conversation_id: &str, limit: i32, offset: i32) -> SqliteResult<Vec<Message>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, conversation_id, role, content, model_id, prompt_tokens, completion_tokens, total_tokens, created_at 
                 FROM messages 
                 WHERE conversation_id = ?1 
                 ORDER BY created_at ASC 
                 LIMIT ?2 OFFSET ?3",
            )?;

            let rows = stmt.query_map(params![conversation_id, limit, offset], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    model_id: row.get(4)?,
                    prompt_tokens: row.get(5)?,
                    completion_tokens: row.get(6)?,
                    total_tokens: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?;

            rows.collect()
        })
    }

    /// 获取会话的最近 N 条消息（用于构建上下文）
    pub fn get_recent(&self, conversation_id: &str, limit: i32) -> SqliteResult<Vec<Message>> {
        self.db.execute(|conn| {
            // 使用索引 idx_messages_created_desc 进行高效查询
            let mut stmt = conn.prepare(
                "SELECT id, conversation_id, role, content, model_id, prompt_tokens, completion_tokens, total_tokens, created_at 
                 FROM messages 
                 WHERE conversation_id = ?1 
                 ORDER BY created_at DESC 
                 LIMIT ?2",
            )?;

            let rows = stmt.query_map(params![conversation_id, limit], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    model_id: row.get(4)?,
                    prompt_tokens: row.get(5)?,
                    completion_tokens: row.get(6)?,
                    total_tokens: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?;

            // 反转顺序，使消息按时间正序排列
            let mut messages: Vec<Message> = rows.collect::<SqliteResult<Vec<_>>>()?;
            messages.reverse();
            Ok(messages)
        })
    }

    /// 获取单条消息
    pub fn get(&self, id: &str) -> SqliteResult<Option<Message>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, conversation_id, role, content, model_id, prompt_tokens, completion_tokens, total_tokens, created_at 
                 FROM messages 
                 WHERE id = ?1",
            )?;

            let mut rows = stmt.query_map(params![id], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    model_id: row.get(4)?,
                    prompt_tokens: row.get(5)?,
                    completion_tokens: row.get(6)?,
                    total_tokens: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?;

            Ok(rows.next().transpose()?)
        })
    }

    /// 删除会话的所有消息
    pub fn delete_by_conversation(&self, conversation_id: &str) -> SqliteResult<i32> {
        self.db.execute(|conn| {
            let affected = conn.execute(
                "DELETE FROM messages WHERE conversation_id = ?1",
                params![conversation_id],
            )?;
            Ok(affected as i32)
        })
    }

    /// 获取会话的消息数量
    pub fn count_by_conversation(&self, conversation_id: &str) -> SqliteResult<i64> {
        self.db.execute(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM messages WHERE conversation_id = ?1",
                params![conversation_id],
                |row| row.get(0),
            )
        })
    }

    /// 获取会话的总 Token 数
    pub fn get_total_tokens(&self, conversation_id: &str) -> SqliteResult<i64> {
        self.db.execute(|conn| {
            conn.query_row(
                "SELECT COALESCE(SUM(total_tokens), 0) FROM messages WHERE conversation_id = ?1",
                params![conversation_id],
                |row| row.get(0),
            )
        })
    }
}
