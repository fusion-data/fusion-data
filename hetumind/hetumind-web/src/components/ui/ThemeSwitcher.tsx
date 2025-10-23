import React from 'react';
import { Button, Dropdown, Tooltip } from 'antd';
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
    { key: 'light', label: '浅色', icon: <SunOutlined /> },
    { key: 'dark', label: '深色', icon: <MoonOutlined /> },
    { key: 'system', label: '跟随系统', icon: <LaptopOutlined /> },
  ];

  const colorOptions = [
    { key: 'blue', label: '蓝色', color: '#1890ff' },
    { key: 'purple', label: '紫色', color: '#722ed1' },
    { key: 'green', label: '绿色', color: '#52c41a' },
    { key: 'orange', label: '橙色', color: '#fa8c16' },
  ];

  const currentThemeOption = themeOptions.find(option => option.key === themeMode);
  const currentColorOption = colorOptions.find(option => option.key === colorScheme);

  const getThemeMenuItems = () => {
    return themeOptions.map(option => ({
      key: option.key,
      label: (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '140px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            {option.icon}
            <div>
              <div style={{ fontSize: '13px', fontWeight: themeMode === option.key ? 'bold' : 'normal' }}>
                {option.label}
              </div>
            </div>
          </div>
          {themeMode === option.key && <CheckOutlined style={{ color: 'var(--color-primary)', fontSize: '12px' }} />}
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
          padding: '6px 8px',
          cursor: 'pointer',
          borderRadius: '4px',
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
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <div
            style={{
              width: '12px',
              height: '12px',
              borderRadius: '50%',
              backgroundColor: option.color,
              border: `2px solid ${colorScheme === option.key ? option.color : 'var(--border-primary)'}`,
            }}
          />
          <div>
            <div style={{ fontSize: '12px', fontWeight: colorScheme === option.key ? 'bold' : 'normal' }}>
              {option.label}
            </div>
          </div>
        </div>
        {colorScheme === option.key && (
          <CheckOutlined style={{ color: option.color, fontSize: '12px' }} />
        )}
      </div>
    ));
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
      {/* 主题模式切换 */}
      <Dropdown
        menu={{
          items: getThemeMenuItems(),
        }}
        trigger={['click']}
        placement="topRight"
      >
        <Tooltip title={`主题模式: ${currentThemeOption?.label}`}>
          <Button
            type="text"
            icon={currentThemeOption?.icon}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '32px',
              height: '32px',
              color: themeMode === 'system' ? 'var(--color-primary)' : 'var(--text-secondary)',
            }}
          />
        </Tooltip>
      </Dropdown>

      {/* 颜色方案切换 */}
      <Dropdown
        trigger={['click']}
        placement="topRight"
        popupRender={() => (
          <div
            style={{
              backgroundColor: 'var(--bg-primary)',
              border: '1px solid var(--border-primary)',
              borderRadius: '6px',
              padding: '6px',
              minWidth: '140px',
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
                  width: '12px',
                  height: '12px',
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
    </div>
  );
};

export default ThemeSwitcher;