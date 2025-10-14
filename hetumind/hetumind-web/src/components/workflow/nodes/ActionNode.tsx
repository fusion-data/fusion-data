import React from 'react';
import {
  ApiOutlined,
  CodeOutlined,
  DatabaseOutlined,
  MailOutlined,
  FileTextOutlined,
  RobotOutlined,
  FunctionOutlined
} from '@ant-design/icons';
import { BaseNodeProps } from './types';
import BaseNodeComponent from './BaseNode';

interface ActionNodeProps extends Omit<BaseNodeProps, 'data'> {
  data: {
    label: string;
    description?: string;
    actionType: 'api' | 'code' | 'database' | 'email' | 'file' | 'aiAgent' | 'function';
    config?: {
      url?: string;
      method?: string;
      headers?: Record<string, string>;
      body?: string;
      code?: string;
      language?: string;
      query?: string;
      collection?: string;
      operation?: string;
      to?: string;
      subject?: string;
      content?: string;
      filePath?: string;
      operationType?: string;
      agentId?: string;
      functionName?: string;
      parameters?: Record<string, any>;
    };
  };
}

const ActionNode: React.FC<ActionNodeProps> = ({ data, ...props }) => {
  const { actionType, config } = data;

  // 根据动作类型获取图标
  const getActionIcon = () => {
    switch (actionType) {
      case 'api':
        return <ApiOutlined />;
      case 'code':
        return <CodeOutlined />;
      case 'database':
        return <DatabaseOutlined />;
      case 'email':
        return <MailOutlined />;
      case 'file':
        return <FileTextOutlined />;
      case 'aiAgent':
        return <RobotOutlined />;
      case 'function':
        return <FunctionOutlined />;
      default:
        return <FunctionOutlined />;
    }
  };

  // 获取动作配置显示
  const getConfigDisplay = () => {
    switch (actionType) {
      case 'api':
        return config?.url ? `API: ${config.method || 'GET'} ${config.url}` : 'API 调用';
      case 'code':
        return config?.language ? `代码: ${config.language}` : '代码执行';
      case 'database':
        return config?.operation ? `数据库: ${config.operation}` : '数据库操作';
      case 'email':
        return config?.to ? `邮件: 发送到 ${config.to}` : '发送邮件';
      case 'file':
        return config?.operationType ? `文件: ${config.operationType}` : '文件操作';
      case 'aiAgent':
        return config?.agentId ? `AI 智能体: ${config.agentId}` : 'AI 智能体';
      case 'function':
        return config?.functionName ? `函数: ${config.functionName}` : '函数调用';
      default:
        return '执行动作';
    }
  };

  // 处理节点编辑
  const handleEdit = () => {
    // TODO: 打开节点编辑弹窗
    console.log('Edit action node:', data);
  };

  // 处理节点删除
  const handleDelete = () => {
    // TODO: 删除节点
    console.log('Delete action node:', data);
  };

  // 处理节点复制
  const handleDuplicate = () => {
    // TODO: 复制节点
    console.log('Duplicate action node:', data);
  };

  return (
    <BaseNodeComponent
      {...props}
      data={{
        ...data,
        type: actionType,
        icon: getActionIcon(),
        config: {
          type: actionType,
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

export default ActionNode;
export type { ActionNodeProps };