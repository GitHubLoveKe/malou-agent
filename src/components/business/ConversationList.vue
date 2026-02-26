<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Delete, Edit, ChatLineRound } from '@element-plus/icons-vue'
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

// 格式化时间
const formatTime = (dateStr: string): string => {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  // 小于 1 分钟
  if (diff < 60000) {
    return '刚刚'
  }
  // 小于 1 小时
  if (diff < 3600000) {
    return `${Math.floor(diff / 60000)} 分钟前`
  }
  // 小于 24 小时
  if (diff < 86400000) {
    return `${Math.floor(diff / 3600000)} 小时前`
  }
  // 小于 7 天
  if (diff < 604800000) {
    return `${Math.floor(diff / 86400000)} 天前`
  }
  // 其他
  return date.toLocaleDateString('zh-CN')
}

// 加载会话列表
const loadConversations = async () => {
  loading.value = true
  try {
    conversations.value = await listConversations(50, 0)
  } catch (error) {
    console.error('加载会话列表失败:', error)
    ElMessage.error('加载会话列表失败')
  } finally {
    loading.value = false
  }
}

// 创建新会话
const handleCreate = async () => {
  try {
    const title = `新会话 ${conversations.value.length + 1}`
    const conversation = await createConversation(title)
    conversations.value.unshift(conversation)
    emit('create', conversation)
    emit('select', conversation)
    ElMessage.success('会话创建成功')
  } catch (error) {
    console.error('创建会话失败:', error)
    ElMessage.error('创建会话失败')
  }
}

// 选择会话
const handleSelect = (conversation: Conversation) => {
  emit('select', conversation)
}

// 开始编辑标题
const startEdit = (conversation: Conversation, event: Event) => {
  event.stopPropagation()
  editingId.value = conversation.id
  editTitle.value = conversation.title
}

// 保存标题
const saveTitle = async (conversation: Conversation) => {
  if (!editTitle.value.trim()) {
    editingId.value = null
    return
  }
  
  try {
    await updateConversationTitle(conversation.id, editTitle.value.trim())
    conversation.title = editTitle.value.trim()
    ElMessage.success('标题已更新')
  } catch (error) {
    console.error('更新标题失败:', error)
    ElMessage.error('更新标题失败')
  } finally {
    editingId.value = null
  }
}

// 取消编辑
const cancelEdit = () => {
  editingId.value = null
}

// 删除会话
const handleDelete = async (conversation: Conversation, event: Event) => {
  event.stopPropagation()
  
  try {
    await ElMessageBox.confirm(
      `确定要删除会话 "${conversation.title}" 吗？此操作不可恢复。`,
      '删除确认',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    
    await deleteConversation(conversation.id)
    conversations.value = conversations.value.filter(c => c.id !== conversation.id)
    
    // 如果删除的是当前选中的会话，选择第一个会话
    if (props.currentId === conversation.id && conversations.value.length > 0) {
      emit('select', conversations.value[0])
    }
    
    ElMessage.success('会话已删除')
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除会话失败:', error)
      ElMessage.error('删除会话失败')
    }
  }
}

// 刷新列表
const refresh = () => {
  loadConversations()
}

// 暴露方法给父组件
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
      <span class="header-title">会话列表</span>
      <el-button 
        type="primary" 
        :icon="Plus" 
        circle 
        size="small"
        @click="handleCreate"
        title="新建会话"
      />
    </div>
    
    <div class="list-content" v-loading="loading">
      <div 
        v-for="conversation in conversations" 
        :key="conversation.id"
        :class="['conversation-item', { active: currentId === conversation.id }]"
        @click="handleSelect(conversation)"
      >
        <div class="item-icon">
          <el-icon><ChatLineRound /></el-icon>
        </div>
        
        <div class="item-content">
          <div class="item-title" v-if="editingId !== conversation.id">
            {{ conversation.title }}
          </div>
          <el-input
            v-else
            v-model="editTitle"
            size="small"
            @blur="saveTitle(conversation)"
            @keyup.enter="saveTitle(conversation)"
            @keyup.escape="cancelEdit"
            @click.stop
            autofocus
          />
          <div class="item-meta">
            <span class="meta-count">{{ conversation.message_count }} 条消息</span>
            <span class="meta-time">{{ formatTime(conversation.updated_at) }}</span>
          </div>
        </div>
        
        <div class="item-actions" v-if="editingId !== conversation.id">
          <el-button 
            :icon="Edit" 
            circle 
            size="small"
            @click="startEdit(conversation, $event)"
            title="编辑标题"
          />
          <el-button 
            :icon="Delete" 
            circle 
            size="small"
            type="danger"
            @click="handleDelete(conversation, $event)"
            title="删除会话"
          />
        </div>
      </div>
      
      <div v-if="conversations.length === 0 && !loading" class="empty-state">
        <p>暂无会话</p>
        <el-button type="primary" @click="handleCreate">
          <el-icon><Plus /></el-icon>
          新建会话
        </el-button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.conversation-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #f5f7fa;
  border-right: 1px solid #e4e7ed;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #e4e7ed;
  background-color: #fff;
}

.header-title {
  font-weight: 600;
  font-size: 14px;
  color: #303133;
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
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  background-color: #fff;
}

.conversation-item:hover {
  background-color: #ecf5ff;
}

.conversation-item.active {
  background-color: #409eff;
  color: #fff;
}

.conversation-item.active .meta-count,
.conversation-item.active .meta-time {
  color: rgba(255, 255, 255, 0.8);
}

.item-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background-color: #f0f0f0;
  margin-right: 12px;
  flex-shrink: 0;
}

.conversation-item.active .item-icon {
  background-color: rgba(255, 255, 255, 0.2);
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
}

.item-meta {
  display: flex;
  gap: 8px;
  font-size: 12px;
  overflow: hidden;
}

.meta-count,
.meta-time {
  color: #909399;
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

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: #909399;
}

.empty-state p {
  margin-bottom: 16px;
}
</style>
