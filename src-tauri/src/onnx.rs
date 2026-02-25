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
            
            log::info!("加载默认模型: {}", model_path);
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
            if let Err(e) = self.get_model(name).await {
                log::warn!("预加载模型 {} 失败: {}", name, e);
            }
        }
        Ok(())
    }
}