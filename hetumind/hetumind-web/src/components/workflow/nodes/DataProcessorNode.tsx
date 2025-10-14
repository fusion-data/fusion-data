import React from 'react';
import { DatabaseOutlined } from '@ant-design/icons';
import { BaseNodeProps } from './types';
import BaseNodeComponent from './BaseNode';

interface DataProcessorNodeProps extends Omit<BaseNodeProps, 'data'> {
  data: {
    label: string;
    description?: string;
    processorType?: 'mapper' | 'filter' | 'aggregator' | 'transformer' | 'validator';
    config?: {
      mappingRules?: Array<{
        sourceField: string;
        targetField: string;
        transform?: string;
      }>;
      filterConditions?: Array<{
        field: string;
        operator: string;
        value: any;
      }>;
      aggregationMethod?: 'sum' | 'count' | 'avg' | 'min' | 'max';
      transformScript?: string;
      validationRules?: Array<{
        field: string;
        rule: string;
        message?: string;
      }>;
    };
  };
}

const DataProcessorNode: React.FC<DataProcessorNodeProps> = ({ data, ...props }) => {
  const { processorType, config } = data;

  // 根据处理器类型获取配置显示
  const getConfigDisplay = () => {
    switch (processorType) {
      case 'mapper':
        return `映射: ${config?.mappingRules?.length || 0} 个规则`;
      case 'filter':
        return `过滤: ${config?.filterConditions?.length || 0} 个条件`;
      case 'aggregator':
        return `聚合: ${config?.aggregationMethod || 'sum'}`;
      case 'transformer':
        return config?.transformScript ? '自定义转换' : '数据转换';
      case 'validator':
        return `验证: ${config?.validationRules?.length || 0} 个规则`;
      default:
        return '数据处理';
    }
  };

  // 处理节点编辑
  const handleEdit = () => {
    // TODO: 打开节点编辑弹窗
    console.log('Edit data processor node:', data);
  };

  // 处理节点删除
  const handleDelete = () => {
    // TODO: 删除节点
    console.log('Delete data processor node:', data);
  };

  // 处理节点复制
  const handleDuplicate = () => {
    // TODO: 复制节点
    console.log('Duplicate data processor node:', data);
  };

  return (
    <BaseNodeComponent
      {...props}
      data={{
        ...data,
        type: 'dataProcessor',
        icon: <DatabaseOutlined />,
        config: {
          type: processorType || 'mapper',
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

export default DataProcessorNode;
export type { DataProcessorNodeProps };