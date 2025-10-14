import React, { useCallback, useRef, useEffect } from 'react';
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Edge,
  Node,
  NodeChange,
  EdgeChange,
  applyNodeChanges,
  applyEdgeChanges,
  BackgroundVariant,
  NodeTypes,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { useWorkflowStore } from '@/stores';
import { NodeProvider } from './nodes/NodeContext';
import { DragDropProvider, CanvasDropZone } from './dnd/DragDropProvider';

// 导入实际的节点组件
import TriggerNode from './nodes/TriggerNode';
import ActionNode from './nodes/ActionNode';
import ConditionNode from './nodes/ConditionNode';
import DataProcessorNode from './nodes/DataProcessorNode';
import WebhookNode from './nodes/WebhookNode';
import TimerNode from './nodes/TimerNode';
import AIAgentNode from './nodes/AIAgentNode';

interface WorkflowCanvasProps {
  workflowId?: string;
  readOnly?: boolean;
}

// 节点类型定义
const nodeTypes: NodeTypes = {
  trigger: TriggerNode,
  action: ActionNode,
  condition: ConditionNode,
  dataProcessor: DataProcessorNode,
  webhook: WebhookNode,
  timer: TimerNode,
  aiAgent: AIAgentNode,
};

export const WorkflowCanvas: React.FC<WorkflowCanvasProps> = ({ readOnly = false }) => {
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const [nodes, setNodes, _onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, _onEdgesChange] = useEdgesState<Edge>([]);

  const {
    currentWorkflow,
    setCurrentWorkflow,
    updateNode,
    deleteNode,
    addEdge: addWorkflowEdge,
    updateEdge,
    deleteEdge,
    selectedNodes,
    selectedEdges,
    selectNode,
    selectEdge,
    clearSelection,
    viewState,
  } = useWorkflowStore();

  // 初始化工作流数据
  useEffect(() => {
    if (currentWorkflow) {
      // 转换 WorkflowNode 到 Node 类型
      const convertedNodes: Node[] = currentWorkflow.nodes.map(node => ({
        id: node.id,
        type: node.type,
        position: node.position,
        data: node.data,
        style: node.style,
      }));
      setNodes(convertedNodes);

      // 转换 WorkflowEdge 到 Edge 类型
      const convertedEdges: Edge[] = currentWorkflow.edges.map(edge => ({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        sourceHandle: edge.sourceHandle,
        targetHandle: edge.targetHandle,
        type: edge.type,
        style: edge.style,
        label: edge.label,
        animated: edge.animated,
        data: edge.data,
      }));
      setEdges(convertedEdges);
    }
  }, [currentWorkflow, setNodes, setEdges]);

  // 处理节点变化
  const onNodesChangeHandler = useCallback(
    (changes: NodeChange[]) => {
      const updatedNodes = applyNodeChanges(changes, nodes);
      setNodes(updatedNodes);

      // 更新 store 中的节点
      updatedNodes.forEach((node) => {
        // 查找与该节点相关的变化
        const nodeChanges = changes.filter((c) => 'id' in c && c.id === node.id);
        if (nodeChanges.length > 0) {
          updateNode(node.id, {
            position: node.position,
            data: node.data,
            style: (node as any).style || {},
          });
        }
      });
    },
    [nodes, setNodes, updateNode]
  );

  // 处理边变化
  const onEdgesChangeHandler = useCallback(
    (changes: EdgeChange[]) => {
      const updatedEdges = applyEdgeChanges(changes, edges);
      setEdges(updatedEdges);

      // 更新 store 中的边
      updatedEdges.forEach((edge) => {
        // 查找与该边相关的变化
        const edgeChanges = changes.filter((c) => 'id' in c && c.id === edge.id);
        if (edgeChanges.length > 0) {
          updateEdge(edge.id, {
            source: edge.source,
            target: edge.target,
            sourceHandle: edge.sourceHandle || undefined,
            targetHandle: edge.targetHandle || undefined,
            style: (edge as any).style || {},
            label: (edge as any).label || undefined,
          });
        }
      });
    },
    [edges, setEdges, updateEdge]
  );

  // 处理新连接
  const onConnectHandler = useCallback(
    (params: Connection) => {
      const newEdge = addEdge(params, edges);
      setEdges(newEdge);
      addWorkflowEdge({
        id: `edge_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        source: params.source,
        target: params.target,
        sourceHandle: params.sourceHandle || undefined,
        targetHandle: params.targetHandle || undefined,
        type: undefined,
        style: undefined,
        label: undefined,
      });
    },
    [edges, setEdges, addWorkflowEdge]
  );

  // 处理节点选择
  const onNodeClickHandler = useCallback(
    (event: React.MouseEvent, node: Node) => {
      selectNode(node.id, event.shiftKey);
    },
    [selectNode]
  );

  // 处理边选择
  const onEdgeClickHandler = useCallback(
    (event: React.MouseEvent, edge: Edge) => {
      selectEdge(edge.id, event.shiftKey);
    },
    [selectEdge]
  );

  // 处理画布点击
  const onPaneClickHandler = useCallback(() => {
    clearSelection();
  }, [clearSelection]);

  // 删除节点
  const onNodesDeleteHandler = useCallback(
    (nodesToDelete: Node[]) => {
      nodesToDelete.forEach((node) => {
        deleteNode(node.id);
      });
    },
    [deleteNode]
  );

  // 删除边
  const onEdgesDeleteHandler = useCallback(
    (edgesToDelete: Edge[]) => {
      edgesToDelete.forEach((edge) => {
        deleteEdge(edge.id);
      });
    },
    [deleteEdge]
  );

  // 保存工作流
  const saveWorkflow = useCallback(() => {
    if (currentWorkflow) {
      // 将 Node 转换回 WorkflowNode
      const convertedNodes = nodes.map(node => ({
        id: node.id,
        type: node.type || 'unknown',
        data: node.data,
        position: node.position,
        style: node.style,
      }));

      // 将 Edge 转换回 WorkflowEdge
      const convertedEdges = edges.map(edge => ({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        sourceHandle: edge.sourceHandle || undefined,
        targetHandle: edge.targetHandle || undefined,
        type: edge.type,
        style: edge.style,
        label: typeof edge.label === 'string' ? edge.label : undefined,
        animated: edge.animated,
        data: edge.data,
      }));

      const updatedWorkflow = {
        ...currentWorkflow,
        nodes: convertedNodes,
        edges: convertedEdges,
      };
      setCurrentWorkflow(updatedWorkflow);
      // TODO: 调用 API 保存工作流
      console.log('Saving workflow:', updatedWorkflow);
    }
  }, [currentWorkflow, nodes, edges, setCurrentWorkflow]);

  // 键盘快捷键
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (readOnly) return;

      // Delete 键删除选中的节点和边
      if (event.key === 'Delete' || event.key === 'Backspace') {
        event.preventDefault();

        // 删除选中的节点
        selectedNodes.forEach((nodeId) => {
          const node = nodes.find((n) => n.id === nodeId);
          if (node) {
            deleteNode(nodeId);
          }
        });

        // 删除选中的边
        selectedEdges.forEach((edgeId) => {
          deleteEdge(edgeId);
        });
      }

      // Ctrl+S 保存工作流
      if ((event.ctrlKey || event.metaKey) && event.key === 's') {
        event.preventDefault();
        saveWorkflow();
      }

      // Ctrl+Z 撤销
      if ((event.ctrlKey || event.metaKey) && event.key === 'z' && !event.shiftKey) {
        event.preventDefault();
        // TODO: 实现撤销功能
      }

      // Ctrl+Y 或 Ctrl+Shift+Z 重做
      if ((event.ctrlKey || event.metaKey) && (event.key === 'y' || (event.key === 'z' && event.shiftKey))) {
        event.preventDefault();
        // TODO: 实现重做功能
      }

      // Ctrl+A 全选
      if ((event.ctrlKey || event.metaKey) && event.key === 'a') {
        event.preventDefault();
        // TODO: 实现全选功能
      }

      // Esc 清除选择
      if (event.key === 'Escape') {
        clearSelection();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [readOnly, selectedNodes, selectedEdges, nodes, deleteNode, deleteEdge, saveWorkflow, clearSelection]);

  return (
    <NodeProvider existingNodes={nodes}>
      <DragDropProvider>
        <div className="workflow-canvas" style={{ width: '100%', height: '100%' }}>
          <CanvasDropZone>
            <ReactFlow
              ref={reactFlowWrapper}
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChangeHandler}
              onEdgesChange={onEdgesChangeHandler}
              onConnect={onConnectHandler}
              onNodeClick={onNodeClickHandler}
              onEdgeClick={onEdgeClickHandler}
              onPaneClick={onPaneClickHandler}
              onNodesDelete={onNodesDeleteHandler}
              onEdgesDelete={onEdgesDeleteHandler}
              nodeTypes={nodeTypes}
              fitView
              attributionPosition="bottom-left"
              defaultViewport={{
                x: viewState.pan.x,
                y: viewState.pan.y,
                zoom: viewState.zoom,
              }}
              minZoom={0.1}
              maxZoom={3}
              snapToGrid={true}
              snapGrid={[15, 15]}
              deleteKeyCode={['Delete', 'Backspace']}
              multiSelectionKeyCode="Shift"
              panOnDrag
              selectionKeyCode="Shift"
              zoomOnScroll
              zoomOnPinch
              preventScrolling={false}
              style={{
                background: 'var(--bg-primary)',
              }}
            >
              <Background
                variant={BackgroundVariant.Dots}
                gap={15}
                size={1}
                color={readOnly ? 'var(--border-secondary)' : 'var(--border-primary)'}
              />

              {!readOnly && (
                <>
                  <MiniMap
                    style={{
                      backgroundColor: 'var(--bg-secondary)',
                      border: '1px solid var(--border-primary)',
                    }}
                    nodeColor={() => {
                      // TODO: 根据节点类型返回不同颜色
                      return 'var(--color-primary)';
                    }}
                    maskColor="rgba(0, 0, 0, 0.1)"
                  />

                  <Controls
                    showZoom={true}
                    showFitView={true}
                    showInteractive={true}
                    position="bottom-right"
                    style={{
                      backgroundColor: 'var(--bg-secondary)',
                      border: '1px solid var(--border-primary)',
                      borderRadius: '6px',
                    }}
                  />
                </>
              )}
            </ReactFlow>
          </CanvasDropZone>
        </div>
      </DragDropProvider>
    </NodeProvider>
  );
};

export default WorkflowCanvas;