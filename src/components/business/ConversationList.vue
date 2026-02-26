<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { 
  listConversations, 
  createConversation, 
  deleteConversation,
  updateConversationTitle,
  type Conversation 
} from '@/api/tauri-api'

const props = defineProps<{
  currentId: string | null
}>()

const emit = defineEmits<{
  (e: 'select', conversation: Conversation): void
  (e: 'create', conversation: Conversation): void
}>()

const conversations = ref<Conversation[]>([])
const loading = ref(false)
const editingId = ref<string | null>(null)
const editTitle = ref('')

// Format time
const formatTime = (dateStr: string): string => {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  if (diff < 60000) return 'Just now'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
  if (diff < 604800000) return `${Math.floor(diff / 86400000)}d ago`
  return date.toLocaleDateString('en-US')
}

// Load conversations
const loadConversations = async () => {
  loading.value = true
  try {
    conversations.value = await listConversations(50, 0)
  } catch (error) {
    console.error('Failed to load conversations:', error)
    ElMessage.error('Failed to load conversations')
  } finally {
    loading.value = false
  }
}

// Create new conversation
const handleCreate = async () => {
  try {
    const title = `New Chat ${conversations.value.length + 1}`
    const conversation = await createConversation(title)
    conversations.value.unshift(conversation)
    emit('create', conversation)
    emit('select', conversation)
    ElMessage.success('Conversation created')
  } catch (error) {
    console.error('Failed to create conversation:', error)
    ElMessage.error('Failed to create conversation')
  }
}

// Select conversation
const handleSelect = (conversation: Conversation) => {
  emit('select', conversation)
}

// Start edit title
const startEdit = (conversation: Conversation, event: Event) => {
  event.stopPropagation()
  editingId.value = conversation.id
  editTitle.value = conversation.title
}

// Save title
const saveTitle = async (conversation: Conversation) => {
  if (!editTitle.value.trim()) {
    editingId.value = null
    return
  }
  
  try {
    await updateConversationTitle(conversation.id, editTitle.value.trim())
    conversation.title = editTitle.value.trim()
    ElMessage.success('Title updated')
  } catch (error) {
    console.error('Failed to update title:', error)
    ElMessage.error('Failed to update title')
  } finally {
    editingId.value = null
  }
}

// Cancel edit
const cancelEdit = () => {
  editingId.value = null
}

// Delete conversation
const handleDelete = async (conversation: Conversation, event: Event) => {
  event.stopPropagation()
  
  try {
    await ElMessageBox.confirm(
      `Delete conversation "${conversation.title}"? This cannot be undone.`,
      'Confirm Delete',
      {
        confirmButtonText: 'Delete',
        cancelButtonText: 'Cancel',
        type: 'warning',
      }
    )
    
    await deleteConversation(conversation.id)
    conversations.value = conversations.value.filter(c => c.id !== conversation.id)
    
    if (props.currentId === conversation.id && conversations.value.length > 0) {
      emit('select', conversations.value[0])
    }
    
    ElMessage.success('Conversation deleted')
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to delete conversation:', error)
      ElMessage.error('Failed to delete conversation')
    }
  }
}

// Refresh list
const refresh = () => {
  loadConversations()
}

defineExpose({
  refresh,
  loadConversations
})

onMounted(() => {
  loadConversations()
})
</script>

