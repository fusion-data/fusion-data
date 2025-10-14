import { useState, useEffect, useCallback, useMemo } from 'react';
import {
  Card,
  Row,
  Col,
  Progress,
  Button,
  Space,
  Typography,
  Statistic,
  Alert,
  Divider,
  Tabs,
  Switch,
  Select,
  Tag,
  List,
  Tooltip,
} from 'antd';
import {
  ThunderboltOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  DashboardOutlined,
  ReloadOutlined,
  SettingOutlined,
  MemoryOutlined,
  ClockCircleOutlined,
  DatabaseOutlined,
  ApiOutlined,
} from '@ant-design/icons';

const { Title, Text } = Typography;
const { TabPane } = Tabs;

// 性能指标接口
interface PerformanceMetrics {
  workflowEngine: {
    executionTime: number;
    memoryUsage: number;
    cacheHitRate: number;
    concurrentWorkflows: number;
    throughput: number;
  };
  canvas: {
    renderTime: number;
    nodeCount: number;
    edgeCount: number;
    fps: number;
    memoryUsage: number;
  };
  dataProcessing: {
    processingTime: number;
    throughput: number;
    queueSize: number;
    errorRate: number;
  };
  system: {
    cpuUsage: number;
    memoryUsage: number;
    networkLatency: number;
    diskIO: number;
  };
}

// 优化建议接口
interface OptimizationSuggestion {
  id: string;
  type: 'warning' | 'error' | 'info';
  category: 'workflow' | 'canvas' | 'data' | 'system';
  title: string;
  description: string;
  impact: 'high' | 'medium' | 'low';
  action: string;
  automated: boolean;
}

interface PerformanceOptimizerProps {
  onOptimize?: (type: string) => void;
  refreshInterval?: number;
  showSettings?: boolean;
}

