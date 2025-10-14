import React from 'react';
import { Button, List, Typography } from 'antd';
import { PlusOutlined } from '@ant-design/icons';

const { Title } = Typography;

const AgentListPage: React.FC = () => {
  return (
    <div style={{ padding: '24px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <Title level={2} style={{ margin: 0, color: 'var(--text-primary)' }}>
          AI 智能体
        </Title>
        <Button type="primary" icon={<PlusOutlined />}>
          新建智能体
        </Button>
      </div>

      <List
        dataSource={[]}
        renderItem={() => null}
        locale={{
          emptyText: (
            <div style={{ textAlign: 'center', padding: '60px 20px' }}>
              <div style={{ fontSize: '48px', marginBottom: '16px', opacity: 0.3 }}>🤖</div>
              <Title level={4} style={{ color: 'var(--text-secondary)', marginBottom: '8px' }}>
                暂无智能体
              </Title>
              <p style={{ color: 'var(--text-tertiary)' }}>
                创建您的第一个 AI 智能体来开始使用
              </p>
              <Button type="primary" icon={<PlusOutlined />} style={{ marginTop: '16px' }}>
                新建智能体
              </Button>
            </div>
          ),
        }}
      />
    </div>
  );
};

export default AgentListPage;