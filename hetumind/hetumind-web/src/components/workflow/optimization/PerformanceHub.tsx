import React, { useState, useCallback, useEffect } from 'react';
import {
  Card,
  Row,
  Col,
  Button,
  Space,
  Typography,
  Tabs,
  Alert,
  Badge,
  Statistic,
  Progress,
  List,
  Tooltip,
  Switch,
} from 'antd';
import {
  ThunderboltOutlined,
  DashboardOutlined,
  SettingOutlined,
  MonitorOutlined,
  EyeOutlined,
  DatabaseOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  RocketOutlined,
  RefreshOutlined,
} from '@ant-design/icons';

import {
  PerformanceOptimizer,
  WorkflowEngineOptimizer,
  CanvasOptimizer,
  PerformanceMonitor,
  useMemoryMonitor,
  useFPSMonitor,
} from './index';

const { Title, Text } = Typography;
const { TabPane } = Tabs;

// 全局性能状态接口
interface GlobalPerformanceStatus {
  overallScore: number;
  criticalIssues: number;
  warnings: number;
  optimizations: number;
  lastOptimization: number | null;
  autoOptimizationEnabled: boolean;
}

// 性能事件接口
interface PerformanceEvent {
  id: string;
  type: 'optimization' | 'warning' | 'error' | 'info';
  category: string;
  title: string;
  description: string;
  timestamp: number;
  impact: 'high' | 'medium' | 'low';
}

interface PerformanceHubProps {
  refreshInterval?: number;
  showSettings?: boolean;
  onOptimizationComplete?: (results: any) => void;
}

