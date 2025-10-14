import React from 'react';
import { ClockCircleOutlined } from '@ant-design/icons';
import { BaseNodeProps } from './types';
import BaseNodeComponent from './BaseNode';

interface TimerNodeProps extends Omit<BaseNodeProps, 'data'> {
  data: {
    label: string;
    description?: string;
    timerType?: 'interval' | 'delay' | 'schedule' | 'once';
    config?: {
      interval?: number; // 毫秒
      delay?: number; // 毫秒
      cronExpression?: string;
      timezone?: string;
      startTime?: string;
      endTime?: string;
      maxExecutions?: number;
    };
  };
}

const TimerNode: React.FC<TimerNodeProps> = ({ data, ...props }) => {
  const { timerType, config } = data;

  // 根据定时器类型获取配置显示
  const getConfigDisplay = () => {
    switch (timerType) {
      case 'interval':
        return config?.interval ? `间隔: ${config.interval}ms` : '定时重复';
      case 'delay':
        return config?.delay ? `延迟: ${config.delay}ms` : '延迟执行';
      case 'schedule':
        return config?.cronExpression ? `计划: ${config.cronExpression}` : '定时计划';
      case 'once':
        return config?.startTime ? `一次: ${config.startTime}` : '单次执行';
      default:
        return '定时器';
    }
  };

  // 处理节点编辑
  const handleEdit = () => {
    // TODO: 打开节点编辑弹窗
    console.log('Edit timer node:', data);
  };

  // 处理节点删除
  const handleDelete = () => {
    // TODO: 删除节点
    console.log('Delete timer node:', data);
  };

  // 处理节点复制
  const handleDuplicate = () => {
    // TODO: 复制节点
    console.log('Duplicate timer node:', data);
  };

  return (
    <BaseNodeComponent
      {...props}
      data={{
        ...data,
        type: 'timer',
        icon: <ClockCircleOutlined />,
        config: {
          type: timerType || 'once',
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

export default TimerNode;
export type { TimerNodeProps };