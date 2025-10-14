import React from 'react';
import { Layout, Menu, Button, Avatar, Dropdown, Space } from 'antd';
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
        <h3 style={{
          margin: 0,
          color: 'var(--color-primary)',
          fontSize: collapsed ? '16px' : '18px',
          fontWeight: 'bold'
        }}>
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

      {/* 底部功能区 - 固定在底部 */}
      <div style={{
        padding: '12px 16px',
        borderTop: '1px solid var(--border-secondary)',
        flexShrink: 0, // 防止压缩
        marginTop: 'auto' // 推到底部
      }}>
        {/* 主题切换器 */}
        <div style={{ marginBottom: collapsed ? '8px' : '12px' }}>
          {collapsed ? (
            <Space direction="vertical" size="small" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
              <ThemeSwitcher />
            </Space>
          ) : (
            <ThemeSwitcher />
          )}
        </div>

        {/* 用户菜单 */}
        {!collapsed && (
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <Space>
              <Avatar
                size="small"
                icon={<UserOutlined />}
                style={{ cursor: 'pointer' }}
              />
              <span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                用户
              </span>
            </Space>
            <Dropdown
              menu={{
                items: userMenuItems,
                onClick: handleUserMenuClick,
              }}
              trigger={['click']}
              placement="topRight"
            >
              <Button
                type="text"
                size="small"
                style={{ padding: '0 4px' }}
              />
            </Dropdown>
          </div>
        )}

        {collapsed && (
          <div style={{ display: 'flex', justifyContent: 'center' }}>
            <Dropdown
              menu={{
                items: userMenuItems,
                onClick: handleUserMenuClick,
              }}
              trigger={['click']}
              placement="topRight"
            >
              <Avatar
                size="small"
                icon={<UserOutlined />}
                style={{ cursor: 'pointer' }}
              />
            </Dropdown>
          </div>
        )}
      </div>
    </Sider>
  );
};

export default Sidebar;