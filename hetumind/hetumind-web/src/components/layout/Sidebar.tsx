import React from 'react';
import { Layout, Menu } from 'antd';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  DashboardOutlined,
  NodeIndexOutlined,
  RobotOutlined,
  SettingOutlined,
} from '@ant-design/icons';

const { Sider } = Layout;

interface SidebarProps {
  collapsed: boolean;
  onCollapse: (collapsed: boolean) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ collapsed, onCollapse }) => {
  const navigate = useNavigate();
  const location = useLocation();

  const menuItems = [
    {
      key: '/dashboard',
      icon: <DashboardOutlined />,
      label: '仪表板',
    },
    {
      key: '/workflows',
      icon: <NodeIndexOutlined />,
      label: '工作流',
      children: [
        {
          key: '/workflows',
          label: '工作流列表',
        },
      ],
    },
    {
      key: '/agents',
      icon: <RobotOutlined />,
      label: 'AI 智能体',
      children: [
        {
          key: '/agents',
          label: '智能体列表',
        },
      ],
    },
    {
      key: '/settings',
      icon: <SettingOutlined />,
      label: '设置',
    },
  ];

  const handleMenuClick = ({ key }: { key: string }) => {
    navigate(key);
  };

  const getSelectedKeys = () => {
    const path = location.pathname;

    // 精确匹配
    if (path === '/dashboard') return ['/dashboard'];
    if (path === '/workflows') return ['/workflows'];
    if (path === '/agents') return ['/agents'];
    if (path.startsWith('/settings')) return ['/settings'];

    // 工作流相关页面
    if (path.startsWith('/workflows/')) return ['/workflows'];

    // 智能体相关页面
    if (path.startsWith('/agents/')) return ['/agents'];

    return [];
  };

  return (
    <Sider
      collapsible
      collapsed={collapsed}
      onCollapse={onCollapse}
      style={{
        background: 'var(--bg-primary)',
        borderRight: '1px solid var(--border-primary)',
      }}
      width={240}
    >
      <div
        style={{
          height: '64px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: collapsed ? 'center' : 'flex-start',
          padding: collapsed ? 0 : '0 24px',
          borderBottom: '1px solid var(--border-secondary)',
        }}
      >
        <h3 style={{
          margin: 0,
          color: 'var(--color-primary)',
          fontSize: collapsed ? '16px' : '18px',
          fontWeight: 'bold'
        }}>
          {collapsed ? 'HM' : 'Hetumind'}
        </h3>
      </div>

      <Menu
        mode="inline"
        selectedKeys={getSelectedKeys()}
        items={menuItems}
        onClick={handleMenuClick}
        style={{
          borderRight: 'none',
          background: 'transparent',
        }}
      />
    </Sider>
  );
};

export default Sidebar;