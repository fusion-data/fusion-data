import React, { useState, useCallback } from 'react';
import {
  Input,
  Radio,
  Space,
  Typography,
  Collapse,
  Badge,
  Empty,
} from 'antd';
import {
  SearchOutlined,
  RobotOutlined,
  DatabaseOutlined,
  PlayCircleOutlined,
  ClockCircleOutlined,
  ApiOutlined,
  BranchesOutlined,
  SettingOutlined,
} from '@ant-design/icons';
import { DndContext, useDraggable } from '@dnd-kit/core';
import { NodeFactory } from '../nodes/NodeFactory';
// import { useNodeContext } from '../nodes/NodeContext'; // Not used yet

const { Text } = Typography;
const { Panel } = Collapse;

interface DraggableNodeItemProps {
  nodeType: {
    type: string;
    displayName: string;
    description: string;
    icon: string;
    category: string;
    color: string;
  };
}

// 可拖拽的节点项
const DraggableNodeItem: React.FC<DraggableNodeItemProps> = ({ nodeType }) => {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: `node-${nodeType.type}`,
    data: {
      nodeType,
    },
  });

  const getIcon = (iconName: string) => {
    const iconMap: Record<string, React.ReactNode> = {
      RobotOutlined: <RobotOutlined />,
      DatabaseOutlined: <DatabaseOutlined />,
      PlayCircleOutlined: <PlayCircleOutlined />,
      ClockCircleOutlined: <ClockCircleOutlined />,
      ApiOutlined: <ApiOutlined />,
      BranchesOutlined: <BranchesOutlined />,
      SettingOutlined: <SettingOutlined />,
    };
    return iconMap[iconName] || <SettingOutlined />;
  };

  return (
    <div
      ref={setNodeRef}
      {...attributes}
      {...listeners}
      className={`draggable-node ${isDragging ? 'dragging' : ''}`}
      style={{
        padding: '8px 12px',
        margin: '4px 0',
        background: isDragging ? 'var(--bg-secondary)' : 'var(--bg-primary)',
        border: `1px solid var(--border-primary)`,
        borderRadius: '6px',
        cursor: 'grab',
        transition: 'all 0.2s ease',
        userSelect: 'none',
      }}
    >
      <Space align="start">
        <div
          style={{
            fontSize: '16px',
            color: nodeType.color,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: '24px',
            height: '24px',
          }}
        >
          {getIcon(nodeType.icon)}
        </div>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              fontSize: '14px',
              fontWeight: 500,
              color: 'var(--text-primary)',
              marginBottom: '2px',
            }}
          >
            {nodeType.displayName}
          </div>
          <Text
            type="secondary"
            style={{
              fontSize: '12px',
              display: 'block',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {nodeType.description}
          </Text>
        </div>
      </Space>
    </div>
  );
};

