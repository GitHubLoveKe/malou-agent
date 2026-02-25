# Tauri v2 与 Vue 3 集成最佳实践指南

## 1. Tauri v2 新特性和API变化

### 核心架构重构
- **模块解耦**：`tauri::api` 模块被移除，功能分散到独立插件中
- **权限系统**：引入新的权限模型，替代原有的白名单机制
- **插件架构**：采用工作区插件系统，提高模块化程度

### 主要API变更

#### 配置文件结构变化
```json
{
  "tauri": {
    "bundle": {
      "identifier": "com.example.app", // 新增必填字段
      "targets": "all"
    }
  }
}
```

#### 插件迁移示例
```rust
// Tauri v1
tauri::Builder::default()
    .setup(|app| {
        // 直接使用 tauri::api
        Ok(())
    })

// Tauri v2
tauri::Builder::default()
    .plugin(tauri_plugin_os::init())  // 插件化
    .plugin(tauri_plugin_cli::init())
```

### 新增功能
- **移动端支持**：原生支持iOS和Android
- **改进的CLI**：增强的命令行工具和自动化迁移
- **更好的类型安全**：Rust API表面的重大重构

## 2. Vue 3 + TypeScript + Vite 标准项目结构

### 推荐目录结构
```
src/
├── assets/              # 静态资源
├── components/          # 可复用组件
│   ├── base/           # 基础组件 (BaseButton.vue等)
│   └── business/       # 业务组件
├── composables/        # Vue组合式函数
├── layouts/            # 页面布局组件
├── pages/             # 页面组件
├── router/            # 路由配置
├── stores/            # Pinia状态管理
├── types/             # TypeScript类型定义
├── utils/             # 工具函数
└── views/             # 视图页面
```

### 关键配置文件

#### tsconfig.json
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "jsx": "preserve",
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
```

#### vite.config.ts
```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src')
    }
  },
  server: {
    port: 3000
  }
})
```

## 3. Tauri命令与Vue组件通信模式

### 基础通信模式

#### Rust后端 (src-tauri/src/commands.rs)
```rust
use tauri::command;

#[command]
pub async fn greet(name: String) -> Result<String, String> {
    Ok(format!("Hello, {}! Welcome to Tauri.", name))
}

#[command]
pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero!".to_string())
    } else {
        Ok(a / b)
    }
}
```

#### Vue前端调用
```typescript
import { invoke } from '@tauri-apps/api/core'

// 简单调用
const greeting = await invoke('greet', { name: 'World' })
console.log(greeting) // "Hello, World! Welcome to Tauri."

// 带错误处理
try {
  const result = await invoke('divide', { a: 10, b: 2 })
  console.log(result) // 5
} catch (error) {
  console.error('Division failed:', error)
}
```

### 类型安全的通信封装

#### 创建API服务层
```typescript
// src/services/tauriApi.ts
import { invoke } from '@tauri-apps/api/core'

export interface ChatMessage {
  id: string
  content: string
  timestamp: number
  sender: 'user' | 'ai'
}

export class TauriApiService {
  static async sendMessage(message: string): Promise<ChatMessage> {
    return await invoke<ChatMessage>('send_message', { message })
  }

  static async searchKnowledge(query: string): Promise<any[]> {
    return await invoke<any[]>('search_knowledge', { query })
  }

  static async runSkill(skillName: string, args: any): Promise<any> {
    return await invoke<any>('run_skill', { 
      skillName, 
      args 
    })
  }
}
```

#### 在组件中使用
```vue
<script setup lang="ts">
import { ref } from 'vue'
import { TauriApiService, type ChatMessage } from '@/services/tauriApi'

const messages = ref<ChatMessage[]>([])
const inputMessage = ref('')

const handleSend = async () => {
  if (!inputMessage.value.trim()) return
  
  try {
    const response = await TauriApiService.sendMessage(inputMessage.value)
    messages.value.push(response)
    inputMessage.value = ''
  } catch (error) {
    console.error('发送消息失败:', error)
  }
}
</script>
```

## 4. 状态管理和数据流设计

### Pinia状态管理架构

#### 核心Store设计
```typescript
// src/stores/chatStore.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { TauriApiService } from '@/services/tauriApi'

