<template>
  <div class="layout">
    <div class="header">
      <div class="header-content">
        <h1>GuixuFlow</h1>
        <tiny-user-head :value="{
          userid: '1',
          username: 'Admin',
          nickname: '管理员',
          avatar: 'https://opentiny.design/static/images/avatar.png'
        }" />
      </div>
    </div>
    <div class="container">
      <div class="aside">
        <tiny-menu :data="menuData" />
      </div>
      <div class="main">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { TinyMenu, TinyUserHead } from '@opentiny/vue'

const menuData = ref([
  {
    title: '仪表盘',
    id: 'dashboard',
    icon: 'icon-dashboard'
  },
  {
    title: '工作流',
    id: 'workflows',
    icon: 'icon-workflow',
    children: [
      {
        title: '工作流列表',
        id: 'workflow-list'
      },
      {
        title: '工作流编辑器',
        id: 'workflow-editor'
      }
    ]
  },
  {
    title: '执行历史',
    id: 'executions',
    icon: 'icon-history'
  },
  {
    title: '节点市场',
    id: 'marketplace',
    icon: 'icon-marketplace'
  },
  {
    title: '设置',
    id: 'settings',
    icon: 'icon-setting'
  }
])
</script>

<style scoped>
.layout {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.header {
  height: 60px;
  border-bottom: 1px solid var(--ti-common-color-line-normal);
  background-color: var(--ti-common-color-bg-white-normal);
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 20px;
  height: 100%;
}

.header-content h1 {
  margin: 0;
  font-size: 20px;
  color: var(--ti-common-color-text-primary);
}

.container {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.aside {
  width: 200px;
  border-right: 1px solid var(--ti-common-color-line-normal);
  background-color: var(--ti-common-color-bg-white-normal);
}

.main {
  flex: 1;
  overflow: auto;
  background-color: var(--ti-common-color-bg-white-normal);
}
</style>
