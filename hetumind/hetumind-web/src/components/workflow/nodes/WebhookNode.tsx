import React from 'react';
import { ApiOutlined } from '@ant-design/icons';
import { BaseNodeProps } from './types';
import BaseNodeComponent from './BaseNode';

interface WebhookNodeProps extends Omit<BaseNodeProps, 'data'> {
  data: {
    label: string;
    description?: string;
    webhookType?: 'trigger' | 'response';
    config?: {
      method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
      path?: string;
      headers?: Record<string, string>;
      authType?: 'none' | 'basic' | 'bearer' | 'apikey';
      auth?: {
        username?: string;
        password?: string;
        token?: string;
        apiKey?: string;
        headerName?: string;
      };
      responseFormat?: 'json' | 'xml' | 'text';
      timeout?: number;
      retryCount?: number;
    };
  };
}

const WebhookNode: React.FC<WebhookNodeProps> = ({ data, ...props }) => {
  const { webhookType, config } = data;

  // 根据 webhook 类型获取配置显示
  const getConfigDisplay = () => {
    if (webhookType === 'trigger') {
      return config?.path ? `Webhook: ${config.method || 'POST'} ${config.path}` : 'HTTP 触发';
    } else {
      return config?.method ? `HTTP ${config.method}` : 'HTTP 响应';
    }
  };

  // 处理节点编辑
  const handleEdit = () => {
    // TODO: 打开节点编辑弹窗
    console.log('Edit webhook node:', data);
  };

  // 处理节点删除
  const handleDelete = () => {
    // TODO: 删除节点
    console.log('Delete webhook node:', data);
  };

  // 处理节点复制
  const handleDuplicate = () => {
    // TODO: 复制节点
    console.log('Duplicate webhook node:', data);
  };

  return (
    <BaseNodeComponent
      {...props}
      data={{
        ...data,
        type: 'webhook',
        icon: <ApiOutlined />,
        config: {
          type: webhookType || 'trigger',
          display: getConfigDisplay(),
          ...config,
        },
      }}
      onEdit={handleEdit}
      onDelete={handleDelete}
      onDuplicate={handleDuplicate}
    />
  );
};

export default WebhookNode;
export type { WebhookNodeProps };