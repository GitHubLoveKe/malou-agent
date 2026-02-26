<script setup lang="ts">
import { ref } from 'vue'
import ChatView from './components/business/ChatView.vue'
import ConversationList from './components/business/ConversationList.vue'
import DocumentManager from './components/business/DocumentManager.vue'
import ModelSettings from './components/settings/ModelSettings.vue'
import type { Conversation } from './api/tauri-api'

const activeTab = ref('chat')
const currentConversation = ref<Conversation | null>(null)
const conversationListRef = ref<InstanceType<typeof ConversationList> | null>(null)

// 选择会话
const handleSelectConversation = (conversation: Conversation) => {
  currentConversation.value = conversation
}

// 创建会话后自动选择
const handleCreateConversation = (conversation: Conversation) => {
  currentConversation.value = conversation
}

// 会话更新后刷新列表
const handleConversationUpdated = () => {
  conversationListRef.value?.refresh()
}

// 打开设置页面
const handleOpenSettings = () => {
  activeTab.value = 'settings'
}
</script>

<template>
  <div class="app-container">
    <el-container style="height: 100vh;">
      <el-header class="app-header">
        <div class="header-title">Malou Agent Desktop</div>
        <el-tabs v-model="activeTab" type="border-card">
          <el-tab-pane label="聊天" name="chat"></el-tab-pane>
          <el-tab-pane label="知识库" name="knowledge"></el-tab-pane>
          <el-tab-pane label="设置" name="settings"></el-tab-pane>
        </el-tabs>
      </el-header>
      
      <el-main class="app-main">
        <!-- 聊天页面：带侧边栏 -->
        <template v-if="activeTab === 'chat'">
          <div class="chat-layout">
            <div class="chat-sidebar">
              <ConversationList 
                ref="conversationListRef"
                :current-id="currentConversation?.id ?? null"
                @select="handleSelectConversation"
                @create="handleCreateConversation"
              />
            </div>
            <div class="chat-main">
              <ChatView 
                :conversation="currentConversation"
                @conversation-updated="handleConversationUpdated"
                @open-settings="handleOpenSettings"
              />
            </div>
          </div>
        </template>
        
        <!-- 知识库页面 -->
        <DocumentManager v-else-if="activeTab === 'knowledge'" />
        
        <!-- 设置页面 -->
        <ModelSettings v-else-if="activeTab === 'settings'" />
      </el-main>
    </el-container>
  </div>
</template>

<style scoped>
.app-container {
  height: 100vh;
  overflow: hidden;
}

.app-header {
  padding: 0;
  background-color: #f5f5f5;
  border-bottom: 1px solid #e4e4e4;
}

.header-title {
  padding: 10px 20px;
  font-size: 18px;
  font-weight: bold;
  background-color: #409eff;
  color: white;
}

.app-main {
  padding: 0;
  overflow: hidden;
}

.chat-layout {
  display: flex;
  height: 100%;
}

.chat-sidebar {
  width: 280px;
  flex-shrink: 0;
  height: 100%;
  overflow: hidden;
}

.chat-main {
  flex: 1;
  height: 100%;
  overflow: hidden;
}
</style>
