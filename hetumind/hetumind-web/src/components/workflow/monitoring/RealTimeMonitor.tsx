import React, { useState, useCallback, useEffect, useRef } from 'react';
import {
  Card,
  Row,
  Col,
  Statistic,
  Progress,
  Table,
  Tabs,
  Tag,
  Button,
  Space,
  Typography,
  Alert,
  List,
  Avatar,
  Timeline,
  Badge,
  Tooltip,
  Switch,
  Select,
  Input,
  DatePicker,
  Divider,
} from 'antd';
import {
  ThunderboltOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  ClockCircleOutlined,
  WarningOutlined,
  InfoCircleOutlined,
  ReloadOutlined,
  SettingOutlined,
  EyeOutlined,
  DownloadOutlined,
  FilterOutlined,
  SearchOutlined,
} from '@ant-design/icons';

import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as RechartsTooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';

const { Text, Title, Paragraph } = Typography;
const { TabPane } = Tabs;
const { RangePicker } = DatePicker;

// 监控数据类型
interface SystemMetrics {
  timestamp: number;
  cpuUsage: number;
  memoryUsage: number;
  diskUsage: number;
  networkIn: number;
  networkOut: number;
  activeConnections: number;
}

interface WorkflowMetrics {
  workflowId: string;
  workflowName: string;
  status: 'running' | 'completed' | 'failed' | 'idle';
  startTime: number;
  duration?: number;
  nodeCount: number;
  completedNodes: number;
  failedNodes: number;
  lastActivity: number;
}

interface NodeMetrics {
  nodeId: string;
  nodeType: string;
  status: 'idle' | 'running' | 'completed' | 'failed';
  executionCount: number;
  avgDuration: number;
  successRate: number;
  lastExecution: number;
}

interface AlertData {
  id: string;
  type: 'info' | 'warning' | 'error' | 'success';
  title: string;
  message: string;
  timestamp: number;
  source: string;
  acknowledged: boolean;
}

interface RealTimeMonitorProps {
  refreshInterval?: number;
  height?: number;
  showSettings?: boolean;
  onExportData?: (data: any) => void;
}

