// src-tauri/src/onnx/mod.rs
//! ONNX模型推理模块
//! 使用tract库实现纯Rust的ONNX模型加载和推理

use tract_onnx::prelude::*;
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};
use std::collections::HashMap;
use std::path::Path;

/// ONNX模型包装器
pub struct OnnxModel {
    model: Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    input_shape: Vec<usize>,
}

impl OnnxModel {
    /// 从文件路径加载ONNX模型
    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path_buf = path.as_ref().to_path_buf();
        
        let model = tokio::task::spawn_blocking(move || {
            tract_onnx::onnx()
                .model_for_path(path_buf)?
                .into_optimized()?
                .into_runnable()
        }).await??;

        // 获取输入形状信息
        let input_fact = model.model().input_fact(0)?;
        let input_shape: Vec<usize> = input_fact.shape.as_concrete().unwrap_or_else(|| vec![1, 384]).to_vec();

        Ok(Self {
            model: Arc::new(model),
            input_shape,
        })
    }

    /// 执行模型推理
    pub async fn infer(&self, input: Tensor) -> Result<Tensor, Box<dyn std::error::Error>> {
        let model = self.model.clone();
        
        let output = tokio::task::spawn_blocking(move || {
            model.run([input])
        }).await??;

        Ok(output.remove(0))
    }

    /// 文本嵌入生成
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // 简化的文本预处理（实际应用中需要完整的tokenizer）
        let tokens = self.tokenize_text(text)?;
        let input_tensor = self.tokens_to_tensor(&tokens)?;
        
        let output_tensor = self.infer(input_tensor).await?;
        let embedding: Vec<f32> = output_tensor.to_array_view::<f32>()?.iter().copied().collect();
        
        Ok(embedding)
    }

    /// 批量文本嵌入
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let mut embeddings = Vec::with_capacity(texts.len());
        
        // 分批处理以控制内存使用
        for chunk in texts.chunks(8) {
            let mut chunk_embeddings = Vec::new();
            for text in chunk {
                let embedding = self.embed_text(text).await?;
                chunk_embeddings.push(embedding);
            }
            embeddings.extend(chunk_embeddings);
        }
        
        Ok(embeddings)
    }

    fn tokenize_text(&self, text: &str) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        // 简化的tokenization，实际应该使用proper tokenizer
        let tokens: Vec<i64> = text
            .split_whitespace()
            .take(self.input_shape[1] - 2) // 保留[CLS]和[SEP]位置
            .map(|word| word.chars().map(|c| c as i64).sum())
            .collect();
        
        Ok(tokens)
    }

    fn tokens_to_tensor(&self, tokens: &[i64]) -> Result<Tensor, Box<dyn std::error::Error>> {
        let mut padded_tokens = vec![101]; // [CLS]
        padded_tokens.extend_from_slice(tokens);
        padded_tokens.push(102); // [SEP]
        
        // 填充到固定长度
        while padded_tokens.len() < self.input_shape[1] {
            padded_tokens.push(0); // [PAD]
        }
        
        let tensor = tract_ndarray::Array2::from_shape_vec(
            (1, self.input_shape[1]),
            padded_tokens,
        )?.into_dyn();
        
        Ok(tensor.into())
    }
}

/// 模型管理器，支持缓存和并发访问
pub struct ModelManager {
    models: RwLock<HashMap<String, Arc<OnnxModel>>>,
    default_model: OnceCell<Arc<OnnxModel>>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: RwLock::new(HashMap::new()),
            default_model: OnceCell::new(),
        }
    }

    /// 获取默认模型（延迟加载）
    pub async fn get_default_model(&self) -> Result<Arc<OnnxModel>, Box<dyn std::error::Error>> {
        let model = self.default_model.get_or_try_init(|| async {
            let model_path = std::env::var("EMBEDDING_MODEL_PATH")
                .unwrap_or_else(|_| "models/all-MiniLM-L6-v2.onnx".to_string());
            
            println!("Loading default model from: {}", model_path);
            OnnxModel::from_path(model_path).await.map(Arc::new)
        }).await?;
        
        Ok(model.clone())
    }

    /// 获取指定名称的模型
    pub async fn get_model(&self, name: &str) -> Result<Arc<OnnxModel>, Box<dyn std::error::Error>> {
        // 检查缓存
        {
            let models = self.models.read().await;
            if let Some(model) = models.get(name) {
                return Ok(model.clone());
            }
        }

        // 加载新模型
        let model_path = format!("models/{}.onnx", name);
        let model = OnnxModel::from_path(model_path).await?;
        let model_arc = Arc::new(model);

        // 更新缓存
        {
            let mut models = self.models.write().await;
            models.insert(name.to_string(), model_arc.clone());
        }

        Ok(model_arc)
    }

    /// 预加载常用模型
    pub async fn preload_models(&self, model_names: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        for &name in model_names {
            self.get_model(name).await?;
        }
        Ok(())
    }
}

