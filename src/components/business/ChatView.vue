<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { sendMessage, testDatabase } from '@/api/tauri-api'

interface Message {
  id: string
  content: string
  sender: 'user' | 'ai'
  timestamp: Date
}

const messages = ref<Message[]>([])
const inputMessage = ref('')
const isLoading = ref(false)

const handleSendMessage = async () => {
  if (!inputMessage.value.trim() || isLoading.value) return
  
  const userMessage: Message = {
    id: Date.now().toString(),
    content: inputMessage.value,
    sender: 'user',
    timestamp: new Date()
  }
  
  messages.value.push(userMessage)
  const userInput = inputMessage.value
  inputMessage.value = ''
  isLoading.value = true
  
  try {
    const response = await sendMessage({
      content: userInput,
      conversationId: 'default'
    })
    
    const aiMessage: Message = {
      id: (Date.now() + 1).toString(),
      content: response.content,
      sender: 'ai',
      timestamp: new Date()
    }
    
    messages.value.push(aiMessage)
  } catch (error) {
    console.error('发送消息失败:', error)
    ElMessage.error('发送消息失败')
  } finally {
    isLoading.value = false
  }
}

const handleTestDatabase = async () => {
  try {
    const result = await testDatabase()
    ElMessage.success(result)
    
    // 添加测试结果到聊天记录
    const testMessage: Message = {
      id: Date.now().toString(),
      content: `数据库测试结果: ${result}`,
      sender: 'ai',
      timestamp: new Date()
    }
    messages.value.push(testMessage)
  } catch (error) {
    console.error('数据库测试失败:', error)
    ElMessage.error(`数据库测试失败: ${(error as Error).message}`)
    
    const errorMessage: Message = {
      id: Date.now().toString(),
      content: `数据库测试失败: ${(error as Error).message}`,
      sender: 'ai',
      timestamp: new Date()
    }
    messages.value.push(errorMessage)
  }
}

const formatTime = (date: Date): string => {
  return date.toLocaleTimeString('zh-CN', { 
    hour: '2-digit', 
    minute: '2-digit' 
  })
}

onMounted(() => {
  // 添加欢迎消息
  messages.value.push({
    id: 'welcome',
    content: '您好！我是Malou Agent，您的AI助手。请问有什么可以帮助您的吗？',
    sender: 'ai',
    timestamp: new Date()
  })
})
</script>

<template>
  <div class="chat-container">
    <div class="chat-messages">
      <div 
        v-for="message in messages" 
        :key="message.id"
        :class="['message', message.sender]"
      >
        <div class="message-content">
          <div class="message-bubble">
            {{ message.content }}
          </div>
          <div class="message-time">
            {{ formatTime(message.timestamp) }}
          </div>
        </div>
      </div>
      
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
    
    <div class="chat-input-area">
      <div class="input-actions">
        <el-button 
          type="success" 
          size="small"
          @click="handleTestDatabase"
          style="margin-right: 10px;"
        >
          测试数据库
        </el-button>
      </div>
      <el-input
        v-model="inputMessage"
        placeholder="请输入消息..."
        @keyup.enter="handleSendMessage"
        :disabled="isLoading"
      >
        <template #append>
          <el-button 
            type="primary" 
            @click="handleSendMessage"
            :loading="isLoading"
            :disabled="!inputMessage.trim()"
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

.message-time {
  font-size: 12px;
  color: #999;
  margin-top: 4px;
}

.chat-input-area {
  padding: 20px;
  background-color: white;
  border-top: 1px solid #e4e4e4;
}

.input-actions {
  margin-bottom: 10px;
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