<template>
  <div class="document-manager">
    <div class="toolbar">
      <el-button type="primary" @click="showCreateDialog = true">
        新建文档
      </el-button>
      <el-button @click="refreshDocuments">
        刷新
      </el-button>
      <el-input
        v-model="searchQuery"
        placeholder="搜索文档..."
        style="width: 300px; margin-left: 20px;"
        clearable
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
    </div>

    <el-table
      :data="filteredDocuments"
      style="width: 100%"
      v-loading="loading"
      @selection-change="handleSelectionChange"
    >
      <el-table-column type="selection" width="55" />
      <el-table-column prop="title" label="标题" width="200" />
      <el-table-column prop="content" label="内容摘要" min-width="300">
        <template #default="scope">
          <div class="content-preview">{{ getContentPreview(scope.row.content) }}</div>
        </template>
      </el-table-column>
      <el-table-column prop="createdAt" label="创建时间" width="180">
        <template #default="scope">
          {{ formatDate(scope.row.createdAt) }}
        </template>
      </el-table-column>
      <el-table-column prop="updatedAt" label="更新时间" width="180">
        <template #default="scope">
          {{ formatDate(scope.row.updatedAt) }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="200">
        <template #default="scope">
          <el-button size="small" @click="editDocument(scope.row)">编辑</el-button>
          <el-button size="small" type="danger" @click="deleteDocument(scope.row.id)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 新建/编辑文档对话框 -->
    <el-dialog
      v-model="showCreateDialog"
      :title="editingDocument ? '编辑文档' : '新建文档'"
      width="600px"
    >
      <el-form :model="documentForm" label-width="80px">
        <el-form-item label="标题">
          <el-input v-model="documentForm.title" placeholder="请输入文档标题" />
        </el-form-item>
        <el-form-item label="内容">
          <el-input
            v-model="documentForm.content"
            type="textarea"
            :rows="8"
            placeholder="请输入文档内容"
          />
        </el-form-item>
        <el-form-item label="元数据">
          <el-input
            v-model="metadataJson"
            type="textarea"
            :rows="4"
            placeholder='{"key": "value"}'
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showCreateDialog = false">取消</el-button>
          <el-button type="primary" @click="saveDocument">保存</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import { 
  listDocuments, 
  createDocument, 
  updateDocument, 
  deleteDocument as deleteDocApi,
  Document,
  DocumentCreate,
  DocumentUpdate
} from '@/api/tauri-api'

const documents = ref<Document[]>([])
const loading = ref(false)
const searchQuery = ref('')
const showCreateDialog = ref(false)
const editingDocument = ref<Document | null>(null)
const selectedDocuments = ref<Document[]>([])

const documentForm = ref({
  title: '',
  content: '',
  metadata: {}
})

const metadataJson = ref('')

const filteredDocuments = computed(() => {
  if (!searchQuery.value) return documents.value
  const query = searchQuery.value.toLowerCase()
  return documents.value.filter(doc => 
    doc.title.toLowerCase().includes(query) || 
    doc.content.toLowerCase().includes(query)
  )
})

const getContentPreview = (content: string) => {
  return content.length > 100 ? content.substring(0, 100) + '...' : content
}

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-CN')
}

const refreshDocuments = async () => {
  loading.value = true
  try {
    documents.value = await listDocuments(100, 0)
  } catch (error) {
    ElMessage.error('获取文档列表失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

const handleSelectionChange = (selection: Document[]) => {
  selectedDocuments.value = selection
}

const editDocument = (doc: Document) => {
  editingDocument.value = doc
  documentForm.value = {
    title: doc.title,
    content: doc.content,
    metadata: doc.metadata || {}
  }
  metadataJson.value = JSON.stringify(doc.metadata || {}, null, 2)
  showCreateDialog.value = true
}

const saveDocument = async () => {
  try {
    let metadata = {}
    if (metadataJson.value) {
      metadata = JSON.parse(metadataJson.value)
    }
    
    if (editingDocument.value) {
      // 更新文档
      const updateData: DocumentUpdate = {
        title: documentForm.value.title || undefined,
        content: documentForm.value.content || undefined,
        metadata: Object.keys(metadata).length > 0 ? metadata : undefined
      }
      
      const result = await updateDocument(editingDocument.value.id, updateData)
      if (result) {
        ElMessage.success('文档更新成功')
        refreshDocuments()
      }
    } else {
      // 创建新文档
      const createData: DocumentCreate = {
        title: documentForm.value.title,
        content: documentForm.value.content,
        metadata: Object.keys(metadata).length > 0 ? metadata : undefined
      }
      
      await createDocument(createData)
      ElMessage.success('文档创建成功')
      refreshDocuments()
    }
    
    showCreateDialog.value = false
    resetForm()
  } catch (error) {
    ElMessage.error('保存文档失败: ' + (error as Error).message)
    console.error(error)
  }
}

const deleteDocument = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要删除这个文档吗？', '确认删除', {
      type: 'warning'
    })
    
    const result = await deleteDocApi(id)
    if (result) {
      ElMessage.success('文档删除成功')
      refreshDocuments()
    }
  } catch (error) {
    if ((error as Error).message !== 'cancel') {
      ElMessage.error('删除文档失败')
      console.error(error)
    }
  }
}

const resetForm = () => {
  documentForm.value = { title: '', content: '', metadata: {} }
  metadataJson.value = ''
  editingDocument.value = null
}

onMounted(() => {
  refreshDocuments()
})
</script>

<style scoped>
.document-manager {
  padding: 20px;
}

.toolbar {
  margin-bottom: 20px;
  display: flex;
  align-items: center;
}

.content-preview {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>