export const useChatStore = defineStore('chat', () => {
  // 状态
  const messages = ref<ChatMessage[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // 计算属性
  const messageCount = computed(() => messages.value.length)
  const hasError = computed(() => error.value !== null)

  // Actions
  const sendMessage = async (content: string) => {
    isLoading.value = true
    error.value = null
    
    try {
      const response = await TauriApiService.sendMessage(content)
      messages.value.push(response)
    } catch (err) {
      error.value = err instanceof Error ? err.message : '未知错误'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  const clearMessages = () => {
    messages.value = []
    error.value = null
  }

  return {
    // 状态
    messages,
    isLoading,
    error,
    
    // 计算属性
    messageCount,
    hasError,
    
    // Actions
    sendMessage,
    clearMessages
  }
})
```

#### 配置Store
```typescript
// src/stores/configStore.ts
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { storeToRefs } from 'pinia'

interface AppConfig {
  localComputation: {
    enabled: boolean
    embedding: {
      enabled: boolean
      modelPath: string
    }
  }
  vllmUrl: string
}

export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig>({
    localComputation: {
      enabled: true,
      embedding: {
        enabled: true,
        modelPath: 'models/all-MiniLM-L6-v2.onnx'
      }
    },
    vllmUrl: 'http://localhost:8000'
  })

  // 自动保存配置到本地存储
  watch(config, async (newConfig) => {
    try {
      await window.__TAURI__.invoke('save_config', { config: newConfig })
    } catch (error) {
      console.error('保存配置失败:', error)
    }
  }, { deep: true })

  const updateConfig = (newConfig: Partial<AppConfig>) => {
    config.value = { ...config.value, ...newConfig }
  }

  return {
    config,
    updateConfig
  }
})
```

### 数据流模式

#### 单向数据流架构
```
Vue Component → Pinia Store → Tauri Command → Rust Backend → Database/External Service
     ↑              ↑              ↑              ↓              ↓
     └──────────────┴──────────────┴←←←←←←←←←←←←←←←←←←←←←←←←←←←←←┘
```

#### 异步数据处理模式
```typescript
// src/composables/useAsyncData.ts
import { ref, type Ref } from 'vue'

interface AsyncState<T> {
  data: Ref<T | null>
  loading: Ref<boolean>
  error: Ref<Error | null>
}

export function useAsyncData<T>(
  asyncFn: () => Promise<T>
): AsyncState<T> & { execute: () => Promise<void> } {
  const data = ref<T | null>(null) as Ref<T | null>
  const loading = ref(false)
  const error = ref<Error | null>(null)

  const execute = async () => {
    loading.value = true
    error.value = null
    
    try {
      data.value = await asyncFn()
    } catch (err) {
      error.value = err instanceof Error ? err : new Error(String(err))
    } finally {
      loading.value = false
    }
  }

  return { data, loading, error, execute }
}
```

## 5. 本地存储和配置管理最佳实践

### Tauri Store插件使用

#### Rust后端配置
```rust
// src-tauri/src/main.rs
use tauri_plugin_store::{StoreBuilder, with_store};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            save_config,
            load_config,
            clear_storage
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn save_config(
    app_handle: tauri::AppHandle,
    config: serde_json::Value
) -> Result<(), String> {
    with_store(app_handle, |store| {
        store.set("config", config)?;
        store.save()
    }).map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_config(app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
    with_store(app_handle, |store| {
        store.get("config").cloned().unwrap_or_default()
    }).map_err(|e| e.to_string())
}
```

#### 前端存储封装
```typescript
// src/services/storageService.ts
import { load } from '@tauri-apps/plugin-store'

class StorageService {
  private store: any = null

  async init() {
    if (!this.store) {
      this.store = await load('app-storage.json', { autoSave: true })
    }
  }

  async set(key: string, value: any): Promise<void> {
    await this.init()
    await this.store.set(key, value)
  }

  async get<T>(key: string, defaultValue?: T): Promise<T> {
    await this.init()
    return await this.store.get(key) ?? defaultValue
  }

  async remove(key: string): Promise<void> {
    await this.init()
    await this.store.delete(key)
  }

  async clear(): Promise<void> {
    await this.init()
    const keys = await this.store.keys()
    for (const key of keys) {
      await this.store.delete(key)
    }
  }
}

export const storageService = new StorageService()
```

### 配置管理策略

#### 分层配置架构
```typescript
// src/types/config.ts
export interface LocalComputationConfig {
  enabled: boolean
  embedding: {
    enabled: boolean
    modelPath: string
  }
  classification: {
    enabled: boolean
    modelPath: string
  }
}

export interface AppConfig {
  localComputation: LocalComputationConfig
  vllm: {
    url: string
    apiKey?: string
    timeout: number
  }
  ui: {
    theme: 'light' | 'dark' | 'auto'
    language: string
  }
  storage: {
    autoSave: boolean
    backupInterval: number
  }
}

// 默认配置
export const DEFAULT_CONFIG: AppConfig = {
  localComputation: {
    enabled: true,
    embedding: {
      enabled: true,
      modelPath: 'models/all-MiniLM-L6-v2.onnx'
    },
    classification: {
      enabled: false,
      modelPath: ''
    }
  },
  vllm: {
    url: 'http://localhost:8000',
    timeout: 30000
  },
  ui: {
    theme: 'auto',
    language: 'zh-CN'
  },
  storage: {
    autoSave: true,
    backupInterval: 300000 // 5分钟
  }
}
```

#### 配置验证和迁移
```typescript
// src/utils/configValidator.ts
import { z } from 'zod'
import { DEFAULT_CONFIG, type AppConfig } from '@/types/config'

const ConfigSchema = z.object({
  localComputation: z.object({
    enabled: z.boolean(),
    embedding: z.object({
      enabled: z.boolean(),
      modelPath: z.string().min(1)
    }),
    classification: z.object({
      enabled: z.boolean(),
      modelPath: z.string()
    })
  }),
  vllm: z.object({
    url: z.string().url(),
    apiKey: z.string().optional(),
    timeout: z.number().min(1000)
  }),
  ui: z.object({
    theme: z.enum(['light', 'dark', 'auto']),
    language: z.string().min(2)
  }),
  storage: z.object({
    autoSave: z.boolean(),
    backupInterval: z.number().min(60000)
  })
})

export class ConfigValidator {
  static validate(config: unknown): config is AppConfig {
    try {
      ConfigSchema.parse(config)
      return true
    } catch {
      return false
    }
  }

  static migrate(oldConfig: any): AppConfig {
    // 版本迁移逻辑
    const migrated = { ...DEFAULT_CONFIG }
    
    // 合并现有配置
    if (oldConfig.localComputation) {
      migrated.localComputation = {
        ...migrated.localComputation,
        ...oldConfig.localComputation
      }
    }
    
    if (oldConfig.vllm?.url) {
      migrated.vllm.url = oldConfig.vllm.url
    }
    
    return migrated
  }
}
```

## 6. 错误处理和类型安全机制

### TypeScript错误处理模式

#### Result类型模式
```typescript
// src/types/result.ts
export type Result<T, E = Error> = 
  | { ok: true; value: T }
  | { ok: false; error: E }

export const Ok = <T>(value: T): Result<T, never> => ({ ok: true, value })
export const Err = <E>(error: E): Result<never, E> => ({ ok: false, error })

// 使用示例
export async function fetchData(): Promise<Result<string>> {
  try {
    const response = await fetch('/api/data')
    if (!response.ok) {
      return Err(new Error(`HTTP ${response.status}: ${response.statusText}`))
    }
    const data = await response.text()
    return Ok(data)
  } catch (error) {
    return Err(error instanceof Error ? error : new Error(String(error)))
  }
}

// 在组件中使用
const result = await fetchData()
if (result.ok) {
  console.log('Success:', result.value)
} else {
  console.error('Error:', result.error.message)
}
```

#### 自定义错误类型
```typescript
// src/types/errors.ts
export class AppError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly cause?: Error
  ) {
    super(message)
    this.name = 'AppError'
  }
}