export const NodePanel: React.FC = () => {
  const [searchText, setSearchText] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');

  // useNodeContext() // Context not needed yet but will be useful later

  // 获取所有节点分类
  const categories = [
    { key: 'all', name: '全部节点', icon: <SettingOutlined /> },
    { key: 'triggers', name: '触发器', icon: <PlayCircleOutlined /> },
    { key: 'actions', name: '动作', icon: <DatabaseOutlined /> },
    { key: 'control', name: '控制流', icon: <BranchesOutlined /> },
    { key: 'data', name: '数据处理', icon: <DatabaseOutlined /> },
    { key: 'ai', name: 'AI 智能体', icon: <RobotOutlined /> },
    { key: 'integration', name: '集成服务', icon: <ApiOutlined /> },
  ];

  // 获取所有节点配置
  const nodeFactoryInstance = NodeFactory.getInstance();
  const allNodeConfigs = nodeFactoryInstance.getAllNodeConfigs();

  // 过滤节点
  const filteredNodes = allNodeConfigs.filter(node => {
    const matchesSearch = !searchText ||
      node.displayName.toLowerCase().includes(searchText.toLowerCase()) ||
      node.description.toLowerCase().includes(searchText.toLowerCase());

    const matchesCategory = selectedCategory === 'all' || node.category === selectedCategory;

    return matchesSearch && matchesCategory;
  });

  // 按分类分组节点
  const groupedNodes = categories.reduce((acc, category) => {
    if (category.key === 'all') return acc;

    acc[category.key] = filteredNodes.filter(node => node.category === category.key);
    return acc;
  }, {} as Record<string, typeof allNodeConfigs>);

  // 搜索处理
  const handleSearch = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchText(e.target.value);
  }, []);

  // 分类选择处理
  const handleCategoryChange = useCallback((e: any) => {
    setSelectedCategory(e.target.value);
  }, []);

  return (
    <div className="node-panel" style={{
      width: '280px',
      height: '100%',
      background: 'var(--bg-primary)',
      borderRight: '1px solid var(--border-primary)',
      display: 'flex',
      flexDirection: 'column',
      overflow: 'hidden',
    }}>
      {/* 面板头部 */}
      <div style={{
        padding: '16px',
        borderBottom: '1px solid var(--border-primary)',
      }}>
        <div style={{
          fontSize: '16px',
          fontWeight: 600,
          color: 'var(--text-primary)',
          marginBottom: '12px',
        }}>
          节点库
        </div>

        {/* 搜索框 */}
        <Input
          placeholder="搜索节点..."
          prefix={<SearchOutlined />}
          value={searchText}
          onChange={handleSearch}
          allowClear
          style={{ marginBottom: '12px' }}
        />

        {/* 分类筛选 */}
        <Radio.Group
          value={selectedCategory}
          onChange={handleCategoryChange}
          size="small"
          style={{ width: '100%' }}
        >
          <Space wrap>
            {categories.map(category => (
              <Radio.Button
                key={category.key}
                value={category.key}
                style={{
                  fontSize: '12px',
                  height: '24px',
                  lineHeight: '22px',
                }}
              >
                {category.name}
              </Radio.Button>
            ))}
          </Space>
        </Radio.Group>
      </div>

      {/* 节点列表 */}
      <div style={{
        flex: 1,
        overflow: 'auto',
        padding: '16px',
      }}>
        <DndContext>
          {filteredNodes.length === 0 ? (
            <Empty
              description="没有找到匹配的节点"
              style={{ marginTop: '40px' }}
            />
          ) : selectedCategory === 'all' ? (
            // 显示所有节点
            <div>
              {categories
                .filter(cat => cat.key !== 'all')
                .map(category => {
                  const categoryNodes = groupedNodes[category.key] || [];
                  if (categoryNodes.length === 0) return null;

                  return (
                    <div key={category.key} style={{ marginBottom: '20px' }}>
                      <div style={{
                        fontSize: '13px',
                        fontWeight: 600,
                        color: 'var(--text-secondary)',
                        marginBottom: '8px',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '4px',
                      }}>
                        {category.icon}
                        {category.name}
                        <Badge
                          count={categoryNodes.length}
                          size="small"
                          style={{ marginLeft: '4px' }}
                        />
                      </div>
                      {categoryNodes.map(nodeType => (
                        <DraggableNodeItem
                          key={nodeType.displayName}
                          nodeType={{
                            ...nodeType,
                            type: nodeType.displayName, // Use displayName as type for now
                          }}
                        />
                      ))}
                    </div>
                  );
                })}
            </div>
          ) : (
            // 按分类显示节点
            <Collapse
              ghost
              defaultActiveKey={[selectedCategory]}
              size="small"
            >
              {Object.entries(groupedNodes).map(([categoryKey, nodes]) => {
                if (nodes.length === 0) return null;

                const category = categories.find(cat => cat.key === categoryKey);
                return (
                  <Panel
                    key={categoryKey}
                    header={
                      <span style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                        {category?.icon}
                        {category?.name}
                        <Badge count={nodes.length} size="small" />
                      </span>
                    }
                  >
                    {nodes.map(nodeType => (
                      <DraggableNodeItem
                        key={nodeType.displayName}
                        nodeType={{
                          ...nodeType,
                          type: nodeType.displayName, // Use displayName as type for now
                        }}
                      />
                    ))}
                  </Panel>
                );
              })}
            </Collapse>
          )}
        </DndContext>
      </div>

      {/* 面板底部 */}
      <div style={{
        padding: '12px 16px',
        borderTop: '1px solid var(--border-primary)',
        fontSize: '12px',
        color: 'var(--text-tertiary)',
        textAlign: 'center',
      }}>
        拖拽节点到画布进行添加
      </div>
    </div>
  );
};

export default NodePanel;