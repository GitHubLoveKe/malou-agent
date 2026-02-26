<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { useConfig } from '@/config';
import { 
  getAppConfig, 
  updateAppConfig, 
  updateModelSelection,
  toggleLocalModels,
  toggleONNXFeature
} from '@/api/tauri-api';
import type { AppConfig, RemoteModelConfig } from '@/config';

const { config, updateConfig } = useConfig();

// 表单数据
const formData = reactive<AppConfig>({
  localModels: {
    enabled: true,
    models: []
  },
  remoteModels: [],
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
    currentModel: '',
    autoSwitch: true,
    fallbackModel: ''
  },
  general: {
    autoSave: true,
    theme: 'light',
    language: 'zh'
  }
});

// 加载状态
const loading = ref(false);

// 可用模型列表（用于选择器）
interface AvailableModel {
  id: string;
  name: string;
  type: 'remote' | 'local';
}

const availableModels = computed(() => {
  const models: AvailableModel[] = [];
  
  // 添加启用的远程模型
  config.value.remoteModels
    .filter(model => model.enabled)
    .forEach(model => {
      models.push({
        id: model.id,
        name: `${model.name} (${model.provider})`,
        type: 'remote'
      });
    });
  
  // 添加启用的本地模型
  if (config.value.localModels.enabled) {
    config.value.localModels.models
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
});

// 初始化数据
const loadData = async () => {
  try {
    loading.value = true;
    const appConfig = await getAppConfig();
    Object.assign(formData, appConfig);
  } catch (error) {
    console.error('加载配置失败:', error);
    ElMessage.error('加载配置失败');
  } finally {
    loading.value = false;
  }
};

// 保存配置
const saveConfig = async () => {
  try {
    loading.value = true;
    await updateAppConfig(formData);
    
    // 同步到前端配置管理器
    updateConfig(formData);
    
    ElMessage.success('配置保存成功');
  } catch (error) {
    console.error('保存配置失败:', error);
    ElMessage.error('保存配置失败');
  } finally {
    loading.value = false;
  }
};

// 切换本地模型启用状态
const handleToggleLocalModels = async (enabled: boolean) => {
  try {
    await toggleLocalModels(enabled);
    formData.localModels.enabled = enabled;
    ElMessage.success(`本地模型已${enabled ? '启用' : '禁用'}`);
  } catch (error) {
    console.error('切换本地模型失败:', error);
    ElMessage.error('切换本地模型失败');
  }
};

// 切换ONNX功能
const handleToggleONNXFeature = async (feature: string, enabled: boolean) => {
  try {
    await toggleONNXFeature(feature, enabled);
    formData.onnx.models[feature as keyof typeof formData.onnx.models] = enabled;
    ElMessage.success(`${feature}功能已${enabled ? '启用' : '禁用'}`);
  } catch (error) {
    console.error('切换ONNX功能失败:', error);
    ElMessage.error('切换ONNX功能失败');
  }
};

// 更改当前模型
const handleChangeCurrentModel = async (modelId: string) => {
  try {
    await updateModelSelection(modelId);
    formData.modelSelector.currentModel = modelId;
    ElMessage.success('模型切换成功');
  } catch (error) {
    console.error('切换模型失败:', error);
    ElMessage.error('切换模型失败');
  }
};

// 添加远程模型
const handleAddRemoteModel = () => {
  const newModel: RemoteModelConfig = {
    id: `custom-${Date.now()}`,
    name: '自定义模型',
    provider: 'custom',
    api_key: '',
    api_url: '',
    model: '',
    enabled: true
  };
  formData.remoteModels.push(newModel);
};

// 删除远程模型
const handleRemoveRemoteModel = async (index: number) => {
  try {
    await ElMessageBox.confirm('确定要删除这个模型配置吗？', '确认删除', {
      type: 'warning'
    });
    formData.remoteModels.splice(index, 1);
  } catch {
    // 用户取消删除
  }
};

// 导出配置
const handleExportConfig = () => {
  const configStr = JSON.stringify(formData, null, 2);
  const blob = new Blob([configStr], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `malou-agent-config-${new Date().toISOString().split('T')[0]}.json`;
  a.click();
  URL.revokeObjectURL(url);
};

// 导入配置
const handleImportConfig = (event: Event) => {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  
  const reader = new FileReader();
  reader.onload = async (e) => {
    try {
      const configStr = e.target?.result as string;
      const importedConfig = JSON.parse(configStr);
      
      await ElMessageBox.confirm('导入配置将覆盖当前设置，确定要继续吗？', '确认导入', {
        type: 'warning'
      });
      
      Object.assign(formData, importedConfig);
      await saveConfig();
    } catch (error) {
      ElMessage.error('导入配置失败：' + (error as Error).message);
    }
  };
  reader.readAsText(file);
};

onMounted(() => {
  loadData();
});
</script>

<template>
  <div class="model-settings">
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <span>模型配置管理</span>
          <div class="header-actions">
            <el-button @click="handleExportConfig" size="small">
              导出配置
            </el-button>
            <input 
              type="file" 
              accept=".json" 
              @change="handleImportConfig" 
              style="display: none" 
              ref="importInput"
            >
            <el-button @click="($refs.importInput as HTMLInputElement).click()" size="small">
              导入配置
            </el-button>
            <el-button type="primary" @click="saveConfig" :loading="loading" size="small">
              保存配置
            </el-button>
          </div>
        </div>
      </template>

      <el-tabs type="border-card">
        <!-- 模型选择 -->
        <el-tab-pane label="模型选择">
          <el-form :model="formData" label-width="120px">
            <el-form-item label="当前模型">
              <el-select 
                v-model="formData.modelSelector.currentModel" 
                @change="handleChangeCurrentModel"
                placeholder="请选择模型"
              >
                <el-option
                  v-for="model in availableModels"
                  :key="model.id"
                  :label="model.name"
                  :value="model.id"
                />
              </el-select>
            </el-form-item>
            
            <el-form-item label="自动切换">
              <el-switch v-model="formData.modelSelector.autoSwitch" />
              <span class="form-tip">当首选模型不可用时自动切换到备用模型</span>
            </el-form-item>
            
            <el-form-item label="备用模型" v-if="formData.modelSelector.autoSwitch">
              <el-select 
                v-model="formData.modelSelector.fallbackModel" 
                placeholder="请选择备用模型"
              >
                <el-option
                  v-for="model in availableModels.filter(m => m.id !== formData.modelSelector.currentModel)"
                  :key="model.id"
                  :label="model.name"
                  :value="model.id"
                />
              </el-select>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 本地模型 -->
        <el-tab-pane label="本地模型">
          <el-alert
            title="本地模型设置"
            type="info"
            description="管理本地ONNX模型的启用状态和配置"
            show-icon
            :closable="false"
          />
          
          <div class="section-divider"></div>
          
          <el-form :model="formData" label-width="120px">
            <el-form-item label="启用本地模型">
              <el-switch 
                v-model="formData.localModels.enabled" 
                @change="handleToggleLocalModels"
              />
            </el-form-item>
            
            <div v-if="formData.localModels.enabled">
              <el-form-item 
                v-for="model in formData.localModels.models" 
                :key="model.id"
                :label="`${model.name}`"
              >
                <div class="model-config-item">
                  <el-switch 
                    v-model="model.enabled"
                    :inactive-text="model.description"
                  />
                  <div class="model-details" v-if="model.enabled">
                    <el-tag size="small" type="info">{{ model.path }}</el-tag>
                    <div class="capabilities">
                      <el-tag 
                        v-for="cap in model.capabilities" 
                        :key="cap" 
                        size="small"
                        style="margin-right: 5px;"
                      >
                        {{ cap }}
                      </el-tag>
                    </div>
                  </div>
                </div>
              </el-form-item>
            </div>
          </el-form>
        </el-tab-pane>

        <!-- 远程模型 -->
        <el-tab-pane label="远程模型">
          <el-alert
            title="远程模型配置"
            type="warning"
            description="配置外部AI服务商的API密钥和模型参数"
            show-icon
            :closable="false"
          />
          
          <div class="section-divider"></div>
          
          <el-button @click="handleAddRemoteModel" type="primary" size="small" style="margin-bottom: 15px;">
            添加远程模型
          </el-button>
          
          <div 
            v-for="(model, index) in formData.remoteModels" 
            :key="model.id" 
            class="remote-model-card"
          >
            <el-card shadow="hover">
              <template #header>
                <div class="model-header">
                  <span>{{ model.name }}</span>
                  <el-button 
                    @click="handleRemoveRemoteModel(index)" 
                    type="danger" 
                    size="small" 
                    link
                  >
                    删除
                  </el-button>
                </div>
              </template>
              
              <el-form :model="model" label-width="100px" size="small">
                <el-row :gutter="20">
                  <el-col :span="12">
                    <el-form-item label="模型名称">
                      <el-input v-model="model.name" />
                    </el-form-item>
                  </el-col>
                  <el-col :span="12">
                    <el-form-item label="服务商">
                      <el-select v-model="model.provider" placeholder="选择服务商">
                        <el-option label="OpenAI" value="openai" />
                        <el-option label="Azure OpenAI" value="azure" />
                        <el-option label="自定义" value="custom" />
                      </el-select>
                    </el-form-item>
                  </el-col>
                </el-row>
                
                <el-form-item label="API地址">
                  <el-input v-model="model.api_url" placeholder="https://api.example.com/v1" />
                </el-form-item>
                
                <el-form-item label="API密钥">
                  <el-input 
                    v-model="model.api_key" 
                    type="password" 
                    show-password 
                    placeholder="sk-..." 
                  />
                </el-form-item>
                
                <el-form-item label="模型标识">
                  <el-input v-model="model.model" placeholder="gpt-3.5-turbo" />
                </el-form-item>
                
                <el-row :gutter="20">
                  <el-col :span="12">
                    <el-form-item label="最大Token">
                      <el-input-number 
                        v-model="model.max_tokens" 
                        :min="1" 
                        :max="32768" 
                        controls-position="right"
                      />
                    </el-form-item>
                  </el-col>
                  <el-col :span="12">
                    <el-form-item label="温度">
                      <el-slider 
                        v-model="model.temperature" 
                        :min="0" 
                        :max="2" 
                        :step="0.1" 
                        show-input
                      />
                    </el-form-item>
                  </el-col>
                </el-row>
                
                <el-form-item label="启用状态">
                  <el-switch v-model="model.enabled" />
                </el-form-item>
              </el-form>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- ONNX设置 -->
        <el-tab-pane label="ONNX辅助">
          <el-alert
            title="ONNX辅助功能"
            type="success"
            description="轻量级本地模型，用于文本处理等辅助功能"
            show-icon
            :closable="false"
          />
          
          <div class="section-divider"></div>
          
          <el-form :model="formData" label-width="120px">
            <el-form-item label="启用ONNX">
              <el-switch v-model="formData.onnx.enabled" />
              <span class="form-tip">启用后可在聊天中使用本地文本处理功能</span>
            </el-form-item>
            
            <div v-if="formData.onnx.enabled">
              <el-form-item label="嵌入模型">
                <el-switch 
                  v-model="formData.onnx.models.embedding"
                  @change="(val: boolean) => handleToggleONNXFeature('embedding', val)"
                />
                <span class="form-tip">文本向量化处理</span>
              </el-form-item>
              
              <el-form-item label="实体识别">
                <el-switch 
                  v-model="formData.onnx.models.ner"
                  @change="(val: boolean) => handleToggleONNXFeature('ner', val)"
                />
                <span class="form-tip">命名实体识别</span>
              </el-form-item>
              
              <el-form-item label="文本分类">
                <el-switch 
                  v-model="formData.onnx.models.classification"
                  @change="(val: boolean) => handleToggleONNXFeature('classification', val)"
                />
                <span class="form-tip">文本情感分析等分类任务</span>
              </el-form-item>
              
              <el-form-item label="资源限制">
                <div class="resource-limit">
                  <el-form-item label="最大内存(MB)">
                    <el-input-number 
                      v-model="formData.onnx.resourceLimit.maxMemoryMB" 
                      :min="128" 
                      :max="2048" 
                      controls-position="right"
                    />
                  </el-form-item>
                  <el-form-item label="最大线程数">
                    <el-input-number 
                      v-model="formData.onnx.resourceLimit.maxThreads" 
                      :min="1" 
                      :max="16" 
                      controls-position="right"
                    />
                  </el-form-item>
                </div>
              </el-form-item>
            </div>
          </el-form>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<style scoped>
.model-settings {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.settings-card {
  height: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.section-divider {
  height: 1px;
  background-color: #e4e7ed;
  margin: 20px 0;
}

.form-tip {
  margin-left: 10px;
  color: #909399;
  font-size: 12px;
}

.model-config-item {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.model-details {
  margin-top: 10px;
}

.capabilities {
  margin-top: 5px;
}

.remote-model-card {
  margin-bottom: 20px;
}

.model-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.resource-limit {
  display: flex;
  gap: 30px;
  flex-wrap: wrap;
}

.resource-limit :deep(.el-form-item) {
  margin-bottom: 0;
}
</style>