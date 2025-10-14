import React from 'react';
import { PlayCircleOutlined, CalendarOutlined, ApiOutlined } from '@ant-design/icons';
import { BaseNodeProps } from './types';
import BaseNodeComponent from './BaseNode';

interface TriggerNodeProps extends Omit<BaseNodeProps, 'data'> {
  data: {
    label: string;
    description?: string;
    triggerType: 'manual' | 'schedule' | 'webhook';
    config?: {
      schedule?: string;
      cronExpression?: string;
      webhookUrl?: string;
      method?: string;
      headers?: Record<string, string>;
    };
  };
}

const TriggerNode: React.FC<TriggerNodeProps> = ({ data, ...props }) => {
  const { triggerType, config } = data;

  // 根据触发器类型获取图标
  const getTriggerIcon = () => {
    switch (triggerType) {
      case 'manual':
        return <PlayCircleOutlined />;
      case 'schedule':
        return <CalendarOutlined />;
      case 'webhook':
        return <ApiOutlined />;
      default:
        return <PlayCircleOutlined />;
    }
  };

  // 获取触发器配置显示
  const getConfigDisplay = () => {
    switch (triggerType) {
      case 'schedule':
        return config?.schedule || config?.cronExpression || '定时触发';
      case 'webhook':
        return config?.webhookUrl ? `Webhook: ${config.webhookUrl}` : 'HTTP 触发';
      case 'manual':
      default:
        return '手动触发';
    }
  };

  // 处理节点编辑
  const handleEdit = () => {
    // TODO: 打开节点编辑弹窗
    console.log('Edit trigger node:', data);
  };

  // 处理节点删除
  const handleDelete = () => {
    // TODO: 删除节点
    console.log('Delete trigger node:', data);
  };

  // 处理节点复制
  const handleDuplicate = () => {
    // TODO: 复制节点
    console.log('Duplicate trigger node:', data);
  };

  return (
    <BaseNodeComponent
      {...props}
      data={{
        ...data,
        type: triggerType,
        icon: getTriggerIcon(),
        config: {
          type: triggerType,
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

export default TriggerNode;
export type { TriggerNodeProps };