export class ValidationError extends AppError {
  constructor(message: string, public readonly field?: string) {
    super('VALIDATION_ERROR', message)
    this.name = 'ValidationError'
  }
}

export class NetworkError extends AppError {
  constructor(message: string, public readonly status?: number) {
    super('NETWORK_ERROR', message)
    this.name = 'NetworkError'
  }
}

export class StorageError extends AppError {
  constructor(message: string) {
    super('STORAGE_ERROR', message)
    this.name = 'StorageError'
  }
}
```

### 全局错误处理

#### Vue错误边界
```typescript
// src/components/ErrorBoundary.vue
<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'

const props = defineProps<{
  fallback?: Component
}>()

const error = ref<Error | null>(null)
const showError = ref(false)

onErrorCaptured((err) => {
  error.value = err
  showError.value = true
  // 阻止错误继续传播
  return false
})

const reset = () => {
  error.value = null
  showError.value = false
}
</script>

<template>
  <div v-if="showError" class="error-boundary">
    <div class="error-content">
      <h3>发生错误</h3>
      <p>{{ error?.message }}</p>
      <button @click="reset">重试</button>
    </div>
  </div>
  <slot v-else />
</template>
```

#### 全局错误处理器
```typescript
// src/plugins/errorHandler.ts
import { App } from 'vue'
import { AppError } from '@/types/errors'

export default {
  install(app: App) {
    // Vue错误处理
    app.config.errorHandler = (error, instance, info) => {
      console.error('Vue Error:', error)
      console.error('Component:', instance?.$options.name || 'Unknown')
      console.error('Info:', info)
      
      // 发送到错误监控服务
      reportError(error, { type: 'vue', component: instance?.$options.name, info })
    }

    // Promise拒绝处理
    window.addEventListener('unhandledrejection', (event) => {
      console.error('Unhandled Promise Rejection:', event.reason)
      reportError(event.reason, { type: 'promise' })
      event.preventDefault()
    })

    // 全局错误处理
    window.addEventListener('error', (event) => {
      console.error('Global Error:', event.error)
      reportError(event.error, { type: 'global', filename: event.filename, lineno: event.lineno })
    })
  }
}

