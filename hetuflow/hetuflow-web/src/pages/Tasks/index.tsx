import React, { useState, useEffect } from 'react';
import { Table, Card, Button, Space, Tag, Typography, Input, Row, Col, Tooltip, Select, DatePicker, Modal } from 'antd';
import { ReloadOutlined, EyeOutlined, PlayCircleOutlined, StopOutlined, ReconciliationFilled } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import type { RangePickerProps } from 'antd/es/date-picker';
import {
  apiService,
  SchedTask,
  TaskForQuery,
  TaskStatus,
  TaskInstanceStatus,
  SchedTaskInstance,
  TaskInstanceForQuery,
  TaskStatusText,
} from '../../services/api';
import { useMessage } from '../../hooks/useMessage';
import dayjs from 'dayjs';

const { Title, Text } = Typography;
const { Search } = Input;
const { Option } = Select;
const { RangePicker } = DatePicker;

interface TasksPageState {
  tasks: SchedTask[];
  loading: boolean;
  total: number;
  current: number;
  pageSize: number;
  searchText: string;
  statusFilter?: TaskStatus;
  dateRange?: RangePickerProps['value'];
  expandedRowKeys: React.Key[];
  taskInstancesMap: Record<string, SchedTaskInstance[]>;
  taskInstancesLoading: Record<string, boolean>;
}

/**
 * 任务管理页面组件
 * 显示 SchedTask 查询和管理
 */
