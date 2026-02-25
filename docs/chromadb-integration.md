# ChromaDB 向量数据库集成方案

## 设计目标
集成 ChromaDB 向量数据库，提供高效的向量搜索和相似度匹配功能。

## 技术架构

### 1. 后端集成 (Rust)
```rust
// src-tauri/src/vector_db.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChromaCollection {
    pub id: String,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorQuery {
    pub query_embeddings: Vec<Vec<f32>>,
    pub n_results: usize,
    pub where_filter: Option<serde_json::Value>,
}

pub struct ChromaDBClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ChromaDBClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
        }
    }

    pub async fn create_collection(&self, name: &str) -> Result<ChromaCollection, Box<dyn std::error::Error>> {
        // 实现集合创建逻辑
        todo!()
    }

    pub async fn add_documents(&self, collection_id: &str, documents: Vec<DocumentEmbedding>) -> Result<(), Box<dyn std::error::Error>> {
        // 实现文档添加逻辑
        todo!()
    }

    pub async fn query_similar(&self, collection_id: &str, query: VectorQuery) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        // 实现相似度查询逻辑
        todo!()
    }
}
```

### 2. 前端集成 (Vue)
```typescript
// src/api/vector-db.ts
import { invoke } from '@tauri-apps/api/core';

export interface VectorSearchRequest {
  query: string;
  collection: string;
  limit: number;
  filters?: Record<string, any>;
}

export interface SearchResult {
  id: string;
  document: string;
  metadata: Record<string, any>;
  similarity: number;
}

export async function searchSimilarDocuments(request: VectorSearchRequest): Promise<SearchResult[]> {
  return await invoke('search_similar_documents', { request });
}

export async function addDocumentToVectorDB(document: DocumentEmbedding): Promise<void> {
  return await invoke('add_document_to_vector_db', { document });
}
```

## 核心功能

### 1. 集合管理
- 创建和删除向量集合
- 集合元数据管理
- 索引配置优化

### 2. 文档操作
- 文本向量化处理
- 批量文档导入
- 实时文档更新

### 3. 向量搜索
- 语义相似度搜索
- 过滤条件支持
- 多向量查询

## 集成步骤

1. 添加 ChromaDB HTTP 客户端依赖
2. 实现向量数据库操作 API
3. 集成文本嵌入生成功能
4. 建立前后端通信接口
5. 添加错误处理和重试机制

## 配置选项

```json
{
  "vector_db": {
    "chroma_url": "http://localhost:8000",
    "api_key": "your-api-key",
    "default_collection": "documents",
    "embedding_model": "sentence-transformers/all-MiniLM-L6-v2"
  }
}
```