// src-tauri/src/database/mod.rs
//! 数据库访问层
//! 包含SQLite和ChromaDB的集成

use rusqlite::{Connection, Result as SqlResult};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// SQLite数据库封装
pub struct SQLiteDatabase {
    conn: Mutex<Connection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: Option<serde_json::Value>,
}

impl SQLiteDatabase {
    pub async fn new<P: AsRef<Path>>(db_path: P) -> SqlResult<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        
        let conn = tokio::task::spawn_blocking(move || {
            let conn = Connection::open(db_path)?;
            Self::initialize_schema(&conn)?;
            Ok(conn)
        }).await??;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn initialize_schema(conn: &Connection) -> SqlResult<()> {
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                embedding BLOB,
                metadata TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at);
            CREATE INDEX IF NOT EXISTS idx_documents_title ON documents(title);
        "#)?;
        
        Ok(())
    }

    pub async fn insert_document(&self, doc: &Document) -> SqlResult<()> {
        let conn = self.conn.lock().await;
        let doc = doc.clone();
        
        tokio::task::spawn_blocking(move || {
            conn.execute(
                "INSERT OR REPLACE INTO documents (id, title, content, embedding, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    &doc.id,
                    &doc.title,
                    &doc.content,
                    &Self::serialize_embedding(&doc.embedding),
                    &doc.metadata.as_ref().map(|m| m.to_string()).unwrap_or_default(),
                ),
            )?;
            Ok(())
        }).await?
    }

    pub async fn search_documents(&self, query: &str, limit: usize) -> SqlResult<Vec<Document>> {
        let conn = self.conn.lock().await;
        let query = query.to_string();
        
        tokio::task::spawn_blocking(move || {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, embedding, metadata FROM documents WHERE title LIKE ?1 OR content LIKE ?1 ORDER BY created_at DESC LIMIT ?2"
            )?;
            
            let docs = stmt.query_map((&format!("%{}%", query), limit), |row| {
                Ok(Document {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    embedding: Self::deserialize_embedding(row.get(3)?),
                    metadata: row.get::<_, String>(4).ok()
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?;
            
            docs.collect()
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

// src-tauri/src/vector_search/mod.rs
//! 向量搜索和相似度计算模块

use std::sync::Arc;

pub trait SimilarityMetric: Send + Sync {
    fn calculate(&self, a: &[f32], b: &[f32]) -> f32;
}

pub struct CosineSimilarity;
pub struct EuclideanDistance;

impl SimilarityMetric for CosineSimilarity {
    fn calculate(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            0.0
        } else {
            dot_product / (magnitude_a * magnitude_b)
        }
    }
}

impl SimilarityMetric for EuclideanDistance {
    fn calculate(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::INFINITY;
        }
        
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

/// 向量搜索引擎
pub struct VectorSearchEngine {
    metric: Arc<dyn SimilarityMetric>,
    documents: Vec<(String, Vec<f32>)>, // (doc_id, embedding)
}

impl VectorSearchEngine {
    pub fn new(metric: Arc<dyn SimilarityMetric>) -> Self {
        Self {
            metric,
            documents: Vec::new(),
        }
    }

    pub fn add_document(&mut self, doc_id: String, embedding: Vec<f32>) {
        self.documents.push((doc_id, embedding));
    }

    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(String, f32)> {
        let mut similarities: Vec<(String, f32)> = self.documents
            .iter()
            .map(|(doc_id, doc_embedding)| {
                let similarity = self.metric.calculate(query_embedding, doc_embedding);
                (doc_id.clone(), similarity)
            })
            .collect();

        // 按相似度降序排列
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // 返回top-k结果
        similarities.truncate(top_k);
        similarities
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

// src-tauri/src/concurrency/mod.rs
//! 并发控制和资源管理

use tokio::sync::{Semaphore, RwLock};
use std::collections::HashMap;
use std::sync::Arc;

/// 资源配额管理器
pub struct ResourceManager {
    // 不同类型资源的信号量
    cpu_permits: Semaphore,
    io_permits: Semaphore,
    memory_permits: Semaphore,
    // 特定资源的锁
    resource_locks: RwLock<HashMap<String, Arc<Semaphore>>>,
}

impl ResourceManager {
    pub fn new(cpu_limit: usize, io_limit: usize, memory_limit: usize) -> Self {
        Self {
            cpu_permits: Semaphore::new(cpu_limit),
            io_permits: Semaphore::new(io_limit),
            memory_permits: Semaphore::new(memory_limit),
            resource_locks: RwLock::new(HashMap::new()),
        }
    }

    /// 获取CPU资源许可
    pub async fn acquire_cpu_permit(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.cpu_permits.acquire().await.unwrap()
    }

    /// 获取IO资源许可
    pub async fn acquire_io_permit(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.io_permits.acquire().await.unwrap()
    }

    /// 获取特定资源的互斥锁
    pub async fn acquire_resource_lock(&self, resource_id: &str, max_concurrent: usize) -> tokio::sync::SemaphorePermit<'_> {
        let semaphore = {
            let locks = self.resource_locks.read().await;
            if let Some(sem) = locks.get(resource_id) {
                sem.clone()
            } else {
                drop(locks);
                let mut locks = self.resource_locks.write().await;
                locks.entry(resource_id.to_string())
                    .or_insert_with(|| Arc::new(Semaphore::new(max_concurrent)))
                    .clone()
            }
        };

        semaphore.acquire().await.unwrap()
    }
}

/// 异步任务包装器，自动管理资源
pub struct AsyncTaskWrapper<T> {
    resource_manager: Arc<ResourceManager>,
    task_type: TaskType,
    _phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone)]
pub enum TaskType {
    CPUIntensive,
    IOIntensive,
    MemorySensitive { resource_id: String, max_concurrent: usize },
}

impl<T> AsyncTaskWrapper<T> {
    pub fn new(resource_manager: Arc<ResourceManager>, task_type: TaskType) -> Self {
        Self {
            resource_manager,
            task_type,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn spawn<F, Fut, R>(&self, f: F) -> Result<R, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<R, Box<dyn std::error::Error>>> + Send,
        R: Send + 'static,
    {
        match &self.task_type {
            TaskType::CPUIntensive => {
                let _permit = self.resource_manager.acquire_cpu_permit().await;
                f().await
            }
            TaskType::IOIntensive => {
                let _permit = self.resource_manager.acquire_io_permit().await;
                f().await
            }
            TaskType::MemorySensitive { resource_id, max_concurrent } => {
                let _permit = self.resource_manager.acquire_resource_lock(resource_id, *max_concurrent).await;
                f().await
            }
        }
    }
}

// src-tauri/src/main.rs
//! 主程序入口

mod onnx;
mod database;
mod vector_search;
mod concurrency;

use tauri::Manager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化资源管理器
    let resource_manager = Arc::new(concurrency::ResourceManager::new(4, 20, 2));
    
    // 初始化模型管理器
    let model_manager = Arc::new(onnx::ModelManager::new());
    
    // 预加载默认模型
    model_manager.preload_models(&["all-MiniLM-L6-v2"]).await?;
    
    // 启动Tauri应用
    tauri::Builder::default()
        .manage(resource_manager)
        .manage(model_manager)
        .invoke_handler(tauri::generate_handler![
            embed_text,
            search_documents,
            add_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

// Tauri命令处理函数
#[tauri::command]
async fn embed_text(
    text: &str,
    model_manager: tauri::State<'_, Arc<onnx::ModelManager>>
) -> Result<Vec<f32>, String> {
    let model = model_manager.get_default_model().await
        .map_err(|e| e.to_string())?;
    
    model.embed_text(text).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_documents(
    query: &str,
    db: tauri::State<'_, Arc<database::SQLiteDatabase>>
) -> Result<Vec<database::Document>, String> {
    db.search_documents(query, 10).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_document(
    doc: database::Document,
    db: tauri::State<'_, Arc<database::SQLiteDatabase>>
) -> Result<(), String> {
    db.insert_document(&doc).await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cosine_similarity() {
        let metric = Arc::new(vector_search::CosineSimilarity);
        let engine = vector_search::VectorSearchEngine::new(metric);
        
        assert_eq!(engine.len(), 0);
        assert!(engine.is_empty());
    }

    #[tokio::test]
    async fn test_resource_manager() {
        let manager = concurrency::ResourceManager::new(2, 4, 1);
        
        // 应该能获取到许可
        let permit1 = manager.acquire_cpu_permit().await;
        let permit2 = manager.acquire_cpu_permit().await;
        
        // 第三个应该阻塞（因为我们只有2个许可）
        let fut = manager.acquire_cpu_permit();
        assert!(!fut.now_or_never().is_some());
        
        drop(permit1); // 释放一个许可
        let permit3 = manager.acquire_cpu_permit().await; // 现在应该能获取到
    }
}