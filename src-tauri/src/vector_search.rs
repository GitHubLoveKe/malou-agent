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