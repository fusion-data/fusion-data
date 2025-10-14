import React, { useCallback, useRef, useState } from 'react';
import { DndContext, DragOverlay, useDroppable, DragStartEvent, DragOverEvent, DragEndEvent } from '@dnd-kit/core';
import { useReactFlow } from '@xyflow/react';
import { SettingOutlined } from '@ant-design/icons';
import { useWorkflowStore } from '@/stores';
import { useNodeContext } from '../nodes/NodeContext';

interface DragDropProviderProps {
  children: React.ReactNode;
}

interface DraggableData {
  nodeType: {
    type: string;
    displayName: string;
    description: string;
    icon: string;
    category: string;
    color: string;
  };
}

interface DropZoneProps {
  children: React.ReactNode;
}

// 画布拖拽接收区域
export const CanvasDropZone: React.FC<DropZoneProps> = ({ children }) => {
  const { setNodeRef, isOver } = useDroppable({ id: 'canvas-drop-zone' });

  return (
    <div
      ref={setNodeRef}
      className={`canvas-drop-zone ${isOver ? 'drop-active' : ''}`}
      style={{
        width: '100%',
        height: '100%',
        position: 'relative',
      }}
    >
      {children}
      {isOver && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(24, 144, 255, 0.1)',
            border: '2px dashed #1890ff',
            pointerEvents: 'none',
            zIndex: 1000,
          }}
        />
      )}
    </div>
  );
};

// 拖拽预览组件
const DragPreview: React.FC<{ data: DraggableData['nodeType'] | null }> = ({ data }) => {
  if (!data) return null;

  return (
    <div
      style={{
        padding: '8px 12px',
        background: 'var(--bg-primary)',
        border: '1px solid var(--border-primary)',
        borderRadius: '6px',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
        cursor: 'grabbing',
        userSelect: 'none',
        transform: 'rotate(-5deg)',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
        }}
      >
        <div
          style={{
            fontSize: '16px',
            color: data.color,
          }}
        >
          {/* 这里应该根据 icon 名称渲染实际的图标 */}
          <SettingOutlined />
        </div>
        <div>
          <div
            style={{
              fontSize: '14px',
              fontWeight: 500,
              color: 'var(--text-primary)',
            }}
          >
            {data.displayName}
          </div>
          <div
            style={{
              fontSize: '12px',
              color: 'var(--text-secondary)',
            }}
          >
            {data.description}
          </div>
        </div>
      </div>
    </div>
  );
};

// 主要的拖拽提供者组件
export const DragDropProvider: React.FC<DragDropProviderProps> = ({ children }) => {
  const [activeDragData, setActiveDragData] = useState<DraggableData['nodeType'] | null>(null);
  const [pointerPosition, setPointerPosition] = useState({ x: 0, y: 0 });

  const { addNode: _addNode } = useWorkflowStore(); // Temporarily unused
  const { createNode, calculateNodePosition } = useNodeContext();
  const reactFlowInstance = useReactFlow();
  const canvasRef = useRef<HTMLDivElement>(null);

  // 处理拖拽开始
  const handleDragStart = useCallback((event: DragStartEvent) => {
    const data = event.active.data.current as DraggableData;
    if (data?.nodeType) {
      setActiveDragData(data.nodeType);
    }
  }, []);

  // 处理拖拽移动
  const handleDragMove = useCallback((event: DragOverEvent) => {
    const activatorEvent = event.activatorEvent as MouseEvent;
    if (activatorEvent) {
      setPointerPosition({
        x: activatorEvent.clientX,
        y: activatorEvent.clientY,
      });
    }
  }, []);

  // 处理拖拽结束
  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;
      const data = active.data.current as DraggableData;

      // 清理状态
      setActiveDragData(null);

      // 检查是否拖拽到画布上
      if (!data?.nodeType || over?.id !== 'canvas-drop-zone') {
        return;
      }

      // 计算在画布中的位置
      const bounds = canvasRef.current?.getBoundingClientRect();
      if (!bounds) return;

      const localPosition = {
        x: pointerPosition.x - bounds.left,
        y: pointerPosition.y - bounds.top,
      };

      // 将屏幕坐标转换为画布坐标 - simplified for now
      const canvasPosition = localPosition; // TODO: Implement proper coordinate transformation

      // 计算合适的位置避免重叠 - 简化版本，不依赖现有节点
      const finalPosition = calculateNodePosition([], canvasPosition);

      // 创建新节点
      const newNode = createNode(data.nodeType.type || 'unknown', finalPosition);
      // TODO: Fix type mismatch between Node and WorkflowNode
      console.log('Created node:', newNode);
      // addNode(newNode); // Temporarily disabled due to type mismatch
    },
    [pointerPosition, createNode, calculateNodePosition, reactFlowInstance]
  );

  return (
    <DndContext onDragStart={handleDragStart} onDragMove={handleDragMove} onDragEnd={handleDragEnd}>
      <div ref={canvasRef} style={{ width: '100%', height: '100%' }}>
        {children}
      </div>

      <DragOverlay>
        <DragPreview data={activeDragData} />
      </DragOverlay>
    </DndContext>
  );
};

// 拖拽上下文 Hook
export const useDragDrop = () => {
  const [isDragging, setIsDragging] = useState(false);
  const [draggedNodeType, setDraggedNodeType] = useState<string | null>(null);

  const startDrag = useCallback((nodeType: string) => {
    setIsDragging(true);
    setDraggedNodeType(nodeType);
  }, []);

  const endDrag = useCallback(() => {
    setIsDragging(false);
    setDraggedNodeType(null);
  }, []);

  return {
    isDragging,
    draggedNodeType,
    startDrag,
    endDrag,
  };
};

export default DragDropProvider;
