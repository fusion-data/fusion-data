import React, { useState, useCallback, useEffect } from 'react';
import {
  Card,
  Row,
  Col,
  Button,
  Space,
  Typography,
  Alert,
  Tabs,
  InputNumber,
  Switch,
  Divider,
  Tag,
  Timeline,
  Progress,
} from 'antd';
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  StopOutlined,
  SettingOutlined,
  ThunderboltOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons';

import {
  WorkflowEngine,
  WorkflowDefinition,
  WorkflowNode,
  WorkflowEdge,
  defaultWorkflowEngine,
  nodeExecutors,
  ExecutionMonitor,
} from './index';

const { Title, Text, Paragraph } = Typography;
const { TabPane } = Tabs;

export const ExecutionDemo: React.FC = () => {
  const [engine] = useState(() => {
    const newEngine = new WorkflowEngine({
      maxConcurrentNodes: 3,
      timeout: 30000,
      retryAttempts: 2,
      enableLogging: true,
      enableMetrics: true,
    });

    // 注册所有节点执行器
    newEngine.registerExecutor('trigger', nodeExecutors.trigger);
    newEngine.registerExecutor('aiAgent', nodeExecutors.aiAgent);
    newEngine.registerExecutor('condition', nodeExecutors.condition);
    newEngine.registerExecutor('action', nodeExecutors.action);
    newEngine.registerExecutor('dataProcessor', nodeExecutors.dataProcessor);

    return newEngine;
  });

  const [engineConfig, setEngineConfig] = useState({
    maxConcurrentNodes: 3,
    timeout: 30000,
    retryAttempts: 2,
    enableLogging: true,
    enableMetrics: true,
  });

  const [currentWorkflow, setCurrentWorkflow] = useState<WorkflowDefinition | null>(null);
  const [executions, setExecutions] = useState<string[]>([]);

  // 示例工作流定义
  const sampleWorkflow: WorkflowDefinition = {
    id: 'demo_workflow',
    name: '演示工作流',
    description: '这是一个演示工作流，展示各种节点类型的执行',
    nodes: [
      {
        id: 'trigger_1',
        type: 'trigger',
        data: {
          triggerType: 'manual',
          config: {},
        },
        inputs: [],
        outputs: ['ai_agent_1'],
        position: { x: 100, y: 100 },
      },
      {
        id: 'ai_agent_1',
        type: 'aiAgent',
        data: {
          agentType: 'chat',
          config: {
            model: 'gpt-3.5-turbo',
            temperature: 0.7,
            maxTokens: 1024,
          },
          input: {
            message: '请分析这个数据并给出建议',
          },
        },
        inputs: ['trigger_1'],
        outputs: ['condition_1'],
        position: { x: 300, y: 100 },
      },
      {
        id: 'condition_1',
        type: 'condition',
        data: {
          conditionType: 'if',
          config: {
            expression: 'response.length > 100',
            trueValue: 'long_response',
            falseValue: 'short_response',
          },
        },
        inputs: ['ai_agent_1'],
        outputs: ['action_1', 'action_2'],
        position: { x: 500, y: 100 },
      },
      {
        id: 'action_1',
        type: 'action',
        data: {
          actionType: 'api',
          config: {
            url: 'https://api.example.com/webhook',
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
          },
        },
        inputs: ['condition_1'],
        outputs: ['data_processor_1'],
        position: { x: 700, y: 50 },
      },
      {
        id: 'action_2',
        type: 'action',
        data: {
          actionType: 'email',
          config: {
            to: 'admin@example.com',
            subject: '工作流执行结果',
            body: '工作流已执行完成',
          },
        },
        inputs: ['condition_1'],
        outputs: ['data_processor_1'],
        position: { x: 700, y: 150 },
      },
      {
        id: 'data_processor_1',
        type: 'dataProcessor',
        data: {
          processorType: 'aggregator',
          config: {
            operation: 'sum',
            field: 'score',
          },
        },
        inputs: ['action_1', 'action_2'],
        outputs: [],
        position: { x: 900, y: 100 },
      },
    ],
    edges: [
      {
        id: 'edge_1',
        source: 'trigger_1',
        target: 'ai_agent_1',
      },
      {
        id: 'edge_2',
        source: 'ai_agent_1',
        target: 'condition_1',
      },
      {
        id: 'edge_3',
        source: 'condition_1',
        target: 'action_1',
        sourceHandle: 'true',
      },
      {
        id: 'edge_4',
        source: 'condition_1',
        target: 'action_2',
        sourceHandle: 'false',
      },
      {
        id: 'edge_5',
        source: 'action_1',
        target: 'data_processor_1',
      },
      {
        id: 'edge_6',
        source: 'action_2',
        target: 'data_processor_1',
      },
    ],
    variables: {
      environment: 'demo',
      debug: true,
    },
  };

  // 执行工作流
  const executeWorkflow = useCallback(async () => {
    try {
      const execution = await engine.execute(sampleWorkflow, {
        variables: { ...sampleWorkflow.variables, timestamp: new Date().toISOString() },
      });

      setExecutions(prev => [...prev, execution.executionId]);
      setCurrentWorkflow(sampleWorkflow);

      console.log('工作流执行完成:', execution);
    } catch (error: any) {
      console.error('工作流执行失败:', error);
    }
  }, [engine, sampleWorkflow]);

  // 监听执行事件
  useEffect(() => {
    const handleExecutionStarted = ({ context }: any) => {
      console.log('执行开始:', context.executionId);
    };

    const handleNodeStarted = ({ result }: any) => {
      console.log('节点开始执行:', result.nodeId);
    };

    const handleNodeCompleted = ({ result }: any) => {
      console.log('节点执行完成:', result.nodeId, result.status);
    };

    const handleExecutionCompleted = ({ context }: any) => {
      console.log('执行完成:', context.executionId, context.status);
    };

    const handleExecutionFailed = ({ error }: any) => {
      console.error('执行失败:', error);
    };

    engine.on('execution-started', handleExecutionStarted);
    engine.on('node-started', handleNodeStarted);
    engine.on('node-completed', handleNodeCompleted);
    engine.on('execution-completed', handleExecutionCompleted);
    engine.on('execution-failed', handleExecutionFailed);

    return () => {
      engine.off('execution-started', handleExecutionStarted);
      engine.off('node-started', handleNodeStarted);
      engine.off('node-completed', handleNodeCompleted);
      engine.off('execution-completed', handleExecutionCompleted);
      engine.off('execution-failed', handleExecutionFailed);
    };
  }, [engine]);

  // 更新引擎配置
  const updateEngineConfig = useCallback((key: string, value: any) => {
    const newConfig = { ...engineConfig, [key]: value };
    setEngineConfig(newConfig);

    // 更新引擎配置（注意：这需要重新创建引擎实例）
    // 这里只是演示，实际使用中可能需要更复杂的配置管理
  }, [engineConfig]);

  // 渲染引擎配置
  const renderEngineConfig = () => (
    <Card title="引擎配置" size="small">
      <Row gutter={16}>
        <Col span={12}>
          <div style={{ marginBottom: 16 }}>
            <Text strong>最大并发节点数:</Text>
            <div style={{ marginTop: 8 }}>
              <InputNumber
                min={1}
                max={10}
                value={engineConfig.maxConcurrentNodes}
                onChange={(value) => updateEngineConfig('maxConcurrentNodes', value)}
              />
            </div>
          </div>
        </Col>
        <Col span={12}>
          <div style={{ marginBottom: 16 }}>
            <Text strong>超时时间 (秒):</Text>
            <div style={{ marginTop: 8 }}>
              <InputNumber
                min={5}
                max={300}
                value={engineConfig.timeout / 1000}
                onChange={(value) => updateEngineConfig('timeout', (value || 30) * 1000)}
              />
            </div>
          </div>
        </Col>
        <Col span={12}>
          <div style={{ marginBottom: 16 }}>
            <Text strong>重试次数:</Text>
            <div style={{ marginTop: 8 }}>
              <InputNumber
                min={0}
                max={5}
                value={engineConfig.retryAttempts}
                onChange={(value) => updateEngineConfig('retryAttempts', value)}
              />
            </div>
          </div>
        </Col>
        <Col span={12}>
          <div style={{ marginBottom: 16 }}>
            <Text strong>启用日志:</Text>
            <div style={{ marginTop: 8 }}>
              <Switch
                checked={engineConfig.enableLogging}
                onChange={(checked) => updateEngineConfig('enableLogging', checked)}
              />
            </div>
          </div>
        </Col>
      </Row>

      <div style={{ marginTop: 16 }}>
        <Button
          type="primary"
          icon={<PlayCircleOutlined />}
          onClick={executeWorkflow}
        >
          执行演示工作流
        </Button>
      </div>
    </Card>
  );

  // 渲染工作流信息
  const renderWorkflowInfo = () => {
    if (!currentWorkflow) {
      return (
        <Alert
          message="请先执行工作流"
          description="点击上方的执行按钮来运行演示工作流"
          type="info"
          showIcon
        />
      );
    }

    return (
      <Card title="工作流信息" size="small">
        <Row gutter={16}>
          <Col span={12}>
            <Text strong>工作流ID: </Text>
            <Text code>{currentWorkflow.id}</Text>
          </Col>
          <Col span={12}>
            <Text strong>名称: </Text>
            <Text>{currentWorkflow.name}</Text>
          </Col>
        </Row>
        <Row gutter={16} style={{ marginTop: 12 }}>
          <Col span={12}>
            <Text strong>节点数量: </Text>
            <Tag color="blue">{currentWorkflow.nodes.length}</Tag>
          </Col>
          <Col span={12}>
            <Text strong>连接数量: </Text>
            <Tag color="green">{currentWorkflow.edges.length}</Tag>
          </Col>
        </Row>
        <Row gutter={16} style={{ marginTop: 12 }}>
          <Col span={24}>
            <Text strong>描述: </Text>
            <Text>{currentWorkflow.description}</Text>
          </Col>
        </Row>

        <Divider />

        <Title level={5}>工作流结构</Title>
        <Timeline>
          {currentWorkflow.nodes.map((node, index) => (
            <Timeline.Item
              key={node.id}
              color="blue"
              dot={<ThunderboltOutlined />}
            >
              <div>
                <Text strong>{node.data.triggerType || node.data.agentType || node.data.actionType || node.data.processorType || node.type}</Text>
                <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>
                  节点ID: {node.id}
                </div>
                {node.data.input?.message && (
                  <div style={{ fontSize: 12, color: '#666' }}>
                    输入: {node.data.input.message}
                  </div>
                )}
              </div>
            </Timeline.Item>
          ))}
        </Timeline>
      </Card>
    );
  };

  // 渲染引擎统计
  const renderEngineStats = () => {
    const stats = engine.getStats();

    return (
      <Card title="引擎统计" size="small">
        <Row gutter={16}>
          <Col span={8}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: 24, fontWeight: 'bold', color: '#1890ff' }}>
                {stats.activeExecutions}
              </div>
              <div style={{ fontSize: 14, color: '#666' }}>活跃执行</div>
            </div>
          </Col>
          <Col span={8}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: 24, fontWeight: 'bold', color: '#52c41a' }}>
                {stats.registeredExecutors}
              </div>
              <div style={{ fontSize: 14, color: '#666' }}>注册执行器</div>
            </div>
          </Col>
          <Col span={8}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: 24, fontWeight: 'bold', color: '#fa8c16' }}>
                {executions.length}
              </div>
              <div style={{ fontSize: 14, color: '#666' }}>总执行次数</div>
            </div>
          </Col>
        </Row>
      </Card>
    );
  };

  // 渲染执行历史
  const renderExecutionHistory = () => {
    if (executions.length === 0) {
      return (
        <Alert
          message="暂无执行历史"
          description="执行工作流后，历史记录将显示在这里"
          type="info"
          showIcon
        />
      );
    }

    return (
      <Card title="执行历史" size="small">
        {executions.map((executionId, index) => (
          <div key={executionId} style={{ marginBottom: 12, padding: 12, border: '1px solid #f0f0f0', borderRadius: 4 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div>
                <Text strong>执行 #{index + 1}</Text>
                <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>
                  ID: {executionId}
                </div>
              </div>
              <Tag color="green">
                <CheckCircleOutlined /> 已完成
              </Tag>
            </div>
          </div>
        ))}
      </Card>
    );
  };

  return (
    <div style={{ padding: 24 }}>
      <Title level={2}>工作流执行引擎演示</Title>
      <Paragraph>
        这个演示展示了工作流执行引擎的核心功能，包括节点执行、并发控制、错误处理等。
      </Paragraph>

      <Tabs defaultActiveKey="config" size="small">
        <TabPane tab="引擎配置" key="config">
          {renderEngineConfig()}
        </TabPane>

        <TabPane tab="工作流信息" key="workflow">
          {renderWorkflowInfo()}
        </TabPane>

        <TabPane tab="执行监控" key="monitor">
          <ExecutionMonitor
            engine={engine}
            workflowId={currentWorkflow?.id}
            height={400}
            showDetails={true}
          />
        </TabPane>

        <TabPane tab="引擎统计" key="stats">
          {renderEngineStats()}
        </TabPane>

        <TabPane tab="执行历史" key="history">
          {renderExecutionHistory()}
        </TabPane>
      </Tabs>
    </div>
  );
};

export default ExecutionDemo;