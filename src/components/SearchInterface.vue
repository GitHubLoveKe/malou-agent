// src/services/onnxService.ts
/**
 * ONNX模型服务 - 前端与Rust后端的桥接
 */

import { invoke } from '@tauri-apps/api/core'

export interface EmbeddingResult {
  vector: number[]
  processingTimeMs: number
}

export interface SearchResult {
  id: string
  title: string
  content: string
  similarity: number
}

export class OnnxService {
  private static instance: OnnxService
  private isInitialized = false

  private constructor() {}

  static getInstance(): OnnxService {
    if (!OnnxService.instance) {
      OnnxService.instance = new OnnxService()
    }
    return OnnxService.instance
  }

  /**
   * 初始化服务
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) return
    
    try {
      // 预热模型
      await this.warmupModel()
      this.isInitialized = true
      console.log('ONNX服务初始化完成')
    } catch (error) {
      console.error('ONNX服务初始化失败:', error)
      throw error
    }
  }

  /**
   * 文本嵌入生成
   */
  async embedText(text: string): Promise<EmbeddingResult> {
    if (!this.isInitialized) {
      await this.initialize()
    }

    const startTime = performance.now()
    
    try {
      const vector = await invoke<number[]>('embed_text', { text })
      const processingTimeMs = performance.now() - startTime
      
      return {
        vector,
        processingTimeMs
      }
    } catch (error) {
      console.error('文本嵌入失败:', error)
      throw new Error(`嵌入生成失败: ${error}`)
    }
  }

  /**
   * 批量文本嵌入
   */
  async embedTextBatch(texts: string[]): Promise<EmbeddingResult[]> {
    const results: EmbeddingResult[] = []
    
    // 分批处理以避免内存压力
    const batchSize = 8
    for (let i = 0; i < texts.length; i += batchSize) {
      const batch = texts.slice(i, i + batchSize)
      const batchResults = await Promise.all(
        batch.map(text => this.embedText(text))
      )
      results.push(...batchResults)
    }
    
    return results
  }

  /**
   * 语义搜索
   */
  async semanticSearch(query: string, limit: number = 10): Promise<SearchResult[]> {
    try {
      // 生成查询向量
      const queryEmbedding = await this.embedText(query)
      
      // 调用后端搜索
      const results = await invoke<SearchResult[]>('semantic_search', {
        queryVector: queryEmbedding.vector,
        limit
      })
      
      return results
    } catch (error) {
      console.error('语义搜索失败:', error)
      throw new Error(`搜索失败: ${error}`)
    }
  }

  /**
   * 预热模型
   */
  private async warmupModel(): Promise<void> {
    // 使用测试文本预加载模型
    await this.embedText("warmup text for model loading")
  }

  /**
   * 获取服务状态
   */
  async getStatus(): Promise<{ 
    isReady: boolean; 
    modelLoaded: boolean; 
    version?: string 
  }> {
    try {
      const status = await invoke<any>('get_onnx_status')
      return {
        isReady: status.is_ready,
        modelLoaded: status.model_loaded,
        version: status.version
      }
    } catch (error) {
      return {
        isReady: false,
        modelLoaded: false
      }
    }
  }
}

// src/services/databaseService.ts
/**
 * 数据库服务 - 统一的数据访问接口
 */

import { invoke } from '@tauri-apps/api/core'

export interface Document {
  id: string
  title: string
  content: string
  createdAt: string
  updatedAt: string
  metadata?: Record<string, any>
}

export class DatabaseService {
  private static instance: DatabaseService

  private constructor() {}

  static getInstance(): DatabaseService {
    if (!DatabaseService.instance) {
      DatabaseService.instance = new DatabaseService()
    }
    return DatabaseService.instance
  }

  /**
   * 添加文档
   */
  async addDocument(document: Omit<Document, 'id' | 'createdAt' | 'updatedAt'>): Promise<Document> {
    try {
      const result = await invoke<Document>('add_document', {
        title: document.title,
        content: document.content,
        metadata: document.metadata
      })
      return result
    } catch (error) {
      console.error('添加文档失败:', error)
      throw new Error(`文档添加失败: ${error}`)
    }
  }

