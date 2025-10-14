import React, { useState, useCallback, useEffect, useRef } from 'react';
import {
  Card,
  Row,
  Col,
  Statistic,
  Progress,
  Typography,
  Space,
  Tag,
  Alert,
  Table,
  Button,
  Select,
  DatePicker,
  Tooltip,
  Badge,
  Divider,
  List,
  Avatar,
  Timeline,
  Empty,
} from 'antd';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as RechartsTooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
  BarChart,
  Bar,
  Legend,
} from 'recharts';
import {
  RobotOutlined,
  ThunderboltOutlined,
  ClockCircleOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  DatabaseOutlined,
  ApiOutlined,
  UserOutlined,
  TrendingUpOutlined,
  TrendingDownOutlined,
  ReloadOutlined,
  FilterOutlined,
} from '@ant-design/icons';

const { Text, Title, Paragraph } = Typography;
const { RangePicker } = DatePicker;

// 性能指标接口
interface PerformanceMetrics {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageResponseTime: number;
  totalTokens: number;
  totalCost: number;
  errorRate: number;
  successRate: number;
}

// 请求数据接口
interface RequestData {
  id: string;
  timestamp: Date;
  status: 'success' | 'error' | 'timeout';
  responseTime: number;
  tokens: number;
  cost: number;
  model: string;
  errorType?: string;
  errorMessage?: string;
}

// 时间序列数据接口
interface TimeSeriesData {
  timestamp: string;
  requests: number;
  errors: number;
  avgResponseTime: number;
  tokens: number;
  cost: number;
}

// 模型使用数据接口
interface ModelUsageData {
  model: string;
  requests: number;
  tokens: number;
  cost: number;
  avgResponseTime: number;
}

interface AIAgentMonitorProps {
  nodeId: string;
  agentName?: string;
  refreshInterval?: number;
  height?: number;
  showDetails?: boolean;
}

