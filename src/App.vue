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

// Tab items
const tabItems = [
  { key: 'chat', label: 'Chat' },
  { key: 'knowledge', label: 'Knowledge' },
  { key: 'settings', label: 'Settings' }
]

// Select conversation
const handleSelectConversation = (conversation: Conversation) => {
  currentConversation.value = conversation
}

// Auto select after create
const handleCreateConversation = (conversation: Conversation) => {
  currentConversation.value = conversation
}

// Refresh list after update
const handleConversationUpdated = () => {
  conversationListRef.value?.refresh()
}

// Open settings
const handleOpenSettings = () => {
  activeTab.value = 'settings'
}
</script>

<template>
  <div class="app-container">
    <!-- Header -->
    <header class="app-header">
      <div class="header-content">
        <div class="header-brand">
          <span class="brand-icon">M</span>
          <span class="brand-name">Malou Agent</span>
        </div>
        <nav class="header-nav">
          <button 
            v-for="item in tabItems" 
            :key="item.key"
            :class="['nav-item', { active: activeTab === item.key }]"
            @click="activeTab = item.key"
          >
            {{ item.label }}
          </button>
        </nav>
      </div>
    </header>

    <!-- Main Content -->
    <main class="app-main">
      <!-- Chat Page with Sidebar -->
      <template v-if="activeTab === 'chat'">
        <div class="chat-layout">
          <aside class="chat-sidebar">
            <ConversationList 
              ref="conversationListRef"
              :current-id="currentConversation?.id ?? null"
              @select="handleSelectConversation"
              @create="handleCreateConversation"
            />
          </aside>
          <section class="chat-main">
            <ChatView 
              :conversation="currentConversation"
              @conversation-updated="handleConversationUpdated"
              @open-settings="handleOpenSettings"
            />
          </section>
        </div>
      </template>
      
      <!-- Knowledge Page -->
      <DocumentManager v-else-if="activeTab === 'knowledge'" />
      
      <!-- Settings Page -->
      <ModelSettings v-else-if="activeTab === 'settings'" />
    </main>
  </div>
</template>

<style>
/* Global Styles */
:root {
  --color-primary: #007AFF;
  --color-primary-light: #5AC8FA;
  --color-bg: #F5F5F7;
  --color-bg-secondary: #FFFFFF;
  --color-text: #1D1D1F;
  --color-text-secondary: #86868B;
  --color-border: rgba(0, 0, 0, 0.08);
  --color-success: #34C759;
  --color-danger: #FF3B30;
  --color-user-bubble: linear-gradient(135deg, #007AFF, #5AC8FA);
  --color-ai-bubble: #E5E5EA;
  
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --radius-xl: 20px;
  
  --shadow-sm: 0 2px 8px rgba(0, 0, 0, 0.06);
  --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.08);
  --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.12);
  
  --font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', 'Helvetica Neue', Arial, sans-serif;
  
  --header-height: 52px;
  --sidebar-width: 280px;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: var(--font-family);
  background-color: var(--color-bg);
  color: var(--color-text);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Scrollbar Styling */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.15);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.25);
}

/* Override Element Plus Primary Color */
.el-button--primary {
  --el-button-bg-color: var(--color-primary);
  --el-button-border-color: var(--color-primary);
  --el-button-hover-bg-color: #0066DD;
  --el-button-hover-border-color: #0066DD;
}

.el-tabs__item.is-active {
  color: var(--color-primary);
}

.el-tabs__active-bar {
  background-color: var(--color-primary);
}
</style>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background-color: var(--color-bg);
}

/* Header */
.app-header {
  height: var(--header-height);
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
  z-index: 100;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  max-width: 100%;
  padding: 0 20px;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-icon {
  width: 28px;
  height: 28px;
  background: var(--color-primary);
  color: white;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
}

.brand-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.header-nav {
  display: flex;
  gap: 4px;
  background: rgba(0, 0, 0, 0.04);
  padding: 4px;
  border-radius: var(--radius-md);
}

.nav-item {
  padding: 8px 20px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all 0.2s ease;
  font-family: inherit;
}

.nav-item:hover {
  color: var(--color-text);
}

.nav-item.active {
  background: var(--color-bg-secondary);
  color: var(--color-primary);
  box-shadow: var(--shadow-sm);
}

/* Main Content */
.app-main {
  flex: 1;
  overflow: hidden;
  display: flex;
}

/* Chat Layout */
.chat-layout {
  display: flex;
  width: 100%;
  height: 100%;
}

.chat-sidebar {
  width: var(--sidebar-width);
  flex-shrink: 0;
  background: var(--color-bg);
  border-right: 1px solid var(--color-border);
  overflow: hidden;
}

.chat-main {
  flex: 1;
  background: var(--color-bg-secondary);
  overflow: hidden;
}
</style>
