// 配置管理API接口
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from '../config';

// 获取应用配置
export async function getAppConfig(): Promise<AppConfig> {
  const backendConfig = await invoke<any>('get_app_config');
  return convertFromBackendConfig(backendConfig);
}

// 转换后端配置为前端格式
function convertFromBackendConfig(config: any): AppConfig {
  return {
    localModels: {
      enabled: config.local_models?.enabled ?? true,
      models: config.local_models?.models?.map((m: any) => ({
        id: m.id,
        name: m.name,
        description: m.description,
        enabled: m.enabled,
        path: m.path,
        capabilities: m.capabilities
      })) ?? []
    },
    remoteModels: config.remote_models?.map((m: any) => ({
      id: m.id,
      name: m.name,
      provider: m.provider,
      api_key: m.api_key ?? '',
      api_url: m.api_url ?? '',
      model: m.model,
      enabled: m.enabled,
      max_tokens: m.max_tokens,
      temperature: m.temperature
    })) ?? [],
    onnx: {
      enabled: config.onnx?.enabled ?? true,
      models: {
        embedding: config.onnx?.models?.embedding ?? true,
        ner: config.onnx?.models?.ner ?? false,
        classification: config.onnx?.models?.classification ?? false
      },
      resourceLimit: {
        maxMemoryMB: config.onnx?.resource_limit?.max_memory_mb ?? 512,
        maxThreads: config.onnx?.resource_limit?.max_threads ?? 4
      }
    },
    modelSelector: {
      currentModel: config.model_selector?.current_model ?? '',
      autoSwitch: config.model_selector?.auto_switch ?? true,
      fallbackModel: config.model_selector?.fallback_model
    },
    general: {
      autoSave: config.general?.auto_save ?? true,
      theme: config.general?.theme ?? 'light',
      language: config.general?.language ?? 'zh'
    }
  };
}

// 更新应用配置
export async function updateAppConfig(config: AppConfig): Promise<void> {
  return await invoke('update_app_config', { newConfig: convertToBackendConfig(config) });
}

// 转换前端配置为后端格式
function convertToBackendConfig(config: AppConfig): any {
  return {
    local_models: {
      enabled: config.localModels.enabled,
      models: config.localModels.models.map(m => ({
        id: m.id,
        name: m.name,
        description: m.description,
        enabled: m.enabled,
        path: m.path,
        capabilities: m.capabilities
      }))
    },
    remote_models: config.remoteModels.map(m => ({
      id: m.id,
      name: m.name,
      provider: m.provider,
      api_key: m.api_key,
      api_url: m.api_url,
      model: m.model,
      enabled: m.enabled,
      max_tokens: m.max_tokens,
      temperature: m.temperature
    })),
    onnx: {
      enabled: config.onnx.enabled,
      models: {
        embedding: config.onnx.models.embedding,
        ner: config.onnx.models.ner,
        classification: config.onnx.models.classification
      },
      resource_limit: {
        max_memory_mb: config.onnx.resourceLimit.maxMemoryMB,
        max_threads: config.onnx.resourceLimit.maxThreads
      }
    },
    model_selector: {
      current_model: config.modelSelector.currentModel,
      auto_switch: config.modelSelector.autoSwitch,
      fallback_model: config.modelSelector.fallbackModel
    },
    general: {
      auto_save: config.general.autoSave,
      theme: config.general.theme,
      language: config.general.language
    }
  };
}

// 获取当前模型信息
export async function getCurrentModelInfo(): Promise<any> {
  return await invoke('get_current_model_info');
}

// 更新模型选择
export async function updateModelSelection(modelId: string): Promise<void> {
  return await invoke('update_model_selection', { modelId });
}

// 切换本地模型启用状态
export async function toggleLocalModels(enabled: boolean): Promise<void> {
  return await invoke('toggle_local_models', { enabled });
}

// 切换ONNX功能启用状态
export async function toggleONNXFeature(feature: string, enabled: boolean): Promise<void> {
  return await invoke('toggle_onnx_feature', { feature, enabled });
}

// 配置变更事件监听
class ConfigEventManager {
  private listeners: Map<string, Array<(data: any) => void>> = new Map();
  
  subscribe(event: string, callback: (data: any) => void): () => void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(callback);
    
    // 返回取消订阅函数
    return () => {
      const eventListeners = this.listeners.get(event);
      if (eventListeners) {
        const index = eventListeners.indexOf(callback);
        if (index > -1) {
          eventListeners.splice(index, 1);
        }
      }
    };
  }
  
  emit(event: string, data: any): void {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      eventListeners.forEach(callback => callback(data));
    }
  }
}

export const configEvents = new ConfigEventManager();

// 配置同步管理器
export class ConfigSyncManager {
  private syncInterval: number | null = null;
  private lastConfigHash: string = '';
  
  async startSync(intervalMs: number = 5000): Promise<void> {
    if (this.syncInterval) {
      this.stopSync();
    }
    
    this.syncInterval = window.setInterval(async () => {
      try {
        const currentConfig = await getAppConfig();
        const configHash = JSON.stringify(currentConfig);
        
        if (configHash !== this.lastConfigHash) {
          this.lastConfigHash = configHash;
          configEvents.emit('configChanged', currentConfig);
        }
      } catch (error) {
        console.error('配置同步失败:', error);
      }
    }, intervalMs);
  }
  
  stopSync(): void {
    if (this.syncInterval) {
      clearInterval(this.syncInterval);
      this.syncInterval = null;
    }
  }
}

export const configSyncManager = new ConfigSyncManager();