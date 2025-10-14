import React from 'react';
import { Handle, Position } from '@xyflow/react';
import { Button, Tooltip } from 'antd';
import { EditOutlined, DeleteOutlined, CopyOutlined } from '@ant-design/icons';
import { BaseNodeProps, NodeStatus, NodeStyleConfig } from './types';
import '@/styles/workflow-nodes.css';

interface BaseNodeComponentProps extends BaseNodeProps {
  nodeStyle?: NodeStyleConfig;
  showHandles?: boolean;
  editable?: boolean;
  onEdit?: () => void;
  onDelete?: () => void;
  onDuplicate?: () => void;
}

const BaseNodeComponent: React.FC<BaseNodeComponentProps> = ({
  data,
  selected,
  nodeStyle,
  showHandles = true,
  editable = true,
  onEdit,
  onDelete,
  onDuplicate,
}) => {
  const {
    label,
    description,
    type,
    status = NodeStatus.IDLE,
    icon,
    config = {},
  } = data;

  // 获取节点状态样式
  const getStatusStyle = (): React.CSSProperties => {
    switch (status) {
      case NodeStatus.RUNNING:
        return {
          borderLeftColor: '#1890ff',
          boxShadow: '0 0 0 2px rgba(24, 144, 255, 0.2)',
        };
      case NodeStatus.SUCCESS:
        return {
          borderLeftColor: '#52c41a',
          boxShadow: '0 0 0 2px rgba(82, 196, 26, 0.2)',
        };
      case NodeStatus.ERROR:
        return {
          borderLeftColor: '#ff4d4f',
          boxShadow: '0 0 0 2px rgba(255, 77, 79, 0.2)',
        };
      default:
        return {};
    }
  };

  // 获取节点默认样式
  const getDefaultStyle = (): React.CSSProperties => {
    return {
      backgroundColor: 'var(--bg-primary)',
      border: `1px solid var(--border-primary)`,
      borderRadius: '8px',
      padding: '12px 16px',
      minWidth: '200px',
      boxShadow: selected ? '0 4px 12px rgba(0, 0, 0, 0.15)' : '0 2px 4px rgba(0, 0, 0, 0.1)',
      cursor: 'default',
      borderLeft: '4px solid var(--color-primary)',
      transition: 'all 0.2s ease',
      ...nodeStyle,
      ...getStatusStyle(),
    };
  };

  // 获取节点类型颜色
  const getNodeColor = (): string => {
    const colorMap: Record<string, string> = {
      trigger: '#52c41a',
      action: '#1890ff',
      condition: '#fa8c16',
      dataProcessor: '#722ed1',
      webhook: '#eb2f96',
      timer: '#13c2c2',
      apiCall: '#fa541c',
      codeExecution: '#722ed1',
      database: '#1890ff',
      email: '#52c41a',
      fileHandler: '#fa8c16',
      aiAgent: '#1890ff',
    };
    return colorMap[type] || 'var(--color-primary)';
  };

  // 处理节点操作
  const handleEdit = (e: React.MouseEvent) => {
    e.stopPropagation();
    onEdit?.();
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    onDelete?.();
  };

  const handleDuplicate = (e: React.MouseEvent) => {
    e.stopPropagation();
    onDuplicate?.();
  };

  // 获取状态图标
  const getStatusIcon = () => {
    switch (status) {
      case NodeStatus.RUNNING:
        return <div className="node-status node-status-running" />;
      case NodeStatus.SUCCESS:
        return <div className="node-status node-status-success" />;
      case NodeStatus.ERROR:
        return <div className="node-status node-status-error" />;
      default:
        return null;
    }
  };

  return (
    <div className="workflow-node" style={getDefaultStyle()}>
      {/* 输入连接点 */}
      {showHandles && (
        <Handle
          type="target"
          position={Position.Left}
          style={{
            background: getNodeColor(),
            border: '2px solid var(--bg-primary)',
          }}
        />
      )}

      {/* 节点头部 */}
      <div className="node-header">
        <div className="node-title">
          <div className="node-icon" style={{ color: getNodeColor() }}>
            {icon}
          </div>
          <span className="node-label">{label}</span>
          {getStatusIcon()}
        </div>

        {/* 节点操作按钮 */}
        {editable && (
          <div className="node-actions">
            <Tooltip title="编辑">
              <Button
                type="text"
                size="small"
                icon={<EditOutlined />}
                onClick={handleEdit}
                className="node-action-btn"
              />
            </Tooltip>
            <Tooltip title="复制">
              <Button
                type="text"
                size="small"
                icon={<CopyOutlined />}
                onClick={handleDuplicate}
                className="node-action-btn"
              />
            </Tooltip>
            <Tooltip title="删除">
              <Button
                type="text"
                size="small"
                icon={<DeleteOutlined />}
                onClick={handleDelete}
                className="node-action-btn node-action-delete"
              />
            </Tooltip>
          </div>
        )}
      </div>

      {/* 节点描述 */}
      {description && (
        <div className="node-description">
          {description}
        </div>
      )}

      {/* 节点配置显示 */}
      {Object.keys(config).length > 0 && (
        <div className="node-config">
          {Object.entries(config).map(([key, value]) => (
            <div key={key} className="config-item">
              <span className="config-label">{key}:</span>
              <span className="config-value">{String(value)}</span>
            </div>
          ))}
        </div>
      )}

      {/* 输出连接点 */}
      {showHandles && (
        <Handle
          type="source"
          position={Position.Right}
          style={{
            background: getNodeColor(),
            border: '2px solid var(--bg-primary)',
          }}
        />
      )}
    </div>
  );
};

export default BaseNodeComponent;