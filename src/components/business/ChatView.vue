<script setup lang="ts">
import { ref, watch, nextTick, onMounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { 
  sendMessage, 
  getConversationMessages,
  clearConversationMessages,
  testDatabase,
  getAppConfig,
  updateModelSelection,
  type Conversation,
  type Message as ApiMessage
} from '@/api/tauri-api'
import { logger } from '@/utils/logger'
import type { AppConfig } from '@/config'

interface DisplayMessage {
  id: string
  content: string
  sender: 'user' | 'ai'
  timestamp: Date
  tokens?: number
}

const props = defineProps<{
  conversation: Conversation | null
}>()

const emit = defineEmits<{
  (e: 'conversation-updated'): void
  (e: 'open-settings'): void
}>()

const messages = ref<DisplayMessage[]>([])
const inputMessage = ref('')
const isLoading = ref(false)
const messagesContainer = ref<HTMLElement | null>(null)

// 模型选择相关
const appConfig = ref<AppConfig | null>(null)
const showModelSelector = ref(false)

// 可用模型列表
const availableModels = computed(() => {
  if (!appConfig.value) return []
  
  const models: Array<{ id: string; name: string; type: 'remote' | 'local' }> = []
  
  // 添加启用的远程模型
  appConfig.value.remoteModels
    .filter(model => model.enabled)
    .forEach(model => {
      models.push({
        id: model.id,
        name: `${model.name} (${model.provider})`,
        type: 'remote'
      })
    })
  
  // 添加启用的本地模型
  if (appConfig.value.localModels.enabled) {
    appConfig.value.localModels.models
      .filter(model => model.enabled)
      .forEach(model => {
        models.push({
          id: model.id,
          name: `${model.name} (本地)`,
          type: 'local'
        })
      })
  }
  
  return models
})

// 当前选中的模型
const currentModel = computed(() => {
  if (!appConfig.value) return null
  const modelId = appConfig.value.modelSelector.currentModel
  return availableModels.value.find(m => m.id === modelId) || availableModels.value[0]
})

// 加载配置
const loadConfig = async () => {
  try {
    appConfig.value = await getAppConfig()
  } catch (error) {
    console.error('加载配置失败:', error)
  }
}

// 切换模型
const handleModelChange = async (modelId: string) => {
  try {
    await updateModelSelection(modelId)
    if (appConfig.value) {
      appConfig.value.modelSelector.currentModel = modelId
    }
    ElMessage.success('模型切换成功')
    showModelSelector.value = false
  } catch (error) {
    console.error('切换模型失败:', error)
    ElMessage.error('切换模型失败')
  }
}

// 将 API 消息转换为显示消息
const convertMessage = (msg: ApiMessage): DisplayMessage => ({
  id: msg.id,
  content: msg.content,
  sender: msg.role === 'user' ? 'user' : 'ai',
  timestamp: new Date(msg.created_at),
  tokens: msg.total_tokens
})

// 加载会话消息
const loadMessages = async () => {
  if (!props.conversation) {
    messages.value = []
    return
  }
  
  try {
    const apiMessages = await getConversationMessages(props.conversation.id, 100, 0)
    messages.value = apiMessages.map(convertMessage)
    
    // 如果没有消息，添加欢迎消息
    if (messages.value.length === 0) {
      messages.value.push({
        id: 'welcome',
        content: '您好！我是Malou Agent，您的AI助手。请问有什么可以帮助您的吗？',
        sender: 'ai',
        timestamp: new Date()
      })
    }
    
    // 滚动到底部
    await nextTick()
    scrollToBottom()
  } catch (error) {
    console.error('加载消息失败:', error)
    ElMessage.error('加载消息失败')
  }
}

// 监听会话变化
watch(() => props.conversation, () => {
  loadMessages()
}, { immediate: true })

onMounted(() => {
  loadConfig()
})

// 滚动到底部
const scrollToBottom = () => {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

// 发送消息
const handleSendMessage = async () => {
  if (!inputMessage.value.trim() || isLoading.value) return
  if (!props.conversation) {
    ElMessage.warning('请先选择或创建一个会话')
    return
  }
  
  const userContent = inputMessage.value.trim()
  
  // 添加用户消息到界面
  const userMessage: DisplayMessage = {
    id: `temp-${Date.now()}`,
    content: userContent,
    sender: 'user',
    timestamp: new Date()
  }
  messages.value.push(userMessage)
  
  inputMessage.value = ''
  isLoading.value = true
  
  await nextTick()
  scrollToBottom()
  
  try {
    const response = await sendMessage({
      conversation_id: props.conversation.id,
      content: userContent
    })
    
    // 更新用户消息 ID
    userMessage.id = `user-${response.id}`
    
    // 添加 AI 回复
    const aiMessage: DisplayMessage = {
      id: response.id,
      content: response.content,
      sender: 'ai',
      timestamp: new Date(response.timestamp),
      tokens: response.total_tokens
    }
    messages.value.push(aiMessage)
    
    // 通知父组件更新会话信息
    emit('conversation-updated')
    
    logger.info('消息发送成功', 'chat')
  } catch (error) {
    console.error('发送消息失败:', error)
    logger.error('发送消息失败', 'chat')
    ElMessage.error('发送消息失败')
    
    // 移除临时用户消息
    messages.value = messages.value.filter(m => m.id !== userMessage.id)
  } finally {
    isLoading.value = false
    await nextTick()
    scrollToBottom()
  }
}

// 测试数据库
const handleTestDatabase = async () => {
  try {
    const result = await testDatabase()
    logger.info(`数据库测试成功: ${result}`, 'chat')
    ElMessage.success(result)
    
    const testMessage: DisplayMessage = {
      id: `test-${Date.now()}`,
      content: `数据库测试结果: ${result}`,
      sender: 'ai',
      timestamp: new Date()
    }
    messages.value.push(testMessage)
  } catch (error) {
    console.error('数据库测试失败:', error)
    logger.error(`数据库测试失败: ${(error as Error).message}`, 'chat')
    ElMessage.error(`数据库测试失败: ${(error as Error).message}`)
  }
}

// 清空聊天记录
const handleClearChatHistory = async () => {
  if (!props.conversation) {
    ElMessage.warning('请先选择一个会话')
    return
  }
  
  try {
    logger.info('开始清空聊天记录', 'chat')
    
    await clearConversationMessages(props.conversation.id)
    
    // 清空界面消息，添加确认消息
    messages.value = [{
      id: `clear-${Date.now()}`,
      content: '聊天记录已清空',
      sender: 'ai',
      timestamp: new Date()
    }]
    
    emit('conversation-updated')
    
    logger.info('聊天记录清空完成', 'chat')
    ElMessage.success('聊天记录已清空')
  } catch (error) {
    console.error('清空聊天记录失败:', error)
    logger.error(`清空聊天记录失败: ${(error as Error).message}`, 'chat')
    ElMessage.error('清空聊天记录失败')
  }
}

// 格式化时间
const formatTime = (date: Date): string => {
  return date.toLocaleTimeString('zh-CN', { 
    hour: '2-digit', 
    minute: '2-digit' 
  })
}
</script>

<template>
  <div class="chat-container">
    <!-- 会话标题栏 -->
    <div class="chat-header" v-if="conversation">
      <div class="header-main">
        <div class="header-info">
          <div class="header-title">{{ conversation.title }}</div>
          <div class="header-meta">
            <span>{{ conversation.message_count }} 条消息</span>
            <span v-if="conversation.total_tokens > 0">| {{ conversation.total_tokens }} tokens</span>
          </div>
        </div>
        <!-- 模型选择器 -->
        <div class="model-selector">
          <el-dropdown trigger="click" @command="handleModelChange">
            <el-button type="primary" size="small" class="model-select-btn">
              <el-icon><Cpu /></el-icon>
              <span class="model-name">{{ currentModel?.name || '选择模型' }}</span>
              <el-icon class="el-icon--right"><ArrowDown /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item 
                  v-for="model in availableModels" 
                  :key="model.id"
                  :command="model.id"
                  :class="{ 'is-active': model.id === currentModel?.id }"
                >
                  <el-icon v-if="model.type === 'remote'"><Connection /></el-icon>
                  <el-icon v-else><Cpu /></el-icon>
                  <span>{{ model.name }}</span>
                  <el-icon v-if="model.id === currentModel?.id" class="check-icon"><Check /></el-icon>
                </el-dropdown-item>
                <el-dropdown-item divided command="" @click="$emit('open-settings')">
                  <el-icon><Setting /></el-icon>
                  <span>管理模型...</span>
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
    </div>
    <div class="chat-header empty" v-else>
      <div class="header-title">请选择或创建会话</div>
    </div>
    
    <!-- 消息列表 -->
    <div class="chat-messages" ref="messagesContainer">
      <div 
        v-for="message in messages" 
        :key="message.id"
        :class="['message', message.sender]"
      >
        <div class="message-content">
          <div class="message-bubble">
            {{ message.content }}
          </div>
          <div class="message-meta">
            <span class="message-time">{{ formatTime(message.timestamp) }}</span>
            <span class="message-tokens" v-if="message.tokens">{{ message.tokens }} tokens</span>
          </div>
        </div>
      </div>
      
      <!-- 加载指示器 -->
      <div v-if="isLoading" class="message ai">
        <div class="message-content">
          <div class="message-bubble typing-indicator">
            <span></span>
            <span></span>
            <span></span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 输入区域 -->
    <div class="chat-input-area">
      <div class="input-actions">
        <el-button 
          type="success" 
          size="small"
          @click="handleTestDatabase"
        >
          测试数据库
        </el-button>
        <el-button 
          type="danger" 
          size="small"
          @click="handleClearChatHistory"
          :disabled="!conversation"
        >
          清空聊天记录
        </el-button>
      </div>
      <el-input
        v-model="inputMessage"
        placeholder="请输入消息..."
        @keyup.enter="handleSendMessage"
        :disabled="isLoading || !conversation"
      >
        <template #append>
          <el-button 
            type="primary" 
            @click="handleSendMessage"
            :loading="isLoading"
            :disabled="!inputMessage.trim() || !conversation"
          >
            发送
          </el-button>
        </template>
      </el-input>
    </div>
  </div>
</template>

<style scoped>
.chat-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #f5f5f5;
}

.chat-header {
  padding: 16px 20px;
  background-color: #fff;
  border-bottom: 1px solid #e4e4e4;
}

.chat-header.empty {
  color: #909399;
}

.header-main {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-info {
  flex: 1;
}

.header-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.header-meta {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.model-selector {
  margin-left: 16px;
}

.model-select-btn {
  display: flex;
  align-items: center;
  gap: 6px;
}

.model-name {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.check-icon {
  margin-left: auto;
  color: #67c23a;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.message {
  display: flex;
  width: 100%;
}

.message.user {
  justify-content: flex-end;
}

.message.ai {
  justify-content: flex-start;
}

.message-content {
  max-width: 70%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.message.user .message-content {
  align-items: flex-end;
}

.message-bubble {
  padding: 12px 16px;
  border-radius: 18px;
  word-wrap: break-word;
  line-height: 1.4;
  white-space: pre-wrap;
}

.message.user .message-bubble {
  background-color: #409eff;
  color: white;
  border-bottom-right-radius: 4px;
}

.message.ai .message-bubble {
  background-color: white;
  color: #333;
  border: 1px solid #e4e4e4;
  border-bottom-left-radius: 4px;
}

.message-meta {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.message-time {
  font-size: 12px;
  color: #999;
}

.message-tokens {
  font-size: 12px;
  color: #67c23a;
}

.chat-input-area {
  padding: 16px 20px;
  background-color: white;
  border-top: 1px solid #e4e4e4;
}

.input-actions {
  margin-bottom: 12px;
  display: flex;
  gap: 8px;
}

.typing-indicator {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 12px 16px;
}

.typing-indicator span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #999;
  animation: typing 1.4s infinite ease-in-out;
}

.typing-indicator span:nth-child(1) { animation-delay: -0.32s; }
.typing-indicator span:nth-child(2) { animation-delay: -0.16s; }

@keyframes typing {
  0%, 80%, 100% { transform: scale(0.8); opacity: 0.5; }
  40% { transform: scale(1); opacity: 1; }
}
</style>
