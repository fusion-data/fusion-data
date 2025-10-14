import React from 'react';
import { Navigate } from 'react-router-dom';
import { Spin } from 'antd';

// 通用加载组件
const LoadingFallback: React.FC = () => (
  <div style={{
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    height: '200px'
  }}>
    <Spin size="large" />
  </div>
);

// 认证守卫组件
interface AuthGuardProps {
  children: React.ReactNode;
}

const AuthGuard: React.FC<AuthGuardProps> = ({ children }) => {
  // TODO: 实现真实的认证逻辑
  // 这里暂时返回 true，后续会实现真实的认证检查
  const isAuthenticated = true;

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
};

// 路由懒加载包装器
export const LazyWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <React.Suspense fallback={<LoadingFallback />}>
    {children}
  </React.Suspense>
);

// 公共路由（不需要认证）
export const publicRoutes = [
  {
    path: '/login',
    component: React.lazy(() => import('@/pages/Login')),
    title: '登录',
  },
];

// 受保护的路由（需要认证）
export const protectedRoutes = [
  // 仪表板
  {
    path: '/dashboard',
    component: React.lazy(() => import('@/pages/Dashboard')),
    title: '仪表板',
  },

  // 工作流
  {
    path: '/workflows',
    component: React.lazy(() => import('@/pages/Workflows')),
    title: '工作流列表',
  },
  {
    path: '/workflows/:id',
    component: React.lazy(() => import('@/pages/Workflows/Editor')),
    title: '工作流编辑器',
  },
  {
    path: '/workflows/:id/executions',
    component: React.lazy(() => import('@/pages/Workflows/Executions')),
    title: '工作流执行记录',
  },
  {
    path: '/workflows/:id/detail',
    component: React.lazy(() => import('@/pages/Workflows/Detail')),
    title: '工作流详情',
  },

  // AI 智能体
  {
    path: '/agents',
    component: React.lazy(() => import('@/pages/Agents')),
    title: 'AI 智能体列表',
  },
  {
    path: '/agents/new',
    component: React.lazy(() => import('@/pages/Agents/Editor')),
    title: '创建 AI 智能体',
  },
  {
    path: '/agents/:id',
    component: React.lazy(() => import('@/pages/Agents/Editor')),
    title: '编辑 AI 智能体',
  },
  {
    path: '/agents/:id/test',
    component: React.lazy(() => import('@/pages/Agents/Test')),
    title: '测试 AI 智能体',
  },

  // 设置页面（嵌套路由）
  {
    path: '/settings/*',
    component: React.lazy(() => import('@/pages/Settings')),
    title: '设置',
    children: [
      {
        path: '/settings',
        component: React.lazy(() => import('@/pages/Settings/General')),
        title: '通用设置',
        index: true,
      },
      {
        path: '/settings/models',
        component: React.lazy(() => import('@/pages/Settings/Models')),
        title: '模型设置',
      },
      {
        path: '/settings/integrations',
        component: React.lazy(() => import('@/pages/Settings/Integrations')),
        title: '集成设置',
      },
      {
        path: '/settings/credentials',
        component: React.lazy(() => import('@/pages/Settings/Credentials')),
        title: '认证设置',
      },
    ],
  },
];

// 404 页面
export const notFoundRoute = {
  path: '*',
  component: React.lazy(() => import('@/pages/NotFound')),
  title: '页面未找到',
};

// 重定向路由
export const redirectRoutes = [
  {
    from: '/',
    to: '/dashboard',
  },
];

export { AuthGuard };