<template>
  <div class="conversation-list">
    <div class="list-header">
      <span class="header-title">Chats</span>
      <button 
        class="add-btn"
        @click="handleCreate"
        title="New Chat"
      >
        <svg viewBox="0 0 24 24" width="18" height="18">
          <path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </button>
    </div>
    
    <div class="list-content" v-loading="loading">
      <div 
        v-for="conversation in conversations" 
        :key="conversation.id"
        :class="['conversation-item', { active: currentId === conversation.id }]"
        @click="handleSelect(conversation)"
      >
        <div class="item-icon">
          <svg viewBox="0 0 24 24" width="18" height="18">
            <path fill="currentColor" d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H5.17L4 17.17V4h16v12z"/>
          </svg>
        </div>
        
        <div class="item-content">
          <div class="item-title" v-if="editingId !== conversation.id">
            {{ conversation.title }}
          </div>
          <input
            v-else
            v-model="editTitle"
            class="title-input"
            @blur="saveTitle(conversation)"
            @keyup.enter="saveTitle(conversation)"
            @keyup.escape="cancelEdit"
            @click.stop
            autofocus
          />
          <div class="item-meta">
            <span class="meta-count">{{ conversation.message_count }} messages</span>
            <span class="meta-time">{{ formatTime(conversation.updated_at) }}</span>
          </div>
        </div>
        
        <div class="item-actions" v-if="editingId !== conversation.id">
          <button 
            class="action-btn"
            @click="startEdit(conversation, $event)"
            title="Edit Title"
          >
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path fill="currentColor" d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
            </svg>
          </button>
          <button 
            class="action-btn danger"
            @click="handleDelete(conversation, $event)"
            title="Delete"
          >
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path fill="currentColor" d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
            </svg>
          </button>
        </div>
      </div>
      
      <div v-if="conversations.length === 0 && !loading" class="empty-state">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" width="48" height="48">
            <path fill="currentColor" d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H5.17L4 17.17V4h16v12z"/>
          </svg>
        </div>
        <p>No conversations yet</p>
        <button class="create-btn" @click="handleCreate">
          <svg viewBox="0 0 24 24" width="18" height="18">
            <path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
          </svg>
          New Chat
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.conversation-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-bg);
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
}

.header-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--color-text);
}

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--color-primary);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: white;
  transition: all 0.2s ease;
}

.add-btn:hover {
  background: #0066DD;
  transform: scale(1.05);
}

.list-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.conversation-item {
  display: flex;
  align-items: flex-start;
  padding: 12px;
  margin-bottom: 4px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s ease;
  background: transparent;
}

.conversation-item:hover {
  background: rgba(0, 0, 0, 0.04);
}

.conversation-item.active {
  background: var(--color-primary);
  box-shadow: var(--shadow-md);
}

.conversation-item.active:hover {
  background: var(--color-primary);
}

.conversation-item.active .item-title,
.conversation-item.active .meta-count,
.conversation-item.active .meta-time {
  color: white;
}

.conversation-item.active .item-icon {
  background: rgba(255, 255, 255, 0.2);
}

.item-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.06);
  margin-right: 12px;
  flex-shrink: 0;
  color: var(--color-text-secondary);
  transition: all 0.2s ease;
}

.conversation-item.active .item-icon {
  color: white;
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-title {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
  color: var(--color-text);
}

.title-input {
  width: 100%;
  padding: 4px 8px;
  border: 1px solid var(--color-primary);
  border-radius: var(--radius-sm);
  font-size: 14px;
  outline: none;
  margin-bottom: 4px;
}

.item-meta {
  display: flex;
  gap: 8px;
  font-size: 12px;
  overflow: hidden;
}

.meta-count,
.meta-time {
  color: var(--color-text-secondary);
  white-space: nowrap;
  flex-shrink: 0;
}

.meta-count {
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-actions {
  display: none;
  gap: 4px;
  margin-left: 8px;
}

.conversation-item:hover .item-actions {
  display: flex;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: rgba(0, 0, 0, 0.08);
  color: var(--color-text);
}

.action-btn.danger:hover {
  background: rgba(255, 59, 48, 0.1);
  color: var(--color-danger);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--color-text-secondary);
}

.empty-icon {
  opacity: 0.3;
  margin-bottom: 16px;
}

.empty-state p {
  margin-bottom: 20px;
  font-size: 14px;
}

.create-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: var(--color-primary);
  border: none;
  border-radius: var(--radius-md);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
}

.create-btn:hover {
  background: #0066DD;
  transform: scale(1.02);
}
</style>