  /**
   * 搜索文档
   */
  async searchDocuments(query: string, limit: number = 20): Promise<Document[]> {
    try {
      const results = await invoke<Document[]>('search_documents', {
        query,
        limit
      })
      return results
    } catch (error) {
      console.error('文档搜索失败:', error)
      throw new Error(`文档搜索失败: ${error}`)
    }
  }

  /**
   * 获取文档详情
   */
  async getDocument(id: string): Promise<Document | null> {
    try {
      const document = await invoke<Document | null>('get_document', { id })
      return document
    } catch (error) {
      console.error('获取文档失败:', error)
      return null
    }
  }

  /**
   * 删除文档
   */
  async deleteDocument(id: string): Promise<boolean> {
    try {
      await invoke<void>('delete_document', { id })
      return true
    } catch (error) {
      console.error('删除文档失败:', error)
      return false
    }
  }

  /**
   * 获取统计信息
   */
  async getStats(): Promise<{ 
    totalDocuments: number; 
    totalSizeMb: number 
  }> {
    try {
      const stats = await invoke<any>('get_database_stats')
      return {
        totalDocuments: stats.total_documents,
        totalSizeMb: stats.total_size_mb
      }
    } catch (error) {
      console.error('获取统计信息失败:', error)
      return { totalDocuments: 0, totalSizeMb: 0 }
    }
  }
}

// src/composables/useVectorSearch.ts
/**
 * 向量搜索组合式函数
 */

import { ref, computed } from 'vue'
import { OnnxService, type SearchResult } from '@/services/onnxService'
import { DatabaseService, type Document } from '@/services/databaseService'

export interface SearchState {
  query: string
  results: SearchResult[]
  isLoading: boolean
  error: string | null
}

export function useVectorSearch() {
  const onnxService = OnnxService.getInstance()
  const dbService = DatabaseService.getInstance()
  
  const state = ref<SearchState>({
    query: '',
    results: [],
    isLoading: false,
    error: null
  })

  const hasResults = computed(() => state.value.results.length > 0)
  const resultCount = computed(() => state.value.results.length)

  /**
   * 执行语义搜索
   */
  const search = async (query: string, limit: number = 10) => {
    if (!query.trim()) return

    state.value.query = query
    state.value.isLoading = true
    state.value.error = null

    try {
      // 并行执行两种搜索：语义搜索 + 传统文本搜索
      const [semanticResults, textResults] = await Promise.all([
        onnxService.semanticSearch(query, Math.floor(limit / 2)),
        dbService.searchDocuments(query, Math.floor(limit / 2))
      ])

      // 合并和去重结果
      const mergedResults = mergeSearchResults(semanticResults, textResults)
      
      state.value.results = mergedResults.slice(0, limit)
    } catch (error) {
      state.value.error = error instanceof Error ? error.message : '搜索失败'
      console.error('搜索错误:', error)
    } finally {
      state.value.isLoading = false
    }
  }

  /**
   * 清空搜索结果
   */
  const clear = () => {
    state.value.query = ''
    state.value.results = []
    state.value.error = null
  }

  /**
   * 重新搜索
   */
  const refresh = () => {
    if (state.value.query) {
      search(state.value.query)
    }
  }

  return {
    // 状态
    state: readonly(state),
    hasResults,
    resultCount,
    
    // 方法
    search,
    clear,
    refresh
  }
}

/**
 * 合并不同来源的搜索结果
 */
function mergeSearchResults(
  semantic: SearchResult[], 
  text: Document[]
): SearchResult[] {
  const resultMap = new Map<string, SearchResult>()
  
  // 处理语义搜索结果
  semantic.forEach(result => {
    resultMap.set(result.id, {
      ...result,
      similarity: result.similarity * 0.7 // 权重调整
    })
  })
  
  // 处理文本搜索结果
  text.forEach(doc => {
    const existing = resultMap.get(doc.id)
    if (existing) {
      // 如果已存在，提高相似度分数
      resultMap.set(doc.id, {
        ...existing,
        similarity: Math.min(1.0, existing.similarity + 0.3)
      })
    } else {
      // 新增文本搜索结果
      resultMap.set(doc.id, {
        id: doc.id,
        title: doc.title,
        content: doc.content,
        similarity: 0.4 // 基础分数
      })
    }
  })
  
  // 按相似度排序
  return Array.from(resultMap.values())
    .sort((a, b) => b.similarity - a.similarity)
}

