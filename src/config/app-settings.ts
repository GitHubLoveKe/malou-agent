// 应用设置管理模块

import { ref, watch } from 'vue';
import { AppConfig, DEFAULT_CONFIG, mergeConfig, validateConfig } from './model-config';

class ConfigManager {
  private config = ref<AppConfig>(DEFAULT_CONFIG);
  private storageKey = 'malou-agent-config';
  
  constructor() {
    this.loadConfig();
    this.setupAutoSave();
  }
  
  // 获取配置
  getConfig(): AppConfig {
    return this.config.value;
  }
  
  // 获取配置响应式引用
  getConfigRef() {
    return this.config;
  }
  
  // 更新配置
  updateConfig(newConfig: Partial<AppConfig>): { success: boolean; errors?: string[] } {
    const errors = validateConfig(newConfig);
    if (errors.length > 0) {
      return { success: false, errors };
    }
    
    this.config.value = mergeConfig(this.config.value, newConfig);
    this.saveConfig();
    return { success: true };
  }
  
  // 更新特定部分配置
  updateSection<K extends keyof AppConfig>(
    section: K, 
    data: Partial<AppConfig[K]>
  ): { success: boolean; errors?: string[] } {
    const newConfig = {
      ...this.config.value,
      [section]: {
        ...this.config.value[section],
        ...data
      }
    };
    
    return this.updateConfig(newConfig);
  }
  
  // 重置为默认配置
  resetToDefault(): void {
    this.config.value = DEFAULT_CONFIG;
    this.saveConfig();
  }
  
  // 导出配置
  exportConfig(): string {
    return JSON.stringify(this.config.value, null, 2);
  }
  
  // 导入配置
  importConfig(configStr: string): { success: boolean; errors?: string[] } {
    try {
      const importedConfig = JSON.parse(configStr);
      return this.updateConfig(importedConfig);
    } catch (error) {
      return { 
        success: false, 
        errors: [`配置文件格式错误: ${(error as Error).message}`] 
      };
    }
  }
  
  // 获取当前激活的模型
  getCurrentModel() {
    const selector = this.config.value.modelSelector;
    const currentModelId = selector.currentModel;
    
    // 查找远程模型
    const remoteModel = this.config.value.remoteModels.find(
      model => model.id === currentModelId && model.enabled
    );
    if (remoteModel) return { type: 'remote' as const, model: remoteModel };
    
    // 查找本地模型
    const localModel = this.config.value.localModels.models.find(
      model => model.id === currentModelId && model.enabled
    );
    if (localModel) return { type: 'local' as const, model: localModel };
    
    // 返回备用模型
    if (selector.fallbackModel) {
      const fallbackModel = this.config.value.localModels.models.find(
        model => model.id === selector.fallbackModel
      );
      if (fallbackModel) return { type: 'local' as const, model: fallbackModel };
    }
    
    return null;
  }

  // 获取所有可用模型列表
  getAvailableModels() {
    const models: Array<{ id: string; name: string; type: 'remote' | 'local' }> = [];
    
    // 添加启用的远程模型
    this.config.value.remoteModels
      .filter(model => model.enabled)
      .forEach(model => {
        models.push({
          id: model.id,
          name: `${model.name} (${model.provider})`,
          type: 'remote'
        });
      });
    
    // 添加启用的本地模型
    if (this.config.value.localModels.enabled) {
      this.config.value.localModels.models
        .filter(model => model.enabled)
        .forEach(model => {
          models.push({
            id: model.id,
            name: `${model.name} (本地)`,
            type: 'local'
          });
        });
    }
    
    return models;
  }

  // 设置当前模型
  setCurrentModel(modelId: string) {
    this.config.value.modelSelector.currentModel = modelId;
    this.saveConfig();
  }
  
  // 检查ONNX功能是否启用
  isONNXEnabled(): boolean {
    return this.config.value.onnx.enabled;
  }
  
  // 检查特定ONNX模型是否启用
  isONNXModelEnabled(modelType: keyof typeof DEFAULT_CONFIG.onnx.models): boolean {
    return this.config.value.onnx.enabled && this.config.value.onnx.models[modelType];
  }
  
  private loadConfig(): void {
    try {
      const stored = localStorage.getItem(this.storageKey);
      if (stored) {
        const parsed = JSON.parse(stored);
        this.config.value = mergeConfig(DEFAULT_CONFIG, parsed);
      }
    } catch (error) {
      console.warn('加载配置失败，使用默认配置:', error);
      this.config.value = DEFAULT_CONFIG;
    }
  }
  
  private saveConfig(): void {
    try {
      localStorage.setItem(this.storageKey, JSON.stringify(this.config.value));
    } catch (error) {
      console.error('保存配置失败:', error);
    }
  }
  
  private setupAutoSave(): void {
    watch(this.config, () => {
      if (this.config.value.general.autoSave) {
        this.saveConfig();
      }
    }, { deep: true });
  }
}

// 创建全局配置管理实例
export const configManager = new ConfigManager();

// 导出常用的配置获取函数
export const useConfig = () => ({
  config: configManager.getConfigRef(),
  updateConfig: configManager.updateConfig.bind(configManager),
  updateSection: configManager.updateSection.bind(configManager),
  getCurrentModel: configManager.getCurrentModel.bind(configManager),
  getAvailableModels: configManager.getAvailableModels.bind(configManager),
  setCurrentModel: configManager.setCurrentModel.bind(configManager),
  isONNXEnabled: configManager.isONNXEnabled.bind(configManager),
  isONNXModelEnabled: configManager.isONNXModelEnabled.bind(configManager)
});