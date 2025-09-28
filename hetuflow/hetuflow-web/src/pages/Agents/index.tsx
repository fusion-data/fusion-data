import React, { useState, useEffect } from 'react';
import { Table, Card, Button, Space, Tag, Typography, Input, Row, Col, Tooltip, Badge, Popconfirm } from 'antd';
import {
  ReloadOutlined,
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import {
  apiService,
  SchedAgent,
  AgentForQuery,
  AgentStatistics,
  AgentCapabilities,
  AgentStatus,
  AgentStatusColor,
  AgentStatusText,
} from '../../services/api';
import { useMessage } from '../../hooks/useMessage';
import dayjs from 'dayjs';

const { Title } = Typography;
const { Search } = Input;

interface AgentsPageState {
  agents: SchedAgent[];
  loading: boolean;
  total: number;
  current: number;
  pageSize: number;
  searchText: string;
}

/**
 * 执行代理管理管理页面组件
 * 显示 Agent 列表和配置管理
 */
const Agents: React.FC = () => {
  const message = useMessage();
  const [state, setState] = useState<AgentsPageState>({
    agents: [],
    loading: false,
    total: 0,
    current: 1,
    pageSize: 10,
    searchText: '',
  });
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);

  /**
   * 获取代理列表
   */
  const fetchAgents = async (params?: Partial<AgentForQuery>) => {
    try {
      setState(prev => ({ ...prev, loading: true }));

      const query: AgentForQuery = {
        page: {
          page: params?.page?.page || state.current,
          limit: params?.page?.limit || state.pageSize,
        },
        filter: params?.filter || {},
      };

      const result = await apiService.agents.queryAgents(query);

      setState(prev => ({
        ...prev,
        agents: result.result || [],
        total: result.page.total || 0,
        current: query.page.page || 1,
        loading: false,
      }));
    } catch (error) {
      console.error('获取代理列表失败:', error);
      message.error('获取代理列表失败');
      setState(prev => ({ ...prev, loading: false }));
    }
  };

  /**
   * 渲染状态标签（使用模拟状态）
   */
  const renderStatus = (status: AgentStatus) => {
    const color = AgentStatusColor[status];
    return <Tag color={color}>{AgentStatusText[status]}</Tag>;
  };

  /**
   * 渲染任务统计
   */
  const renderTaskStats = (stats: AgentStatistics) => {
    const successed = stats.success_tasks || 0;
    const failed = stats.failure_tasks || 0;
    const running = (stats.total_tasks || 0) - successed - failed;
    return (
      <Space>
        <Tooltip title="失败任务">
          <Badge count={failed} color="red" />
        </Tooltip>
        <Tooltip title="运行中任务">
          <Badge count={running} color="blue" />
        </Tooltip>
        <Tooltip title="成功任务">
          <Badge count={successed} color="green" />
        </Tooltip>
      </Space>
    );
  };

  useEffect(() => {
    fetchAgents();
  }, []);

  /**
   * 表格列定义
   */
  const columns: ColumnsType<SchedAgent> = [
    {
      title: '代理 ID',
      dataIndex: 'id',
      key: 'id',
      render: (id: string, item) => <Tooltip title={item.name || id}>{id}</Tooltip>,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: renderStatus,
      width: 100,
    },
    {
      title: '能力',
      dataIndex: 'capabilities',
      key: 'capabilities',
      render: (capabilities: AgentCapabilities) => (
        <Tooltip title={JSON.stringify(capabilities)}>
          <Space wrap>
            {capabilities.labels ? (
              Object.entries(capabilities.labels).map(([key, value]) => (
                <Tag key={key}>
                  {key}={value}
                </Tag>
              ))
            ) : (
              <span>无标签</span>
            )}
          </Space>
        </Tooltip>
      ),
    },
    {
      title: '任务统计',
      dataIndex: 'statistics',
      key: 'statistics',
      render: renderTaskStats,
      width: 120,
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (time: string) => dayjs(time).format('YYYY-MM-DD HH:mm:ss'),
      width: 150,
    },
    {
      title: '更新时间',
      dataIndex: 'updated_at',
      key: 'updated_at',
      render: (time: string) => dayjs(time).format('YYYY-MM-DD HH:mm:ss'),
      width: 150,
    },
    {
      title: '操作',
      key: 'action',
      render: (_, record) => (
        <Space size="small">
          <Tooltip title="启动">
            <Button type="text" icon={<PlayCircleOutlined />} onClick={() => handleStart(record)} />
          </Tooltip>
          <Tooltip title="暂停">
            <Button type="text" icon={<PauseCircleOutlined />} onClick={() => handlePause(record)} />
          </Tooltip>
          <Tooltip title="编辑">
            <Button type="text" icon={<EditOutlined />} onClick={() => handleEdit(record)} />
          </Tooltip>
          <Tooltip title="删除">
            <Popconfirm
              title="确认删除"
              description={`确定要删除代理 "${record.name}" 吗？`}
              okText="确定"
              cancelText="取消"
              onConfirm={() => handleDelete(record)}
            >
              <Button type="text" danger icon={<DeleteOutlined />} />
            </Popconfirm>
          </Tooltip>
        </Space>
      ),
      width: 120,
    },
  ];

  /**
   * 处理搜索
   */
  const handleSearch = (value: string) => {
    setState(prev => ({ ...prev, searchText: value, current: 1 }));
    fetchAgents({ page: { page: 1, limit: state.pageSize } });
  };

  /**
   * 刷新数据
   */
  const handleRefresh = () => {
    fetchAgents();
  };

  /**
   * 添加代理
   */
  const handleAdd = () => {
    // TODO: 打开添加代理对话框
    message.info('添加代理功能开发中');
  };

  /**
   * 编辑代理
   */
  const handleEdit = (record: SchedAgent) => {
    // TODO: 打开编辑代理对话框
    message.info(`编辑代理 "${record.name}" 功能开发中`);
  };

  /**
   * 删除代理
   */
  const handleDelete = async (record: SchedAgent) => {
    try {
      await apiService.agents.deleteAgent(record.id);
      message.success('删除成功');
      fetchAgents();
    } catch (error) {
      console.error('删除代理失败:', error);
      message.error('删除代理失败');
    }
  };

  /**
   * 启动代理
   */
  const handleStart = (record: SchedAgent) => {
    // TODO: 调用启动 API
    message.info(`启动代理 "${record.name}" 功能开发中`);
  };

  /**
   * 暂停代理
   */
  const handlePause = (record: SchedAgent) => {
    // TODO: 调用暂停 API
    message.info(`暂停代理 "${record.name}" 功能开发中`);
  };

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Row justify="space-between" align="middle">
        <Col>
          <Title level={2}>执行代理管理管理</Title>
        </Col>
      </Row>

      <Card>
        <Row justify="space-between" style={{ marginBottom: 16 }}>
          <Col>
            <Space>
              <Search
                placeholder="搜索代理 ID、描述或地址"
                allowClear
                style={{ width: 300 }}
                onSearch={handleSearch}
                onChange={e => setState(prev => ({ ...prev, searchText: e.target.value }))}
              />
            </Space>
          </Col>
          <Col>
            <Space>
              <Button icon={<ReloadOutlined />} onClick={handleRefresh} loading={state.loading}>
                刷新
              </Button>
              <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>
                添加代理
              </Button>
            </Space>
          </Col>
        </Row>

        <Table
          columns={columns}
          dataSource={state.agents}
          rowKey="id"
          loading={state.loading}
          rowSelection={{
            selectedRowKeys,
            onChange: setSelectedRowKeys,
          }}
          pagination={{
            current: state.current,
            pageSize: state.pageSize,
            total: state.total,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
            onChange: (page, pageSize) => {
              setState(prev => ({
                ...prev,
                current: page,
                pageSize: pageSize || 10,
              }));
              fetchAgents({ page: { page, limit: pageSize } });
            },
          }}
        />
      </Card>
    </Space>
  );
};

export default Agents;
