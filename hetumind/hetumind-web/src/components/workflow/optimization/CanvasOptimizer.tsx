import { useState, useCallback, useEffect, useMemo } from 'react';
import {
  Card,
  Row,
  Col,
  Button,
  Space,
  Typography,
  Switch,
  Slider,
  Select,
  Divider,
  Tag,
  Progress,
  Statistic,
  Alert,
  List,
  Tooltip,
  Badge,
  InputNumber,
  Tabs,
} from 'antd';
import {
  DesktopOutlined,
  MobileOutlined,
  EyeOutlined,
  ThunderboltOutlined,
  SettingOutlined,
  ZoomInOutlined,
  ZoomOutOutlined,
  ReloadOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  MonitorOutlined,
  ClockCircleOutlined,
} from '@ant-design/icons';

const { Title, Text } = Typography;

// 画布优化配置接口
interface CanvasOptimizationConfig {
  rendering: {
    enableVirtualization: boolean;
    virtualizationThreshold: number;
    enableMinimap: boolean;
    minimapPosition: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
    renderQuality: 'low' | 'medium' | 'high';
    enableSmoothPan: boolean;
    enableSmoothZoom: boolean;
  };
  performance: {
    enableCaching: boolean;
    cacheStrategy: 'node' | 'edge' | 'both';
    maxCacheSize: number;
    enableDebouncing: boolean;
    debounceDelay: number;
    enableBatchUpdates: boolean;
    batchSize: number;
  };
  interaction: {
    enableDragOptimization: boolean;
    enableConnectionOptimization: boolean;
    enableSelectionOptimization: boolean;
    maxSelectableNodes: number;
    enableHoverEffects: boolean;
    enableAnimation: boolean;
  };
  memory: {
    enableMemoryManagement: boolean;
    maxMemoryUsage: number;
    enableLazyLoading: boolean;
    enableNodePooling: boolean;
    cleanupInterval: number;
  };
}

// 性能指标接口
interface CanvasPerformanceMetrics {
  renderTime: number;
  fps: number;
  nodeCount: number;
  edgeCount: number;
  memoryUsage: number;
  cacheHitRate: number;
  interactionLatency: number;
  viewportNodes: number;
}

// 优化建议接口
interface CanvasOptimizationTip {
  id: string;
  category: string;
  title: string;
  description: string;
  impact: 'high' | 'medium' | 'low';
  action: string;
  applicable: boolean;
}

interface CanvasOptimizerProps {
  nodeCount?: number;
  edgeCount?: number;
  onOptimize?: (config: CanvasOptimizationConfig) => void;
}

