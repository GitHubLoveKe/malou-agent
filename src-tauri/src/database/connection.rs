use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::schema::get_schema_sql;

/// 数据库连接管理器
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 创建新的数据库连接
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;
        
        // 启用外键约束
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        
        // 启用 WAL 模式以提高并发性能
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        // 初始化表结构
        db.init_schema()?;
        
        Ok(db)
    }

    /// 初始化数据库表结构
    fn init_schema(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(get_schema_sql())
    }

    /// 获取数据库连接的克隆引用
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// 执行带参数的查询
    pub fn execute<F, T>(&self, f: F) -> SqliteResult<T>
    where
        F: FnOnce(&Connection) -> SqliteResult<T>,
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

/// 获取默认数据库路径
pub fn get_default_db_path() -> PathBuf {
    let app_data = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    app_data.join("malou-agent").join("malou.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_database_creation() {
        let db_path = temp_dir().join("test_malou.db");
        let db = Database::new(db_path.clone());
        assert!(db.is_ok());
        // 清理测试文件
        std::fs::remove_file(db_path).ok();
    }
}
