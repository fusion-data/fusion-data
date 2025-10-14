import React, { useState, useCallback, useEffect } from 'react';
import {
  Card,
  Row,
  Col,
  Statistic,
  Progress,
  Button,
  Space,
  Typography,
  Avatar,
  Tag,
  List,
  Timeline,
  Badge,
  Alert,
  Tooltip,
  Switch,
  Select,
  Input,
  Divider,
  Tabs,
} from 'antd';
import {
  DashboardOutlined,
  ThunderboltOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  ClockCircleOutlined,
  WarningOutlined,
  UserOutlined,
  EyeOutlined,
  SettingOutlined,
  ReloadOutlined,
  BellOutlined,
  LineChartOutlined,
  BarChartOutlined,
  PieChartOutlined,
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

import { RealTimeMonitor } from './RealTimeMonitor';
import { WorkflowEngine, defaultWorkflowEngine } from '../execution';

const { Text, Title, Paragraph } = Typography;
const { TabPane } = Tabs;
const { Search } = Input;

// 仪表板数据类型
interface DashboardStats {
  totalWorkflows: number;
  activeWorkflows: number;
  completedWorkflows: number;
  failedWorkflows: number;
  totalNodes: number;
  activeNodes: number;
  avgExecutionTime: number;
  successRate: number;
  totalUsers: number;
  activeUsers: number;
}

interface ActivityLog {
  id: string;
  type: 'workflow' | 'node' | 'system' | 'user';
  action: string;
  entity: string;
  details: string;
  timestamp: number;
  user?: string;
  status: 'success' | 'warning' | 'error' | 'info';
}

interface QuickStats {
  title: string;
  value: number;
  prefix?: React.ReactNode;
  suffix?: string;
  color?: string;
  trend?: number;
  icon?: React.ReactNode;
}

interface DashboardProps {
  engine?: WorkflowEngine;
  refreshInterval?: number;
  height?: number;
  showSettings?: boolean;
}

export const Dashboard: React.FC<DashboardProps> = ({
  engine = defaultWorkflowEngine,
  refreshInterval = 10000,
  height = 900,
  showSettings = true,
}) => {
  const [activeTab, setActiveTab] = useState('overview');
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h' | '7d'>('24h');
  const [dashboardStats, setDashboardStats] = useState<DashboardStats>({
    totalWorkflows: 0,
    activeWorkflows: 0,
    completedWorkflows: 0,
    failedWorkflows: 0,
    totalNodes: 0,
    activeNodes: 0,
    avgExecutionTime: 0,
    successRate: 0,
    totalUsers: 0,
    activeUsers: 0,
  });

  const [activityLogs, setActivityLogs] = useState<ActivityLog[]>([]);
  const [quickStats, setQuickStats] = useState<QuickStats[]>([]);
  const [notifications, setNotifications] = useState<any[]>([]);

  // 生成模拟数据
  const generateDashboardStats = useCallback((): DashboardStats => ({
    totalWorkflows: 25,
    activeWorkflows: 3,
    completedWorkflows: 18,
    failedWorkflows: 2,
    totalNodes: 156,
    activeNodes: 8,
    avgExecutionTime: 45.6,
    successRate: 92.5,
    totalUsers: 128,
    activeUsers: 42,
  }), []);

  const generateActivityLogs = useCallback((): ActivityLog[] => {
    return [
      {
        id: '1',
        type: 'workflow',
        action: '执行完成',
        entity: '数据分析工作流',
        details: '成功处理 1250 条记录',
        timestamp: Date.now() - 120000,
        user: '张三',
        status: 'success',
      },
      {
        id: '2',
        type: 'node',
        action: '执行失败',
        entity: 'AI 分析节点',
        details: 'API 调用超时',
        timestamp: Date.now() - 300000,
        user: '系统',
        status: 'error',
      },
      {
        id: '3',
        type: 'system',
        action: '性能告警',
        entity: 'CPU 使用率',
        details: 'CPU 使用率达到 85%',
        timestamp: Date.now() - 600000,
        status: 'warning',
      },
      {
        id: '4',
        type: 'user',
        action: '用户登录',
        entity: '李四',
        details: '用户登录系统',
        timestamp: Date.now() - 900000,
        user: '李四',
        status: 'info',
      },
      {
        id: '5',
        type: 'workflow',
        action: '工作流创建',
        entity: '报表生成流程',
        details: '创建了新的报表生成工作流',
        timestamp: Date.now() - 1200000,
        user: '王五',
        status: 'success',
      },
    ];
  }, []);

  const generateQuickStats = useCallback((): QuickStats[] => [
    {
      title: '今日执行',
      value: 156,
      prefix: <ThunderboltOutlined />,
      color: '#1890ff',
      trend: 12.5,
    },
    {
      title: '成功率',
      value: 92.5,
      suffix: '%',
      prefix: <CheckCircleOutlined />,
      color: '#52c41a',
      trend: 2.1,
    },
    {
      title: '平均耗时',
      value: 45.6,
      suffix: 's',
      prefix: <ClockCircleOutlined />,
      color: '#fa8c16',
      trend: -5.3,
    },
    {
      title: '活跃节点',
      value: 8,
      prefix: <BarChartOutlined />,
      color: '#722ed1',
      trend: 0,
    },
  ], []);

  const generateNotifications = useCallback((): any[] => [
    {
      id: '1',
      type: 'warning',
      title: '系统性能警告',
      message: 'CPU 使用率持续偏高，建议检查系统负载',
      timestamp: Date.now() - 300000,
      read: false,
    },
    {
      id: '2',
      type: 'error',
      title: '工作流执行失败',
      message: '工作流 "数据处理流程" 执行失败，请检查配置',
      timestamp: Date.now() - 600000,
      read: false,
    },
    {
      id: '3',
      type: 'info',
      title: '系统更新',
      message: '系统已更新到版本 2.1.0',
      timestamp: Date.now() - 1800000,
      read: true,
    },
  ], []);

  // 刷新数据
  const refreshData = useCallback(() => {
    setDashboardStats(generateDashboardStats());
    setActivityLogs(generateActivityLogs());
    setQuickStats(generateQuickStats());
    setNotifications(generateNotifications());
  }, [generateDashboardStats, generateActivityLogs, generateQuickStats, generateNotifications]);

  // 自动刷新
  useEffect(() => {
    if (autoRefresh) {
      refreshData();
      const interval = setInterval(refreshData, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [autoRefresh, refreshInterval, refreshData]);

  // 获取时间范围毫秒数
  const getTimeRangeMs = useCallback(() => {
    switch (timeRange) {
      case '1h': return 60 * 60 * 1000;
      case '6h': return 6 * 60 * 60 * 1000;
      case '24h': return 24 * 60 * 60 * 1000;
      case '7d': return 7 * 24 * 60 * 60 * 1000;
      default: return 24 * 60 * 60 * 1000;
    }
  }, [timeRange]);

  // 过滤活动日志
  const filteredActivityLogs = activityLogs.filter(log => {
    const logTime = log.timestamp;
    const now = Date.now();
    const rangeMs = getTimeRangeMs();
    return now - logTime <= rangeMs;
  });

  // 获取状态颜色
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'success': return '#52c41a';
      case 'warning': return '#faad14';
      case 'error': return '#ff4d4f';
      case 'info': return '#1890ff';
      default: return '#d9d9d9';
    }
  };

  // 获取活动图标
  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'workflow': return <ThunderboltOutlined />;
      case 'node': return <BarChartOutlined />;
      case 'system': return <SettingOutlined />;
      case 'user': return <UserOutlined />;
      default: return <InfoCircleOutlined />;
    }
  };

  // 渲染概览卡片
  const renderOverviewCards = () => {
    const cards = [
      {
        title: '工作流总数',
        value: dashboardStats.totalWorkflows,
        prefix: <ThunderboltOutlined />,
        color: '#1890ff',
      },
      {
        title: '执行成功率',
        value: dashboardStats.successRate,
        suffix: '%',
        prefix: <CheckCircleOutlined />,
        color: '#52c41a',
      },
      {
        title: '平均执行时间',
        value: dashboardStats.avgExecutionTime,
        suffix: 's',
        prefix: <ClockCircleOutlined />,
        color: '#fa8c16',
      },
      {
        title: '活跃用户',
        value: dashboardStats.activeUsers,
        prefix: <UserOutlined />,
        color: '#722ed1',
      },
    ];

    return (
      <Row gutter={16}>
        {cards.map((card, index) => (
          <Col span={6} key={index}>
            <Card>
              <Statistic
                title={card.title}
                value={card.value}
                prefix={card.prefix}
                suffix={card.suffix}
                valueStyle={{ color: card.color }}
              />
            </Card>
          </Col>
        ))}
      </Row>
    );
  };

  // 渲染快速统计
  const renderQuickStatsGrid = () => (
    <Row gutter={16}>
      {quickStats.map((stat, index) => (
        <Col span={6} key={index}>
          <Card size="small">
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div>
                <Text style={{ fontSize: 14, color: '#666' }}>{stat.title}</Text>
                <div style={{ fontSize: 24, fontWeight: 'bold', color: stat.color, marginTop: 4 }}>
                  {stat.prefix}
                  <span style={{ marginLeft: 8 }}>{stat.value}</span>
                  {stat.suffix}
                </div>
              </div>
              <div>
                {stat.trend && (
                  <Text
                    style={{
                      fontSize: 12,
                      color: stat.trend > 0 ? '#52c41a' : '#ff4d4f',
                    }}
                  >
                    {stat.trend > 0 ? '↑' : '↓'} {Math.abs(stat.trend)}%
                  </Text>
                )}
              </div>
            </div>
          </Card>
        </Col>
      ))}
    </Row>
  );

  // 渲染活动时间线
  const renderActivityTimeline = () => (
    <Card title="最近活动" size="small">
      <Timeline mode="left">
        {filteredActivityLogs.map((log, index) => (
          <Timeline.Item
            key={log.id}
            color={getStatusColor(log.status)}
            dot={getActivityIcon(log.type)}
          >
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div>
                  <Text strong>{log.action}</Text>
                  <div style={{ fontSize: 14, color: '#666', marginTop: 4 }}>
                    {log.entity}
                  </div>
                </div>
                <div style={{ fontSize: 12, color: '#999' }}>
                  {new Date(log.timestamp).toLocaleString()}
                </div>
              </div>
              <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>
                {log.details}
              </div>
              {log.user && (
                <div style={{ marginTop: 4 }}>
                  <Tag color="blue" size="small">
                    <UserOutlined style={{ marginRight: 4 }} />
                    {log.user}
                  </Tag>
                </div>
              )}
            </div>
          </Timeline.Item>
        ))}
      </Timeline>
    </Card>
  );

  // 渲染通知中心
  const renderNotificationCenter = () => {
    const unreadCount = notifications.filter(n => !n.read).length;

    return (
      <Card
        title={
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <span>通知中心</span>
            <Badge count={unreadCount} showZero>
              <BellOutlined />
            </Badge>
          </div>
        }
        size="small"
        extra={
          <Button type="text" size="small" onClick={refreshData}>
            <ReloadOutlined />
          </Button>
        }
      >
        <List
          dataSource={notifications}
          renderItem={(item) => (
            <List.Item
              style={{
                padding: '12px 0',
                borderBottom: '1px solid #f0f0f0',
                backgroundColor: item.read ? 'transparent' : '#f6ffed',
              }}
            >
              <List.Item.Meta
                avatar={
                  <Avatar
                    style={{
                      backgroundColor: getStatusColor(item.type),
                    }}
                    icon={
                      item.type === 'warning' ? (
                        <WarningOutlined />
                      ) : item.type === 'error' ? (
                        <ExclamationCircleOutlined />
                      ) : item.type === 'info' ? (
                        <InfoCircleOutlined />
                      ) : (
                        <CheckCircleOutlined />
                      )
                    }
                  />
                }
                title={
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <Text strong>{item.title}</Text>
                  <Text type="secondary" style={{ fontSize: 12 }}>
                    {new Date(item.timestamp).toLocaleString()}
                  </Text>
                </div>
                }
                description={item.message}
              />
            </List.Item>
          )}
        />
      </Card>
    );
  };

  return (
    <div style={{ height, padding: 24 }} className="dashboard">
      <div style={{ marginBottom: 24 }}>
        <Row justify="space-between" align="middle">
          <Col>
            <Title level={2} style={{ margin: 0 }}>
              <DashboardOutlined style={{ marginRight: 8 }} />
              工作流仪表板
            </Title>
            <Text type="secondary">
              实时监控工作流执行状态和系统性能
            </Text>
          </Col>
          <Col>
            <Space>
              <Select
                value={timeRange}
                onChange={setTimeRange}
                style={{ width: 100 }}
                size="small"
              >
                <Select.Option value="1h">1小时</Select.Option>
                <Select.Option value="6h">6小时</Select.Option>
                <Select.Option value="24h">24小时</Select.Option>
                <Select.Option value="7d">7天</Select.Option>
              </Select>
              <Switch
                checkedChildren="自动刷新"
                unCheckedChildren="手动"
                checked={autoRefresh}
                onChange={setAutoRefresh}
              />
              <Button
                icon={<ReloadOutlined />}
                onClick={refreshData}
                size="small"
              >
                刷新
              </Button>
              {showSettings && (
                <Button icon={<SettingOutlined />} size="small">
                  设置
                </Button>
              )}
            </Space>
          </Col>
        </Row>
      </div>

      <Tabs activeKey={activeTab} onChange={setActiveTab}>
        <TabPane tab="概览" key="overview">
          {renderOverviewCards()}
          <div style={{ marginTop: 24 }}>
            {renderQuickStatsGrid()}
          </div>
          <div style={{ marginTop: 24 }}>
            <Row gutter={16}>
              <Col span={16}>
                {renderActivityTimeline()}
              </Col>
              <Col span={8}>
                {renderNotificationCenter()}
              </Col>
            </Row>
          </div>
        </TabPane>

        <TabPane tab="详细监控" key="monitoring">
          <RealTimeMonitor
            engine={engine}
            refreshInterval={refreshInterval}
            height={height - 200}
            showSettings={false}
          />
        </TabPane>

        <TabPane tab="统计分析" key="analytics">
          <Row gutter={16}>
            <Col span={12}>
              <Card title="执行趋势" size="small">
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={[]}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="time" />
                    <YAxis />
                    <RechartsTooltip />
                    <Legend />
                    <Line type="monotone" dataKey="count" stroke="#8884d8" strokeWidth={2} />
                    <Line type="monotone" dataKey="success" stroke="#52c41a" strokeWidth={2} />
                    <Line type="monotone" dataKey="failed" stroke="#ff4d4f" strokeWidth={2} />
                  </LineChart>
                </ResponsiveContainer>
              </Card>
            </Col>

            <Col span={12}>
              <Card title="节点类型分布" size="small">
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={[
                        { name: 'AI Agent', value: 35 },
                        { name: '数据处理器', value: 28 },
                        { name: '触发器', value: 20 },
                        { name: '动作节点', value: 17 },
                      ]}
                      cx="50%"
                      cy="50%"
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="value"
                    >
                      <Cell fill="#0088FE" />
                      <Cell fill="#00C49F" />
                      <Cell fill="#FFBB28" />
                      <Cell fill="#FF8042" />
                    </Pie>
                    <RechartsTooltip />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              </Card>
            </Col>
          </Row>
        </TabPane>

        <TabPane tab="告警管理" key="alerts">
          <Alert
            message="告警中心"
            description="系统告警和通知管理功能正在开发中"
            type="info"
            showIcon
          />
        </TabPane>
      </Tabs>
    </div>
  );
};

export default Dashboard;