export const RealTimeMonitor: React.FC<RealTimeMonitorProps> = ({
  refreshInterval = 5000,
  height = 800,
  showSettings = true,
  onExportData,
}) => {
  const [activeTab, setActiveTab] = useState('overview');
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics[]>([]);
  const [workflowMetrics, setWorkflowMetrics] = useState<WorkflowMetrics[]>([]);
  const [nodeMetrics, setNodeMetrics] = useState<NodeMetrics[]>([]);
  const [alerts, setAlerts] = useState<AlertData[]>([]);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [searchText, setSearchText] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [timeRange, setTimeRange] = useState<[any, any] | null>(null);

  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  // 生成模拟数据
  const generateSystemMetrics = useCallback((): SystemMetrics => {
    return {
      timestamp: Date.now(),
      cpuUsage: Math.random() * 100,
      memoryUsage: 60 + Math.random() * 30,
      diskUsage: 40 + Math.random() * 20,
      networkIn: Math.random() * 1000,
      networkOut: Math.random() * 800,
      activeConnections: Math.floor(Math.random() * 100),
    };
  }, []);

  const generateWorkflowMetrics = useCallback((): WorkflowMetrics[] => {
    return [
      {
        workflowId: 'wf_001',
        workflowName: '数据处理流程',
        status: Math.random() > 0.7 ? 'running' : Math.random() > 0.5 ? 'completed' : 'idle',
        startTime: Date.now() - Math.random() * 3600000,
        duration: Math.floor(Math.random() * 300000),
        nodeCount: 10,
        completedNodes: Math.floor(Math.random() * 10),
        failedNodes: Math.floor(Math.random() * 2),
        lastActivity: Date.now(),
      },
      {
        workflowId: 'wf_002',
        workflowName: 'AI 分析工作流',
        status: 'running',
        startTime: Date.now() - Math.random() * 7200000,
        duration: Math.floor(Math.random() * 600000),
        nodeCount: 15,
        completedNodes: Math.floor(Math.random() * 15),
        failedNodes: Math.floor(Math.random() * 3),
        lastActivity: Date.now(),
      },
      {
        workflowId: 'wf_003',
        workflowName: '报表生成流程',
        status: 'completed',
        startTime: Date.now() - Math.random() * 1800000,
        duration: Math.floor(Math.random() * 120000),
        nodeCount: 8,
        completedNodes: 8,
        failedNodes: 0,
        lastActivity: Date.now(),
      },
    ];
  }, []);

  const generateNodeMetrics = useCallback((): NodeMetrics[] => {
    return [
      {
        nodeId: 'node_001',
        nodeType: 'aiAgent',
        status: 'running',
        executionCount: 156,
        avgDuration: 2500,
        successRate: 94.2,
        lastExecution: Date.now(),
      },
      {
        nodeId: 'node_002',
        nodeType: 'dataProcessor',
        status: 'completed',
        executionCount: 342,
        avgDuration: 1200,
        successRate: 98.5,
        lastExecution: Date.now() - 60000,
      },
      {
        nodeId: 'node_003',
        nodeType: 'condition',
        status: 'idle',
        executionCount: 789,
        avgDuration: 100,
        successRate: 99.9,
        lastExecution: Date.now() - 120000,
      },
      {
        nodeId: 'node_004',
        nodeType: 'action',
        status: 'failed',
        executionCount: 234,
        avgDuration: 3500,
        successRate: 87.3,
        lastExecution: Date.now() - 30000,
      },
    ];
  }, []);

  const generateAlerts = useCallback((): AlertData[] => {
    return [
      {
        id: 'alert_001',
        type: 'warning',
        title: 'CPU 使用率过高',
        message: '系统CPU使用率达到85%，建议检查系统负载',
        timestamp: Date.now() - 120000,
        source: 'system',
        acknowledged: false,
      },
      {
        id: 'alert_002',
        type: 'error',
        title: '工作流执行失败',
        message: '工作流 "数据处理流程" 执行失败，请检查节点配置',
        timestamp: Date.now() - 300000,
        source: 'workflow',
        acknowledged: false,
      },
      {
        id: 'alert_003',
        type: 'info',
        title: '新版本部署',
        message: '系统已成功更新到版本 2.1.0',
        timestamp: Date.now() - 600000,
        source: 'system',
        acknowledged: true,
      },
    ];
  }, []);

  // 刷新数据
  const refreshData = useCallback(() => {
    // 更新系统指标
    setSystemMetrics(prev => {
      const newMetrics = generateSystemMetrics();
      const updated = [...prev, newMetrics].slice(-20); // 保留最近20个数据点
      return updated;
    });

    // 更新工作流指标
    setWorkflowMetrics(generateWorkflowMetrics());

    // 更新节点指标
    setNodeMetrics(generateNodeMetrics());

    // 更新告警
    setAlerts(generateAlerts());
  }, [generateSystemMetrics, generateWorkflowMetrics, generateNodeMetrics, generateAlerts]);

  // 自动刷新
  useEffect(() => {
    if (autoRefresh) {
      refreshData();
      intervalRef.current = setInterval(refreshData, refreshInterval);
    }

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      intervalRef.current = null;
      }
    };
  }, [autoRefresh, refreshInterval, refreshData]);

  // 获取状态颜色
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return 'blue';
      case 'completed': return 'green';
      case 'failed': return 'red';
      case 'idle': return 'default';
      case 'warning': return 'orange';
      case 'error': return 'red';
      case 'info': return 'blue';
      case 'success': return 'green';
      default: return 'default';
    }
  };

  // 过滤数据
  const filteredWorkflowMetrics = workflowMetrics.filter(workflow => {
    const matchesSearch = workflow.workflowName.toLowerCase().includes(searchText.toLowerCase());
    const matchesStatus = statusFilter === 'all' || workflow.status === statusFilter;
    return matchesSearch && matchesStatus;
  });

  // 渲染系统概览
  const renderSystemOverview = () => {
    const latestMetrics = systemMetrics[systemMetrics.length - 1] || {
      cpuUsage: 0,
      memoryUsage: 0,
      diskUsage: 0,
      networkIn: 0,
      networkOut: 0,
      activeConnections: 0,
    };

    const systemStats = [
      {
        title: 'CPU 使用率',
        value: latestMetrics.cpuUsage.toFixed(1),
        suffix: '%',
        color: latestMetrics.cpuUsage > 80 ? '#ff4d4f' : latestMetrics.cpuUsage > 60 ? '#faad14' : '#52c41a',
      },
      {
        title: '内存使用率',
        value: latestMetrics.memoryUsage.toFixed(1),
        suffix: '%',
        color: latestMetrics.memoryUsage > 80 ? '#ff4d4f' : latestMetrics.memoryUsage > 60 ? '#faad14' : '#52c41a',
      },
      {
        title: '磁盘使用率',
        value: latestMetrics.diskUsage.toFixed(1),
        suffix: '%',
        color: latestMetrics.diskUsage > 80 ? '#ff4d4f' : latestMetrics.diskUsage > 60 ? '#faad14' : '#52c41a',
      },
      {
        title: '活跃连接',
        value: latestMetrics.activeConnections,
        color: '#1890ff',
      },
    ];

    return (
      <div>
        <Row gutter={16} style={{ marginBottom: 24 }}>
          {systemStats.map((stat, index) => (
            <Col span={6} key={index}>
              <Card>
                <Statistic
                  title={stat.title}
                  value={stat.value}
                  suffix={stat.suffix}
                  valueStyle={{ color: stat.color }}
                />
              </Card>
            </Col>
          ))}
        </Row>

        <Row gutter={16}>
          <Col span={12}>
            <Card title="系统资源趋势" size="small">
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={systemMetrics}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis
                    dataKey="timestamp"
                    tickFormatter={(value) => new Date(value).toLocaleTimeString()}
                  />
                  <YAxis />
                  <RechartsTooltip
                    labelFormatter={(value) => new Date(value).toLocaleString()}
                  />
                  <Legend />
                  <Line
                    type="monotone"
                    dataKey="cpuUsage"
                    stroke="#8884d8"
                    name="CPU %"
                    strokeWidth={2}
                  />
                  <Line
                    type="monotone"
                    dataKey="memoryUsage"
                    stroke="#82ca9d"
                    name="内存 %"
                    strokeWidth={2}
                  />
                  <Line
                    type="monotone"
                    dataKey="diskUsage"
                    stroke="#ffc658"
                    name="磁盘 %"
                    strokeWidth={2}
                  />
                </LineChart>
              </ResponsiveContainer>
            </Card>
          </Col>

          <Col span={12}>
            <Card title="网络流量" size="small">
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={systemMetrics}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis
                    dataKey="timestamp"
                    tickFormatter={(value) => new Date(value).toLocaleTimeString()}
                  />
                  <YAxis />
                  <RechartsTooltip
                    labelFormatter={(value) => new Date(value).toLocaleString()}
                  />
                  <Legend />
                  <Area
                    type="monotone"
                    dataKey="networkIn"
                    stackId="1"
                    stroke="#8884d8"
                    fill="#8884d8"
                    name="入站 (KB/s)"
                  />
                  <Area
                    type="monotone"
                    dataKey="networkOut"
                    stackId="1"
                    stroke="#82ca9d"
                    fill="#82ca9d"
                    name="出站 (KB/s)"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </Card>
          </Col>
        </Row>
      </div>
    );
  };

  // 渲染工作流监控
  const renderWorkflowMonitor = () => {
    const workflowColumns = [
      {
        title: '工作流名称',
        dataIndex: 'workflowName',
        key: 'workflowName',
        render: (name: string, record: WorkflowMetrics) => (
          <div>
            <Text strong>{name}</Text>
            <div style={{ fontSize: 12, color: '#666' }}>
              {record.workflowId}
            </div>
          </div>
        ),
      },
      {
        title: '状态',
        dataIndex: 'status',
        key: 'status',
        render: (status: string) => (
          <Tag color={getStatusColor(status)}>
            {status === 'running' && <ThunderboltOutlined spin />}
            {status === 'completed' && <CheckCircleOutlined />}
            {status === 'failed' && <ExclamationCircleOutlined />}
            {status === 'idle' && <ClockCircleOutlined />}
            {status}
          </Tag>
        ),
      },
      {
        title: '进度',
        key: 'progress',
        render: (_, record: WorkflowMetrics) => (
          <Progress
            percent={Math.round((record.completedNodes / record.nodeCount) * 100)}
            size="small"
            status={record.failedNodes > 0 ? 'exception' : 'active'}
          />
        ),
      },
      {
        title: '节点统计',
        key: 'nodeStats',
        render: (_, record: WorkflowMetrics) => (
          <Space>
            <Badge count={record.completedNodes} showZero>
              <CheckCircleOutlined style={{ color: '#52c41a' }} />
            </Badge>
            <Badge count={record.failedNodes} showZero>
              <ExclamationCircleOutlined style={{ color: '#ff4d4f' }} />
            </Badge>
            <Text type="secondary">/{record.nodeCount}</Text>
          </Space>
        ),
      },
      {
        title: '执行时间',
        key: 'duration',
        render: (_, record: WorkflowMetrics) => {
          const duration = record.duration || (Date.now() - record.startTime);
          const seconds = Math.floor(duration / 1000);
          const minutes = Math.floor(seconds / 60);
          return (
            <Text>
              {minutes > 0 ? `${minutes}m ${seconds % 60}s` : `${seconds}s`}
            </Text>
          );
        },
      },
      {
        title: '最后活动',
        dataIndex: 'lastActivity',
        key: 'lastActivity',
        render: (time: number) => (
          <Text>{new Date(time).toLocaleString()}</Text>
        ),
      },
    ];

    return (
      <div>
        <div style={{ marginBottom: 16 }}>
          <Row gutter={16} align="middle">
            <Col span={8}>
              <Input
                placeholder="搜索工作流名称"
                prefix={<SearchOutlined />}
                value={searchText}
                onChange={(e) => setSearchText(e.target.value)}
              />
            </Col>
            <Col span={4}>
              <Select
                value={statusFilter}
                onChange={setStatusFilter}
                style={{ width: '100%' }}
              >
                <Select.Option value="all">全部状态</Select.Option>
                <Select.Option value="running">运行中</Select.Option>
                <Select.Option value="completed">已完成</Select.Option>
                <Select.Option value="failed">失败</Select.Option>
                <Select.Option value="idle">空闲</Select.Option>
              </Select>
            </Col>
            <Col span={4}>
              <RangePicker
                showTime
                onChange={setTimeRange}
                style={{ width: '100%' }}
              />
            </Col>
            <Col span={8} style={{ textAlign: 'right' }}>
              <Space>
                <Button
                  icon={<ReloadOutlined />}
                  onClick={refreshData}
                >
                  刷新
                </Button>
                {onExportData && (
                  <Button
                    icon={<DownloadOutlined />}
                    onClick={() => onExportData(filteredWorkflowMetrics)}
                  >
                    导出数据
                  </Button>
                )}
              </Space>
            </Col>
          </Row>
        </div>

        <Table
          dataSource={filteredWorkflowMetrics}
          columns={workflowColumns}
          rowKey="workflowId"
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showQuickJumper: true,
          }}
          size="small"
        />
      </div>
    );
  };

  // 渲染节点性能
  const renderNodePerformance = () => {
    const nodeTypeData = nodeMetrics.reduce((acc, node) => {
      if (!acc[node.nodeType]) {
        acc[node.nodeType] = {
          type: node.nodeType,
          count: 0,
          avgDuration: 0,
          successRate: 0,
        };
      }
      acc[node.nodeType].count += 1;
      acc[node.nodeType].avgDuration += node.avgDuration;
      acc[node.nodeType].successRate += node.successRate;
      return acc;
    }, {} as Record<string, any>);

    const pieData = Object.values(nodeTypeData).map(item => ({
      name: item.type,
      value: item.count,
    }));

    const barData = Object.values(nodeTypeData).map(item => ({
      type: item.type,
      avgDuration: Math.round(item.avgDuration / item.count),
      successRate: Math.round(item.successRate / item.count),
    }));

    const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042', '#8884D8'];

    return (
      <Row gutter={16}>
        <Col span={12}>
          <Card title="节点类型分布" size="small">
            <ResponsiveContainer width="100%" height={300}>
              <PieChart>
                <Pie
                  data={pieData}
                  cx="50%"
                  cy="50%"
                  labelLine={false}
                  label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                  outerRadius={80}
                  fill="#8884d8"
                  dataKey="value"
                >
                  {pieData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <RechartsTooltip />
              </PieChart>
            </ResponsiveContainer>
          </Card>
        </Col>

        <Col span={12}>
          <Card title="节点性能对比" size="small">
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={barData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="type" />
                <YAxis />
                <RechartsTooltip />
                <Legend />
                <Bar dataKey="avgDuration" fill="#8884d8" name="平均耗时 (ms)" />
                <Bar dataKey="successRate" fill="#82ca9d" name="成功率 (%)" />
              </BarChart>
            </ResponsiveContainer>
          </Card>
        </Col>

        <Col span={24}>
          <Card title="节点详细指标" size="small" style={{ marginTop: 16 }}>
            <Table
              dataSource={nodeMetrics}
              columns={[
                {
                  title: '节点ID',
                  dataIndex: 'nodeId',
                  key: 'nodeId',
                },
                {
                  title: '节点类型',
                  dataIndex: 'nodeType',
                  key: 'nodeType',
                  render: (type: string) => <Tag color="blue">{type}</Tag>,
                },
                {
                  title: '状态',
                  dataIndex: 'status',
                  key: 'status',
                  render: (status: string) => (
                    <Tag color={getStatusColor(status)}>
                      {status}
                    </Tag>
                  ),
                },
                {
                  title: '执行次数',
                  dataIndex: 'executionCount',
                  key: 'executionCount',
                },
                {
                  title: '平均耗时',
                  dataIndex: 'avgDuration',
                  key: 'avgDuration',
                  render: (duration: number) => `${duration}ms`,
                },
                {
                  title: '成功率',
                  dataIndex: 'successRate',
                  key: 'successRate',
                  render: (rate: number) => (
                    <Progress
                      percent={rate}
                      size="small"
                      status={rate > 90 ? 'success' : rate > 70 ? 'normal' : 'exception'}
                    />
                  ),
                },
                {
                  title: '最后执行',
                  dataIndex: 'lastExecution',
                  key: 'lastExecution',
                  render: (time: number) => (
                    <Text>{new Date(time).toLocaleString()}</Text>
                  ),
                },
              ]}
              rowKey="nodeId"
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
      </Row>
    );
  };

  // 渲染告警中心
  const renderAlertCenter = () => {
    const alertColumns = [
      {
        title: '类型',
        dataIndex: 'type',
        key: 'type',
        render: (type: string) => (
          <Tag color={getStatusColor(type)}>
            {type === 'info' && <InfoCircleOutlined />}
            {type === 'warning' && <WarningOutlined />}
            {type === 'error' && <ExclamationCircleOutlined />}
            {type === 'success' && <CheckCircleOutlined />}
            {type}
          </Tag>
        ),
      },
      {
        title: '标题',
        dataIndex: 'title',
        key: 'title',
      },
      {
        title: '消息',
        dataIndex: 'message',
        key: 'message',
        ellipsis: true,
      },
      {
        title: '来源',
        dataIndex: 'source',
        key: 'source',
        render: (source: string) => <Tag>{source}</Tag>,
      },
      {
        title: '时间',
        dataIndex: 'timestamp',
        key: 'timestamp',
        render: (time: number) => (
          <Text>{new Date(time).toLocaleString()}</Text>
        ),
      },
      {
        title: '状态',
        dataIndex: 'acknowledged',
        key: 'acknowledged',
        render: (acknowledged: boolean) => (
          <Tag color={acknowledged ? 'green' : 'orange'}>
            {acknowledged ? '已确认' : '待确认'}
          </Tag>
        ),
      },
    ];

    const unacknowledgedCount = alerts.filter(alert => !alert.acknowledged).length;

    return (
      <div>
        <Alert
          message={`当前有 ${unacknowledgedCount} 个待处理告警`}
          type={unacknowledgedCount > 0 ? 'warning' : 'success'}
          showIcon
          style={{ marginBottom: 16 }}
        />

        <Table
          dataSource={alerts}
          columns={alertColumns}
          rowKey="id"
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
          }}
          size="small"
        />
      </div>
    );
  };

  return (
    <div style={{ height }} className="real-time-monitor">
      <div style={{ marginBottom: 16 }}>
        <Row justify="space-between" align="middle">
          <Col>
            <Title level={3} style={{ margin: 0 }}>
              实时监控面板
            </Title>
          </Col>
          <Col>
            <Space>
              {showSettings && (
                <Button icon={<SettingOutlined />} size="small">
                  设置
                </Button>
              )}
              <Switch
                checkedChildren="自动刷新"
                unCheckedChildren="手动刷新"
                checked={autoRefresh}
                onChange={setAutoRefresh}
              />
              <Button
                icon={<ReloadOutlined />}
                onClick={refreshData}
              >
                刷新
              </Button>
            </Space>
          </Col>
        </Row>
      </div>

      <Tabs activeKey={activeTab} onChange={setActiveTab}>
        <TabPane tab="系统概览" key="overview">
          {renderSystemOverview()}
        </TabPane>

        <TabPane tab="工作流监控" key="workflows">
          {renderWorkflowMonitor()}
        </TabPane>

        <TabPane tab="节点性能" key="nodes">
          {renderNodePerformance()}
        </TabPane>

        <TabPane tab="告警中心" key="alerts">
          {renderAlertCenter()}
        </TabPane>
      </Tabs>
    </div>
  );
};

export default RealTimeMonitor;