function reportError(error: unknown, context: Record<string, any>) {
  // 发送到错误监控服务的逻辑
  const errorData = {
    message: error instanceof Error ? error.message : String(error),
    stack: error instanceof Error ? error.stack : undefined,
    timestamp: new Date().toISOString(),
    context
  }
  
  // 这里可以集成 Sentry、LogRocket 等错误监控服务
  console.error('Reporting error:', errorData)
}
```

### 类型安全的API调用

#### 泛型API客户端
```typescript
// src/services/apiClient.ts
import { invoke } from '@tauri-apps/api/core'
import { Result, Ok, Err } from '@/types/result'
import { AppError, NetworkError } from '@/types/errors'

interface ApiRequest {
  command: string
  payload?: Record<string, any>
  timeout?: number
}

class ApiClient {
  async request<T>(request: ApiRequest): Promise<Result<T>> {
    try {
      const result = await invoke<T>(request.command, request.payload)
      return Ok(result)
    } catch (error) {
      if (error instanceof Error) {
        // 分析错误类型
        if (error.message.includes('timeout')) {
          return Err(new NetworkError('请求超时', 408))
        } else if (error.message.includes('validation')) {
          return Err(new ValidationError(error.message))
        } else {
          return Err(new AppError('API_ERROR', error.message, error))
        }
      }
      return Err(new AppError('UNKNOWN_ERROR', String(error)))
    }
  }

  // 特定API方法
  async sendMessage(message: string): Promise<Result<ChatMessage>> {
    return this.request<ChatMessage>({
      command: 'send_message',
      payload: { message },
      timeout: 30000
    })
  }

  async searchKnowledge(query: string): Promise<Result<any[]>> {
    return this.request<any[]>({
      command: 'search_knowledge',
      payload: { query },
      timeout: 15000
    })
  }
}

export const apiClient = new ApiClient()
```

### 性能优化建议

#### 1. 打包优化
```typescript
// vite.config.ts - 生产环境优化
export default defineConfig(({ mode }) => ({
  plugins: [vue()],
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['vue', 'pinia'],
          tauri: ['@tauri-apps/api']
        }
      }
    },
    chunkSizeWarningLimit: 1000
  },
  esbuild: {
    drop: mode === 'production' ? ['console', 'debugger'] : []
  }
}))
```

#### 2. 内存管理
```typescript
// src/composables/useMemoryEfficient.ts
import { onUnmounted } from 'vue'

export function useMemoryEfficient() {
  const cleanupCallbacks: (() => void)[] = []

  const addCleanup = (callback: () => void) => {
    cleanupCallbacks.push(callback)
  }

  onUnmounted(() => {
    cleanupCallbacks.forEach(callback => callback())
  })

  return { addCleanup }
}

// 使用示例
export function useLargeDataSet() {
  const { addCleanup } = useMemoryEfficient()
  const largeData = ref<any[]>([])

  // 加载大数据集
  const loadData = async () => {
    const data = await loadMassiveDataset()
    largeData.value = data
    
    // 添加清理回调
    addCleanup(() => {
      largeData.value = []
    })
  }

  return { largeData, loadData }
}
```

#### 3. 渲染优化
```vue
<!-- 虚拟滚动列表 -->
<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  items: any[]
  itemHeight: number
}>()

const visibleItems = computed(() => {
  // 实现虚拟滚动逻辑
  return props.items.slice(0, 50) // 只渲染可见项
})
</script>

<template>
  <div class="virtual-list" :style="{ height: `${itemHeight * items.length}px` }">
    <div 
      v-for="item in visibleItems" 
      :key="item.id"
      class="list-item"
      :style="{ transform: `translateY(${item.index * itemHeight}px)` }"
    >
      {{ item.content }}
    </div>
  </div>
</template>
```

## 总结

这份最佳实践指南涵盖了Tauri v2与Vue 3集成的核心方面：

1. **现代化架构**：利用Tauri v2的插件化架构和Vue 3的Composition API
2. **类型安全**：通过TypeScript和Result模式确保端到端的类型安全
3. **状态管理**：采用Pinia进行响应式状态管理，配合持久化存储
4. **错误处理**：建立完善的错误处理体系，包含自定义错误类型和全局错误边界
5. **性能优化**：实施打包优化、内存管理和渲染优化策略

这些实践能够帮助开发者构建高性能、可维护的桌面应用程序，充分发挥Tauri和Vue生态系统的优势。