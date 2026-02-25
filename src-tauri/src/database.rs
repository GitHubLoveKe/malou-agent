use rusqlite::{Connection, Result as SqlResult, Error as SqlError};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQL error: {0}")]
    Sql(#[from] SqlError),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Document not found")]
    NotFound,
}

/// SQLite数据库封装
pub struct SQLiteDatabase {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentCreate {
    pub title: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentUpdate {
    pub title: Option<String>,
    pub content: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub metadata: Option<serde_json::Value>,
}

impl SQLiteDatabase {
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, DatabaseError> {
        let db_path = db_path.as_ref().to_path_buf();
        
        let conn = tokio::task::spawn_blocking(move || {
            let conn = Connection::open(db_path)?;
            Self::initialize_schema(&conn)?;
            Ok(conn)
        }).await??;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn get_connection(&self) -> tokio::sync::MutexGuard<Connection> {
        self.conn.lock().await
    }

    fn initialize_schema(conn: &Connection) -> Result<(), DatabaseError> {
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                embedding BLOB,
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at);
            CREATE INDEX IF NOT EXISTS idx_documents_title ON documents(title);
            CREATE INDEX IF NOT EXISTS idx_documents_updated_at ON documents(updated_at);
        "#)?;
        
        Ok(())
    }

    pub async fn create_document(&self, doc_create: DocumentCreate) -> Result<Document, DatabaseError> {
        let conn = self.conn.lock().await;
        let doc_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let doc = Document {
            id: doc_id.clone(),
            title: doc_create.title,
            content: doc_create.content,
            embedding: doc_create.embedding,
            metadata: doc_create.metadata,
            created_at: now,
            updated_at: now,
        };
        
        let doc_clone = doc.clone();
        
        tokio::task::spawn_blocking(move || {
            let tx = conn.transaction()?;
            
            tx.execute(
                "INSERT INTO documents (id, title, content, embedding, metadata, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                (
                    &doc_clone.id,
                    &doc_clone.title,
                    &doc_clone.content,
                    &Self::serialize_embedding(&doc_clone.embedding),
                    &doc_clone.metadata.as_ref().map(|m| m.to_string()).unwrap_or_default(),
                    &doc_clone.created_at.to_rfc3339(),
                    &doc_clone.updated_at.to_rfc3339(),
                ),
            )?;
            
            tx.commit()?;
            Ok(())
        }).await??;
        
        Ok(doc)
    }

    pub async fn get_document(&self, id: &str) -> Result<Option<Document>, DatabaseError> {
        let conn = self.conn.lock().await;
        let id = id.to_string();
        
        tokio::task::spawn_blocking(move || {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, embedding, metadata, created_at, updated_at FROM documents WHERE id = ?1"
            )?;
            
            let mut rows = stmt.query([&id])?;
            
            if let Some(row) = rows.next()? {
                let doc = Document {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    embedding: Self::deserialize_embedding(row.get(3)?),
                    metadata: row.get::<_, String>(4).ok()
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map_err(|e| SqlError::InvalidQuery(Box::new(e)))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .map_err(|e| SqlError::InvalidQuery(Box::new(e)))?
                        .with_timezone(&Utc),
                };
                Ok(Some(doc))
            } else {
                Ok(None)
            }
        }).await?
    }

