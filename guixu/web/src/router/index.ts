import { createRouter, createWebHistory } from 'vue-router'
import MainLayout from '../layouts/MainLayout.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: MainLayout,
      children: [
        {
          path: '',
          redirect: '/workflow',
        },
        {
          path: 'workflow',
          name: 'WorkflowList',
          component: () => import('../views/workflow/WorkflowList.vue'),
        },
        {
          path: 'workflow/editor/:id?',
          name: 'WorkflowEditor',
          component: () => import('../views/workflow/WorkflowEditor.vue'),
        },
      ],
    },
  ],
})

export default router
