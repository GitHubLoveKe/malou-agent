// Tauri配置管理模块
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalModelConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub path: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_key: String,
    pub api_url: String,
    pub model: String,
    pub enabled: bool,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ONNXSettings {
    pub enabled: bool,
    pub models: HashMap<String, bool>,
    pub resource_limit: ResourceLimit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceLimit {
    pub max_memory_mb: u32,
    pub max_threads: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelSelectorConfig {
    pub current_model: String,
    pub auto_switch: bool,
    pub fallback_model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralSettings {
    pub auto_save: bool,
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub local_models: LocalModelsConfig,
    pub remote_models: Vec<RemoteModelConfig>,
    pub onnx: ONNXSettings,
    pub model_selector: ModelSelectorConfig,
    pub general: GeneralSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalModelsConfig {
    pub enabled: bool,
    pub models: Vec<LocalModelConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ConfigStorage {
    pub app_config: AppConfig,
    pub model_configs: HashMap<String, serde_json::Value>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            local_models: LocalModelsConfig {
                enabled: true,
                models: vec![
                    LocalModelConfig {
                        id: "all-minilm-l6-v2".to_string(),
                        name: "All MiniLM L6 v2".to_string(),
                        description: "轻量级嵌入模型".to_string(),
                        enabled: true,
                        path: Some("models/all-MiniLM-L6-v2.onnx".to_string()),
                        capabilities: vec!["embedding".to_string()],
                    },
                    LocalModelConfig {
                        id: "distilbert-ner".to_string(),
                        name: "DistilBERT NER".to_string(),
                        description: "命名实体识别模型".to_string(),
                        enabled: false,
                        path: Some("models/distilbert-ner.onnx".to_string()),
                        capabilities: vec!["ner".to_string()],
                    },
                ],
            },
            remote_models: vec![RemoteModelConfig {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                provider: "openai".to_string(),
                api_key: "".to_string(),
                api_url: "https://api.openai.com/v1".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                enabled: true,
                max_tokens: Some(2048),
                temperature: Some(0.7),
            }],
            onnx: ONNXSettings {
                enabled: true,
                models: [
                    ("embedding".to_string(), true),
                    ("ner".to_string(), false),
                    ("classification".to_string(), false),
                ]
                .iter()
                .cloned()
                .collect(),
                resource_limit: ResourceLimit {
                    max_memory_mb: 512,
                    max_threads: 4,
                },
            },
            model_selector: ModelSelectorConfig {
                current_model: "gpt-3.5-turbo".to_string(),
                auto_switch: true,
                fallback_model: Some("all-minilm-l6-v2".to_string()),
            },
            general: GeneralSettings {
                auto_save: true,
                theme: "light".to_string(),
                language: "zh".to_string(),
            },
        }
    }
}

pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_dir = app_handle.path().app_config_dir()?;
        let config_path = app_dir.join("config.json");
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let config = Self::load_config(&config_path)?;
        
        Ok(Self {
            config,
            config_path,
        })
    }
    
    fn load_config(config_path: &PathBuf) -> Result<AppConfig, Box<dyn std::error::Error>> {
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // 创建默认配置文件
            let default_config = AppConfig::default();
            Self::save_config_to_file(config_path, &default_config)?;
            Ok(default_config)
        }
    }
    
    fn save_config_to_file(
        config_path: &PathBuf,
        config: &AppConfig
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(config_path, content)?;
        Ok(())
    }
    
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
    
    pub fn update_config(&mut self, new_config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config = new_config;
        Self::save_config_to_file(&self.config_path, &self.config)?;
        Ok(())
    }
    
    pub fn update_section<T>(&mut self, updater: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: FnOnce(&mut AppConfig) -> Result<(), Box<dyn std::error::Error>>,
    {
        updater(&mut self.config)?;
        Self::save_config_to_file(&self.config_path, &self.config)?;
        Ok(())
    }
    
    pub fn get_current_model(&self) -> Option<CurrentModel> {
        let current_model_id = &self.config.model_selector.current_model;
        
        // 查找远程模型
        if let Some(remote_model) = self.config.remote_models
            .iter()
            .find(|m| &m.id == current_model_id && m.enabled)
        {
            return Some(CurrentModel::Remote(remote_model.clone()));
        }
        
        // 查找本地模型
        if let Some(local_model) = self.config.local_models.models
            .iter()
            .find(|m| &m.id == current_model_id && m.enabled)
        {
            return Some(CurrentModel::Local(local_model.clone()));
        }
        
        // 返回备用模型
        if let Some(fallback_id) = &self.config.model_selector.fallback_model {
            if let Some(local_model) = self.config.local_models.models
                .iter()
                .find(|m| &m.id == fallback_id)
            {
                return Some(CurrentModel::Local(local_model.clone()));
            }
        }
        
        None
    }
    
    pub fn is_onnx_enabled(&self) -> bool {
        self.config.onnx.enabled
    }
    
    pub fn is_onnx_model_enabled(&self, model_type: &str) -> bool {
        self.config.onnx.enabled 
            && self.config.onnx.models.get(model_type).copied().unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CurrentModel {
    Local(LocalModelConfig),
    Remote(RemoteModelConfig),
}

// Tauri命令
#[tauri::command]
pub async fn get_app_config(
    app_handle: tauri::AppHandle
) -> Result<AppConfig, String> {
    let config_manager = app_handle.state::<std::sync::Mutex<ConfigManager>>();
    let manager = config_manager.lock().unwrap();
    Ok(manager.get_config().clone())
}

#[tauri::command]
pub async fn update_app_config(
    new_config: AppConfig,
    app_handle: tauri::AppHandle
) -> Result<(), String> {
    log::info!("update_app_config called");
    
    let config_manager = app_handle.state::<std::sync::Mutex<ConfigManager>>();
    let mut manager = config_manager.lock().unwrap();
    manager.update_config(new_config.clone()).map_err(|e| e.to_string())?;
    
    // Sync OpenAI config to AppState
    let current_model_id = &new_config.model_selector.current_model;
    if let Some(remote_model) = new_config.remote_models.iter().find(|m| &m.id == current_model_id) {
        if let Some(app_state) = app_handle.try_state::<super::AppState>() {
            if let Ok(mut openai_config) = app_state.openai_config.lock() {
                openai_config.api_key = remote_model.api_key.clone();
                openai_config.api_base = remote_model.api_url.clone();
                openai_config.model = remote_model.model.clone();
                openai_config.temperature = remote_model.temperature.unwrap_or(0.7);
                log::info!("OpenAI config synced to AppState: model={}", openai_config.model);
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_current_model_info(
    app_handle: tauri::AppHandle
) -> Result<Option<CurrentModel>, String> {
    let config_manager = app_handle.state::<std::sync::Mutex<ConfigManager>>();
    let manager = config_manager.lock().unwrap();
    Ok(manager.get_current_model())
}

#[tauri::command]
pub fn update_model_selection(
    model_id: String,
    app_handle: tauri::AppHandle
) -> Result<(), String> {
    let config_manager = app_handle.state::<std::sync::Mutex<ConfigManager>>();
    let mut manager = config_manager.lock().unwrap();
    
    manager.update_section(|config| {
        config.model_selector.current_model = model_id;
        Ok::<(), Box<dyn std::error::Error>>(())
    }).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn toggle_local_models(
    enabled: bool,
    app_handle: tauri::AppHandle
) -> Result<(), String> {
    let config_manager = app_handle.state::<std::sync::Mutex<ConfigManager>>();
    let mut manager = config_manager.lock().unwrap();
    
    manager.update_section(|config| {
        config.local_models.enabled = enabled;
        Ok::<(), Box<dyn std::error::Error>>(())
    }).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn toggle_onnx_feature(
    feature: String,
    enabled: bool,
    app_handle: tauri::AppHandle
) -> Result<(), String> {
    let config_manager = app_handle.state::<std::sync::Mutex<ConfigManager>>();
    let mut manager = config_manager.lock().unwrap();
    
    manager.update_section(|config| {
        config.onnx.models.insert(feature, enabled);
        Ok::<(), Box<dyn std::error::Error>>(())
    }).map_err(|e| e.to_string())?;
    
    Ok(())
}