export const CanvasOptimizer: React.FC<CanvasOptimizerProps> = ({
  nodeCount = 0,
  edgeCount = 0,
  onOptimize,
}) => {
  const [config, setConfig] = useState<CanvasOptimizationConfig>({
    rendering: {
      enableVirtualization: false,
      virtualizationThreshold: 100,
      enableMinimap: true,
      minimapPosition: 'bottom-right',
      renderQuality: 'high',
      enableSmoothPan: true,
      enableSmoothZoom: true,
    },
    performance: {
      enableCaching: true,
      cacheStrategy: 'both',
      maxCacheSize: 500,
      enableDebouncing: true,
      debounceDelay: 100,
      enableBatchUpdates: true,
      batchSize: 10,
    },
    interaction: {
      enableDragOptimization: true,
      enableConnectionOptimization: true,
      enableSelectionOptimization: true,
      maxSelectableNodes: 50,
      enableHoverEffects: true,
      enableAnimation: true,
    },
    memory: {
      enableMemoryManagement: true,
      maxMemoryUsage: 70,
      enableLazyLoading: false,
      enableNodePooling: true,
      cleanupInterval: 30000,
    },
  });

  const [metrics, setMetrics] = useState<CanvasPerformanceMetrics>({
    renderTime: 0,
    fps: 60,
    nodeCount,
    edgeCount,
    memoryUsage: 0,
    cacheHitRate: 0,
    interactionLatency: 0,
    viewportNodes: 0,
  });

  const [optimizationTips, setOptimizationTips] = useState<CanvasOptimizationTip[]>([]);
  const [isOptimizing, setIsOptimizing] = useState(false);

  // 更新配置
  const updateConfig = useCallback((category: keyof CanvasOptimizationConfig, key: string, value: any) => {
    const newConfig = {
      ...config,
      [category]: {
        ...config[category],
        [key]: value,
      },
    };
    setConfig(newConfig);
    onOptimize?.(newConfig);
  }, [config, onOptimize]);

  // 生成性能指标
  const generateMetrics = useCallback((): CanvasPerformanceMetrics => {
    const baseRenderTime = Math.random() * 8 + 2;
    const nodeCountFactor = nodeCount > 100 ? nodeCount / 100 : 1;
    const edgeCountFactor = edgeCount > 200 ? edgeCount / 200 : 1;

    return {
      renderTime: baseRenderTime * nodeCountFactor * edgeCountFactor,
      fps: Math.max(30, 60 - (nodeCountFactor * edgeCountFactor * 5)),
      nodeCount,
      edgeCount,
      memoryUsage: Math.min(90, nodeCount * 0.1 + Math.random() * 20),
      cacheHitRate: config.performance.enableCaching ? Math.random() * 30 + 70 : 0,
      interactionLatency: Math.random() * 50 + 10,
      viewportNodes: Math.min(nodeCount, Math.floor(Math.random() * 50) + 20),
    };
  }, [nodeCount, edgeCount, config.performance.enableCaching]);

  // 生成优化建议
  const generateOptimizationTips = useCallback((): CanvasOptimizationTip[] => {
    const tips: CanvasOptimizationTip[] = [];

    // 节点数量优化
    if (nodeCount > 200) {
      tips.push({
        id: 'enable_virtualization',
        category: '渲染优化',
        title: '启用虚拟化渲染',
        description: `节点数量 (${nodeCount}) 较多，建议启用虚拟化来提升性能`,
        impact: 'high',
        action: 'enable_virtualization',
        applicable: !config.rendering.enableVirtualization,
      });
    }

    // 渲染质量优化
    if (metrics.fps < 45) {
      tips.push({
        id: 'lower_render_quality',
        category: '渲染优化',
        title: '降低渲染质量',
        description: `当前帧率 (${metrics.fps.toFixed(0)}fps) 较低，建议降低渲染质量`,
        impact: 'medium',
        action: 'lower_render_quality',
        applicable: config.rendering.renderQuality === 'high',
      });
    }

    // 缓存优化
    if (!config.performance.enableCaching && nodeCount > 50) {
      tips.push({
        id: 'enable_caching',
        category: '性能优化',
        title: '启用渲染缓存',
        description: '启用缓存可以显著提升重复渲染的性能',
        impact: 'high',
        action: 'enable_caching',
        applicable: true,
      });
    }

    // 动画优化
    if (config.interaction.enableAnimation && metrics.fps < 50) {
      tips.push({
        id: 'disable_animation',
        category: '交互优化',
        title: '禁用动画效果',
        description: '禁用动画可以减少CPU使用并提升响应速度',
        impact: 'medium',
        action: 'disable_animation',
        applicable: true,
      });
    }

    // 悬停效果优化
    if (config.interaction.enableHoverEffects && nodeCount > 150) {
      tips.push({
        id: 'disable_hover_effects',
        category: '交互优化',
        title: '禁用悬停效果',
        description: '大量节点时禁用悬停效果可以提升性能',
        impact: 'low',
        action: 'disable_hover_effects',
        applicable: true,
      });
    }

    // 内存管理优化
    if (metrics.memoryUsage > 80) {
      tips.push({
        id: 'enable_memory_management',
        category: '内存优化',
        title: '优化内存管理',
        description: `内存使用率 (${metrics.memoryUsage.toFixed(1)}%) 较高，建议启用内存优化`,
        impact: 'high',
        action: 'enable_memory_management',
        applicable: !config.memory.enableMemoryManagement,
      });
    }

    // 防抖优化
    if (!config.performance.enableDebouncing && metrics.interactionLatency > 100) {
      tips.push({
        id: 'enable_debouncing',
        category: '性能优化',
        title: '启用防抖处理',
        description: '防抖可以减少不必要的渲染和更新',
        impact: 'medium',
        action: 'enable_debouncing',
        applicable: true,
      });
    }

    return tips.filter(tip => tip.applicable);
  }, [nodeCount, metrics, config]);

  // 执行优化
  const applyOptimization = useCallback((action: string) => {
    setIsOptimizing(true);

    switch (action) {
      case 'enable_virtualization':
        updateConfig('rendering', 'enableVirtualization', true);
        break;
      case 'lower_render_quality':
        updateConfig('rendering', 'renderQuality', 'medium');
        break;
      case 'enable_caching':
        updateConfig('performance', 'enableCaching', true);
        break;
      case 'disable_animation':
        updateConfig('interaction', 'enableAnimation', false);
        break;
      case 'disable_hover_effects':
        updateConfig('interaction', 'enableHoverEffects', false);
        break;
      case 'enable_memory_management':
        updateConfig('memory', 'enableMemoryManagement', true);
        break;
      case 'enable_debouncing':
        updateConfig('performance', 'enableDebouncing', true);
        break;
    }

    setTimeout(() => {
      setIsOptimizing(false);
    }, 1000);
  }, [updateConfig]);

  // 自动应用所有建议
  const applyAllOptimizations = useCallback(() => {
    optimizationTips.forEach(tip => {
      setTimeout(() => {
        applyOptimization(tip.action);
      }, 500);
    });
  }, [optimizationTips, applyOptimization]);

  // 刷新指标
  const refreshMetrics = useCallback(() => {
    setMetrics(generateMetrics());
    setOptimizationTips(generateOptimizationTips());
  }, [generateMetrics, generateOptimizationTips]);

  // 自动刷新指标
  useEffect(() => {
    refreshMetrics();
    const interval = setInterval(refreshMetrics, 3000);
    return () => clearInterval(interval);
  }, [refreshMetrics]);

  // 计算性能分数
  const performanceScore = useMemo(() => {
    let score = 100;

    if (metrics.fps < 30) score -= 30;
    else if (metrics.fps < 45) score -= 15;

    if (metrics.renderTime > 16) score -= 20;
    else if (metrics.renderTime > 10) score -= 10;

    if (metrics.memoryUsage > 80) score -= 20;
    else if (metrics.memoryUsage > 60) score -= 10;

    if (metrics.interactionLatency > 100) score -= 15;
    else if (metrics.interactionLatency > 50) score -= 5;

    return Math.max(0, score);
  }, [metrics]);

  // 渲染性能概览
  const renderPerformanceOverview = () => {
    const scoreColor = performanceScore >= 80 ? '#52c41a' : performanceScore >= 60 ? '#faad14' : '#ff4d4f';

    return (
      <Row gutter={16}>
        <Col span={6}>
          <Card>
            <Statistic
              title="性能分数"
              value={performanceScore}
              suffix="/100"
              valueStyle={{ color: scoreColor }}
              prefix={<MonitorOutlined />}
            />
            <Progress
              percent={performanceScore}
              strokeColor={scoreColor}
              showInfo={false}
              size="small"
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="渲染帧率"
              value={metrics.fps}
              suffix="fps"
              valueStyle={{
                color: metrics.fps >= 45 ? '#52c41a' : metrics.fps >= 30 ? '#faad14' : '#ff4d4f'
              }}
              prefix={<ThunderboltOutlined />}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="渲染时间"
              value={metrics.renderTime}
              suffix="ms"
              valueStyle={{
                color: metrics.renderTime <= 16 ? '#52c41a' : metrics.renderTime <= 25 ? '#faad14' : '#ff4d4f'
              }}
              prefix={<ClockCircleOutlined />}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="内存使用"
              value={metrics.memoryUsage}
              suffix="%"
              valueStyle={{
                color: metrics.memoryUsage <= 60 ? '#52c41a' : metrics.memoryUsage <= 80 ? '#faad14' : '#ff4d4f'
              }}
              prefix={<DesktopOutlined />}
            />
          </Card>
        </Col>
      </Row>
    );
  };

  // 渲染渲染配置
  const renderRenderingConfig = () => (
    <Card title="渲染配置" size="small">
      <Space direction="vertical" style={{ width: '100%' }}>
        <div>
          <Space>
            <Text strong>启用虚拟化: </Text>
            <Switch
              checked={config.rendering.enableVirtualization}
              onChange={(checked) => updateConfig('rendering', 'enableVirtualization', checked)}
            />
            <Tooltip title="当节点数量超过阈值时只渲染可见节点">
              <Text type="secondary">?</Text>
            </Tooltip>
          </Space>
        </div>

        {config.rendering.enableVirtualization && (
          <div>
            <Text strong>虚拟化阈值: </Text>
            <InputNumber
              min={50}
              max={500}
              value={config.rendering.virtualizationThreshold}
              onChange={(value) => updateConfig('rendering', 'virtualizationThreshold', value || 100)}
              style={{ marginTop: 8 }}
            />
          </div>
        )}

        <div>
          <Space>
            <Text strong>启用小地图: </Text>
            <Switch
              checked={config.rendering.enableMinimap}
              onChange={(checked) => updateConfig('rendering', 'enableMinimap', checked)}
            />
          </Space>
        </div>

        {config.rendering.enableMinimap && (
          <div>
            <Text strong>小地图位置: </Text>
            <Select
              value={config.rendering.minimapPosition}
              onChange={(value) => updateConfig('rendering', 'minimapPosition', value)}
              style={{ width: 200, marginTop: 8 }}
            >
              <Select.Option value="top-left">左上</Select.Option>
              <Select.Option value="top-right">右上</Select.Option>
              <Select.Option value="bottom-left">左下</Select.Option>
              <Select.Option value="bottom-right">右下</Select.Option>
            </Select>
          </div>
        )}

        <div>
          <Text strong>渲染质量: </Text>
          <Select
            value={config.rendering.renderQuality}
            onChange={(value) => updateConfig('rendering', 'renderQuality', value)}
            style={{ width: 200, marginTop: 8 }}
          >
            <Select.Option value="low">低 (最快)</Select.Option>
            <Select.Option value="medium">中 (平衡)</Select.Option>
            <Select.Option value="high">高 (最佳)</Select.Option>
          </Select>
        </div>

        <div>
          <Space>
            <Text strong>平滑平移: </Text>
            <Switch
              checked={config.rendering.enableSmoothPan}
              onChange={(checked) => updateConfig('rendering', 'enableSmoothPan', checked)}
            />
          </Space>
        </div>

        <div>
          <Space>
            <Text strong>平滑缩放: </Text>
            <Switch
              checked={config.rendering.enableSmoothZoom}
              onChange={(checked) => updateConfig('rendering', 'enableSmoothZoom', checked)}
            />
          </Space>
        </div>
      </Space>
    </Card>
  );

  // 渲染性能配置
  const renderPerformanceConfig = () => (
    <Card title="性能配置" size="small">
      <Space direction="vertical" style={{ width: '100%' }}>
        <div>
          <Space>
            <Text strong>启用缓存: </Text>
            <Switch
              checked={config.performance.enableCaching}
              onChange={(checked) => updateConfig('performance', 'enableCaching', checked)}
            />
          </Space>
        </div>

        {config.performance.enableCaching && (
          <>
            <div>
              <Text strong>缓存策略: </Text>
              <Select
                value={config.performance.cacheStrategy}
                onChange={(value) => updateConfig('performance', 'cacheStrategy', value)}
                style={{ width: 200, marginTop: 8 }}
              >
                <Select.Option value="node">仅节点</Select.Option>
                <Select.Option value="edge">仅连接</Select.Option>
                <Select.Option value="both">全部</Select.Option>
              </Select>
            </div>

            <div>
              <Text strong>最大缓存大小: </Text>
              <Slider
                min={100}
                max={1000}
                step={50}
                value={config.performance.maxCacheSize}
                onChange={(value) => updateConfig('performance', 'maxCacheSize', value)}
                style={{ marginTop: 8 }}
              />
              <Text type="secondary">当前: {config.performance.maxCacheSize}</Text>
            </div>
          </>
        )}

        <div>
          <Space>
            <Text strong>启用防抖: </Text>
            <Switch
              checked={config.performance.enableDebouncing}
              onChange={(checked) => updateConfig('performance', 'enableDebouncing', checked)}
            />
          </Space>
        </div>

        {config.performance.enableDebouncing && (
          <div>
            <Text strong>防抖延迟: </Text>
            <Slider
              min={50}
              max={500}
              step={50}
              value={config.performance.debounceDelay}
              onChange={(value) => updateConfig('performance', 'debounceDelay', value)}
              style={{ marginTop: 8 }}
            />
            <Text type="secondary">当前: {config.performance.debounceDelay}ms</Text>
          </div>
        )}

        <div>
          <Space>
            <Text strong>启用批量更新: </Text>
            <Switch
              checked={config.performance.enableBatchUpdates}
              onChange={(checked) => updateConfig('performance', 'enableBatchUpdates', checked)}
            />
          </Space>
        </div>

        {config.performance.enableBatchUpdates && (
          <div>
            <Text strong>批量大小: </Text>
            <Slider
              min={5}
              max={50}
              value={config.performance.batchSize}
              onChange={(value) => updateConfig('performance', 'batchSize', value)}
              style={{ marginTop: 8 }}
            />
            <Text type="secondary">当前: {config.performance.batchSize}</Text>
          </div>
        )}
      </Space>
    </Card>
  );

  // 渲染优化建议
  const renderOptimizationTips = () => (
    <Card
      title="优化建议"
      size="small"
      extra={
        <Space>
          <Badge count={optimizationTips.length} showZero>
            <ExclamationCircleOutlined />
          </Badge>
          <Button
            type="primary"
            size="small"
            onClick={applyAllOptimizations}
            disabled={optimizationTips.length === 0}
            loading={isOptimizing}
          >
            应用全部
          </Button>
          <Button
            size="small"
            icon={<ReloadOutlined />}
            onClick={refreshMetrics}
          >
            刷新
          </Button>
        </Space>
      }
    >
      {optimizationTips.length === 0 ? (
        <Alert
          message="无需优化"
          description="当前配置已经是最优状态"
          type="success"
          showIcon
        />
      ) : (
        <List
          dataSource={optimizationTips}
          renderItem={(tip) => (
            <List.Item
              actions={[
                <Button
                  type={tip.impact === 'high' ? 'primary' : 'default'}
                  size="small"
                  onClick={() => applyOptimization(tip.action)}
                  loading={isOptimizing}
                >
                  应用
                </Button>
              ]}
            >
              <List.Item.Meta
                avatar={
                  <div style={{
                    width: 8,
                    height: 8,
                    borderRadius: '50%',
                    backgroundColor: tip.impact === 'high' ? '#ff4d4f' :
                                   tip.impact === 'medium' ? '#faad14' : '#1890ff'
                  }} />
                }
                title={
                  <Space>
                    <span>{tip.title}</span>
                    <Tag color={
                      tip.impact === 'high' ? 'red' :
                      tip.impact === 'medium' ? 'orange' : 'blue'
                    }>
                      {tip.impact === 'high' ? '高' :
                       tip.impact === 'medium' ? '中' : '低'}影响
                    </Tag>
                    <Tag color="blue">{tip.category}</Tag>
                  </Space>
                }
                description={tip.description}
              />
            </List.Item>
          )}
        />
      )}
    </Card>
  );

  return (
    <div style={{ padding: 24 }}>
      <div style={{ marginBottom: 24 }}>
        <Row justify="space-between" align="middle">
          <Col>
            <Title level={2} style={{ margin: 0 }}>
              <EyeOutlined style={{ marginRight: 8 }} />
              画布性能优化器
            </Title>
            <Text type="secondary">
              优化工作流画布的渲染性能和交互体验
            </Text>
          </Col>
          <Col>
            <Space>
              <Button
                icon={<ReloadOutlined />}
                onClick={refreshMetrics}
              >
                刷新指标
              </Button>
            </Space>
          </Col>
        </Row>
      </div>

      <Tabs
        defaultActiveKey="overview"
        size="small"
        items={[
          {
            key: 'overview',
            label: '性能概览',
            children: (
              <>
                {renderPerformanceOverview()}
                <div style={{ marginTop: 24 }}>
                  {renderOptimizationTips()}
                </div>
              </>
            ),
          },
          {
            key: 'rendering',
            label: '渲染配置',
            children: renderRenderingConfig(),
          },
          {
            key: 'performance',
            label: '性能配置',
            children: renderPerformanceConfig(),
          },
          {
            key: 'metrics',
            label: '详细指标',
            children: (
              <Row gutter={16}>
                <Col span={12}>
                  <Card title="基础指标" size="small">
                    <Space direction="vertical" style={{ width: '100%' }}>
                      <div>
                        <Text>节点数量: </Text>
                        <Tag color="blue">{metrics.nodeCount}</Tag>
                      </div>
                      <div>
                        <Text>连接数量: </Text>
                        <Tag color="blue">{metrics.edgeCount}</Tag>
                      </div>
                      <div>
                        <Text>视口节点: </Text>
                        <Tag color="green">{metrics.viewportNodes}</Tag>
                      </div>
                      <div>
                        <Text>交互延迟: </Text>
                        <Tag color={metrics.interactionLatency > 100 ? 'orange' : 'green'}>
                          {metrics.interactionLatency.toFixed(0)}ms
                        </Tag>
                      </div>
                    </Space>
                  </Card>
                </Col>
                <Col span={12}>
                  <Card title="缓存指标" size="small">
                    <Space direction="vertical" style={{ width: '100%' }}>
                      <div>
                        <Text>缓存命中率: </Text>
                        <Progress
                          percent={metrics.cacheHitRate}
                          size="small"
                          format={percent => `${percent?.toFixed(1)}%`}
                        />
                      </div>
                      <div>
                        <Text>缓存状态: </Text>
                        <Tag color={config.performance.enableCaching ? 'green' : 'red'}>
                          {config.performance.enableCaching ? '已启用' : '已禁用'}
                        </Tag>
                      </div>
                      <div>
                        <Text>防抖状态: </Text>
                        <Tag color={config.performance.enableDebouncing ? 'green' : 'red'}>
                          {config.performance.enableDebouncing ? '已启用' : '已禁用'}
                        </Tag>
                      </div>
                      <div>
                        <Text>虚拟化状态: </Text>
                        <Tag color={config.rendering.enableVirtualization ? 'green' : 'red'}>
                          {config.rendering.enableVirtualization ? '已启用' : '已禁用'}
                        </Tag>
                      </div>
                    </Space>
                  </Card>
                </Col>
              </Row>
            ),
          },
        ]}
      />
    </div>
  );
};

export default CanvasOptimizer;