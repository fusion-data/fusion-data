import React from 'react';
import ReactDOM from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ConfigProvider } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import { ThemeProvider } from '@/contexts/ThemeContext';
import App from './App';
import '@/styles/globals.css';
import '@/styles/themes.css';

// 创建 React Query 客户端
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      refetchOnWindowFocus: false,
    },
  },
});

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <ConfigProvider locale={zhCN}>
        <ThemeProvider>
          <App />
        </ThemeProvider>
      </ConfigProvider>
    </QueryClientProvider>
  </React.StrictMode>
);