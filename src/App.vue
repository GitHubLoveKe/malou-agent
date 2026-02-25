<script setup lang="ts">
import { ref } from 'vue'
import ChatView from './components/business/ChatView.vue'
import DocumentManager from './components/business/DocumentManager.vue'

const activeTab = ref('chat')
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
        <ChatView v-if="activeTab === 'chat'" />
        <DocumentManager v-else-if="activeTab === 'knowledge'" />
        <div v-else-if="activeTab === 'settings'">
          <h2>设置功能待实现</h2>
        </div>
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
</style>