// src/components/SearchInterface.vue
<template>
  <div class="search-interface">
    <!-- 搜索输入区域 -->
    <div class="search-header">
      <div class="search-input-container">
        <input
          v-model="searchQuery"
          @keyup.enter="handleSearch"
          placeholder="请输入搜索内容..."
          class="search-input"
          :disabled="isLoading"
        />
        <button 
          @click="handleSearch" 
          :disabled="isLoading || !searchQuery.trim()"
          class="search-button"
        >
          <span v-if="isLoading">搜索中...</span>
          <span v-else>搜索</span>
        </button>
      </div>
      
      <!-- 状态显示 -->
      <div class="search-status" v-if="hasResults || error">
        <span v-if="hasResults">
          找到 {{ resultCount }} 个结果
        </span>
        <span v-else-if="error" class="error-text">
          {{ error }}
        </span>
      </div>
    </div>

    <!-- 搜索结果 -->
    <div class="search-results" v-if="hasResults">
      <div 
        v-for="result in state.results" 
        :key="result.id"
        class="result-item"
      >
        <div class="result-header">
          <h3 class="result-title">{{ result.title }}</h3>
          <span class="similarity-score">
            相似度: {{ (result.similarity * 100).toFixed(1) }}%
          </span>
        </div>
        <p class="result-content">{{ truncateContent(result.content) }}</p>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!isLoading && searchQuery" class="empty-state">
      <p>未找到相关结果</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useVectorSearch } from '@/composables/useVectorSearch'

const props = defineProps<{
  autoSearch?: boolean
  debounceMs?: number
}>()

const emit = defineEmits<{
  (e: 'search', query: string): void
  (e: 'results', results: any[]): void
}>()

// 使用搜索组合式函数
const { state, search, clear } = useVectorSearch()

// 本地状态
const searchQuery = ref('')
const debouncedQuery = ref('')

// 计算属性
const isLoading = computed(() => state.value.isLoading)
const hasResults = computed(() => state.value.results.length > 0)
const resultCount = computed(() => state.value.results.length)
const error = computed(() => state.value.error)

// 处理搜索
const handleSearch = () => {
  if (searchQuery.value.trim()) {
    search(searchQuery.value)
    emit('search', searchQuery.value)
    emit('results', state.value.results)
  }
}

// 内容截断
const truncateContent = (content: string, maxLength: number = 200): string => {
  if (content.length <= maxLength) return content
  return content.substring(0, maxLength) + '...'
}

// 自动搜索功能
watch(searchQuery, (newQuery) => {
  if (props.autoSearch && newQuery.trim()) {
    // 防抖处理
    clearTimeout(debounceTimer.value)
    debounceTimer.value = setTimeout(() => {
      debouncedQuery.value = newQuery
      handleSearch()
    }, props.debounceMs || 300)
  }
})

// 清理定时器
const debounceTimer = ref<NodeJS.Timeout>()

// 组件卸载时清理
onUnmounted(() => {
  if (debounceTimer.value) {
    clearTimeout(debounceTimer.value)
  }
})
</script>

<style scoped>
.search-interface {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
}

.search-header {
  margin-bottom: 20px;
}

.search-input-container {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
}

.search-input {
  flex: 1;
  padding: 12px;
  border: 2px solid #ddd;
  border-radius: 6px;
  font-size: 16px;
}

.search-input:focus {
  outline: none;
  border-color: #007bff;
}

.search-button {
  padding: 12px 24px;
  background-color: #007bff;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
}

.search-button:hover:not(:disabled) {
  background-color: #0056b3;
}

.search-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.search-status {
  color: #666;
  font-size: 14px;
}

.error-text {
  color: #dc3545;
}

.search-results {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.result-item {
  border: 1px solid #eee;
  border-radius: 8px;
  padding: 15px;
  background-color: #fafafa;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.result-title {
  margin: 0;
  color: #333;
  font-size: 18px;
}

.similarity-score {
  background-color: #e9ecef;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  color: #495057;
}

.result-content {
  margin: 0;
  color: #666;
  line-height: 1.5;
}

.empty-state {
  text-align: center;
  color: #666;
  padding: 40px;
}
</style>