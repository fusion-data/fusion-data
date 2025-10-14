import React from 'react';
import { Card, Typography, Radio, Space, Button, message } from 'antd';
import { useTheme } from '@/contexts/ThemeContext';
import {
  SunOutlined,
  MoonOutlined,
  LaptopOutlined,
  BgColorsOutlined,
  ReloadOutlined,
} from '@ant-design/icons';

const { Title, Text } = Typography;

const ThemeSettings: React.FC = () => {
  const { themeMode, colorScheme, setThemeMode, setColorScheme, resetTheme } = useTheme();

  const handleThemeModeChange = (mode: 'light' | 'dark' | 'system') => {
    setThemeMode(mode);
    message.success(`已切换到${mode === 'light' ? '浅色' : mode === 'dark' ? '深色' : '跟随系统'}模式`);
  };

  const handleColorSchemeChange = (scheme: 'blue' | 'purple' | 'green' | 'orange') => {
    setColorScheme(scheme);
    const colorNames = {
      blue: '蓝色',
      purple: '紫色',
      green: '绿色',
      orange: '橙色',
    };
    message.success(`已切换到${colorNames[scheme]}主题`);
  };

  const handleResetTheme = () => {
    resetTheme();
    message.success('主题已重置为默认设置');
  };

  const themeOptions = [
    {
      value: 'light',
      label: '浅色模式',
      description: '使用浅色背景和深色文字，适合白天使用',
      icon: <SunOutlined style={{ color: '#faad14' }} />,
    },
    {
      value: 'dark',
      label: '深色模式',
      description: '使用深色背景和浅色文字，适合夜间使用',
      icon: <MoonOutlined style={{ color: '#1890ff' }} />,
    },
    {
      value: 'system',
      label: '跟随系统',
      description: '自动跟随系统的主题设置',
      icon: <LaptopOutlined style={{ color: '#52c41a' }} />,
    },
  ];

  const colorOptions = [
    {
      value: 'blue',
      label: '蓝色主题',
      description: '经典稳重的蓝色系',
      color: '#1890ff',
    },
    {
      value: 'purple',
      label: '紫色主题',
      description: '优雅神秘的紫色系',
      color: '#722ed1',
    },
    {
      value: 'green',
      label: '绿色主题',
      description: '自然清新的绿色系',
      color: '#52c41a',
    },
    {
      value: 'orange',
      label: '橙色主题',
      description: '活力温暖的橙色系',
      color: '#fa8c16',
    },
  ];

  return (
    <div style={{ maxWidth: '800px', margin: '0 auto' }}>
      <Title level={3} style={{ marginBottom: '24px' }}>
        主题设置
      </Title>

      {/* 主题模式设置 */}
      <Card title={<><SunOutlined /> 主题模式</>} style={{ marginBottom: '16px' }}>
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          {themeOptions.map((option) => (
            <div key={option.value} style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <Radio
                checked={themeMode === option.value}
                onChange={() => handleThemeModeChange(option.value as any)}
                style={{ marginRight: '8px' }}
              />
              <div style={{ flex: 1 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  {option.icon}
                  <Text strong>{option.label}</Text>
                </div>
                <Text type="secondary" style={{ fontSize: '12px' }}>
                  {option.description}
                </Text>
              </div>
            </div>
          ))}
        </Space>
      </Card>

      {/* 颜色方案设置 */}
      <Card title={<><BgColorsOutlined /> 主题颜色</>} style={{ marginBottom: '16px' }}>
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          {colorOptions.map((option) => (
            <div key={option.value} style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <Radio
                checked={colorScheme === option.value}
                onChange={() => handleColorSchemeChange(option.value as any)}
                style={{ marginRight: '8px' }}
              />
              <div style={{
                width: '20px',
                height: '20px',
                borderRadius: '50%',
                backgroundColor: option.color,
                border: `2px solid ${colorScheme === option.value ? option.color : 'var(--border-primary)'}`,
              }} />
              <div style={{ flex: 1 }}>
                <Text strong>{option.label}</Text>
                <br />
                <Text type="secondary" style={{ fontSize: '12px' }}>
                  {option.description}
                </Text>
              </div>
            </div>
          ))}
        </Space>
      </Card>

      {/* 预览区域 */}
      <Card title="主题预览" style={{ marginBottom: '16px' }}>
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <div style={{
            padding: '16px',
            backgroundColor: 'var(--bg-secondary)',
            borderRadius: '8px',
            border: '1px solid var(--border-primary)',
          }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
              <Text strong>示例标题</Text>
              <Button type="primary" size="small">
                主要按钮
              </Button>
            </div>
            <Text type="secondary">
              这是一个示例文本，用于展示当前主题的效果。您可以看到文字颜色、背景色和按钮样式的变化。
            </Text>
          </div>

          <div style={{
            padding: '16px',
            backgroundColor: 'var(--color-primary-bg)',
            borderRadius: '8px',
            border: '1px solid var(--color-primary)',
          }}>
            <Text style={{ color: 'var(--color-primary)' }}>
              当前主题: {themeMode === 'light' ? '浅色' : themeMode === 'dark' ? '深色' : '跟随系统'} +
              {colorScheme === 'blue' ? ' + 蓝色' :
               colorScheme === 'purple' ? ' + 紫色' :
               colorScheme === 'green' ? ' + 绿色' : ' + 橙色'}主题
            </Text>
          </div>
        </Space>
      </Card>

      {/* 操作按钮 */}
      <Card>
        <Space>
          <Button
            icon={<ReloadOutlined />}
            onClick={handleResetTheme}
          >
            重置为默认主题
          </Button>
        </Space>
      </Card>
    </div>
  );
};

export default ThemeSettings;