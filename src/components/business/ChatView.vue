<script setup lang="ts">
import { ref, watch, nextTick, onMounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { 
  sendMessage, 
  getConversationMessages,
  clearConversationMessages,
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

// Model selection
const appConfig = ref<AppConfig | null>(null)
const showModelSelector = ref(false)

// Available models
const availableModels = computed(() => {
  if (!appConfig.value) return []
  
  const models: Array<{ id: string; name: string; type: 'remote' | 'local' }> = []
  
  appConfig.value.remoteModels
    .filter(model => model.enabled)
    .forEach(model => {
      models.push({
        id: model.id,
        name: `${model.name} (${model.provider})`,
        type: 'remote'
      })
    })
  
  if (appConfig.value.localModels.enabled) {
    appConfig.value.localModels.models
      .filter(model => model.enabled)
      .forEach(model => {
        models.push({
          id: model.id,
          name: `${model.name} (Local)`,
          type: 'local'
        })
      })
  }
  
  return models
})

// Current model
const currentModel = computed(() => {
  if (!appConfig.value) return null
  const modelId = appConfig.value.modelSelector.currentModel
  return availableModels.value.find(m => m.id === modelId) || availableModels.value[0]
})

// Load config
const loadConfig = async () => {
  try {
    appConfig.value = await getAppConfig()
  } catch (error) {
    console.error('Failed to load config:', error)
  }
}

// Switch model
const handleModelChange = async (modelId: string) => {
  try {
    await updateModelSelection(modelId)
    if (appConfig.value) {
      appConfig.value.modelSelector.currentModel = modelId
    }
    ElMessage.success('Model switched')
    showModelSelector.value = false
  } catch (error) {
    console.error('Failed to switch model:', error)
    ElMessage.error('Failed to switch model')
  }
}

// Convert API message to display message
const convertMessage = (msg: ApiMessage): DisplayMessage => ({
  id: msg.id,
  content: msg.content,
  sender: msg.role === 'user' ? 'user' : 'ai',
  timestamp: new Date(msg.created_at),
  tokens: msg.total_tokens
})

// Load conversation messages
const loadMessages = async () => {
  if (!props.conversation) {
    messages.value = []
    return
  }
  
  try {
    const apiMessages = await getConversationMessages(props.conversation.id, 100, 0)
    messages.value = apiMessages.map(convertMessage)
    
    if (messages.value.length === 0) {
      messages.value.push({
        id: 'welcome',
        content: 'Hi! I\'m Malou Agent, your AI assistant. How can I help you today?',
        sender: 'ai',
        timestamp: new Date()
      })
    }
    
    await nextTick()
    scrollToBottom()
  } catch (error) {
    console.error('Failed to load messages:', error)
    ElMessage.error('Failed to load messages')
  }
}

// Watch conversation changes
watch(() => props.conversation, () => {
  loadMessages()
}, { immediate: true })

onMounted(() => {
  loadConfig()
})

// Scroll to bottom
const scrollToBottom = () => {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

// Send message
const handleSendMessage = async () => {
  if (!inputMessage.value.trim() || isLoading.value) return
  if (!props.conversation) {
    ElMessage.warning('Please select or create a conversation')
    return
  }
  
  const userContent = inputMessage.value.trim()
  
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
    
    userMessage.id = `user-${response.id}`
    
    const aiMessage: DisplayMessage = {
      id: response.id,
      content: response.content,
      sender: 'ai',
      timestamp: new Date(response.timestamp),
      tokens: response.total_tokens
    }
    messages.value.push(aiMessage)
    
    emit('conversation-updated')
    
    logger.info('Message sent successfully', 'chat')
  } catch (error) {
    console.error('Failed to send message:', error)
    logger.error('Failed to send message', 'chat')
    ElMessage.error('Failed to send message')
    
    messages.value = messages.value.filter(m => m.id !== userMessage.id)
  } finally {
    isLoading.value = false
    await nextTick()
    scrollToBottom()
  }
}

// Clear chat history
const handleClearChatHistory = async () => {
  if (!props.conversation) {
    ElMessage.warning('Please select a conversation')
    return
  }
  
  try {
    logger.info('Clearing chat history', 'chat')
    
    await clearConversationMessages(props.conversation.id)
    
    messages.value = [{
      id: `clear-${Date.now()}`,
      content: 'Chat history cleared',
      sender: 'ai',
      timestamp: new Date()
    }]
    
    emit('conversation-updated')
    
    logger.info('Chat history cleared', 'chat')
    ElMessage.success('Chat history cleared')
  } catch (error) {
    console.error('Failed to clear chat history:', error)
    logger.error(`Failed to clear chat history: ${(error as Error).message}`, 'chat')
    ElMessage.error('Failed to clear chat history')
  }
}

// Format time
const formatTime = (date: Date): string => {
  return date.toLocaleTimeString('zh-CN', { 
    hour: '2-digit', 
    minute: '2-digit' 
  })
}
</script>

<template>
  <div class="chat-container">
    <!-- Chat Header -->
    <div class="chat-header" v-if="conversation">
      <div class="header-main">
        <div class="header-info">
          <div class="header-title">{{ conversation.title }}</div>
          <div class="header-meta">
            <span>{{ conversation.message_count }} messages</span>
            <span v-if="conversation.total_tokens > 0">| {{ conversation.total_tokens }} tokens</span>
          </div>
        </div>
        <div class="header-actions">
          <el-dropdown trigger="click" @command="handleModelChange">
            <button class="model-select-btn">
              <span class="model-name">{{ currentModel?.name || 'Select Model' }}</span>
              <svg class="dropdown-arrow" viewBox="0 0 24 24" width="16" height="16">
                <path fill="currentColor" d="M7 10l5 5 5-5z"/>
              </svg>
            </button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item 
                  v-for="model in availableModels" 
                  :key="model.id"
                  :command="model.id"
                  :class="{ 'is-active': model.id === currentModel?.id }"
                >
                  <span>{{ model.name }}</span>
                </el-dropdown-item>
                <el-dropdown-item divided command="" @click="$emit('open-settings')">
                  <span>Manage Models...</span>
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          <button class="icon-btn" @click="handleClearChatHistory" title="Clear Chat">
            <svg viewBox="0 0 24 24" width="18" height="18">
              <path fill="currentColor" d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
    <div class="chat-header empty" v-else>
      <div class="header-title">Select or create a conversation</div>
    </div>
    
    <!-- Messages -->
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
      
      <!-- Loading -->
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
    
    <!-- Input Area -->
    <div class="chat-input-area">
      <div class="input-wrapper">
        <input
          v-model="inputMessage"
          class="message-input"
          placeholder="Type a message..."
          @keyup.enter="handleSendMessage"
          :disabled="isLoading || !conversation"
        />
        <button 
          class="send-btn"
          @click="handleSendMessage"
          :disabled="!inputMessage.trim() || !conversation"
          :class="{ loading: isLoading }"
        >
          <svg v-if="!isLoading" viewBox="0 0 24 24" width="20" height="20">
            <path fill="currentColor" d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
          </svg>
          <span v-else class="loading-spinner"></span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.chat-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-bg-secondary);
}