export const AIAgentMonitor: React.FC<AIAgentMonitorProps> = ({
  nodeId,
  agentName = 'AI Agent',
  refreshInterval = 30000, // 30秒
  height = 600,
  showDetails = true,
}) => {
  const [metrics, setMetrics] = useState<PerformanceMetrics>({
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    averageResponseTime: 0,
    totalTokens: 0,
    totalCost: 0,
    errorRate: 0,
    successRate: 0,
  });

  const [recentRequests, setRecentRequests] = useState<RequestData[]>([]);
  const [timeSeriesData, setTimeSeriesData] = useState<TimeSeriesData[]>([]);
  const [modelUsageData, setModelUsageData] = useState<ModelUsageData[]>([]);
  const [loading, setLoading] = useState(false);
  const [timeRange, setTimeRange] = useState<[Date, Date]>([
    new Date(Date.now() - 24 * 60 * 60 * 1000), // 24小时前
    new Date(),
  ]);

  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  // 生成模拟数据
  const generateMockData = useCallback(() => {
    const now = new Date();
    const timeSeries: TimeSeriesData[] = [];
    const requests: RequestData[] = [];
    const modelUsage: Record<string, ModelUsageData> = {};

    // 生成时间序列数据
    for (let i = 23; i >= 0; i--) {
      const timestamp = new Date(now.getTime() - i * 60 * 60 * 1000);
      const requestCount = Math.floor(Math.random() * 50) + 10;
      const errorCount = Math.floor(requestCount * (Math.random() * 0.1)); // 0-10% 错误率

      timeSeries.push({
        timestamp: timestamp.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
        requests: requestCount,
        errors: errorCount,
        avgResponseTime: Math.floor(Math.random() * 2000) + 500,
        tokens: requestCount * (Math.floor(Math.random() * 500) + 100),
        cost: (requestCount * (Math.random() * 0.05) + 0.01),
      });
    }

    // 生成最近请求数据
    const models = ['gpt-4', 'gpt-3.5-turbo', 'claude-3-sonnet', 'claude-3-opus'];
    const errorTypes = ['timeout', 'rate_limit', 'invalid_request', 'server_error'];

    for (let i = 0; i < 20; i++) {
      const timestamp = new Date(now.getTime() - Math.random() * 60 * 60 * 1000);
      const status = Math.random() > 0.9 ? 'error' : 'success';
      const model = models[Math.floor(Math.random() * models.length)];

      const request: RequestData = {
        id: `req_${i + 1}`,
        timestamp,
        status,
        responseTime: Math.floor(Math.random() * 3000) + 200,
        tokens: Math.floor(Math.random() * 1000) + 50,
        cost: Math.random() * 0.1 + 0.001,
        model,
        ...(status === 'error' && {
          errorType: errorTypes[Math.floor(Math.random() * errorTypes.length)],
          errorMessage: '模拟错误消息',
        }),
      };

      requests.push(request);

      // 统计模型使用情况
      if (!modelUsage[model]) {
        modelUsage[model] = {
          model,
          requests: 0,
          tokens: 0,
          cost: 0,
          avgResponseTime: 0,
        };
      }

      modelUsage[model].requests++;
      modelUsage[model].tokens += request.tokens;
      modelUsage[model].cost += request.cost;
      modelUsage[model].avgResponseTime += request.responseTime;
    }

    // 计算平均响应时间
    Object.values(modelUsage).forEach(data => {
      data.avgResponseTime = Math.floor(data.avgResponseTime / data.requests);
    });

    return { timeSeries, requests, modelUsage: Object.values(modelUsage) };
  }, []);

  // 刷新数据
  const refreshData = useCallback(async () => {
    setLoading(true);
    try {
      // 模拟API调用
      await new Promise(resolve => setTimeout(resolve, 500));

      const { timeSeries, requests, modelUsage } = generateMockData();

      setTimeSeriesData(timeSeries);
      setRecentRequests(requests.slice(0, 10)); // 最近10条请求
      setModelUsageData(modelUsage);

      // 计算总体指标
      const totalRequests = requests.length;
      const successfulRequests = requests.filter(r => r.status === 'success').length;
      const failedRequests = requests.filter(r => r.status === 'error').length;
      const averageResponseTime = Math.floor(
        requests.reduce((sum, r) => sum + r.responseTime, 0) / totalRequests
      );
      const totalTokens = requests.reduce((sum, r) => sum + r.tokens, 0);
      const totalCost = requests.reduce((sum, r) => sum + r.cost, 0);
      const errorRate = totalRequests > 0 ? (failedRequests / totalRequests) * 100 : 0;
      const successRate = totalRequests > 0 ? (successfulRequests / totalRequests) * 100 : 0;

      setMetrics({
        totalRequests,
        successfulRequests,
        failedRequests,
        averageResponseTime,
        totalTokens,
        totalCost,
        errorRate,
        successRate,
      });
    } catch (error) {
      console.error('刷新数据失败:', error);
    } finally {
      setLoading(false);
    }
  }, [generateMockData]);

  // 初始化和定时刷新
  useEffect(() => {
    refreshData();

    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }

    intervalRef.current = setInterval(refreshData, refreshInterval);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [refreshData, refreshInterval]);

  // 图表颜色
  const COLORS = ['#1890ff', '#52c41a', '#faad14', '#ff4d4f', '#722ed1'];

  // 表格列定义
  const requestColumns = [
    {
      title: '时间',
      dataIndex: 'timestamp',
      key: 'timestamp',
      render: (timestamp: Date) => timestamp.toLocaleTimeString('zh-CN'),
      width: 100,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Badge
          status={status === 'success' ? 'success' : status === 'error' ? 'error' : 'warning'}
          text={status === 'success' ? '成功' : status === 'error' ? '失败' : '超时'}
        />
      ),
      width: 80,
    },
    {
      title: '模型',
      dataIndex: 'model',
      key: 'model',
      width: 120,
    },
    {
      title: '响应时间',
      dataIndex: 'responseTime',
      key: 'responseTime',
      render: (time: number) => `${time}ms`,
      width: 100,
    },
    {
      title: 'Token数',
      dataIndex: 'tokens',
      key: 'tokens',
      width: 80,
    },
    {
      title: '成本',
      dataIndex: 'cost',
      key: 'cost',
      render: (cost: number) => `$${cost.toFixed(4)}`,
      width: 80,
    },
  ];

  return (
    <div className="ai-agent-monitor" style={{ height }}>
      {/* 头部统计 */}
      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={6}>
          <Card size="small">
            <Statistic
              title="总请求数"
              value={metrics.totalRequests}
              prefix={<ThunderboltOutlined />}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card size="small">
            <Statistic
              title="成功率"
              value={metrics.successRate}
              precision={1}
              suffix="%"
              prefix={<CheckCircleOutlined />}
              valueStyle={{
                color: metrics.successRate > 95 ? '#52c41a' : metrics.successRate > 90 ? '#faad14' : '#ff4d4f',
              }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card size="small">
            <Statistic
              title="平均响应时间"
              value={metrics.averageResponseTime}
              suffix="ms"
              prefix={<ClockCircleOutlined />}
              valueStyle={{
                color: metrics.averageResponseTime < 1000 ? '#52c41a' : metrics.averageResponseTime < 2000 ? '#faad14' : '#ff4d4f',
              }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card size="small">
            <Statistic
              title="总成本"
              value={metrics.totalCost}
              precision={3}
              prefix="$"
              prefix={<DatabaseOutlined />}
              valueStyle={{ color: '#722ed1' }}
            />
          </Card>
        </Col>
      </Row>

      <Row gutter={16} style={{ height: showDetails ? 'calc(100% - 140px)' : 'calc(100% - 80px)' }}>
        {showDetails && (
          <>
            {/* 请求趋势图 */}
            <Col span={16}>
              <Card
                title="请求趋势"
                size="small"
                extra={
                  <Button
                    icon={<ReloadOutlined />}
                    onClick={refreshData}
                    loading={loading}
                    size="small"
                  >
                    刷新
                  </Button>
                }
                style={{ height: '100%' }}
              >
                <ResponsiveContainer width="100%" height={250}>
                  <LineChart data={timeSeriesData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="timestamp" />
                    <YAxis yAxisId="left" />
                    <YAxis yAxisId="right" orientation="right" />
                    <RechartsTooltip />
                    <Legend />
                    <Line
                      yAxisId="left"
                      type="monotone"
                      dataKey="requests"
                      stroke="#1890ff"
                      name="请求数"
                      strokeWidth={2}
                    />
                    <Line
                      yAxisId="left"
                      type="monotone"
                      dataKey="errors"
                      stroke="#ff4d4f"
                      name="错误数"
                      strokeWidth={2}
                    />
                    <Line
                      yAxisId="right"
                      type="monotone"
                      dataKey="avgResponseTime"
                      stroke="#52c41a"
                      name="平均响应时间(ms)"
                      strokeWidth={2}
                    />
                  </LineChart>
                </ResponsiveContainer>
              </Card>
            </Col>

            {/* 模型使用分布 */}
            <Col span={8}>
              <Card title="模型使用分布" size="small" style={{ height: '100%' }}>
                <ResponsiveContainer width="100%" height={250}>
                  <PieChart>
                    <Pie
                      data={modelUsageData}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      label={({ model, requests }) => `${model}: ${requests}`}
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="requests"
                    >
                      {modelUsageData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <RechartsTooltip />
                  </PieChart>
                </ResponsiveContainer>
              </Card>
            </Col>
          </>
        )}

        {/* 最近请求列表 */}
        <Col span={showDetails ? 24 : 24}>
          <Card
            title={
              <Space>
                最近请求
                {loading && <Badge status="processing" />}
              </Space>
            }
            size="small"
            style={{ height: showDetails ? 'calc(100% - 290px)' : 'calc(100% - 50px)' }}
            bodyStyle={{ padding: 0 }}
          >
            <Table
              dataSource={recentRequests}
              columns={requestColumns}
              pagination={false}
              size="small"
              scroll={{ y: 200 }}
              rowKey="id"
            />
          </Card>
        </Col>
      </Row>
    </div>
  );
};

export default AIAgentMonitor;