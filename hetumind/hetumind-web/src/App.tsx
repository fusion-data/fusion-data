import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider } from 'antd';
import { useTheme } from '@/contexts/ThemeContext';
import MainLayout from '@/components/layout/MainLayout';
import LoginPage from '@/pages/Login';
import DashboardPage from '@/pages/Dashboard';
import WorkflowListPage from '@/pages/Workflows';
import WorkflowEditorPage from '@/pages/Workflows/Editor';
import WorkflowDetailPage from '@/pages/Workflows/Detail';
import AgentListPage from '@/pages/Agents';
import AgentEditorPage from '@/pages/Agents/Editor';
import SettingsPage from '@/pages/Settings';
import NotFoundPage from '@/pages/NotFound';

// 临时占位组件，稍后会被实际实现替换
const WorkflowExecutionsPage = React.lazy(() => import('@/pages/Workflows/Executions'));
const AgentTestPage = React.lazy(() => import('@/pages/Agents/Test'));
const GeneralSettingsPage = React.lazy(() => import('@/pages/Settings/General'));
const ModelSettingsPage = React.lazy(() => import('@/pages/Settings/Models'));
const IntegrationSettingsPage = React.lazy(() => import('@/pages/Settings/Integrations'));
const CredentialsSettingsPage = React.lazy(() => import('@/pages/Settings/Credentials'));

const App: React.FC = () => {
  const { antdTheme } = useTheme();

  return (
    <ConfigProvider theme={antdTheme}>
      <Router>
        <div className="app">
          <Routes>
            {/* 登录页面 */}
            <Route path="/login" element={<LoginPage />} />

            {/* 主要布局 */}
            <Route path="/" element={<MainLayout />}>
              <Route index element={<Navigate to="/dashboard" replace />} />

              {/* 仪表板 */}
              <Route path="dashboard" element={<DashboardPage />} />

              {/* 工作流 */}
              <Route path="workflows" element={<WorkflowListPage />} />
              <Route path="workflows/:id" element={<WorkflowEditorPage />} />
              <Route path="workflows/:id/executions" element={
                <React.Suspense fallback={<div>加载中...</div>}>
                  <WorkflowExecutionsPage />
                </React.Suspense>
              } />
              <Route path="workflows/:id/detail" element={<WorkflowDetailPage />} />

              {/* AI 智能体 */}
              <Route path="agents" element={<AgentListPage />} />
              <Route path="agents/new" element={<AgentEditorPage />} />
              <Route path="agents/:id" element={<AgentEditorPage />} />
              <Route path="agents/:id/test" element={
                <React.Suspense fallback={<div>加载中...</div>}>
                  <AgentTestPage />
                </React.Suspense>
              } />

              {/* 设置 */}
              <Route path="settings" element={<SettingsPage />}>
                <Route index element={
                  <React.Suspense fallback={<div>加载中...</div>}>
                    <GeneralSettingsPage />
                  </React.Suspense>
                } />
                <Route path="models" element={
                  <React.Suspense fallback={<div>加载中...</div>}>
                    <ModelSettingsPage />
                  </React.Suspense>
                } />
                <Route path="integrations" element={
                  <React.Suspense fallback={<div>加载中...</div>}>
                    <IntegrationSettingsPage />
                  </React.Suspense>
                } />
                <Route path="credentials" element={
                  <React.Suspense fallback={<div>加载中...</div>}>
                    <CredentialsSettingsPage />
                  </React.Suspense>
                } />
              </Route>
            </Route>

            {/* 404 页面 */}
            <Route path="*" element={<NotFoundPage />} />
          </Routes>
        </div>
      </Router>
    </ConfigProvider>
  );
};

export default App;