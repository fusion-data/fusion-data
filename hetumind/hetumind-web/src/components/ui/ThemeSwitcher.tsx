import React from 'react';
import { Button, Dropdown, Space, Tooltip, Badge } from 'antd';
import { useTheme } from '@/contexts/ThemeContext';
import {
  SunOutlined,
  MoonOutlined,
  LaptopOutlined,
  CheckOutlined,
} from '@ant-design/icons';

const ThemeSwitcher: React.FC = () => {
  const { themeMode, colorScheme, setThemeMode, setColorScheme } = useTheme();

  const themeOptions = [
    {
      key: 'light',
      label: '浅色模式',
      icon: <SunOutlined />,
      description: '使用浅色背景和深色文字',
    },
    {
      key: 'dark',
      label: '深色模式',
      icon: <MoonOutlined />,
      description: '使用深色背景和浅色文字',
    },
    {
      key: 'system',
      label: '跟随系统',
      icon: <LaptopOutlined />,
      description: '自动跟随系统主题设置',
    },
  ];

  const colorOptions = [
    { key: 'blue', label: '蓝色', color: '#1890ff', description: '经典蓝色主题' },
    { key: 'purple', label: '紫色', color: '#722ed1', description: '优雅紫色主题' },
    { key: 'green', label: '绿色', color: '#52c41a', description: '自然绿色主题' },
    { key: 'orange', label: '橙色', color: '#fa8c16', description: '活力橙色主题' },
  ];

  const currentThemeOption = themeOptions.find(option => option.key === themeMode);
  const currentColorOption = colorOptions.find(option => option.key === colorScheme);

  const getThemeMenuItems = () => {
    return themeOptions.map(option => ({
      key: option.key,
      label: (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '200px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            {option.icon}
            <div>
              <div style={{ fontWeight: themeMode === option.key ? 'bold' : 'normal' }}>
                {option.label}
              </div>
              <div style={{ fontSize: '12px', color: 'var(--text-tertiary)' }}>
                {option.description}
              </div>
            </div>
          </div>
          {themeMode === option.key && <CheckOutlined style={{ color: 'var(--color-primary)' }} />}
        </div>
      ),
      onClick: () => setThemeMode(option.key as any),
    }));
  };

  const getColorMenuItems = () => {
    return colorOptions.map(option => (
      <div
        key={option.key}
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '8px 12px',
          cursor: 'pointer',
          borderRadius: '6px',
          backgroundColor: colorScheme === option.key ? 'var(--color-primary-bg)' : 'transparent',
        }}
        onClick={() => setColorScheme(option.key as any)}
        onMouseEnter={(e) => {
          if (colorScheme !== option.key) {
            e.currentTarget.style.backgroundColor = 'var(--bg-secondary)';
          }
        }}
        onMouseLeave={(e) => {
          if (colorScheme !== option.key) {
            e.currentTarget.style.backgroundColor = 'transparent';
          }
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <div
            style={{
              width: '16px',
              height: '16px',
              borderRadius: '50%',
              backgroundColor: option.color,
              border: `2px solid ${colorScheme === option.key ? option.color : 'var(--border-primary)'}`,
            }}
          />
          <div>
            <div style={{ fontWeight: colorScheme === option.key ? 'bold' : 'normal' }}>
              {option.label}
            </div>
            <div style={{ fontSize: '12px', color: 'var(--text-tertiary)' }}>
              {option.description}
            </div>
          </div>
        </div>
        {colorScheme === option.key && (
          <CheckOutlined style={{ color: option.color }} />
        )}
      </div>
    ));
  };

  return (
    <Space size="small">
      {/* 主题模式切换 */}
      <Dropdown
        menu={{
          items: getThemeMenuItems(),
        }}
        trigger={['click']}
        placement="bottomRight"
      >
        <Tooltip title={`主题模式: ${currentThemeOption?.label}`}>
          <Button
            type="text"
            icon={
              <Badge dot={themeMode === 'system'} offset={[0, 2]}>
                {currentThemeOption?.icon}
              </Badge>
            }
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '32px',
              height: '32px',
            }}
          />
        </Tooltip>
      </Dropdown>

      {/* 颜色方案切换 */}
      <Dropdown
        trigger={['click']}
        placement="bottomRight"
        dropdownRender={() => (
          <div
            style={{
              backgroundColor: 'var(--bg-primary)',
              border: '1px solid var(--border-primary)',
              borderRadius: '8px',
              padding: '8px',
              minWidth: '220px',
              boxShadow: 'var(--shadow-2)',
            }}
          >
            <div
              style={{
                padding: '4px 8px',
                marginBottom: '4px',
                fontSize: '12px',
                fontWeight: 'bold',
                color: 'var(--text-secondary)',
              }}
            >
              主题颜色
            </div>
            {getColorMenuItems()}
          </div>
        )}
      >
        <Tooltip title={`主题颜色: ${currentColorOption?.label}`}>
          <Button
            type="text"
            icon={
              <div
                style={{
                  width: '16px',
                  height: '16px',
                  borderRadius: '50%',
                  backgroundColor: currentColorOption?.color,
                  border: '1px solid var(--border-primary)',
                }}
              />
            }
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '32px',
              height: '32px',
            }}
          />
        </Tooltip>
      </Dropdown>
    </Space>
  );
};

export default ThemeSwitcher;