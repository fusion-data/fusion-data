import React, { useState } from "react";
import { Card, Space, Typography, Row, Col, Divider, List, Button, message } from "antd";
import {
  DragOutlined,
  SwapOutlined,
  AppstoreOutlined,
  CheckCircleOutlined,
  ClockCircleOutlined,
  ExclamationCircleOutlined,
  HolderOutlined,
} from "@ant-design/icons";
import {
  DndContext,
  DragEndEvent,
  DragOverEvent,
  DragStartEvent,
  PointerSensor,
  useSensor,
  useSensors,
  DragOverlay,
  closestCorners,
  UniqueIdentifier,
  useDroppable,
} from "@dnd-kit/core";
import {
  SortableContext,
  verticalListSortingStrategy,
  horizontalListSortingStrategy,
  useSortable,
  arrayMove,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

const { Title, Paragraph, Text } = Typography;

// 基础拖拽卡片组件
interface DraggableCardProps {
  id: string;
  title: string;
  description: string;
  icon?: React.ReactNode;
}

function DraggableCard({ id, title, description, icon }: DraggableCardProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <Card
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      hoverable
      size="small"
      title={
        <Space>
          {icon}
          {title}
          <DragOutlined style={{ color: "#999" }} />
        </Space>
      }
      className="draggable-card"
    >
      <Paragraph style={{ margin: 0, fontSize: "12px" }}>{description}</Paragraph>
    </Card>
  );
}

// 任务项组件
interface TaskItemProps {
  id: string;
  content: string;
  status: "todo" | "doing" | "done";
}

function TaskItem({ id, content, status }: TaskItemProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const getStatusIcon = () => {
    switch (status) {
      case "todo":
        return <ClockCircleOutlined style={{ color: "#999" }} />;
      case "doing":
        return <ExclamationCircleOutlined style={{ color: "#1890ff" }} />;
      case "done":
        return <CheckCircleOutlined style={{ color: "#52c41a" }} />;
    }
  };

  const getStatusText = () => {
    switch (status) {
      case "todo":
        return "待办";
      case "doing":
        return "进行中";
      case "done":
        return "已完成";
    }
  };

  return (
    <List.Item ref={setNodeRef} style={style} {...attributes} {...listeners} className="task-item">
      <List.Item.Meta
        avatar={getStatusIcon()}
        title={
          <Space>
            {content}
            <DragOutlined style={{ color: "#ccc", fontSize: "12px" }} />
          </Space>
        }
        description={getStatusText()}
      />
    </List.Item>
  );
}

// 看板列组件
interface KanbanColumnProps {
  id: string;
  title: string;
  items: TaskItemProps[];
  color: string;
}

function KanbanColumn({ id, title, items, color }: KanbanColumnProps) {
  const { setNodeRef, isOver } = useDroppable({
    id,
  });

  return (
    <Card
      ref={setNodeRef}
      title={
        <Space>
          <AppstoreOutlined style={{ color }} />
          {title}
          <Text type="secondary">({items.length})</Text>
        </Space>
      }
      size="small"
      style={{
        height: "400px",
        overflow: "auto",
        backgroundColor: isOver ? "#f6ffed" : undefined,
        borderColor: isOver ? "#52c41a" : undefined,
      }}
    >
      <SortableContext items={items.map((item) => item.id)} strategy={verticalListSortingStrategy}>
        {items.length > 0 ? (
          <List dataSource={items} renderItem={(item) => <TaskItem key={item.id} {...item} />} size="small" />
        ) : (
          <div
            style={{
              height: "200px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: "#bfbfbf",
              fontSize: "14px",
            }}
          >
            拖拽任务到此处
          </div>
        )}
      </SortableContext>
    </Card>
  );
}

