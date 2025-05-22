<template>
  <div class="workflow-list">
    <div class="list-header">
      <tiny-input v-model="searchText" placeholder="搜索工作流" clearable>
        <template #prefix>
          <tiny-icon-search />
        </template>
      </tiny-input>
      <tiny-button type="primary" @click="createWorkflow">新建工作流</tiny-button>
    </div>
    <tiny-table :data="workflows" :columns="columns">
      <template #status="{ row }">
        <tiny-tag :type="row.status === 'active' ? 'success' : 'info'">
          {{ row.status === 'active' ? '已启用' : '已禁用' }}
        </tiny-tag>
      </template>
      <template #operations="{ row }">
        <tiny-button-group>
          <tiny-button @click="editWorkflow(row)">编辑</tiny-button>
          <tiny-button @click="toggleStatus(row)">
            {{ row.status === 'active' ? '禁用' : '启用' }}
          </tiny-button>
          <tiny-button @click="deleteWorkflow(row)" type="danger">删除</tiny-button>
        </tiny-button-group>
      </template>
    </tiny-table>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { TinyButton, TinyButtonGroup, TinyInput, TinyTable, TinyTag } from '@opentiny/vue'

const router = useRouter()
const searchText = ref('')

const columns = [
  { title: '名称', field: 'name' },
  { title: '描述', field: 'description' },
  { title: '状态', field: 'status', slot: 'status' },
  { title: '创建时间', field: 'createdAt' },
  { title: '更新时间', field: 'updatedAt' },
  { title: '操作', field: 'operations', slot: 'operations' }
]

// 模拟数据
const workflows = ref([
  {
    id: 1,
    name: '示例工作流',
    description: '这是一个示例工作流',
    status: 'active',
    createdAt: '2024-03-20 10:00:00',
    updatedAt: '2024-03-20 10:00:00'
  }
])

const createWorkflow = () => {
  router.push('/workflow/editor')
}

const editWorkflow = (workflow: any) => {
  router.push(`/workflow/editor/${workflow.id}`)
}

const toggleStatus = (workflow: any) => {
  workflow.status = workflow.status === 'active' ? 'inactive' : 'active'
}

const deleteWorkflow = (workflow: any) => {
  // TODO: 实现删除工作流的逻辑
  console.log('删除工作流', workflow)
}
</script>

<style scoped>
.workflow-list {
  padding: 16px;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.tiny-input {
  width: 300px;
}
</style>
