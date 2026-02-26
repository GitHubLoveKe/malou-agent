// 模型配置管理模块

export interface LocalModelConfig {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  path?: string;
  capabilities: string[]; // ['embedding', 'classification', 'ner']
}

export interface RemoteModelConfig {
  id: string;
  name: string;
  provider: 'openai' | 'azure' | 'custom';
  api_key: string;
  api_url: string;
  model: string;
  enabled: boolean;
  max_tokens?: number;
  temperature?: number;
}

export interface ONNXSettings {
  enabled: boolean;
  models: {
    embedding: boolean;
    ner: boolean;
    classification: boolean;
  };
  resourceLimit: {
    maxMemoryMB: number;
    maxThreads: number;
  };
}

export interface ModelSelectorConfig {
  currentModel: string;
  autoSwitch: boolean;
  fallbackModel?: string;
}

export interface AppConfig {
  localModels: {
    enabled: boolean;
    models: LocalModelConfig[];
  };
  remoteModels: RemoteModelConfig[];
  onnx: ONNXSettings;
  modelSelector: ModelSelectorConfig;
  general: {
    autoSave: boolean;
    theme: 'light' | 'dark' | 'auto';
    language: 'zh' | 'en';
  };
}

// 默认配置
export const DEFAULT_CONFIG: AppConfig = {
  localModels: {
    enabled: true,
    models: [
      {
        id: 'all-minilm-l6-v2',
        name: 'All MiniLM L6 v2',
        description: '轻量级嵌入模型',
        enabled: true,
        path: 'models/all-MiniLM-L6-v2.onnx',
        capabilities: ['embedding']
      },
      {
        id: 'distilbert-ner',
        name: 'DistilBERT NER',
        description: '命名实体识别模型',
        enabled: false,
        path: 'models/distilbert-ner.onnx',
        capabilities: ['ner']
      }
    ]
  },
  remoteModels: [
    {
      id: 'gpt-3.5-turbo',
      name: 'GPT-3.5 Turbo',
      provider: 'openai',
      api_key: '',
      api_url: 'https://api.openai.com/v1',
      model: 'gpt-3.5-turbo',
      enabled: true,
      max_tokens: 2048,
      temperature: 0.7
    }
  ],
  onnx: {
    enabled: true,
    models: {
      embedding: true,
      ner: false,
      classification: false
    },
    resourceLimit: {
      maxMemoryMB: 512,
      maxThreads: 4
    }
  },
  modelSelector: {
    currentModel: 'gpt-3.5-turbo',
    autoSwitch: true,
    fallbackModel: 'all-minilm-l6-v2'
  },
  general: {
    autoSave: true,
    theme: 'light',
    language: 'zh'
  }
};

// 配置验证函数
export function validateConfig(config: Partial<AppConfig>): string[] {
  const errors: string[] = [];
  
  if (config.remoteModels) {
    config.remoteModels.forEach((model) => {
      if (!model.api_key && model.enabled) {
        errors.push(`远程模型 "${model.name}" 需要配置API密钥`);
      }
      if (!model.api_url) {
        errors.push(`远程模型 "${model.name}" 需要配置API地址`);
      }
    });
  }
  
  if (config.localModels?.enabled && config.localModels.models.length === 0) {
    errors.push('启用了本地模型但未配置任何模型');
  }
  
  return errors;
}

// 配置合并函数
export function mergeConfig(base: AppConfig, override: Partial<AppConfig>): AppConfig {
  return {
    ...base,
    ...override,
    localModels: override.localModels ? {
      ...base.localModels,
      ...override.localModels
    } : base.localModels,
    remoteModels: override.remoteModels || base.remoteModels,
    onnx: override.onnx ? {
      ...base.onnx,
      ...override.onnx
    } : base.onnx,
    modelSelector: override.modelSelector ? {
      ...base.modelSelector,
      ...override.modelSelector
    } : base.modelSelector,
    general: override.general ? {
      ...base.general,
      ...override.general
    } : base.general
  };
}