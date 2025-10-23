import React from 'react';
import { Layout, Menu, Button, Avatar, Dropdown } from 'antd';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  DashboardOutlined,
  NodeIndexOutlined,
  RobotOutlined,
  SettingOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  UserOutlined,
  LogoutOutlined,
} from '@ant-design/icons';
import ThemeSwitcher from '@/components/ui/ThemeSwitcher';

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
          key: '/workflows/list',
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
          key: '/agents/list',
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
    if (path === '/workflows' || path === '/workflows/list') return ['/workflows/list'];
    if (path === '/agents' || path === '/agents/list') return ['/agents/list'];
    if (path.startsWith('/settings')) return ['/settings'];

    // 工作流相关页面
    if (path.startsWith('/workflows/')) return ['/workflows/list'];

    // 智能体相关页面
    if (path.startsWith('/agents/')) return ['/agents/list'];

    return [];
  };

  const userMenuItems = [
    {
      key: 'profile',
      label: '个人设置',
      icon: <UserOutlined />,
    },
    {
      type: 'divider' as const,
    },
    {
      key: 'logout',
      label: '退出登录',
      icon: <LogoutOutlined />,
      danger: true,
    },
  ];

  const handleUserMenuClick = ({ key }: { key: string }) => {
    if (key === 'logout') {
      // TODO: 实现登出逻辑
      console.log('Logout clicked');
    }
  };

  return (
    <Sider
      style={{
        background: 'var(--bg-primary)',
        borderRight: '1px solid var(--border-primary)',
        display: 'flex',
        flexDirection: 'column',
      }}
      width={240}
      collapsedWidth={80}
      collapsible
      collapsed={collapsed}
      onCollapse={onCollapse}
      trigger={null}
    >
      {/* 顶部 Logo 和折叠按钮 */}
      <div
        style={{
          height: '64px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: collapsed ? '0 16px' : '0 24px',
          borderBottom: '1px solid var(--border-secondary)',
        }}
      >
        <h3
          style={{
            margin: 0,
            color: 'var(--color-primary)',
            fontSize: collapsed ? '16px' : '18px',
            fontWeight: 'bold',
          }}
        >
          {collapsed ? 'HM' : 'Hetumind'}
        </h3>
        <Button
          type="text"
          icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
          onClick={() => onCollapse(!collapsed)}
          style={{ fontSize: '16px' }}
        />
      </div>

      {/* 导航菜单 */}
      <div style={{ flex: 1, overflow: 'auto' }}>
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
      </div>

      {/* 底部功能区 - 固定在浏览器视口左下角 */}
      <div
        style={{
          position: 'fixed',
          left: collapsed ? 0 : 0,
          bottom: 0,
          width: collapsed ? 80 : 240,
          height: 'auto',
          padding: collapsed ? '12px 8px' : '12px 16px',
          borderTop: '1px solid var(--border-secondary)',
          background: 'var(--bg-primary)',
          boxShadow: '0 -2px 12px rgba(0, 0, 0, 0.15)',
          zIndex: 1000,
          transition: 'width 0.2s, left 0.2s',
        }}
      >
        {/* 底部菜单项 - 左对齐显示，参考顶部导航菜单样式 */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'stretch',
            gap: '4px',
          }}
        >
          {/* 主题切换器 - 左对齐布局 */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              padding: collapsed ? '8px' : '8px 16px',
              borderRadius: '4px',
              transition: 'background-color 0.2s',
              justifyContent: collapsed ? 'center' : 'flex-start',
              gap: collapsed ? '0' : '8px',
            }}
          >
            <ThemeSwitcher />
          </div>

          {/* 用户菜单 - 左对齐布局 */}
          <Dropdown
            menu={{
              items: userMenuItems,
              onClick: handleUserMenuClick,
            }}
            trigger={['click']}
            placement="topRight"
          >
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                cursor: 'pointer',
                padding: collapsed ? '8px' : '8px 16px',
                borderRadius: '4px',
                transition: 'background-color 0.2s',
                justifyContent: collapsed ? 'center' : 'flex-start',
                gap: collapsed ? '0' : '8px',
              }}
              onMouseEnter={e => {
                e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              <Avatar size="small" icon={<UserOutlined />} style={{ cursor: 'pointer' }} />
              {!collapsed && (
                <span style={{ fontSize: '14px', color: 'var(--text-primary)', fontWeight: 500 }}>用户</span>
              )}
            </div>
          </Dropdown>
        </div>
      </div>
    </Sider>
  );
};

export default Sidebar;
