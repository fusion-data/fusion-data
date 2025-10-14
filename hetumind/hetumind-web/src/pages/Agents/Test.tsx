import React from 'react';
import { Typography } from 'antd';

const { Title } = Typography;

const AgentTestPage: React.FC = () => {
  return (
    <div style={{ padding: '24px' }}>
      <Title level={2} style={{ color: 'var(--text-primary)' }}>
        AI 智能体测试
      </Title>
      <div style={{
        height: '400px',
        border: '1px solid var(--border-primary)',
        borderRadius: '8px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        backgroundColor: 'var(--bg-secondary)'
      }}>
        <div style={{ textAlign: 'center', color: 'var(--text-secondary)' }}>
          <div style={{ fontSize: '48px', marginBottom: '16px', opacity: 0.3 }}>🧪</div>
          <Title level={4} style={{ color: 'var(--text-secondary)', marginBottom: '8px' }}>
            AI 智能体测试
          </Title>
          <p>智能体测试功能正在开发中...</p>
        </div>
      </div>
    </div>
  );
};

export default AgentTestPage;