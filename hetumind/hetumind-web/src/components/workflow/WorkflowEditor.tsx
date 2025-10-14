import React, { useCallback } from 'react';
import { Layout } from 'antd';
import { useWorkflowStore } from '@/stores';
import WorkflowToolbar from './toolbar/WorkflowToolbar';
import WorkflowCanvas from './WorkflowCanvas';
import NodePanel from './panel/NodePanel';
import PropertyPanel from './panel/PropertyPanel';
import { useNodeContext } from './nodes/NodeContext';

const { Content, Sider } = Layout;

interface WorkflowEditorProps {
  workflowId?: string;
  readOnly?: boolean;
}

export const WorkflowEditor: React.FC<WorkflowEditorProps> = ({
  workflowId,
  readOnly = false,
}) => {
  const { selectedNodes } = useWorkflowStore();
  const { selectNode } = useNodeContext();

  // 处理节点选择
  const selectedNodeId = selectedNodes.length === 1 ? selectedNodes[0] : null;

  // 处理节点选择
  const handleNodeSelect = useCallback((nodeId: string | null) => {
    selectNode(nodeId);
  }, [selectNode]);

  return (
    <Layout style={{ height: '100vh' }}>
      {/* 顶部工具栏 */}
      <div style={{
        height: '56px',
        background: 'var(--bg-primary)',
        borderBottom: '1px solid var(--border-primary)',
        padding: '0 16px',
        display: 'flex',
        alignItems: 'center',
      }}>
        <WorkflowToolbar
          workflowId={workflowId}
          readOnly={readOnly}
        />
      </div>

      <Layout style={{ height: 'calc(100vh - 56px)' }}>
        {/* 左侧节点面板 */}
        <Sider
          width={280}
          style={{
            background: 'var(--bg-primary)',
            borderRight: 'none',
          }}
        >
          <NodePanel />
        </Sider>

        {/* 中间画布区域 */}
        <Content style={{
          background: 'var(--bg-canvas)',
          position: 'relative',
          overflow: 'hidden',
        }}>
          <WorkflowCanvas
            workflowId={workflowId}
            readOnly={readOnly}
          />
        </Content>

        {/* 右侧属性面板 */}
        <Sider
          width={320}
          style={{
            background: 'var(--bg-primary)',
            borderLeft: 'none',
          }}
        >
          <PropertyPanel
            nodeId={selectedNodeId || undefined}
            onClose={() => handleNodeSelect(null)}
          />
        </Sider>
      </Layout>
    </Layout>
  );
};

export default WorkflowEditor;