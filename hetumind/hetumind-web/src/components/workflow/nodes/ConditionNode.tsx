import React from 'react';
import { BranchesOutlined } from '@ant-design/icons';
import { BaseNodeProps } from './types';
import BaseNodeComponent from './BaseNode';

interface ConditionNodeProps extends Omit<BaseNodeProps, 'data'> {
  data: {
    label: string;
    description?: string;
    conditionType?: 'if' | 'switch' | 'custom';
    config?: {
      expression?: string;
      branches?: Array<{
        id: string;
        name: string;
        condition?: string;
      }>;
    };
  };
}

const ConditionNode: React.FC<ConditionNodeProps> = ({ data, ...props }) => {
  const { conditionType, config } = data;

  // 根据条件类型获取配置显示
  const getConfigDisplay = () => {
    switch (conditionType) {
      case 'if':
        return config?.expression ? `条件: ${config.expression}` : '条件判断';
      case 'switch':
        return `分支: ${config?.branches?.length || 0} 个分支`;
      case 'custom':
        return '自定义条件';
      default:
        return '条件控制';
    }
  };

  // 处理节点编辑
  const handleEdit = () => {
    // TODO: 打开节点编辑弹窗
    console.log('Edit condition node:', data);
  };

  // 处理节点删除
  const handleDelete = () => {
    // TODO: 删除节点
    console.log('Delete condition node:', data);
  };

  // 处理节点复制
  const handleDuplicate = () => {
    // TODO: 复制节点
    console.log('Duplicate condition node:', data);
  };

  return (
    <BaseNodeComponent
      {...props}
      data={{
        ...data,
        type: 'condition',
        icon: <BranchesOutlined />,
        config: {
          type: conditionType || 'if',
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

export default ConditionNode;
export type { ConditionNodeProps };