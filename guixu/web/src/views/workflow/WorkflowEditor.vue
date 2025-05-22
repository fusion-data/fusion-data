<template>
  <div class="workflow-editor">
    <div class="editor-header">
      <tiny-button type="primary" @click="saveWorkflow">保存</tiny-button>
      <tiny-button @click="executeWorkflow">执行</tiny-button>
      <tiny-button @click="toggleActive">{{ isActive ? '禁用' : '启用' }}</tiny-button>
    </div>
    <div class="editor-content">
      <div class="node-panel">
        <tiny-collapse v-model="activeCollapse">
          <tiny-collapse-item title="触发器" name="triggers">
            <div class="node-list">
              <div v-for="node in triggerNodes" :key="node.type" class="node-item" draggable="true"
                @dragstart="onDragStart($event, node)">
                <tiny-icon :name="node.icon" />
                <span>{{ node.label }}</span>
              </div>
            </div>
          </tiny-collapse-item>
          <tiny-collapse-item title="操作" name="actions">
            <div class="node-list">
              <div v-for="node in actionNodes" :key="node.type" class="node-item" draggable="true"
                @dragstart="onDragStart($event, node)">
                <tiny-icon :name="node.icon" />
                <span>{{ node.label }}</span>
              </div>
            </div>
          </tiny-collapse-item>
          <tiny-collapse-item title="AI 编排" name="ai">
            <div class="node-list">
              <div v-for="node in aiNodes" :key="node.type" class="node-item" draggable="true"
                @dragstart="onDragStart($event, node)">
                <tiny-icon :name="node.icon" />
                <span>{{ node.label }}</span>
              </div>
            </div>
          </tiny-collapse-item>
        </tiny-collapse>
      </div>
      <div class="flow-container">
        <VueFlow v-model="elements" :default-viewport="{ x: 0, y: 0, zoom: 1.5 }" @connect="onConnect"
          @node-click="onNodeClick">
          <Background pattern-color="#aaa" :gap="8" />
          <Controls />
          <MiniMap />
        </VueFlow>
      </div>
      <div class="properties-panel" v-if="selectedNode">
        <h3>节点属性</h3>
        <tiny-form :model="nodeProperties" label-position="top">
          <tiny-form-item label="名称">
            <tiny-input v-model="nodeProperties.name" />
          </tiny-form-item>
          <tiny-form-item label="描述">
            <tiny-input type="textarea" v-model="nodeProperties.description" />
          </tiny-form-item>
          <!-- 根据节点类型显示不同的配置项 -->
        </tiny-form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { TinyButton, TinyCollapse, TinyCollapseItem, TinyForm, TinyFormItem, TinyInput } from '@opentiny/vue'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import type { Node, Edge, Connection, NodeMouseEvent } from '@vue-flow/core'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import '@vue-flow/controls/dist/style.css'
import '@vue-flow/minimap/dist/style.css'

const activeCollapse = ref(['triggers'])
const isActive = ref(false)
const selectedNode = ref<Node | null>(null)
const nodeProperties = ref({
  name: '',
  description: ''
})

const triggerNodes = [
  { type: 'manual', label: '手动触发', icon: 'icon-manual' },
  { type: 'schedule', label: '定时触发', icon: 'icon-schedule' },
  { type: 'webhook', label: 'Webhook', icon: 'icon-webhook' }
]

const actionNodes = [
  { type: 'http', label: 'HTTP 请求', icon: 'icon-http' },
  { type: 'db', label: '数据库查询', icon: 'icon-database' },
  { type: 'code', label: '代码执行', icon: 'icon-code' }
]

const aiNodes = [
  { type: 'llm', label: 'LLM 调用', icon: 'icon-ai' },
  { type: 'prompt', label: 'Prompt 模板', icon: 'icon-prompt' },
  { type: 'vector', label: '向量存储', icon: 'icon-vector' }
]

const elements = ref<(Node | Edge)[]>([])

const onDragStart = (event: DragEvent, node: any) => {
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/vueflow', JSON.stringify(node))
    event.dataTransfer.effectAllowed = 'move'
  }
}

const onConnect = (params: Connection) => {
  const edge: Edge = {
    id: `e${params.source}-${params.target}`,
    source: params.source,
    target: params.target,
    type: 'smoothstep',
    sourceHandle: params.sourceHandle,
    targetHandle: params.targetHandle
  }
  elements.value = [...elements.value, edge]
}

const onNodeClick = (event: NodeMouseEvent) => {
  selectedNode.value = event.node
  nodeProperties.value = {
    name: event.node.data?.name || '',
    description: event.node.data?.description || ''
  }
}

const saveWorkflow = () => {
  // TODO: 实现保存工作流的逻辑
  console.log('保存工作流', elements.value)
}

const executeWorkflow = () => {
  // TODO: 实现执行工作流的逻辑
  console.log('执行工作流', elements.value)
}

const toggleActive = () => {
  isActive.value = !isActive.value
}
</script>

<style scoped>
.workflow-editor {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.editor-header {
  padding: 12px;
  border-bottom: 1px solid var(--ti-common-color-line-normal);
  display: flex;
  gap: 8px;
}

.editor-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.node-panel {
  width: 250px;
  border-right: 1px solid var(--ti-common-color-line-normal);
  overflow-y: auto;
}

.flow-container {
  flex: 1;
  height: 100%;
}

.properties-panel {
  width: 300px;
  border-left: 1px solid var(--ti-common-color-line-normal);
  padding: 16px;
  overflow-y: auto;
}

.node-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px;
}

.node-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border: 1px solid var(--ti-common-color-line-normal);
  border-radius: 4px;
  cursor: move;
}

.node-item:hover {
  background-color: var(--ti-common-color-bg-light-normal);
}
</style>
