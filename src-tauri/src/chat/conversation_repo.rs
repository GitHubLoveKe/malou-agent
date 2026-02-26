use rusqlite::{params, Result as SqliteResult};
use crate::database::{Database, Conversation, CreateConversation};

/// 会话数据访问层
pub struct ConversationRepo {
    db: Database,
}

impl ConversationRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 创建新会话
    pub fn create(&self, req: CreateConversation) -> SqliteResult<Conversation> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.db.execute(|conn| {
            conn.execute(
                "INSERT INTO conversations (id, title, model_id, total_tokens, message_count, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, 0, 0, ?4, ?4)",
                params![id, req.title, req.model_id, now],
            )?;
            Ok(())
        })?;

        Ok(Conversation {
            id,
            title: req.title,
            model_id: req.model_id,
            total_tokens: 0,
            message_count: 0,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// 获取会话列表
    pub fn list(&self, limit: i32, offset: i32) -> SqliteResult<Vec<Conversation>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, model_id, total_tokens, message_count, created_at, updated_at 
                 FROM conversations 
                 ORDER BY updated_at DESC 
                 LIMIT ?1 OFFSET ?2",
            )?;

            let rows = stmt.query_map(params![limit, offset], |row| {
                Ok(Conversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    model_id: row.get(2)?,
                    total_tokens: row.get(3)?,
                    message_count: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?;

            rows.collect()
        })
    }

    /// 获取单个会话
    pub fn get(&self, id: &str) -> SqliteResult<Option<Conversation>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, model_id, total_tokens, message_count, created_at, updated_at 
                 FROM conversations 
                 WHERE id = ?1",
            )?;

            let mut rows = stmt.query_map(params![id], |row| {
                Ok(Conversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    model_id: row.get(2)?,
                    total_tokens: row.get(3)?,
                    message_count: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?;

            Ok(rows.next().transpose()?)
        })
    }

    /// 更新会话标题
    pub fn update_title(&self, id: &str, title: &str) -> SqliteResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
                params![title, now, id],
            )?;
            Ok(())
        })
    }

    /// 更新会话的 Token 统计和消息数
    pub fn update_stats(&self, id: &str, add_tokens: i32, add_messages: i32) -> SqliteResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE conversations 
                 SET total_tokens = total_tokens + ?1, 
                     message_count = message_count + ?2, 
                     updated_at = ?3 
                 WHERE id = ?4",
                params![add_tokens, add_messages, now, id],
            )?;
            Ok(())
        })
    }

    /// 重置会话统计为 0
    pub fn reset_stats(&self, id: &str) -> SqliteResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE conversations 
                 SET total_tokens = 0, 
                     message_count = 0, 
                     updated_at = ?1 
                 WHERE id = ?2",
                params![now, id],
            )?;
            Ok(())
        })
    }

    /// 更新会话使用的模型
    pub fn update_model(&self, id: &str, model_id: &str) -> SqliteResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE conversations SET model_id = ?1, updated_at = ?2 WHERE id = ?3",
                params![model_id, now, id],
            )?;
            Ok(())
        })
    }

    /// 删除会话（级联删除消息）
    pub fn delete(&self, id: &str) -> SqliteResult<bool> {
        self.db.execute(|conn| {
            let affected = conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
            Ok(affected > 0)
        })
    }

    /// 获取会话数量
    pub fn count(&self) -> SqliteResult<i64> {
        self.db.execute(|conn| {
            conn.query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
        })
    }
}