    pub async fn update_document(&self, id: &str, doc_update: DocumentUpdate) -> Result<Option<Document>, DatabaseError> {
        let conn = self.conn.lock().await;
        let id = id.to_string();
        let now = Utc::now();
        
        tokio::task::spawn_blocking(move || {
            // 先检查文档是否存在
            let mut check_stmt = conn.prepare("SELECT COUNT(*) FROM documents WHERE id = ?1")?;
            let count: i32 = check_stmt.query_row([&id], |row| row.get(0))?;
            
            if count == 0 {
                return Ok(None);
            }
            
            let tx = conn.transaction()?;
            
            // 构建动态更新语句
            let mut sql = "UPDATE documents SET updated_at = ?1".to_string();
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
                Box::new(now.to_rfc3339()),
            ];
            
            if let Some(ref title) = doc_update.title {
                sql.push_str(", title = ?2");
                params.push(Box::new(title.clone()));
            }
            
            if let Some(ref content) = doc_update.content {
                let param_index = params.len() + 1;
                sql.push_str(&format!(", content = ?{}", param_index));
                params.push(Box::new(content.clone()));
            }
            
            if let Some(ref embedding) = doc_update.embedding {
                let param_index = params.len() + 1;
                sql.push_str(&format!(", embedding = ?{}", param_index));
                params.push(Box::new(Self::serialize_embedding(&Some(embedding.clone()))));
            }
            
            if let Some(ref metadata) = doc_update.metadata {
                let param_index = params.len() + 1;
                sql.push_str(&format!(", metadata = ?{}", param_index));
                params.push(Box::new(metadata.to_string()));
            }
            
            sql.push_str(" WHERE id = ?");
            let param_index = params.len() + 1;
            sql.push_str(&format!("{}", param_index));
            params.push(Box::new(id.clone()));
            
            // 执行更新
            {
                let mut stmt = tx.prepare(&sql)?;
                let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
                stmt.execute(&param_refs[..])?;
            }
            
            tx.commit()?;
            
            // 获取更新后的文档
            let mut stmt = conn.prepare(
                "SELECT id, title, content, embedding, metadata, created_at, updated_at FROM documents WHERE id = ?1"
            )?;
            
            let doc = stmt.query_row([&id], |row| {
                Ok(Document {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    embedding: Self::deserialize_embedding(row.get(3)?),
                    metadata: row.get::<_, String>(4).ok()
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map_err(|e| SqlError::InvalidQuery(Box::new(e)))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .map_err(|e| SqlError::InvalidQuery(Box::new(e)))?
                        .with_timezone(&Utc),
                })
            })?;
            
            Ok(Some(doc))
        }).await?
    }

    pub async fn delete_document(&self, id: &str) -> Result<bool, DatabaseError> {
        let conn = self.conn.lock().await;
        let id = id.to_string();
        
        tokio::task::spawn_blocking(move || {
            let affected = conn.execute("DELETE FROM documents WHERE id = ?1", [&id])?;
            Ok(affected > 0)
        }).await?
    }

    pub async fn list_documents(&self, limit: usize, offset: usize) -> Result<Vec<Document>, DatabaseError> {
        let conn = self.conn.lock().await;
        
        tokio::task::spawn_blocking(move || {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, embedding, metadata, created_at, updated_at FROM documents ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
            )?;
            
            let docs = stmt.query_map((limit, offset), |row| {
                Ok(Document {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    embedding: Self::deserialize_embedding(row.get(3)?),
                    metadata: row.get::<_, String>(4).ok()
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map_err(|e| SqlError::InvalidQuery(Box::new(e)))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .map_err(|e| SqlError::InvalidQuery(Box::new(e)))?
                        .with_timezone(&Utc),
                })
            })?;
            
            docs.collect()
        }).await?
    }

    pub async fn get_document_count(&self) -> Result<i64, DatabaseError> {
        let conn = self.conn.lock().await;
        
        tokio::task::spawn_blocking(move || {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
            Ok(count)
        }).await?
    }

    fn serialize_embedding(embedding: &Option<Vec<f32>>) -> Option<Vec<u8>> {
        embedding.as_ref().map(|vec| {
            let bytes: Vec<u8> = vec.iter()
                .flat_map(|&f| f.to_le_bytes())
                .collect();
            bytes
        })
    }

    fn deserialize_embedding(bytes: Option<Vec<u8>>) -> Option<Vec<f32>> {
        bytes.map(|bytes| {
            bytes.chunks_exact(4)
                .map(|chunk| {
                    let mut array = [0u8; 4];
                    array.copy_from_slice(chunk);
                    f32::from_le_bytes(array)
                })
                .collect()
        })
    }
}