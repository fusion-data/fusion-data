import React from 'react';
import { Layout, Button, Space, Avatar, Dropdown } from 'antd';
import ThemeSwitcher from '@/components/ui/ThemeSwitcher';
import {
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  UserOutlined,
  LogoutOutlined,
} from '@ant-design/icons';

const { Header: AntHeader } = Layout;

interface HeaderProps {
  collapsed: boolean;
  onToggle: () => void;
}

const Header: React.FC<HeaderProps> = ({ collapsed, onToggle }) => {
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
    <AntHeader
      style={{
        background: 'var(--bg-primary)',
        padding: '0 16px',
        borderBottom: '1px solid var(--border-primary)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
      }}
    >
      <Button
        type="text"
        icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
        onClick={onToggle}
        style={{ fontSize: '16px' }}
      />

      <Space size="middle">
        {/* 主题切换器 */}
        <ThemeSwitcher />

        {/* 用户菜单 */}
        <Dropdown
          menu={{
            items: userMenuItems,
            onClick: handleUserMenuClick,
          }}
          trigger={['click']}
        >
          <Avatar
            icon={<UserOutlined />}
            style={{ cursor: 'pointer' }}
          />
        </Dropdown>
      </Space>
    </AntHeader>
  );
};

export default Header;