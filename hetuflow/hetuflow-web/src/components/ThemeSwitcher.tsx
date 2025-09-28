import React from 'react';
import { Button, Dropdown, Space, Spin } from 'antd';
import { SunOutlined, MoonOutlined, MonitorOutlined, DownOutlined } from '@ant-design/icons';
import { useTheme, ThemeMode } from '../contexts/ThemeContext';

const ThemeSwitcher: React.FC = () => {
  const { themeMode, setThemeMode, currentTheme, isInitialized } = useTheme();

  const handleThemeChange = (mode: ThemeMode) => {
    setThemeMode(mode);
  };

  const themeMenuItems = [
    {
      key: 'light',
      label: '浅色模式',
      icon: <SunOutlined />,
      onClick: () => handleThemeChange('light'),
    },
    {
      key: 'dark',
      label: '深色模式',
      icon: <MoonOutlined />,
      onClick: () => handleThemeChange('dark'),
    },
    {
      key: 'system',
      label: '跟随系统',
      icon: <MonitorOutlined />,
      onClick: () => handleThemeChange('system'),
    },
  ];

  const getCurrentThemeIcon = () => {
    if (!isInitialized) {
      return <Spin size="small" />;
    }
    if (themeMode === 'system') {
      return <MonitorOutlined />;
    }
    return currentTheme === 'dark' ? <MoonOutlined /> : <SunOutlined />;
  };

  const getCurrentThemeLabel = () => {
    if (!isInitialized) {
      return '加载中...';
    }
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
      disabled={!isInitialized}
    >
      <Button
        type="text"
        style={{
          display: 'flex',
          alignItems: 'center',
          opacity: isInitialized ? 1 : 0.6,
        }}
        title={!isInitialized ? '正在初始化主题...' : '切换主题'}
      >
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