export const PerformanceHub: React.FC<PerformanceHubProps> = ({
  refreshInterval = 10000,
  showSettings = true,
  onOptimizationComplete,
}) => {
  const [activeTab, setActiveTab] = useState('overview');
  const [globalStatus, setGlobalStatus] = useState<GlobalPerformanceStatus>({
    overallScore: 85,
    criticalIssues: 0,
    warnings: 2,
    optimizations: 5,
    lastOptimization: null,
    autoOptimizationEnabled: false,
  });

  const [recentEvents, setRecentEvents] = useState<PerformanceEvent[]>([]);
  const [isOptimizing, setIsOptimizing] = useState(false);
  const [optimizationHistory, setOptimizationHistory] = useState<any[]>([]);

  const memoryUsage = useMemoryMonitor();
  const fps = useFPSMonitor();
  const monitor = PerformanceMonitor.getInstance();

  // 生成性能事件
  const generatePerformanceEvents = useCallback((): PerformanceEvent[] => {
    const events: PerformanceEvent[] = [
      {
        id: '1',
        type: 'optimization',
        category: '引擎优化',
        title: '并发配置已优化',
        description: '最大并发节点数已从3提升到5，吞吐量提升73%',
        timestamp: Date.now() - 120000,
        impact: 'high',
      },
      {
        id: '2',
        type: 'warning',
        category: '内存警告',
        title: '内存使用率偏高',
        description: '当前内存使用率为78%，建议清理缓存',
        timestamp: Date.now() - 300000,
        impact: 'medium',
      },
      {
        id: '3',
        type: 'info',
        category: '缓存优化',
        title: '缓存命中率提升',
        description: '缓存命中率从65%提升到92%',
        timestamp: Date.now() - 600000,
        impact: 'medium',
      },
      {
        id: '4',
        type: 'optimization',
        category: '画布优化',
        title: '虚拟化渲染已启用',
        description: '节点数量超过阈值，自动启用虚拟化渲染',
        timestamp: Date.now() - 900000,
        impact: 'high',
      },
      {
        id: '5',
        type: 'error',
        category: '性能异常',
        title: '帧率过低',
        description: '画布帧率降至25fps，建议降低渲染质量',
        timestamp: Date.now() - 1200000,
        impact: 'high',
      },
    ];

    return events.sort((a, b) => b.timestamp - a.timestamp);
  }, []);

  // 刷新性能状态
  const refreshPerformanceStatus = useCallback(() => {
    // 模拟性能分数计算
    const baseScore = 85;
    let scoreDeduction = 0;

    if (memoryUsage && memoryUsage.percentage > 80) scoreDeduction += 15;
    if (fps < 30) scoreDeduction += 20;
    if (fps < 45) scoreDeduction += 10;

    const overallScore = Math.max(0, baseScore - scoreDeduction);

    setGlobalStatus(prev => ({
      ...prev,
      overallScore,
      lastOptimization: prev.lastOptimization || Date.now() - 3600000,
    }));

    setRecentEvents(generatePerformanceEvents());
  }, [memoryUsage, fps, generatePerformanceEvents]);

  // 执行全局优化
  const performGlobalOptimization = useCallback(async () => {
    setIsOptimizing(true);

    try {
      // 模拟全局优化过程
      await new Promise(resolve => setTimeout(resolve, 3000));

      const optimizationResults = {
        engineOptimization: {
          throughput: '+73%',
          executionTime: '-29%',
          successRate: '+10%',
        },
        canvasOptimization: {
          renderTime: '-45%',
          memoryUsage: '-25%',
          fps: '+120%',
        },
        systemOptimization: {
          cpuUsage: '-15%',
          memoryUsage: '-20%',
          responseTime: '-35%',
        },
      };

      setOptimizationHistory(prev => [
        {
          id: Date.now(),
          timestamp: Date.now(),
          results: optimizationResults,
          score: 92,
        },
        ...prev.slice(0, 9), // Keep only last 10 records
      ]);

      setGlobalStatus(prev => ({
        ...prev,
        overallScore: 92,
        lastOptimization: Date.now(),
      }));

      onOptimizationComplete?.(optimizationResults);

    } catch (error) {
      console.error('全局优化失败:', error);
    } finally {
      setIsOptimizing(false);
    }
  }, [onOptimizationComplete]);

  // 自动优化
  useEffect(() => {
    if (globalStatus.autoOptimizationEnabled && globalStatus.overallScore < 70) {
      performGlobalOptimization();
    }
  }, [globalStatus.autoOptimizationEnabled, globalStatus.overallScore, performGlobalOptimization]);

  // 自动刷新
  useEffect(() => {
    refreshPerformanceStatus();
    const interval = setInterval(refreshPerformanceStatus, refreshInterval);
    return () => clearInterval(interval);
  }, [refreshInterval, refreshPerformanceStatus]);

  // 获取状态颜色
  const getStatusColor = (score: number) => {
    if (score >= 80) return '#52c41a';
    if (score >= 60) return '#faad14';
    return '#ff4d4f';
  };

  // 渲染性能概览
  const renderPerformanceOverview = () => (
    <Row gutter={16}>
      <Col span={6}>
        <Card>
          <Statistic
            title="整体性能分数"
            value={globalStatus.overallScore}
            suffix="/100"
            valueStyle={{ color: getStatusColor(globalStatus.overallScore) }}
            prefix={<DashboardOutlined />}
          />
          <Progress
            percent={globalStatus.overallScore}
            strokeColor={getStatusColor(globalStatus.overallScore)}
            showInfo={false}
            size="small"
          />
        </Card>
      </Col>
      <Col span={6}>
        <Card>
          <Statistic
            title="内存使用率"
            value={memoryUsage ? memoryUsage.percentage.toFixed(1) : 0}
            suffix="%"
            valueStyle={{
              color: memoryUsage && memoryUsage.percentage > 80 ? '#ff4d4f' :
                     memoryUsage && memoryUsage.percentage > 60 ? '#faad14' : '#52c41a'
            }}
            prefix={<DatabaseOutlined />}
          />
          {memoryUsage && (
            <Text type="secondary" style={{ fontSize: 12 }}>
              {memoryUsage.used.toFixed(1)}MB / {memoryUsage.total.toFixed(1)}MB
            </Text>
          )}
        </Card>
      </Col>
      <Col span={6}>
        <Card>
          <Statistic
            title="渲染帧率"
            value={fps}
            suffix="fps"
            valueStyle={{
              color: fps >= 45 ? '#52c41a' : fps >= 30 ? '#faad14' : '#ff4d4f'
            }}
            prefix={<EyeOutlined />}
          />
        </Card>
      </Col>
      <Col span={6}>
        <Card>
          <Statistic
            title="活跃优化项"
            value={globalStatus.optimizations}
            valueStyle={{ color: '#1890ff' }}
            prefix={<ThunderboltOutlined />}
          />
          <Space style={{ marginTop: 8 }}>
            <Badge count={globalStatus.criticalIssues} style={{ backgroundColor: '#ff4d4f' }}>
              <Text type="secondary">严重</Text>
            </Badge>
            <Badge count={globalStatus.warnings} style={{ backgroundColor: '#faad14' }}>
              <Text type="secondary">警告</Text>
            </Badge>
          </Space>
        </Card>
      </Col>
    </Row>
  );

  // 渲染性能事件
  const renderPerformanceEvents = () => (
    <Card
      title="性能事件"
      size="small"
      extra={
        <Space>
          <Badge count={recentEvents.length} showZero>
            <ExclamationCircleOutlined />
          </Badge>
          <Button
            type="text"
            size="small"
            icon={<RefreshOutlined />}
            onClick={refreshPerformanceStatus}
          >
            刷新
          </Button>
        </Space>
      }
    >
      <List
        dataSource={recentEvents}
        renderItem={(event) => (
          <List.Item>
            <List.Item.Meta
              avatar={
                <div style={{
                  width: 8,
                  height: 8,
                  borderRadius: '50%',
                  backgroundColor: event.type === 'error' ? '#ff4d4f' :
                                 event.type === 'warning' ? '#faad14' :
                                 event.type === 'optimization' ? '#52c41a' : '#1890ff'
                }} />
              }
              title={
                <Space>
                  <span>{event.title}</span>
                  <Tag color="blue" size="small">{event.category}</Tag>
                  <Tag color={
                    event.impact === 'high' ? 'red' :
                    event.impact === 'medium' ? 'orange' : 'blue'
                  } size="small">
                    {event.impact === 'high' ? '高' :
                     event.impact === 'medium' ? '中' : '低'}影响
                  </Tag>
                </Space>
              }
              description={
                <div>
                  <Text>{event.description}</Text>
                  <div style={{ marginTop: 4 }}>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      {new Date(event.timestamp).toLocaleString()}
                    </Text>
                  </div>
                </div>
              }
            />
          </List.Item>
        )}
      />
    </Card>
  );

  // 渲染优化历史
  const renderOptimizationHistory = () => {
    if (optimizationHistory.length === 0) {
      return (
        <Alert
          message="暂无优化历史"
          description="执行优化操作后，历史记录将显示在这里"
          type="info"
          showIcon
        />
      );
    }

    return (
      <List
        dataSource={optimizationHistory}
        renderItem={(record) => (
          <List.Item>
            <List.Item.Meta
              avatar={
                <div style={{
                  width: 40,
                  height: 40,
                  borderRadius: '50%',
                  backgroundColor: '#f0f0f0',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  fontSize: 12,
                  fontWeight: 'bold',
                  color: '#52c41a',
                }}>
                  {record.score}
                </div>
              }
              title={
                <Space>
                  <span>全局优化完成</span>
                  <Tag color="green">性能分数: {record.score}</Tag>
                </Space>
              }
              description={
                <div>
                  <Text type="secondary">
                    优化时间: {new Date(record.timestamp).toLocaleString()}
                  </Text>
                  <div style={{ marginTop: 4 }}>
                    <Space size="large">
                      <Text>吞吐量: {record.results.engineOptimization.throughput}</Text>
                      <Text>渲染时间: {record.results.canvasOptimization.renderTime}</Text>
                      <Text>响应时间: {record.results.systemOptimization.responseTime}</Text>
                    </Space>
                  </div>
                </div>
              }
            />
          </List.Item>
        )}
      />
    );
  };

  return (
    <div style={{ padding: 24 }}>
      <div style={{ marginBottom: 24 }}>
        <Row justify="space-between" align="middle">
          <Col>
            <Title level={2} style={{ margin: 0 }}>
              <RocketOutlined style={{ marginRight: 8 }} />
              性能优化中心
            </Title>
            <Text type="secondary">
              全方位监控和优化系统性能
            </Text>
          </Col>
          <Col>
            <Space>
              <Space>
                <Text>自动优化:</Text>
                <Switch
                  checked={globalStatus.autoOptimizationEnabled}
                  onChange={(checked) => setGlobalStatus(prev => ({
                    ...prev,
                    autoOptimizationEnabled: checked
                  }))}
                />
              </Space>
              <Button
                type="primary"
                icon={<ThunderboltOutlined />}
                onClick={performGlobalOptimization}
                loading={isOptimizing}
              >
                全局优化
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

      {renderPerformanceOverview()}

      <div style={{ marginTop: 24 }}>
        <Tabs activeKey={activeTab} onChange={setActiveTab}>
          <TabPane tab="性能监控" key="overview">
            <Row gutter={16}>
              <Col span={16}>
                {renderPerformanceEvents()}
              </Col>
              <Col span={8}>
                <Card title="快速优化" size="small">
                  <Space direction="vertical" style={{ width: '100%' }}>
                    <Button
                      type="primary"
                      icon={<ThunderboltOutlined />}
                      onClick={performGlobalOptimization}
                      loading={isOptimizing}
                      block
                    >
                      执行全局优化
                    </Button>
                    <Button
                      icon={<DatabaseOutlined />}
                      onClick={() => {
                        // 模拟内存清理
                        console.log('清理内存缓存');
                      }}
                      block
                    >
                      清理内存缓存
                    </Button>
                    <Button
                      icon={<EyeOutlined />}
                      onClick={() => {
                        // 模拟画布优化
                        console.log('优化画布渲染');
                      }}
                      block
                    >
                      优化画布渲染
                    </Button>
                    <Button
                      icon={<MonitorOutlined />}
                      onClick={() => {
                        // 模拟引擎优化
                        console.log('优化执行引擎');
                      }}
                      block
                    >
                      优化执行引擎
                    </Button>
                  </Space>
                </Card>
              </Col>
            </Row>
          </TabPane>

          <TabPane tab="引擎优化" key="engine">
            <WorkflowEngineOptimizer />
          </TabPane>

          <TabPane tab="画布优化" key="canvas">
            <CanvasOptimizer />
          </TabPane>

          <TabPane tab="系统优化" key="system">
            <PerformanceOptimizer />
          </TabPane>

          <TabPane tab="优化历史" key="history">
            <Card title="优化历史记录" size="small">
              {renderOptimizationHistory()}
            </Card>
          </TabPane>
        </Tabs>
      </div>

      {globalStatus.overallScore < 60 && (
        <Alert
          message="性能警告"
          description="系统性能分数较低，建议立即执行优化操作"
          type="warning"
          showIcon
          style={{ marginTop: 16 }}
          action={
            <Button
              type="primary"
              size="small"
              onClick={performGlobalOptimization}
              loading={isOptimizing}
            >
              立即优化
            </Button>
          }
        />
      )}
    </div>
  );
};

export default PerformanceHub;