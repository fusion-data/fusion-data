import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider } from 'antd';
import { useTheme } from '@/contexts/ThemeContext';
import MainLayout from '@/components/layout/MainLayout';
import { AuthGuard, LazyWrapper } from '@/router';
import { getPageTitle } from '@/router/utils';

// 页面组件导入
const LoginPage = React.lazy(() => import('@/pages/Login'));
const DashboardPage = React.lazy(() => import('@/pages/Dashboard'));
const WorkflowListPage = React.lazy(() => import('@/pages/Workflows'));
const WorkflowEditorPage = React.lazy(() => import('@/pages/Workflows/Editor'));
const WorkflowExecutionsPage = React.lazy(() => import('@/pages/Workflows/Executions'));
const WorkflowDetailPage = React.lazy(() => import('@/pages/Workflows/Detail'));
const AgentListPage = React.lazy(() => import('@/pages/Agents'));
const AgentEditorPage = React.lazy(() => import('@/pages/Agents/Editor'));
const AgentTestPage = React.lazy(() => import('@/pages/Agents/Test'));
const SettingsPage = React.lazy(() => import('@/pages/Settings'));
const GeneralSettingsPage = React.lazy(() => import('@/pages/Settings/General'));
const ModelSettingsPage = React.lazy(() => import('@/pages/Settings/Models'));
const IntegrationSettingsPage = React.lazy(() => import('@/pages/Settings/Integrations'));
const CredentialsSettingsPage = React.lazy(() => import('@/pages/Settings/Credentials'));
const NotFoundPage = React.lazy(() => import('@/pages/NotFound'));

const App: React.FC = () => {
  const { antdTheme } = useTheme();

  // 动态设置页面标题
  React.useEffect(() => {
    const handleRouteChange = () => {
      const title = getPageTitle(window.location.pathname);
      document.title = title;
    };

    // 初始设置
    handleRouteChange();

    // 监听路由变化
    window.addEventListener('popstate', handleRouteChange);

    return () => {
      window.removeEventListener('popstate', handleRouteChange);
    };
  }, []);

  return (
    <ConfigProvider theme={antdTheme}>
      <Router>
        <div className="app">
          <Routes>
            {/* 登录页面 */}
            <Route path="/login" element={
              <LazyWrapper>
                <LoginPage />
              </LazyWrapper>
            } />

            {/* 主要布局 */}
            <Route path="/" element={<AuthGuard><MainLayout /></AuthGuard>}>
              <Route index element={<Navigate to="/dashboard" replace />} />

              {/* 仪表板 */}
              <Route path="dashboard" element={
                <LazyWrapper>
                  <DashboardPage />
                </LazyWrapper>
              } />

              {/* 工作流 */}
              <Route path="workflows" element={
                <LazyWrapper>
                  <WorkflowListPage />
                </LazyWrapper>
              } />
              <Route path="workflows/:id" element={
                <LazyWrapper>
                  <WorkflowEditorPage />
                </LazyWrapper>
              } />
              <Route path="workflows/:id/executions" element={
                <LazyWrapper>
                  <WorkflowExecutionsPage />
                </LazyWrapper>
              } />
              <Route path="workflows/:id/detail" element={
                <LazyWrapper>
                  <WorkflowDetailPage />
                </LazyWrapper>
              } />

              {/* AI 智能体 */}
              <Route path="agents" element={
                <LazyWrapper>
                  <AgentListPage />
                </LazyWrapper>
              } />
              <Route path="agents/new" element={
                <LazyWrapper>
                  <AgentEditorPage />
                </LazyWrapper>
              } />
              <Route path="agents/:id" element={
                <LazyWrapper>
                  <AgentEditorPage />
                </LazyWrapper>
              } />
              <Route path="agents/:id/test" element={
                <LazyWrapper>
                  <AgentTestPage />
                </LazyWrapper>
              } />

              {/* 设置 */}
              <Route path="settings" element={
                <LazyWrapper>
                  <SettingsPage />
                </LazyWrapper>
              }>
                <Route index element={
                  <LazyWrapper>
                    <GeneralSettingsPage />
                  </LazyWrapper>
                } />
                <Route path="models" element={
                  <LazyWrapper>
                    <ModelSettingsPage />
                  </LazyWrapper>
                } />
                <Route path="integrations" element={
                  <LazyWrapper>
                    <IntegrationSettingsPage />
                  </LazyWrapper>
                } />
                <Route path="credentials" element={
                  <LazyWrapper>
                    <CredentialsSettingsPage />
                  </LazyWrapper>
                } />
              </Route>
            </Route>

            {/* 404 页面 */}
            <Route path="*" element={
              <LazyWrapper>
                <NotFoundPage />
              </LazyWrapper>
            } />
          </Routes>
        </div>
      </Router>
    </ConfigProvider>
  );
};

export default App;