# 技能插件系统设计方案

## 系统架构

### 1. 插件管理器 (Rust 后端)
```rust
// src-tauri/src/plugins/mod.rs
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub parameters: serde_json::Value,
    pub settings: serde_json::Value,
}

pub trait SkillPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, String>;
    fn validate_params(&self, params: &serde_json::Value) -> Result<(), String>;
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn SkillPlugin>>,
    configs: HashMap<String, PluginConfig>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn SkillPlugin>) -> Result<(), String> {
        let id = plugin.id().to_string();
        self.plugins.insert(id, plugin);
        Ok(())
    }

    pub fn execute_skill(&self, skill_id: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        match self.plugins.get(skill_id) {
            Some(plugin) => plugin.execute(params),
            None => Err(format!("Skill plugin '{}' not found", skill_id)),
        }
    }
}
```

### 2. 内置技能示例
```rust
// src-tauri/src/plugins/calculator.rs
pub struct CalculatorPlugin;

impl SkillPlugin for CalculatorPlugin {
    fn id(&self) -> &str { "calculator" }
    fn name(&self) -> &str { "计算器" }
    fn description(&self) -> &str { "执行数学计算" }

    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        // 实现计算逻辑
        todo!()
    }

    fn validate_params(&self, params: &serde_json::Value) -> Result<(), String> {
        // 参数验证逻辑
        todo!()
    }
}
```

## 前端插件管理界面

### 1. 插件商店组件
```vue
<!-- src/components/business/PluginStore.vue -->
<template>
  <div class="plugin-store">
    <el-tabs v-model="activeTab">
      <el-tab-pane label="已安装" name="installed">
        <InstalledPlugins />
      </el-tab-pane>
      <el-tab-pane label="发现" name="discover">
        <PluginDiscovery />
      </el-tab-pane>
    </el-tabs>
  </div>
</template>
```

### 2. 插件配置界面
```vue
<!-- src/components/business/PluginConfig.vue -->
<template>
  <div class="plugin-config">
    <el-form :model="configForm">
      <el-form-item 
        v-for="param in pluginParams" 
        :key="param.name"
        :label="param.displayName"
      >
        <component 
          :is="getParamComponent(param.type)"
          v-model="configForm[param.name]"
          :options="param.options"
        />
      </el-form-item>
    </el-form>
  </div>
</template>
```

## API 接口设计

### 1. 后端命令
```rust
#[tauri::command]
async fn list_plugins() -> Result<Vec<PluginMetadata>, String> {
    // 返回所有已安装插件列表
    todo!()
}

#[tauri::command]
async fn execute_plugin_skill(
    plugin_id: String, 
    params: serde_json::Value
) -> Result<serde_json::Value, String> {
    // 执行指定插件技能
    todo!()
}

#[tauri::command]
async fn install_plugin(plugin_source: String) -> Result<PluginMetadata, String> {
    // 安装新插件
    todo!()
}
```

### 2. 前端 API 封装
```typescript
// src/api/plugins.ts
export async function listInstalledPlugins(): Promise<PluginMetadata[]> {
  return await invoke('list_plugins');
}

export async function executePluginSkill(
  pluginId: string, 
  params: Record<string, any>
): Promise<any> {
  return await invoke('execute_plugin_skill', { pluginId, params });
}

export async function installPlugin(source: string): Promise<PluginMetadata> {
  return await invoke('install_plugin', { source });
}
```

## 插件开发规范

### 1. 插件结构
```
plugins/
├── calculator/
│   ├── src/
│   │   └── lib.rs
│   ├── Cargo.toml
│   └── plugin.json
└── web-search/
    ├── src/
    │   └── lib.rs
    ├── Cargo.toml
    └── plugin.json
```

### 2. 插件配置文件
```json
{
  "id": "web-search",
  "name": "网络搜索",
  "version": "1.0.0",
  "description": "执行网络搜索查询",
  "author": "Malou Team",
  "entry_point": "lib.rs",
  "permissions": ["network", "filesystem"],
  "parameters": [
    {
      "name": "query",
      "type": "string",
      "required": true,
      "description": "搜索关键词"
    }
  ]
}
```

## 安全机制

1. **沙箱隔离**：每个插件运行在独立的沙箱环境中
2. **权限控制**：细粒度的权限管理系统
3. **代码签名**：插件代码完整性验证
4. **审计日志**：记录所有插件执行活动