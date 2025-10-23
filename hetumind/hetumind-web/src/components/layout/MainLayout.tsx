import React, { useState } from 'react';
import { Layout } from 'antd';
import { Outlet } from 'react-router-dom';
import Sidebar from './Sidebar';

const { Content } = Layout;

const MainLayout: React.FC = () => {
  const [collapsed, setCollapsed] = useState(false);

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sidebar collapsed={collapsed} onCollapse={setCollapsed} />
      <Layout style={{
        marginLeft: collapsed ? 0 : 0, // 保持不变，因为侧边栏不是固定的
        paddingBottom: '80px', // 为底部功能区预留空间
      }}>
        <Content style={{
          margin: '16px',
          background: 'var(--bg-primary)',
          padding: '16px',
          borderRadius: '8px',
          minHeight: 'calc(100vh - 112px)', // 调整高度以适应底部padding
          overflow: 'auto'
        }}>
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
};

export default MainLayout;