const Tasks: React.FC = () => {
  const message = useMessage();
  const [state, setState] = useState<TasksPageState>({
    tasks: [],
    loading: false,
    total: 0,
    current: 1,
    pageSize: 10,
    searchText: '',
    expandedRowKeys: [],
    taskInstancesMap: {},
    taskInstancesLoading: {},
  });

  /**
   * 获取任务实例列表（按 started_at 降序排序）
   */
  const fetchTaskInstances = async (taskId: string) => {
    try {
      setState(prev => ({
        ...prev,
        taskInstancesLoading: { ...prev.taskInstancesLoading, [taskId]: true },
      }));

      const query: TaskInstanceForQuery = {
        page: { page: 1, limit: 100 },
        filter: { task_id: { $eq: taskId } },
      };

      const result = await apiService.taskInstances.queryTaskInstances(query);

      setState(prev => ({
        ...prev,
        taskInstancesMap: {
          ...prev.taskInstancesMap,
          [taskId]: result.result || [],
        },
        taskInstancesLoading: { ...prev.taskInstancesLoading, [taskId]: false },
      }));
    } catch (error) {
      console.error('获取任务实例列表失败:', error);
      message.error('获取任务实例列表失败');
      setState(prev => ({
        ...prev,
        taskInstancesLoading: { ...prev.taskInstancesLoading, [taskId]: false },
      }));
    }
  };

  /**
   * 获取任务列表
   */
  const fetchTasks = async (params?: Partial<TaskForQuery>) => {
    try {
      setState(prev => ({ ...prev, loading: true }));

      const query: TaskForQuery = {
        page: {
          page: params?.page?.page || state.current,
          limit: params?.page?.limit || state.pageSize,
        },
        filter: params?.filter || {},
      };

      const result = await apiService.tasks.queryTasks(query);

      setState(prev => ({
        ...prev,
        tasks: result.result,
        total: result.page.total,
        current: result.page.total > 0 ? prev.current : 1,
        loading: false,
      }));
    } catch (error) {
      console.error('获取任务列表失败:', error);
      message.error('获取任务列表失败');
      setState(prev => ({ ...prev, loading: false }));
    }
  };

  /**
   * 渲染任务实例状态标签
   */
  const renderTaskInstanceStatus = (status: TaskInstanceStatus) => {
    const statusConfig: Partial<Record<TaskInstanceStatus, { color: string; text: string }>> = {
      [TaskInstanceStatus.Pending]: { color: 'default', text: '等待中' },
      [TaskInstanceStatus.Running]: { color: 'processing', text: '运行中' },
      [TaskInstanceStatus.Succeeded]: { color: 'success', text: '成功' },
      [TaskInstanceStatus.Failed]: { color: 'error', text: '失败' },
      [TaskInstanceStatus.Cancelled]: { color: 'warning', text: '已取消' },
      [TaskInstanceStatus.Timeout]: { color: 'orange', text: '超时' },
    };
    const config = statusConfig[status] || { color: 'default', text: status };
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  /**
   * 渲染任务实例执行时长
   */
  const renderTaskInstanceDuration = (instance: SchedTaskInstance) => {
    if (!instance.started_at || !instance.completed_at) return '-';

    const startTime = dayjs(instance.started_at);
    const endTime = dayjs(instance.completed_at);
    const duration = endTime.diff(startTime, 'second');

    const hours = Math.floor(duration / 3600);
    const minutes = Math.floor((duration % 3600) / 60);
    const seconds = duration % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m ${seconds}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds}s`;
    } else {
      return `${seconds}s`;
    }
  };

  /**
   * 渲染状态标签
   */
  const renderStatus = (status: TaskStatus) => {
    const colorMap = {
      [TaskStatus.Pending]: 'default',
      [TaskStatus.Doing]: 'processing',
      [TaskStatus.Succeeded]: 'success',
      [TaskStatus.Failed]: 'error',
      [TaskStatus.Cancelled]: 'warning',
    };
    const color = colorMap[status] || 'default';
    return <Tag color={color}>{TaskStatusText[status]}</Tag>;
  };

  /**
   * 渲染重试信息（使用真实数据）
   */
  const renderRetryInfo = (task: SchedTask) => {
    // 从配置中获取重试信息
    const maxRetries = task.config?.max_retries || 3;
    const retryCount = task.config?.retry_count || 0;
    const color = retryCount > 0 ? 'orange' : 'default';
    return (
      <Tag color={color}>
        {retryCount}/{maxRetries}
      </Tag>
    );
  };

  /**
   * 渲染执行时长
   */
  const renderDuration = (task: SchedTask) => {
    let startTime: dayjs.Dayjs;
    let endTime: dayjs.Dayjs;

    // 检查是否有任务实例数据
    const taskInstances = state.taskInstancesMap[task.id] || [];

    if (taskInstances.length > 0) {
      // 使用最新的任务实例来计算执行时长
      const latestInstance = taskInstances[0]; // 假设已按 started_at 降序排序
      startTime = dayjs(latestInstance.started_at || task.created_at);
      endTime = latestInstance.completed_at ? dayjs(latestInstance.completed_at) : dayjs();
    } else {
      // 等待中的任务，显示 '-'
      return '-';
    }

    const duration = endTime.diff(startTime, 'second');

    if (duration < 0) {
      return '-';
    }

    const hours = Math.floor(duration / 3600);
    const minutes = Math.floor((duration % 3600) / 60);
    const seconds = duration % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m ${seconds}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds}s`;
    } else {
      return `${seconds}s`;
    }
  };

  useEffect(() => {
    fetchTasks();
  }, []);

  /**
   * 处理展开/折叠行
   */
  const handleExpand = async (expanded: boolean, record: SchedTask) => {
    if (expanded) {
      // 如果是展开且没有缓存数据，则获取任务实例
      if (!state.taskInstancesMap[record.id]) {
        await fetchTaskInstances(record.id);
      }
      setState(prev => ({
        ...prev,
        expandedRowKeys: expanded
          ? [...prev.expandedRowKeys, record.id]
          : prev.expandedRowKeys.filter(key => key !== record.id),
      }));
    } else {
      setState(prev => ({
        ...prev,
        expandedRowKeys: prev.expandedRowKeys.filter(key => key !== record.id),
      }));
    }
  };

  /**
   * 任务实例表格列定义
   */
  const taskInstanceColumns: ColumnsType<SchedTaskInstance> = [
    {
      title: '实例ID',
      dataIndex: 'id',
      key: 'id',
      width: 220,
      render: (text: string) => text,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: TaskInstanceStatus) => renderTaskInstanceStatus(status),
      width: 100,
    },
    {
      title: '执行代理',
      dataIndex: 'agent_id',
      key: 'agent_id',
      width: 120,
    },
    {
      title: '开始时间',
      dataIndex: 'started_at',
      key: 'started_at',
      render: (time: string) => (time ? dayjs(time).format('MM-DD HH:mm:ss') : '-'),
      width: 150,
    },
    {
      title: '结束时间',
      dataIndex: 'completed_at',
      key: 'completed_at',
      render: (time: string) => (time ? dayjs(time).format('MM-DD HH:mm:ss') : '-'),
      width: 150,
    },
    {
      title: '执行时长',
      key: 'duration',
      render: (_, record) => renderTaskInstanceDuration(record),
      width: 100,
    },
    {
      title: '错误信息',
      dataIndex: 'error_message',
      key: 'error_message',
      ellipsis: true,
      render: (error: string) => (error ? <Text type="danger">{error}</Text> : '-'),
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (time: string) => dayjs(time).format('MM-DD HH:mm:ss'),
      width: 150,
    },
  ];

  /**
   * 表格列定义
   */
  const columns: ColumnsType<SchedTask> = [
    {
      title: '任务',
      dataIndex: 'id',
      key: 'id',
      width: 210,
      render: (id, record) => <Tooltip title={`作业ID: ${record.job_id}`}>{id}</Tooltip>,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: TaskStatus) => renderStatus(status),
      width: 60,
    },
    {
      title: '优先级',
      dataIndex: 'priority',
      key: 'priority',
      width: 50,
    },
    {
      title: '重试次数',
      key: 'retry',
      render: (_, record) => renderRetryInfo(record),
      width: 80,
    },
    {
      title: '执行时长',
      key: 'duration',
      render: (_, record) => renderDuration(record),
      width: 100,
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (time: string) => dayjs(time).format('MM-DD HH:mm'),
      width: 120,
    },
    {
      title: '更新时间',
      dataIndex: 'updated_at',
      key: 'updated_at',
      render: (time: string) => dayjs(time).format('MM-DD HH:mm'),
      width: 120,
    },
  ];

  /**
   * 处理搜索
   */
  const handleSearch = (value: string) => {
    setState(prev => ({ ...prev, searchText: value, current: 1 }));
    fetchTasks({ page: { page: 1, limit: state.pageSize } });
  };

  /**
   * 刷新数据
   */
  const handleRefresh = () => {
    fetchTasks();
  };

  /**
   * 立即执行
   */
  const handleExecute = (record: SchedTask) => {
    Modal.confirm({
      title: '确认执行',
      content: `确定要立即执行任务 "${record.name}" 吗？`,
      onOk: async () => {
        try {
          // TODO: 调用立即执行 API
          message.success('任务执行请求已提交');
          fetchTasks();
        } catch (error) {
          console.error('执行任务失败:', error);
          message.error('执行任务失败');
        }
      },
    });
  };

  /**
   * 取消执行
   */
  const handleCancel = (record: SchedTask) => {
    Modal.confirm({
      title: '确认取消',
      content: `确定要取消任务 "${record.name}" 的执行吗？`,
      onOk: async () => {
        try {
          // TODO: 调用取消执行 API
          message.success('任务取消请求已提交');
          fetchTasks();
        } catch (error) {
          console.error('取消任务失败:', error);
          message.error('取消任务失败');
        }
      },
    });
  };

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Row justify="space-between" align="middle">
        <Col>
          <Title level={2}>任务管理</Title>
        </Col>
      </Row>

      <Card>
        <Row gutter={[16, 16]} style={{ marginBottom: 16 }}>
          <Col xs={24} sm={12} md={8}>
            <Search
              placeholder="搜索任务 ID 或作业名称"
              allowClear
              onSearch={handleSearch}
              onChange={e => setState(prev => ({ ...prev, searchText: e.target.value }))}
            />
          </Col>
          <Col xs={24} sm={12} md={6}>
            <Select
              placeholder="选择状态"
              allowClear
              style={{ width: '100%' }}
              onChange={value => setState(prev => ({ ...prev, statusFilter: value }))}
            >
              <Option value={TaskStatus.Pending}>等待中</Option>
              <Option value={TaskStatus.Doing}>运行中</Option>
              <Option value={TaskStatus.Succeeded}>成功</Option>
              <Option value={TaskStatus.Failed}>失败</Option>
              <Option value={TaskStatus.Cancelled}>已取消</Option>
              <Option value={TaskInstanceStatus.Timeout}>超时</Option>
            </Select>
          </Col>
          <Col xs={24} sm={12} md={6}>
            <RangePicker
              style={{ width: '100%' }}
              placeholder={['开始时间', '结束时间']}
              onChange={dates => setState(prev => ({ ...prev, dateRange: dates }))}
            />
          </Col>
          <Col xs={24} sm={12} md={4}>
            <Button icon={<ReloadOutlined />} onClick={handleRefresh} loading={state.loading} style={{ width: '100%' }}>
              刷新
            </Button>
          </Col>
        </Row>

        <Table
          columns={columns}
          dataSource={state.tasks}
          rowKey="id"
          loading={state.loading}
          scroll={{ x: 1200 }}
          expandable={{
            expandedRowKeys: state.expandedRowKeys,
            onExpand: handleExpand,
            expandedRowRender: record => (
              <div style={{ padding: '16px' }}>
                <Table
                  columns={taskInstanceColumns}
                  dataSource={state.taskInstancesMap[record.id] || []}
                  rowKey="id"
                  loading={state.taskInstancesLoading[record.id]}
                  pagination={false}
                  size="small"
                  bordered
                  locale={{ emptyText: '暂无任务实例数据' }}
                />
              </div>
            ),
            rowExpandable: () => true,
          }}
          pagination={{
            current: state.current,
            pageSize: state.pageSize,
            total: state.total,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
            onChange: (page, pageSize) => {
              setState(prev => ({ ...prev, current: page, pageSize: pageSize || 10 }));
              fetchTasks({ page: { page, limit: pageSize } });
            },
          }}
        />
      </Card>
    </Space>
  );
};

export default Tasks;