export const PerformanceOptimizer: React.FC<PerformanceOptimizerProps> = ({
  onOptimize,
  refreshInterval = 5000,
  showSettings = true,
}) => {
  const [metrics, setMetrics] = useState<PerformanceMetrics>({
    workflowEngine: {
      executionTime: 0,
      memoryUsage: 0,
      cacheHitRate: 0,
      concurrentWorkflows: 0,
      throughput: 0,
    },
    canvas: {
      renderTime: 0,
      nodeCount: 0,
      edgeCount: 0,
      fps: 0,
      memoryUsage: 0,
    },
    dataProcessing: {
      processingTime: 0,
      throughput: 0,
      queueSize: 0,
      errorRate: 0,
    },
    system: {
      cpuUsage: 0,
      memoryUsage: 0,
      networkLatency: 0,
      diskIO: 0,
    },
  });

  const [suggestions, setSuggestions] = useState<OptimizationSuggestion[]>([]);
  const [autoOptimize, setAutoOptimize] = useState(false);
  const [selectedCategory, setSelectedCategory] = useState<string>('all');

  // 生成模拟性能指标
  const generateMetrics = useCallback((): PerformanceMetrics => ({
    workflowEngine: {
      executionTime: Math.random() * 1000 + 200,
      memoryUsage: Math.random() * 100 + 50,
      cacheHitRate: Math.random() * 40 + 60,
      concurrentWorkflows: Math.floor(Math.random() * 10) + 1,
      throughput: Math.random() * 50 + 10,
    },
    canvas: {
      renderTime: Math.random() * 16 + 2,
      nodeCount: Math.floor(Math.random() * 200) + 50,
      edgeCount: Math.floor(Math.random() * 300) + 100,
      fps: Math.random() * 30 + 30,
      memoryUsage: Math.random() * 80 + 20,
    },
    dataProcessing: {
      processingTime: Math.random() * 500 + 100,
      throughput: Math.random() * 1000 + 200,
      queueSize: Math.floor(Math.random() * 50) + 10,
      errorRate: Math.random() * 5,
    },
    system: {
      cpuUsage: Math.random() * 60 + 20,
      memoryUsage: Math.random() * 70 + 30,
      networkLatency: Math.random() * 100 + 10,
      diskIO: Math.random() * 200 + 50,
    },
  }), []);

  // 生成优化建议
  const generateSuggestions = useCallback((): OptimizationSuggestion[] => {
    const suggestions: OptimizationSuggestion[] = [];

    // 工作流引擎优化建议
    if (metrics.workflowEngine.executionTime > 800) {
      suggestions.push({
        id: 'workflow_execution_time',
        type: 'warning',
        category: 'workflow',
        title: '工作流执行时间过长',
        description: `当前执行时间 ${metrics.workflowEngine.executionTime.toFixed(0)}ms，建议优化节点执行逻辑`,
        impact: 'high',
        action: 'optimize_execution',
        automated: true,
      });
    }

    if (metrics.workflowEngine.cacheHitRate < 70) {
      suggestions.push({
        id: 'cache_hit_rate',
        type: 'info',
        category: 'workflow',
        title: '缓存命中率偏低',
        description: `当前缓存命中率 ${metrics.workflowEngine.cacheHitRate.toFixed(1)}%，建议调整缓存策略`,
        impact: 'medium',
        action: 'optimize_cache',
        automated: true,
      });
    }

    // 画布性能优化建议
    if (metrics.canvas.fps < 30) {
      suggestions.push({
        id: 'canvas_fps',
        type: 'error',
        category: 'canvas',
        title: '画布渲染帧率过低',
        description: `当前帧率 ${metrics.canvas.fps.toFixed(0)}fps，建议启用虚拟化渲染`,
        impact: 'high',
        action: 'enable_virtualization',
        automated: true,
      });
    }

    if (metrics.canvas.nodeCount > 150) {
      suggestions.push({
        id: 'canvas_node_count',
        type: 'warning',
        category: 'canvas',
        title: '节点数量过多',
        description: `当前节点数量 ${metrics.canvas.nodeCount}，建议启用节点分组或懒加载`,
        impact: 'medium',
        action: 'enable_grouping',
        automated: false,
      });
    }

    // 数据处理优化建议
    if (metrics.dataProcessing.errorRate > 3) {
      suggestions.push({
        id: 'data_error_rate',
        type: 'error',
        category: 'data',
        title: '数据处理错误率过高',
        description: `当前错误率 ${metrics.dataProcessing.errorRate.toFixed(1)}%，建议检查数据源配置`,
        impact: 'high',
        action: 'check_data_sources',
        automated: false,
      });
    }

    if (metrics.dataProcessing.queueSize > 40) {
      suggestions.push({
        id: 'data_queue_size',
        type: 'warning',
        category: 'data',
        title: '数据处理队列积压',
        description: `当前队列大小 ${metrics.dataProcessing.queueSize}，建议增加处理并发数`,
        impact: 'medium',
        action: 'increase_concurrency',
        automated: true,
      });
    }

    // 系统资源优化建议
    if (metrics.system.memoryUsage > 80) {
      suggestions.push({
        id: 'system_memory',
        type: 'error',
        category: 'system',
        title: '系统内存使用率过高',
        description: `当前内存使用率 ${metrics.system.memoryUsage.toFixed(1)}%，建议清理缓存或增加内存`,
        impact: 'high',
        action: 'cleanup_memory',
        automated: true,
      });
    }

    if (metrics.system.cpuUsage > 75) {
      suggestions.push({
        id: 'system_cpu',
        type: 'warning',
        category: 'system',
        title: '系统CPU使用率较高',
        description: `当前CPU使用率 ${metrics.system.cpuUsage.toFixed(1)}%，建议优化计算密集型操作`,
        impact: 'medium',
        action: 'optimize_cpu',
        automated: true,
      });
    }

    return suggestions;
  }, [metrics]);

  // 刷新数据
  const refreshData = useCallback(() => {
    const newMetrics = generateMetrics();
    setMetrics(newMetrics);
    setSuggestions(generateSuggestions());
  }, [generateMetrics, generateSuggestions]);

  // 自动刷新
  useEffect(() => {
    refreshData();
    const interval = setInterval(refreshData, refreshInterval);
    return () => clearInterval(interval);
  }, [refreshInterval, refreshData]);

  // 执行优化
  const handleOptimize = useCallback((type: string) => {
    console.log('执行优化:', type);
    onOptimize?.(type);

    // 模拟优化效果
    setTimeout(() => {
      refreshData();
    }, 1000);
  }, [onOptimize, refreshData]);

  // 自动优化
  useEffect(() => {
    if (autoOptimize) {
      const automatedSuggestions = suggestions.filter(s => s.automated && s.type !== 'error');
      automatedSuggestions.forEach(suggestion => {
        setTimeout(() => {
          handleOptimize(suggestion.action);
        }, 1000);
      });
    }
  }, [autoOptimize, suggestions, handleOptimize]);

  // 过滤建议
  const filteredSuggestions = useMemo(() => {
    if (selectedCategory === 'all') return suggestions;
    return suggestions.filter(s => s.category === selectedCategory);
  }, [suggestions, selectedCategory]);

  // 获取性能分数
  const getPerformanceScore = useCallback(() => {
    let score = 100;

    // 工作流引擎分数
    if (metrics.workflowEngine.executionTime > 800) score -= 15;
    if (metrics.workflowEngine.cacheHitRate < 70) score -= 10;

    // 画布性能分数
    if (metrics.canvas.fps < 30) score -= 20;
    if (metrics.canvas.nodeCount > 150) score -= 10;

    // 数据处理分数
    if (metrics.dataProcessing.errorRate > 3) score -= 15;
    if (metrics.dataProcessing.queueSize > 40) score -= 10;

    // 系统资源分数
    if (metrics.system.memoryUsage > 80) score -= 15;
    if (metrics.system.cpuUsage > 75) score -= 10;

    return Math.max(0, score);
  }, [metrics]);

  // 获取状态颜色
  const getStatusColor = (value: number, thresholds: { warning: number; error: number }) => {
    if (value >= thresholds.error) return '#ff4d4f';
    if (value >= thresholds.warning) return '#faad14';
    return '#52c41a';
  };

  // 渲染性能概览
  const renderPerformanceOverview = () => {
    const score = getPerformanceScore();
    const scoreColor = score >= 80 ? '#52c41a' : score >= 60 ? '#faad14' : '#ff4d4f';

    return (
      <Row gutter={16}>
        <Col span={6}>
          <Card>
            <Statistic
              title="性能评分"
              value={score}
              suffix="/100"
              valueStyle={{ color: scoreColor }}
              prefix={<DashboardOutlined />}
            />
            <Progress
              percent={score}
              strokeColor={scoreColor}
              showInfo={false}
              size="small"
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="工作流执行时间"
              value={metrics.workflowEngine.executionTime}
              suffix="ms"
              valueStyle={{
                color: getStatusColor(metrics.workflowEngine.executionTime, { warning: 500, error: 800 })
              }}
              prefix={<ClockCircleOutlined />}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="画布帧率"
              value={metrics.canvas.fps}
              suffix="fps"
              valueStyle={{
                color: getStatusColor(metrics.canvas.fps, { warning: 30, error: 20 })
              }}
              prefix={<ThunderboltOutlined />}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="内存使用率"
              value={metrics.system.memoryUsage}
              suffix="%"
              valueStyle={{
                color: getStatusColor(metrics.system.memoryUsage, { warning: 70, error: 85 })
              }}
              prefix={<MemoryOutlined />}
            />
          </Card>
        </Col>
      </Row>
    );
  };

  // 渲染详细指标
  const renderDetailedMetrics = () => (
    <Row gutter={16}>
      <Col span={12}>
        <Card title="工作流引擎指标" size="small">
          <Space direction="vertical" style={{ width: '100%' }}>
            <div>
              <Text>内存使用: </Text>
              <Progress
                percent={metrics.workflowEngine.memoryUsage}
                size="small"
                format={percent => `${percent}%`}
              />
            </div>
            <div>
              <Text>缓存命中率: </Text>
              <Progress
                percent={metrics.workflowEngine.cacheHitRate}
                size="small"
                strokeColor="#52c41a"
                format={percent => `${percent.toFixed(1)}%`}
              />
            </div>
            <div>
              <Text>并发工作流: </Text>
              <Tag color="blue">{metrics.workflowEngine.concurrentWorkflows}</Tag>
            </div>
            <div>
              <Text>吞吐量: </Text>
              <Tag color="green">{metrics.workflowEngine.throughput.toFixed(1)} ops/s</Tag>
            </div>
          </Space>
        </Card>
      </Col>
      <Col span={12}>
        <Card title="画布性能指标" size="small">
          <Space direction="vertical" style={{ width: '100%' }}>
            <div>
              <Text>渲染时间: </Text>
              <Tag color={metrics.canvas.renderTime > 10 ? 'orange' : 'green'}>
                {metrics.canvas.renderTime.toFixed(1)}ms
              </Tag>
            </div>
            <div>
              <Text>节点数量: </Text>
              <Tag color="blue">{metrics.canvas.nodeCount}</Tag>
            </div>
            <div>
              <Text>连接数量: </Text>
              <Tag color="blue">{metrics.canvas.edgeCount}</Tag>
            </div>
            <div>
              <Text>画布内存: </Text>
              <Progress
                percent={metrics.canvas.memoryUsage}
                size="small"
                format={percent => `${percent}%`}
              />
            </div>
          </Space>
        </Card>
      </Col>
    </Row>
  );

  // 渲染优化建议
  const renderOptimizationSuggestions = () => (
    <Card
      title={
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span>优化建议</span>
          <Space>
            <Select
              value={selectedCategory}
              onChange={setSelectedCategory}
              style={{ width: 120 }}
              size="small"
            >
              <Select.Option value="all">全部</Select.Option>
              <Select.Option value="workflow">工作流</Select.Option>
              <Select.Option value="canvas">画布</Select.Option>
              <Select.Option value="data">数据</Select.Option>
              <Select.Option value="system">系统</Select.Option>
            </Select>
            <Switch
              checkedChildren="自动优化"
              unCheckedChildren="手动"
              checked={autoOptimize}
              onChange={setAutoOptimize}
              size="small"
            />
          </Space>
        </div>
      }
      size="small"
      extra={
        <Button
          type="text"
          size="small"
          icon={<ReloadOutlined />}
          onClick={refreshData}
        >
          刷新
        </Button>
      }
    >
      <List
        dataSource={filteredSuggestions}
        renderItem={(suggestion) => (
          <List.Item
            actions={[
              <Tooltip title={suggestion.automated ? '可自动优化' : '需要手动处理'}>
                <Button
                  type={suggestion.type === 'error' ? 'primary' : 'default'}
                  size="small"
                  onClick={() => handleOptimize(suggestion.action)}
                  disabled={!suggestion.automated}
                >
                  {suggestion.automated ? '自动优化' : '手动处理'}
                </Button>
              </Tooltip>
            ]}
          >
            <List.Item.Meta
              avatar={
                <div style={{ width: 8, height: 8, borderRadius: '50%', backgroundColor:
                  suggestion.type === 'error' ? '#ff4d4f' :
                  suggestion.type === 'warning' ? '#faad14' : '#1890ff'
                }} />
              }
              title={
                <Space>
                  <span>{suggestion.title}</span>
                  <Tag color={
                    suggestion.impact === 'high' ? 'red' :
                    suggestion.impact === 'medium' ? 'orange' : 'blue'
                  } size="small">
                    {suggestion.impact === 'high' ? '高' :
                     suggestion.impact === 'medium' ? '中' : '低'}影响
                  </Tag>
                </Space>
              }
              description={suggestion.description}
            />
          </List.Item>
        )}
      />
    </Card>
  );

  return (
    <div style={{ padding: 24 }}>
      <div style={{ marginBottom: 24 }}>
        <Row justify="space-between" align="middle">
          <Col>
            <Title level={2} style={{ margin: 0 }}>
              <ThunderboltOutlined style={{ marginRight: 8 }} />
              性能优化中心
            </Title>
            <Text type="secondary">
              实时监控系统性能并提供优化建议
            </Text>
          </Col>
          <Col>
            <Space>
              <Button
                icon={<ReloadOutlined />}
                onClick={refreshData}
              >
                刷新数据
              </Button>
              {showSettings && (
                <Button icon={<SettingOutlined />}>
                  设置
                </Button>
              )}
            </Space>
          </Col>
        </Row>
      </div>

      <Tabs defaultActiveKey="overview" size="small">
        <TabPane tab="性能概览" key="overview">
          {renderPerformanceOverview()}
          <div style={{ marginTop: 24 }}>
            {renderOptimizationSuggestions()}
          </div>
        </TabPane>

        <TabPane tab="详细指标" key="metrics">
          {renderDetailedMetrics()}
          <div style={{ marginTop: 24 }}>
            <Row gutter={16}>
              <Col span={12}>
                <Card title="数据处理指标" size="small">
                  <Space direction="vertical" style={{ width: '100%' }}>
                    <div>
                      <Text>处理时间: </Text>
                      <Tag color={metrics.dataProcessing.processingTime > 300 ? 'orange' : 'green'}>
                        {metrics.dataProcessing.processingTime.toFixed(0)}ms
                      </Tag>
                    </div>
                    <div>
                      <Text>吞吐量: </Text>
                      <Tag color="green">{metrics.dataProcessing.throughput.toFixed(0)} ops/s</Tag>
                    </div>
                    <div>
                      <Text>队列大小: </Text>
                      <Tag color={metrics.dataProcessing.queueSize > 40 ? 'orange' : 'blue'}>
                        {metrics.dataProcessing.queueSize}
                      </Tag>
                    </div>
                    <div>
                      <Text>错误率: </Text>
                      <Tag color={metrics.dataProcessing.errorRate > 3 ? 'red' : 'green'}>
                        {metrics.dataProcessing.errorRate.toFixed(1)}%
                      </Tag>
                    </div>
                  </Space>
                </Card>
              </Col>
              <Col span={12}>
                <Card title="系统资源指标" size="small">
                  <Space direction="vertical" style={{ width: '100%' }}>
                    <div>
                      <Text>CPU使用率: </Text>
                      <Progress
                        percent={metrics.system.cpuUsage}
                        size="small"
                        strokeColor={getStatusColor(metrics.system.cpuUsage, { warning: 60, error: 80 })}
                      />
                    </div>
                    <div>
                      <Text>内存使用率: </Text>
                      <Progress
                        percent={metrics.system.memoryUsage}
                        size="small"
                        strokeColor={getStatusColor(metrics.system.memoryUsage, { warning: 70, error: 85 })}
                      />
                    </div>
                    <div>
                      <Text>网络延迟: </Text>
                      <Tag color={metrics.system.networkLatency > 100 ? 'orange' : 'green'}>
                        {metrics.system.networkLatency.toFixed(0)}ms
                      </Tag>
                    </div>
                    <div>
                      <Text>磁盘IO: </Text>
                      <Tag color="blue">{metrics.system.diskIO.toFixed(0)} MB/s</Tag>
                    </div>
                  </Space>
                </Card>
              </Col>
            </Row>
          </div>
        </TabPane>

        <TabPane tab="优化历史" key="history">
          <Alert
            message="优化历史记录"
            description="系统会记录所有优化操作的历史记录和效果"
            type="info"
            showIcon
          />
        </TabPane>
      </Tabs>
    </div>
  );
};

export default PerformanceOptimizer;