function DragDropDemo() {
  // 基础拖拽卡片数据
  const [basicCards, setBasicCards] = useState([
    { id: "1", title: "数据处理", description: "高性能数据流处理引擎", icon: <HolderOutlined /> },
    { id: "2", title: "AI Agent", description: "智能代理工作流编排", icon: <SwapOutlined /> },
    { id: "3", title: "任务调度", description: "分布式任务调度系统", icon: <AppstoreOutlined /> },
  ]);

  // 看板数据
  const [kanbanData, setKanbanData] = useState<{
    todo: TaskItemProps[];
    doing: TaskItemProps[];
    done: TaskItemProps[];
  }>({
    todo: [
      { id: "task-1", content: "设计用户界面", status: "todo" as const },
      { id: "task-2", content: "编写API文档", status: "todo" as const },
      { id: "task-3", content: "数据库设计", status: "todo" as const },
    ],
    doing: [
      { id: "task-4", content: "实现拖拽功能", status: "doing" as const },
      { id: "task-5", content: "优化性能", status: "doing" as const },
    ],
    done: [
      { id: "task-6", content: "项目初始化", status: "done" as const },
      { id: "task-7", content: "环境配置", status: "done" as const },
    ],
  });

  const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
  );

  // 处理基础拖拽结束
  const handleBasicDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (active.id !== over?.id) {
      setBasicCards((items) => {
        const oldIndex = items.findIndex((item) => item.id === active.id);
        const newIndex = items.findIndex((item) => item.id === over?.id);
        return arrayMove(items, oldIndex, newIndex);
      });
    }
    setActiveId(null);
  };

  const [draggedItemContainer, setDraggedItemContainer] = useState<string | null>(null);

  // 处理拖拽悬停事件
  const handleKanbanDragOver = (event: DragOverEvent) => {
    const { active, over } = event;

    if (!over) return;

    const activeContainer = findContainer(active.id);
    const overContainer = findContainer(over.id) || over.id;

    if (!activeContainer || !overContainer || activeContainer === overContainer) {
      return;
    }

    // 记录拖拽元素的原始容器
    if (draggedItemContainer === null) {
      setDraggedItemContainer(activeContainer as string);
    }

    // 在不同列之间移动时的实时更新
    setKanbanData((prev) => {
      const activeItems = prev[activeContainer as keyof typeof prev];
      const overItems = prev[overContainer as keyof typeof prev];
      const activeIndex = activeItems.findIndex((item) => item.id === active.id);
      const overIndex = over.id in prev ? overItems.length : overItems.findIndex((item) => item.id === over.id);

      const movedItem = { ...activeItems[activeIndex], status: overContainer as any };

      return {
        ...prev,
        [activeContainer]: activeItems.filter((item) => item.id !== active.id),
        [overContainer]: [...overItems.slice(0, overIndex), movedItem, ...overItems.slice(overIndex)],
      };
    });
  };
  // 处理看板拖拽结束
  const handleKanbanDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (!over) {
      setActiveId(null);
      setDraggedItemContainer(null);
      return;
    }

    const activeContainer = findContainer(active.id);
    const overContainer = findContainer(over.id) || over.id;

    if (!activeContainer || !overContainer) {
      setActiveId(null);
      setDraggedItemContainer(null);
      return;
    }

    if (activeContainer === overContainer) {
      // 同一列内排序
      const items = kanbanData[activeContainer as keyof typeof kanbanData];
      const oldIndex = items.findIndex((item) => item.id === active.id);
      const newIndex = items.findIndex((item) => item.id === over.id);

      if (oldIndex !== newIndex) {
        setKanbanData((prev) => ({
          ...prev,
          [activeContainer]: arrayMove(items, oldIndex, newIndex),
        }));
      }
    } else {
      // 跨列移动时显示成功消息
      if (draggedItemContainer !== overContainer) {
        message.success(
          `任务已移动到${overContainer === "todo" ? "待办" : overContainer === "doing" ? "进行中" : "已完成"}`
        );
      }
    }

    setActiveId(null);
    setDraggedItemContainer(null);
  };

  const findContainer = (id: UniqueIdentifier) => {
    if (id in kanbanData) {
      return id;
    }

    return Object.keys(kanbanData).find((key) =>
      kanbanData[key as keyof typeof kanbanData].some((item) => item.id === id)
    );
  };

  const handleDragStart = (event: DragStartEvent) => {
    setActiveId(event.active.id);
    setDraggedItemContainer(null); // 重置状态
  };

  const resetBasicCards = () => {
    setBasicCards([
      { id: "1", title: "数据处理", description: "高性能数据流处理引擎", icon: <HolderOutlined /> },
      { id: "2", title: "AI Agent", description: "智能代理工作流编排", icon: <SwapOutlined /> },
      { id: "3", title: "任务调度", description: "分布式任务调度系统", icon: <AppstoreOutlined /> },
    ]);
    message.success("基础拖拽卡片已重置");
  };

  const resetKanban = () => {
    setKanbanData({
      todo: [
        { id: "task-1", content: "设计用户界面", status: "todo" },
        { id: "task-2", content: "编写API文档", status: "todo" },
        { id: "task-3", content: "数据库设计", status: "todo" },
      ],
      doing: [
        { id: "task-4", content: "实现拖拽功能", status: "doing" },
        { id: "task-5", content: "优化性能", status: "doing" },
      ],
      done: [
        { id: "task-6", content: "项目初始化", status: "done" },
        { id: "task-7", content: "环境配置", status: "done" },
      ],
    });
    message.success("看板数据已重置");
  };

  return (
    <div style={{ padding: "16px" }}>
      <Title level={3}>
        <HolderOutlined /> 拖拽功能演示
      </Title>
      <Paragraph>
        使用 <Text code>@dnd-kit</Text> 实现的多种拖拽交互效果，支持排序、跨容器移动等功能。
      </Paragraph>

      <Divider />

      {/* 基础拖拽排序 */}
      <Title level={4}>1. 基础卡片排序</Title>
      <Paragraph>
        拖动下面的卡片来重新排序：
        <Button type="link" size="small" onClick={resetBasicCards}>
          重置
        </Button>
      </Paragraph>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCorners}
        onDragStart={handleDragStart}
        onDragEnd={handleBasicDragEnd}
      >
        <SortableContext items={basicCards.map((card) => card.id)} strategy={horizontalListSortingStrategy}>
          <Row gutter={[16, 16]}>
            {basicCards.map((card) => (
              <Col key={card.id} xs={24} sm={12} md={8}>
                <DraggableCard {...card} />
              </Col>
            ))}
          </Row>
        </SortableContext>
      </DndContext>

      <Divider />

      {/* 看板拖拽 */}
      <Title level={4}>2. 看板任务管理</Title>
      <Paragraph>
        拖动任务在不同状态列之间移动，或在同一列内重新排序：
        <Button type="link" size="small" onClick={resetKanban}>
          重置
        </Button>
      </Paragraph>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCorners}
        onDragStart={handleDragStart}
        onDragOver={handleKanbanDragOver}
        onDragEnd={handleKanbanDragEnd}
      >
        <Row gutter={[16, 16]}>
          <Col xs={24} md={8}>
            <KanbanColumn id="todo" title="待办事项" items={kanbanData.todo} color="#999" />
          </Col>
          <Col xs={24} md={8}>
            <KanbanColumn id="doing" title="进行中" items={kanbanData.doing} color="#1890ff" />
          </Col>
          <Col xs={24} md={8}>
            <KanbanColumn id="done" title="已完成" items={kanbanData.done} color="#52c41a" />
          </Col>
        </Row>

        <DragOverlay>
          {activeId ? (
            <Card size="small" style={{ opacity: 0.8, transform: "rotate(5deg)" }}>
              拖拽中...
            </Card>
          ) : null}
        </DragOverlay>
      </DndContext>

      <Divider />

      <Paragraph type="secondary">
        💡 提示：
        <ul>
          <li>拖拽需要按住并移动一定距离才会激活，避免误触</li>
          <li>基础卡片支持水平排序</li>
          <li>看板支持跨列移动和同列排序</li>
          <li>使用了 Ant Design 组件保持界面一致性</li>
        </ul>
      </Paragraph>
    </div>
  );
}

export default DragDropDemo;
