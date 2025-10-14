import React from 'react';
import { RobotOutlined } from '@ant-design/icons';
import { BaseNodeProps } from './types';
import BaseNodeComponent from './BaseNode';

interface AIAgentNodeProps extends Omit<BaseNodeProps, 'data'> {
  data: {
    label: string;
    description?: string;
    agentType?: 'chat' | 'completion' | 'embedding' | 'image' | 'speech';
    config?: {
      agentId?: string;
      model?: string;
      prompt?: string;
      temperature?: number;
      maxTokens?: number;
      topP?: number;
      frequencyPenalty?: number;
      presencePenalty?: number;
      systemPrompt?: string;
      tools?: Array<{
        type: string;
        name: string;
        description?: string;
      }>;
      responseFormat?: 'text' | 'json' | 'markdown';
      timeout?: number;
    };
  };
}

const AIAgentNode: React.FC<AIAgentNodeProps> = ({ data, ...props }) => {
  const { agentType, config } = data;

  // 根据 AI Agent 类型获取配置显示
  const getConfigDisplay = () => {
    switch (agentType) {
      case 'chat':
        return config?.agentId ? `对话: ${config.agentId}` : 'AI 对话';
      case 'completion':
        return config?.model ? `文本生成: ${config.model}` : '文本生成';
      case 'embedding':
        return config?.model ? `向量嵌入: ${config.model}` : '向量嵌入';
      case 'image':
        return config?.model ? `图像生成: ${config.model}` : '图像生成';
      case 'speech':
        return config?.model ? `语音处理: ${config.model}` : '语音处理';
      default:
        return 'AI 智能体';
    }
  };

  // 处理节点编辑
  const handleEdit = () => {
    // TODO: 打开节点编辑弹窗
    console.log('Edit AI agent node:', data);
  };

  // 处理节点删除
  const handleDelete = () => {
    // TODO: 删除节点
    console.log('Delete AI agent node:', data);
  };

  // 处理节点复制
  const handleDuplicate = () => {
    // TODO: 复制节点
    console.log('Duplicate AI agent node:', data);
  };

  return (
    <BaseNodeComponent
      {...props}
      data={{
        ...data,
        type: 'aiAgent',
        icon: <RobotOutlined />,
        config: {
          type: agentType || 'chat',
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

export default AIAgentNode;
export type { AIAgentNodeProps };