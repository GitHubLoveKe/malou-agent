# 配置管理功能实现

## 系统架构

### 1. 配置管理器 (Rust 后端)
```rust
// src-tauri/src/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub ai: AIConfig,
    pub database: DatabaseConfig,
    pub vector_db: VectorDBConfig,
    pub plugins: PluginsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub theme: String,
    pub language: String,
    pub auto_save: bool,
    pub backup_interval: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIConfig {
    pub model_path: Option<String>,
    pub use_local_compute: bool,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub backup_enabled: bool,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VectorDBConfig {
    pub chroma_url: String,
    pub api_key: Option<String>,
    pub default_collection: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginsConfig {
    pub allowed_origins: Vec<String>,
    pub auto_update: bool,
    pub sandbox_enabled: bool,
}

pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Self::load_config(&config_path)?;
        Ok(Self { config, config_path })
    }

    fn load_config(path: &PathBuf) -> Result<AppConfig, Box<dyn std::error::Error>> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let default_config = Self::default_config();
            Self::save_config(path, &default_config)?;
            Ok(default_config)
        }
    }

    fn save_config(path: &PathBuf, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn update_config(&mut self, new_config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config = new_config;
        Self::save_config(&self.config_path, &self.config)?;
        Ok(())
    }
}
```

## 前端配置界面

### 1. 主配置组件
```vue
<!-- src/components/business/SettingsView.vue -->
<template>
  <div class="settings-view">
    <el-tabs v-model="activeTab">
      <el-tab-pane label="通用设置" name="general">
        <GeneralSettings />
      </el-tab-pane>
      <el-tab-pane label="AI 配置" name="ai">
        <AISettings />
      </el-tab-pane>
      <el-tab-pane label="数据库" name="database">
        <DatabaseSettings />
      </el-tab-pane>
      <el-tab-pane label="向量数据库" name="vector-db">
        <VectorDBSettings />
      </el-tab-pane>
    </el-tabs>
  </div>
</template>
```

### 2. 通用设置组件
```vue
<!-- src/components/settings/GeneralSettings.vue -->
<template>
  <div class="general-settings">
    <el-form :model="form" label-width="120px">
      <el-form-item label="主题">
        <el-select v-model="form.theme">
          <el-option label="浅色" value="light" />
          <el-option label="深色" value="dark" />
          <el-option label="自动" value="auto" />
        </el-select>
      </el-form-item>
      
      <el-form-item label="语言">
        <el-select v-model="form.language">
          <el-option label="中文" value="zh-CN" />
          <el-option label="English" value="en-US" />
        </el-select>
      </el-form-item>
      
      <el-form-item label="自动保存">
        <el-switch v-model="form.autoSave" />
      </el-form-item>
      
      <el-form-item label="备份间隔(分钟)">
        <el-input-number 
          v-model="form.backupInterval" 
          :min="1" 
          :max="1440"
        />
      </el-form-item>
      
      <el-form-item>
        <el-button type="primary" @click="saveSettings">保存设置</el-button>
        <el-button @click="resetSettings">重置</el-button>
      </el-form-item>
    </el-form>
  </div>
</template>
```

## API 接口设计

### 1. 后端命令
```rust
#[tauri::command]
async fn get_app_config() -> Result<AppConfig, String> {
    // 获取当前应用配置
    todo!()
}

#[tauri::command]
async fn update_app_config(config: AppConfig) -> Result<(), String> {
    // 更新应用配置
    todo!()
}

#[tauri::command]
async fn reset_config_to_default() -> Result<AppConfig, String> {
    // 重置为默认配置
    todo!()
}

#[tauri::command]
async fn export_config() -> Result<String, String> {
    // 导出配置文件
    todo!()
}

#[tauri::command]
async fn import_config(config_data: String) -> Result<AppConfig, String> {
    // 导入配置文件
    todo!()
}
```

### 2. 前端 API 封装
```typescript
// src/api/config.ts
export async function getAppConfig(): Promise<AppConfig> {
  return await invoke('get_app_config');
}

export async function updateAppConfig(config: AppConfig): Promise<void> {
  return await invoke('update_app_config', { config });
}

export async function resetConfigToDefault(): Promise<AppConfig> {
  return await invoke('reset_config_to_default');
}

export async function exportConfig(): Promise<string> {
  return await invoke('export_config');
}

export async function importConfig(configData: string): Promise<AppConfig> {
  return await invoke('import_config', { configData });
}
```

## 配置文件结构

### 1. 主配置文件 (config.json)
```json
{
  "general": {
    "theme": "light",
    "language": "zh-CN",
    "auto_save": true,
    "backup_interval": 30
  },
  "ai": {
    "model_path": "/models/ggml-model.bin",
    "use_local_compute": true,
    "max_tokens": 2048,
    "temperature": 0.7
  },
  "database": {
    "path": "%APPDATA%/malou-agent/database.sqlite",
    "backup_enabled": true,
    "max_connections": 10
  },
  "vector_db": {
    "chroma_url": "http://localhost:8000",
    "api_key": null,
    "default_collection": "documents"
  },
  "plugins": {
    "allowed_origins": ["localhost"],
    "auto_update": true,
    "sandbox_enabled": true
  }
}
```

## 热重载机制

```typescript
// src/composables/useConfigWatcher.ts
import { watch } from 'vue';
import { getAppConfig, updateAppConfig } from '@/api/config';

export function useConfigWatcher() {
  const config = ref<AppConfig | null>(null);
  
  // 监听配置变化
  watch(config, async (newConfig) => {
    if (newConfig) {
      await updateAppConfig(newConfig);
    }
  }, { deep: true });
  
  // 加载初始配置
  const loadConfig = async () => {
    config.value = await getAppConfig();
  };
  
  return { config, loadConfig };
}
```

## 验证和错误处理

```rust
impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.general.backup_interval == 0 {
            return Err("Backup interval must be greater than 0".to_string());
        }
        
        if self.ai.max_tokens == 0 {
            return Err("Max tokens must be greater than 0".to_string());
        }
        
        if !self.vector_db.chroma_url.starts_with("http") {
            return Err("Invalid ChromaDB URL".to_string());
        }
        
        Ok(())
    }
}
```