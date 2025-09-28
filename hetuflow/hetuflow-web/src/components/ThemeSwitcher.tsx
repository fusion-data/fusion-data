import React from 'react';
import { Button, Dropdown, Space } from 'antd';
import { SunOutlined, MoonOutlined, MonitorOutlined, DownOutlined } from '@ant-design/icons';
import { useTheme, ThemeMode } from '../contexts/ThemeContext';

const ThemeSwitcher: React.FC = () => {
  const { themeMode, setThemeMode, currentTheme } = useTheme();

  const themeMenuItems = [
    {
      key: 'light',
      label: '浅色模式',
      icon: <SunOutlined />,
      onClick: () => setThemeMode('light' as ThemeMode),
    },
    {
      key: 'dark',
      label: '深色模式',
      icon: <MoonOutlined />,
      onClick: () => setThemeMode('dark' as ThemeMode),
    },
    {
      key: 'system',
      label: '跟随系统',
      icon: <MonitorOutlined />,
      onClick: () => setThemeMode('system' as ThemeMode),
    },
  ];

  const getCurrentThemeIcon = () => {
    if (themeMode === 'system') {
      return <MonitorOutlined />;
    }
    return currentTheme === 'dark' ? <MoonOutlined /> : <SunOutlined />;
  };

  const getCurrentThemeLabel = () => {
    if (themeMode === 'system') {
      return '跟随系统';
    }
    return currentTheme === 'dark' ? '深色模式' : '浅色模式';
  };

  return (
    <Dropdown
      menu={{
        items: themeMenuItems,
      }}
      placement="bottomRight"
    >
      <Button type="text" style={{ display: 'flex', alignItems: 'center' }}>
        <Space>
          {getCurrentThemeIcon()}
          <span>{getCurrentThemeLabel()}</span>
          <DownOutlined style={{ fontSize: '10px' }} />
        </Space>
      </Button>
    </Dropdown>
  );
};

export default ThemeSwitcher;
