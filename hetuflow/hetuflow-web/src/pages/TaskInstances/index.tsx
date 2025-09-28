import React, { useState, useEffect } from 'react';
import {
  Table,
  Card,
  Button,
  Space,
  Tag,
  Typography,
  Input,
  Row,
  Col,
  Tooltip,
  Select,
  DatePicker,
  Progress,
} from 'antd';
import { ReloadOutlined, EyeOutlined, FileTextOutlined, DownloadOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import type { RangePickerProps } from 'antd/es/date-picker';
import { apiService, SchedTaskInstance, TaskInstanceForQuery, TaskInstanceStatus } from '../../services/api';
import { useMessage } from '../../hooks/useMessage';
import dayjs from 'dayjs';

const { Title } = Typography;
const { Search } = Input;
const { Option } = Select;
const { RangePicker } = DatePicker;

interface TaskInstancesPageState {
  taskInstances: SchedTaskInstance[];
  loading: boolean;
  total: number;
  current: number;
  pageSize: number;
  searchText: string;
  statusFilter?: TaskInstanceStatus;
  dateRange?: RangePickerProps['value'];
}

/**
 * 任务实例管理页面组件
 * 显示 SchedTaskInstance 查询和管理
 */
const TaskInstances: React.FC = () => {
  const message = useMessage();
  const [state, setState] = useState<TaskInstancesPageState>({
    taskInstances: [],
    loading: false,
    total: 0,
    current: 1,
    pageSize: 10,
    searchText: '',
  });

  /**
   * 获取任务实例列表
   */
  const fetchTaskInstances = async (params?: Partial<TaskInstanceForQuery>) => {
    try {
      setState(prev => ({ ...prev, loading: true }));

      const query: TaskInstanceForQuery = {
        page: {
          page: params?.page?.page || state.current,
          limit: params?.page?.limit || state.pageSize,
        },
        filter: params?.filter || {},
      };

      const result = await apiService.taskInstances.queryTaskInstances(query);

      setState(prev => ({
        ...prev,
        taskInstances: result.result || [],
        total: result.page.total || 0,
        current: query.page.page || 1,
        loading: false,
      }));
    } catch (error) {
      console.error('获取任务实例列表失败:', error);
      message.error('获取任务实例列表失败');
      setState(prev => ({ ...prev, loading: false }));
    }
  };

  useEffect(() => {
    fetchTaskInstances();
  }, []);

  /**
   * 渲染状态标签
   */
  const renderStatus = (status: TaskInstanceStatus) => {
    const statusConfig: Partial<Record<TaskInstanceStatus, { color: string; text: string }>> = {
      [TaskInstanceStatus.Pending]: { color: 'default', text: '等待中' },
      [TaskInstanceStatus.Running]: { color: 'processing', text: '运行中' },
      [TaskInstanceStatus.Succeeded]: { color: 'success', text: '成功' },
      [TaskInstanceStatus.Failed]: { color: 'error', text: '失败' },
      [TaskInstanceStatus.Cancelled]: { color: 'warning', text: '已取消' },
      [TaskInstanceStatus.Timeout]: { color: 'orange', text: '超时' },
    };
    const config = statusConfig[status] || { color: 'default', text: status.toString() };
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  /**
   * 渲染进度（使用模拟数据）
   */
  const renderProgress = (status: TaskInstanceStatus) => {
    let progress = 0;
    let strokeColor = '#1890ff';

    switch (status) {
      case TaskInstanceStatus.Succeeded:
        progress = 100;
        strokeColor = '#52c41a';
        break;
      case TaskInstanceStatus.Running:
        progress = Math.floor(Math.random() * 80) + 10; // 10-90%
        strokeColor = '#1890ff';
        break;
      case TaskInstanceStatus.Failed:
        progress = Math.floor(Math.random() * 50) + 10; // 10-60%
        strokeColor = '#ff4d4f';
        break;
      case TaskInstanceStatus.Cancelled:
        progress = Math.floor(Math.random() * 30) + 5; // 5-35%
        strokeColor = '#faad14';
        break;
      default:
        progress = 0;
    }

    return <Progress percent={progress} size="small" strokeColor={strokeColor} showInfo={false} />;
  };

  /**
   * 渲染资源使用情况（使用模拟数据）
   */
  const renderResourceUsage = () => {
    const cpuUsage = Math.floor(Math.random() * 100);
    const memoryUsage = Math.floor(Math.random() * 100);

    return (
      <Space direction="vertical" size="small">
        <div>
          <span style={{ fontSize: '12px', color: '#666' }}>CPU: </span>
          <Progress
            percent={cpuUsage}
            size="small"
            strokeColor={cpuUsage > 80 ? '#ff4d4f' : '#1890ff'}
            format={percent => `${percent}%`}
          />
        </div>
        <div>
          <span style={{ fontSize: '12px', color: '#666' }}>内存: </span>
          <Progress
            percent={memoryUsage}
            size="small"
            strokeColor={memoryUsage > 80 ? '#ff4d4f' : '#52c41a'}
            format={percent => `${percent}%`}
          />
        </div>
      </Space>
    );
  };

  /**
   * 渲染执行时长（使用模拟数据）
   */
  const renderDuration = (_instance: SchedTaskInstance) => {
    // 模拟执行时长
    const duration = Math.floor(Math.random() * 7200) + 60; // 1分钟到2小时

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
   * 表格列定义
   */
  const columns: ColumnsType<SchedTaskInstance> = [
    {
      title: '实例 ID',
      dataIndex: 'id',
      key: 'id',
      width: 120,
      render: (text: string, record: SchedTaskInstance) => (
        <Button type="link" onClick={() => handleViewDetails(record)} style={{ padding: 0, height: 'auto' }}>
          {text}
        </Button>
      ),
    },
    {
      title: '任务 ID',
      dataIndex: 'task_id',
      key: 'task_id',
      ellipsis: true,
    },
    {
      title: '所属作业',
      dataIndex: 'job_id',
      key: 'job_id',
      ellipsis: true,
      width: 120,
    },
    {
      title: '执行代理管理',
      dataIndex: 'agent_id',
      key: 'agent_id',
      ellipsis: true,
      width: 120,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: TaskInstanceStatus) => renderStatus(status),
      width: 100,
    },
    {
      title: '进度',
      key: 'progress',
      render: (_, record) => renderProgress(record.status),
      width: 120,
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (time: string) => dayjs(time).format('MM-DD HH:mm'),
      width: 120,
    },
    {
      title: '执行时长',
      key: 'duration',
      render: (_, record) => renderDuration(record),
      width: 100,
    },
    {
      title: '资源使用',
      key: 'resource',
      render: renderResourceUsage,
      width: 150,
    },

    {
      title: '操作',
      key: 'action',
      render: (_, record) => (
        <Space size="small">
          <Tooltip title="查看详情">
            <Button type="text" icon={<EyeOutlined />} onClick={() => handleViewDetails(record)} />
          </Tooltip>
          <Tooltip title="查看日志">
            <Button type="text" icon={<FileTextOutlined />} onClick={() => handleViewLogs(record)} />
          </Tooltip>
          <Tooltip title="下载输出">
            <Button type="text" icon={<DownloadOutlined />} onClick={() => handleDownloadOutput(record)} />
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
    fetchTaskInstances({ page: { page: 1, limit: state.pageSize } });
  };

  /**
   * 刷新数据
   */
  const handleRefresh = () => {
    fetchTaskInstances();
  };

  /**
   * 查看详情
   */
  const handleViewDetails = (record: SchedTaskInstance) => {
    // TODO: 打开任务实例详情对话框
    message.info(`查看任务实例 "${record.id}" 详情功能开发中`);
  };

  /**
   * 查看日志
   */
  const handleViewLogs = (record: SchedTaskInstance) => {
    // TODO: 打开日志查看对话框
    message.info(`查看任务实例 "${record.id}" 日志功能开发中`);
  };

  /**
   * 下载输出
   */
  const handleDownloadOutput = (record: SchedTaskInstance) => {
    // TODO: 下载任务输出文件
    message.info(`下载任务实例 "${record.id}" 输出功能开发中`);
  };

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Row justify="space-between" align="middle">
        <Col>
          <Title level={2}>任务实例管理</Title>
        </Col>
      </Row>

      <Card>
        <Row gutter={[16, 16]} style={{ marginBottom: 16 }}>
          <Col xs={24} sm={12} md={8}>
            <Search
              placeholder="搜索实例 ID、任务 ID、作业名称或代理"
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
              <Option value={TaskInstanceStatus.Pending}>等待中</Option>
              <Option value={TaskInstanceStatus.Running}>运行中</Option>
              <Option value={TaskInstanceStatus.Succeeded}>成功</Option>
              <Option value={TaskInstanceStatus.Failed}>失败</Option>
              <Option value={TaskInstanceStatus.Cancelled}>已取消</Option>
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
          dataSource={state.taskInstances}
          rowKey="id"
          loading={state.loading}
          pagination={{
            current: state.current,
            pageSize: state.pageSize,
            total: state.total,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
            onChange: (page, pageSize) => {
              setState(prev => ({ ...prev, current: page, pageSize: pageSize || 10 }));
              fetchTaskInstances({ page: { page, limit: pageSize } });
            },
          }}
        />
      </Card>
    </Space>
  );
};

export default TaskInstances;