.chat-header {
  padding: 16px 24px;
  background-color: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.chat-header.empty {
  color: var(--color-text-secondary);
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
  color: var(--color-text);
}

.header-meta {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 4px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.model-select-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text);
  transition: all 0.2s ease;
  font-family: inherit;
}

.model-select-btn:hover {
  background: var(--color-bg-secondary);
  border-color: var(--color-primary);
}

.dropdown-arrow {
  opacity: 0.5;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all 0.2s ease;
}

.icon-btn:hover {
  background: var(--color-bg);
  color: var(--color-danger);
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.message {
  display: flex;
  width: 100%;
  animation: messageIn 0.3s ease;
}

@keyframes messageIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
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
  border-radius: var(--radius-lg);
  word-wrap: break-word;
  line-height: 1.5;
  white-space: pre-wrap;
  font-size: 14px;
}

.message.user .message-bubble {
  background: var(--color-user-bubble);
  color: white;
  border-bottom-right-radius: 4px;
  box-shadow: var(--shadow-sm);
}

.message.ai .message-bubble {
  background: var(--color-ai-bubble);
  color: var(--color-text);
  border-bottom-left-radius: 4px;
}

.message-meta {
  display: flex;
  gap: 8px;
  margin-top: 6px;
  padding: 0 4px;
}

.message-time {
  font-size: 11px;
  color: var(--color-text-secondary);
}

.message-tokens {
  font-size: 11px;
  color: var(--color-success);
}

.chat-input-area {
  padding: 16px 24px 20px;
  background: var(--color-bg-secondary);
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

.input-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  padding: 4px;
  transition: all 0.2s ease;
}

.input-wrapper:focus-within {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

.message-input {
  flex: 1;
  border: none;
  background: transparent;
  padding: 12px 16px;
  font-size: 14px;
  color: var(--color-text);
  outline: none;
  font-family: inherit;
}

.message-input::placeholder {
  color: var(--color-text-secondary);
}

.message-input:disabled {
  opacity: 0.5;
}

.send-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: var(--color-primary);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  color: white;
  transition: all 0.2s ease;
}

.send-btn:hover:not(:disabled) {
  background: #0066DD;
  transform: scale(1.05);
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.send-btn.loading {
  background: var(--color-primary);
}

.loading-spinner {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
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
  background-color: var(--color-text-secondary);
  animation: typing 1.4s infinite ease-in-out;
}

.typing-indicator span:nth-child(1) { animation-delay: -0.32s; }
.typing-indicator span:nth-child(2) { animation-delay: -0.16s; }

@keyframes typing {
  0%, 80%, 100% { transform: scale(0.8); opacity: 0.5; }
  40% { transform: scale(1); opacity: 1; }
}
</style>
