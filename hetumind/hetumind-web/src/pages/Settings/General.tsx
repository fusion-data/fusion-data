import React from 'react';
import { Tabs } from 'antd';
import ThemeSettings from '@/components/ui/ThemeSettings';

const GeneralSettingsPage: React.FC = () => {
  const tabItems = [
    {
      key: 'theme',
      label: '主题设置',
      children: <ThemeSettings />,
    },
    {
      key: 'preferences',
      label: '偏好设置',
      children: (
        <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
          <div style={{ fontSize: '48px', marginBottom: '16px', opacity: 0.3 }}>⚙️</div>
          <h3 style={{ color: 'var(--text-secondary)', marginBottom: '8px' }}>
            偏好设置
          </h3>
          <p>偏好设置功能正在开发中...</p>
        </div>
      ),
    },
    {
      key: 'accessibility',
      label: '无障碍设置',
      children: (
        <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
          <div style={{ fontSize: '48px', marginBottom: '16px', opacity: 0.3 }}>♿</div>
          <h3 style={{ color: 'var(--text-secondary)', marginBottom: '8px' }}>
            无障碍设置
          </h3>
          <p>无障碍设置功能正在开发中...</p>
        </div>
      ),
    },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Tabs
        defaultActiveKey="theme"
        items={tabItems}
      />
    </div>
  );
};

export default GeneralSettingsPage;