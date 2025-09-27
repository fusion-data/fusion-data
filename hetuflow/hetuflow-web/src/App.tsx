import { BrowserRouter as Router, Routes, Route, Navigate } from "react-router-dom";
import { ConfigProvider } from "antd";
import zhCN from "antd/locale/zh_CN";
import { ThemeProvider, useTheme, getAntdTheme } from "./contexts/ThemeContext";
import MainLayout from "./components/Layout/MainLayout";
import Login from "./pages/Login";
import Dashboard from "./pages/Dashboard";
import Servers from "./pages/Servers";
import Agents from "./pages/Agents";
import Jobs from "./pages/Jobs";
import Tasks from "./pages/Tasks";
import TaskInstances from "./pages/TaskInstances";
import "./App.css";

/**
 * HetuFlow 分布式作业调度系统主应用组件
 * 配置路由系统和全局主题
 */
const AppContent: React.FC = () => {
  const { currentTheme } = useTheme();

  return (
    <ConfigProvider
      locale={zhCN}
      theme={getAntdTheme(currentTheme)}
    >
      <Router>
        <Routes>
          {/* 登录页面 */}
          <Route path="/login" element={<Login />} />

          {/* 主应用布局 */}
          <Route path="/" element={<MainLayout />}>
            {/* 默认重定向到仪表板 */}
            <Route index element={<Navigate to="/dashboard" replace />} />

            {/* 仪表板 */}
            <Route path="dashboard" element={<Dashboard />} />

            {/* 服务器管理 */}
            <Route path="servers" element={<Servers />} />

            {/* 执行代理管理 */}
            <Route path="agents" element={<Agents />} />

            {/* 作业管理 */}
            <Route path="jobs" element={<Jobs />} />

            {/* 任务管理 */}
            <Route path="tasks" element={<Tasks />} />

            {/* 任务实例管理 */}
            <Route path="task-instances" element={<TaskInstances />} />
          </Route>

          {/* 未匹配路由重定向到登录页 */}
          <Route path="*" element={<Navigate to="/login" replace />} />
        </Routes>
      </Router>
    </ConfigProvider>
  );
};

function App() {
  return (
    <ThemeProvider>
      <AppContent />
    </ThemeProvider>
  );
}

export default App;
