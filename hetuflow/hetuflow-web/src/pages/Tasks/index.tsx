import React, { useState, useEffect } from "react";
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
  message,
  Modal,
} from "antd";
import {
  ReloadOutlined,
  EyeOutlined,
  PlayCircleOutlined,
  StopOutlined,
} from "@ant-design/icons";
import type { ColumnsType } from "antd/es/table";
import type { RangePickerProps } from "antd/es/date-picker";
import { apiService, SchedTask, TaskForQuery, TaskStatus, TaskInstanceStatus } from "../../services/api";
import dayjs from "dayjs";

const { Title } = Typography;
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
}

/**
 * 任务管理页面组件
 * 显示 SchedTask 查询和管理
 */
const Tasks: React.FC = () => {
  const [state, setState] = useState<TasksPageState>({
    tasks: [],
    loading: false,
    total: 0,
    current: 1,
    pageSize: 10,
    searchText: "",
  });

  /**
   * 获取任务列表
   */
  const fetchTasks = async (params?: Partial<TaskForQuery>) => {
    try {
      setState((prev) => ({ ...prev, loading: true }));

      const query: TaskForQuery = {
        page: {
          page: params?.page?.page || state.current,
          limit: params?.page?.limit || state.pageSize,
        },
        filter: params?.filter || {},
      };

      const result = await apiService.tasks.queryTasks(query);

      setState((prev) => ({
        ...prev,
        tasks: result.result,
        total: result.page.total,
        current: result.page.total > 0 ? prev.current : 1,
        loading: false,
      }));
    } catch (error) {
      console.error("获取任务列表失败:", error);
      message.error("获取任务列表失败");
      setState((prev) => ({ ...prev, loading: false }));
    }
  };

  /**
   * 渲染状态标签
   */
  const renderStatus = (status: TaskStatus) => {
    const statusConfig = {
      [TaskStatus.Pending]: { color: "default", text: "等待中" },
      [TaskStatus.Doing]: { color: "processing", text: "运行中" },
      [TaskStatus.Succeeded]: { color: "success", text: "成功" },
      [TaskStatus.Failed]: { color: "error", text: "失败" },
      [TaskStatus.Cancelled]: { color: "warning", text: "已取消" },
    };
    const config = statusConfig[status] || { color: "default", text: status };
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  /**
   * 渲染优先级（使用模拟数据）
   */
  const renderPriority = () => {
    const priorities = ["high", "medium", "low"];
    const priority = priorities[Math.floor(Math.random() * priorities.length)];
    const priorityConfig = {
      high: { color: "red", text: "高" },
      medium: { color: "orange", text: "中" },
      low: { color: "green", text: "低" },
    };
    const config = priorityConfig[priority as keyof typeof priorityConfig];
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  /**
   * 渲染重试信息（使用模拟数据）
   */
  const renderRetryInfo = () => {
    const retryCount = Math.floor(Math.random() * 3);
    const maxRetries = 3;
    const color = retryCount > 0 ? "orange" : "default";
    return (
      <Tag color={color}>
        {retryCount}/{maxRetries}
      </Tag>
    );
  };

  /**
   * 渲染执行时长（使用模拟数据）
   */
  const renderDuration = (_task: SchedTask) => {
    // 模拟执行时长
    const duration = Math.floor(Math.random() * 3600) + 60; // 1分钟到1小时

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
   * 表格列定义
   */
  const columns: ColumnsType<SchedTask> = [
    {
      title: "任务名称",
      dataIndex: "name",
      key: "name",
      ellipsis: true,
      render: (text: string, record: SchedTask) => (
        <Tooltip title={record.description}>
          <Button type="link" onClick={() => handleViewDetails(record)}>
            {text}
          </Button>
        </Tooltip>
      ),
    },
    {
      title: "状态",
      dataIndex: "status",
      key: "status",
      render: (status: TaskStatus) => renderStatus(status),
      width: 100,
    },
    {
      title: "优先级",
      key: "priority",
      render: renderPriority,
      width: 80,
    },
    {
      title: "所属作业",
      dataIndex: "job_id",
      key: "job_id",
      width: 120,
    },
    {
      title: "执行代理",
      dataIndex: "agent_id",
      key: "agent_id",
      width: 120,
    },
    {
      title: "重试次数",
      key: "retry",
      render: renderRetryInfo,
      width: 80,
    },
    {
      title: "执行时长",
      key: "duration",
      render: (_, record) => renderDuration(record),
      width: 100,
    },
    {
      title: "创建时间",
      dataIndex: "created_at",
      key: "created_at",
      render: (time: string) => dayjs(time).format("MM-DD HH:mm"),
      width: 120,
    },
    {
      title: "更新时间",
      dataIndex: "updated_at",
      key: "updated_at",
      render: (time: string) => dayjs(time).format("MM-DD HH:mm"),
      width: 120,
    },
    {
      title: "操作",
      key: "action",
      render: (_, record) => (
        <Space size="small">
          <Tooltip title="查看详情">
            <Button type="text" icon={<EyeOutlined />} onClick={() => handleViewDetails(record)} />
          </Tooltip>
          {record.status === TaskStatus.Pending && (
            <Tooltip title="立即执行">
              <Button type="text" icon={<PlayCircleOutlined />} onClick={() => handleExecute(record)} />
            </Tooltip>
          )}
          {record.status === TaskStatus.Doing && (
            <Tooltip title="取消执行">
              <Button type="text" danger icon={<StopOutlined />} onClick={() => handleCancel(record)} />
            </Tooltip>
          )}
        </Space>
      ),
      width: 100,
    },
  ];

  /**
   * 处理搜索
   */
  const handleSearch = (value: string) => {
    setState((prev) => ({ ...prev, searchText: value, current: 1 }));
    fetchTasks({ page: { page: 1, limit: state.pageSize } });
  };

  /**
   * 刷新数据
   */
  const handleRefresh = () => {
    fetchTasks();
  };

  /**
   * 查看详情
   */
  const handleViewDetails = (record: SchedTask) => {
    // TODO: 打开任务详情对话框
    message.info(`查看任务 "${record.name}" 详情功能开发中`);
  };

  /**
   * 立即执行
   */
  const handleExecute = (record: SchedTask) => {
    Modal.confirm({
      title: "确认执行",
      content: `确定要立即执行任务 "${record.name}" 吗？`,
      onOk: async () => {
        try {
          // TODO: 调用立即执行 API
          message.success("任务执行请求已提交");
          fetchTasks();
        } catch (error) {
          console.error("执行任务失败:", error);
          message.error("执行任务失败");
        }
      },
    });
  };

  /**
   * 取消执行
   */
  const handleCancel = (record: SchedTask) => {
    Modal.confirm({
      title: "确认取消",
      content: `确定要取消任务 "${record.name}" 的执行吗？`,
      onOk: async () => {
        try {
          // TODO: 调用取消执行 API
          message.success("任务取消请求已提交");
          fetchTasks();
        } catch (error) {
          console.error("取消任务失败:", error);
          message.error("取消任务失败");
        }
      },
    });
  };

  return (
    <Space direction="vertical" size="large" style={{ width: "100%" }}>
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
              onChange={(e) => setState((prev) => ({ ...prev, searchText: e.target.value }))}
            />
          </Col>
          <Col xs={24} sm={12} md={6}>
            <Select
              placeholder="选择状态"
              allowClear
              style={{ width: "100%" }}
              onChange={(value) => setState((prev) => ({ ...prev, statusFilter: value }))}
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
              style={{ width: "100%" }}
              placeholder={["开始时间", "结束时间"]}
              onChange={(dates) => setState((prev) => ({ ...prev, dateRange: dates }))}
            />
          </Col>
          <Col xs={24} sm={12} md={4}>
            <Button icon={<ReloadOutlined />} onClick={handleRefresh} loading={state.loading} style={{ width: "100%" }}>
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
          pagination={{
            current: state.current,
            pageSize: state.pageSize,
            total: state.total,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
            onChange: (page, pageSize) => {
              setState((prev) => ({ ...prev, current: page, pageSize: pageSize || 10 }));
              fetchTasks({ page: { page, limit: pageSize } });
            },
          }}
        />
      </Card>
    </Space>
  );
};

export default Tasks;
