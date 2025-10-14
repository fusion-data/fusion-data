import React, { useState, useCallback, useEffect } from 'react';
import {
  Card,
  Row,
  Col,
  Button,
  Space,
  Typography,
  Tag,
  Progress,
  Table,
  Timeline,
  Alert,
  Tabs,
  Badge,
  Tooltip,
  Modal,
  Empty,
  Statistic,
  List,
  Avatar,
} from 'antd';
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  StopOutlined,
  ReloadOutlined,
  ClockCircleOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  CloseCircleOutlined,
  InfoCircleOutlined,
  ThunderboltOutlined,
  EyeOutlined,
} from '@ant-design/icons';

import { WorkflowEngine, ExecutionContext, NodeExecutionResult, ExecutionStatus } from './WorkflowEngine';

const { Text, Title, Paragraph } = Typography;
const { TabPane } = Tabs;

interface ExecutionMonitorProps {
  engine: WorkflowEngine;
  workflowId?: string;
  height?: number;
  showDetails?: boolean;
}

export const ExecutionMonitor: React.FC<ExecutionMonitorProps> = ({
  engine,
  workflowId,
  height = 600,
  showDetails = true,
}) => {
  const [executions, setExecutions] = useState<ExecutionContext[]>([]);
  const [selectedExecution, setSelectedExecution] = useState<ExecutionContext | null>(null);
  const [detailsModalVisible, setDetailsModalVisible] = useState(false);
  const [activeTab, setActiveTab] = useState('overview');

  // 监听执行事件
  useEffect(() => {
    const handleExecutionStarted = ({ context }: any) => {
      if (!workflowId || context.workflowId === workflowId) {
        setExecutions(prev => [context, ...prev]);
      }
    };

    const handleExecutionCompleted = ({ context }: any) => {
      if (!workflowId || context.workflowId === workflowId) {
        setExecutions(prev =>
          prev.map(exec => exec.executionId === context.executionId ? context : exec)
        );
      }
    };

    const handleNodeStarted = ({ context, result }: any) => {
      if (!workflowId || context.workflowId === workflowId) {
        setExecutions(prev =>
          prev.map(exec => {
            if (exec.executionId === context.executionId) {
              return {
                ...exec,
                nodeResults: {
                  ...exec.nodeResults,
                  [result.nodeId]: result,
                },
              };
            }
            return exec;
          })
        );
      }
    };

    const handleNodeCompleted = ({ context, result }: any) => {
      if (!workflowId || context.workflowId === workflowId) {
        setExecutions(prev =>
          prev.map(exec => {
            if (exec.executionId === context.executionId) {
              return {
                ...exec,
                nodeResults: {
                  ...exec.nodeResults,
                  [result.nodeId]: result,
                },
              };
            }
            return exec;
          })
        );
      }
    };

    // 注册事件监听器
    engine.on('execution-started', handleExecutionStarted);
    engine.on('execution-completed', handleExecutionCompleted);
    engine.on('node-started', handleNodeStarted);
    engine.on('node-completed', handleNodeCompleted);

    // 清理函数
    return () => {
      engine.off('execution-started', handleExecutionStarted);
      engine.off('execution-completed', handleExecutionCompleted);
      engine.off('node-started', handleNodeStarted);
      engine.off('node-completed', handleNodeCompleted);
    };
  }, [engine, workflowId]);

  // 刷新执行列表
  const refreshExecutions = useCallback(() => {
    const activeExecutions = engine.getActiveExecutions();
    if (workflowId) {
      setExecutions(activeExecutions.filter(exec => exec.workflowId === workflowId));
    } else {
      setExecutions(activeExecutions);
    }
  }, [engine, workflowId]);

  // 获取执行状态颜色
  const getStatusColor = (status: ExecutionStatus) => {
    switch (status) {
      case 'running': return 'blue';
      case 'completed': return 'green';
      case 'failed': return 'red';
      case 'paused': return 'orange';
      case 'cancelled': return 'default';
      default: return 'default';
    }
  };

  // 获取执行状态图标
  const getStatusIcon = (status: ExecutionStatus) => {
    switch (status) {
      case 'running': return <ThunderboltOutlined spin />;
      case 'completed': return <CheckCircleOutlined />;
      case 'failed': return <ExclamationCircleOutlined />;
      case 'paused': return <PauseCircleOutlined />;
      case 'cancelled': return <CloseCircleOutlined />;
      default: return <ClockCircleOutlined />;
    }
  };

  // 计算执行进度
  const calculateProgress = (execution: ExecutionContext) => {
    const nodeResults = Object.values(execution.nodeResults);
    if (nodeResults.length === 0) return 0;

    const completedNodes = nodeResults.filter(result =>
      result.status === 'completed' || result.status === 'failed'
    ).length;

    return Math.round((completedNodes / nodeResults.length) * 100);
  };

  // 计算执行持续时间
  const getDuration = (execution: ExecutionContext) => {
    const endTime = execution.endTime || new Date();
    const duration = endTime.getTime() - execution.startTime.getTime();
    return Math.round(duration / 1000); // 返回秒数
  };

  // 格式化持续时间
  const formatDuration = (seconds: number) => {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
  };

  // 执行控制操作
  const handlePause = useCallback((executionId: string) => {
    engine.pause(executionId);
    refreshExecutions();
  }, [engine, refreshExecutions]);

  const handleResume = useCallback((executionId: string) => {
    engine.resume(executionId);
    refreshExecutions();
  }, [engine, refreshExecutions]);

  const handleCancel = useCallback((executionId: string) => {
    engine.cancel(executionId);
    refreshExecutions();
  }, [engine, refreshExecutions]);

  // 查看执行详情
  const handleViewDetails = useCallback((execution: ExecutionContext) => {
    setSelectedExecution(execution);
    setDetailsModalVisible(true);
  }, []);

  // 渲染执行统计
  const renderStatistics = () => {
    const stats = {
      total: executions.length,
      running: executions.filter(e => e.status === 'running').length,
      completed: executions.filter(e => e.status === 'completed').length,
      failed: executions.filter(e => e.status === 'failed').length,
    };

    return (
      <Row gutter={16} style={{ marginBottom: 24 }}>
        <Col span={6}>
          <Card>
            <Statistic
              title="总执行数"
              value={stats.total}
              prefix={<ClockCircleOutlined />}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="运行中"
              value={stats.running}
              prefix={<ThunderboltOutlined spin />}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="已完成"
              value={stats.completed}
              prefix={<CheckCircleOutlined />}
              valueStyle={{ color: '#52c41a' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="失败"
              value={stats.failed}
              prefix={<ExclamationCircleOutlined />}
              valueStyle={{ color: '#ff4d4f' }}
            />
          </Card>
        </Col>
      </Row>
    );
  };

  // 渲染执行列表
  const renderExecutionList = () => {
    const columns = [
      {
        title: '执行ID',
        dataIndex: 'executionId',
        key: 'executionId',
        render: (id: string) => (
          <Text code style={{ fontSize: 12 }}>{id}</Text>
        ),
      },
      {
        title: '状态',
        dataIndex: 'status',
        key: 'status',
        render: (status: ExecutionStatus) => (
          <Tag color={getStatusColor(status)} icon={getStatusIcon(status)}>
            {status}
          </Tag>
        ),
      },
      {
        title: '进度',
        key: 'progress',
        render: (_, record: ExecutionContext) => (
          <Progress
            percent={calculateProgress(record)}
            size="small"
            status={record.status === 'failed' ? 'exception' : 'active'}
          />
        ),
      },
      {
        title: '持续时间',
        key: 'duration',
        render: (_, record: ExecutionContext) => (
          <Text>{formatDuration(getDuration(record))}</Text>
        ),
      },
      {
        title: '开始时间',
        dataIndex: 'startTime',
        key: 'startTime',
        render: (time: Date) => (
          <Text>{new Date(time).toLocaleString()}</Text>
        ),
      },
      {
        title: '操作',
        key: 'actions',
        render: (_, record: ExecutionContext) => (
          <Space size="small">
            {record.status === 'running' && (
              <Tooltip title="暂停">
                <Button
                  type="text"
                  size="small"
                  icon={<PauseCircleOutlined />}
                  onClick={() => handlePause(record.executionId)}
                />
              </Tooltip>
            )}
            {record.status === 'paused' && (
              <Tooltip title="恢复">
                <Button
                  type="text"
                  size="small"
                  icon={<PlayCircleOutlined />}
                  onClick={() => handleResume(record.executionId)}
                />
              </Tooltip>
            )}
            {(record.status === 'running' || record.status === 'paused') && (
              <Tooltip title="取消">
                <Button
                  type="text"
                  size="small"
                  danger
                  icon={<StopOutlined />}
                  onClick={() => handleCancel(record.executionId)}
                />
              </Tooltip>
            )}
            {showDetails && (
              <Tooltip title="查看详情">
                <Button
                  type="text"
                  size="small"
                  icon={<EyeOutlined />}
                  onClick={() => handleViewDetails(record)}
                />
              </Tooltip>
            )}
          </Space>
        ),
      },
    ];

    return (
      <Table
        dataSource={executions}
        columns={columns}
        rowKey="executionId"
        pagination={{
          pageSize: 10,
          showSizeChanger: true,
          showQuickJumper: true,
        }}
        size="small"
      />
    );
  };

  // 渲染节点执行时间线
  const renderNodeTimeline = (execution: ExecutionContext) => {
    const nodeResults = Object.values(execution.nodeResults);

    if (nodeResults.length === 0) {
      return <Empty description="暂无节点执行记录" />;
    }

    return (
      <Timeline mode="left">
        {nodeResults.map((result, index) => (
          <Timeline.Item
            key={result.nodeId}
            color={result.status === 'completed' ? 'green' : result.status === 'failed' ? 'red' : 'blue'}
            dot={getStatusIcon(result.status as any)}
          >
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <Text strong>{result.nodeId}</Text>
                <Tag color={result.status === 'completed' ? 'green' : result.status === 'failed' ? 'red' : 'blue'}>
                  {result.status}
                </Tag>
              </div>
              <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>
                开始: {new Date(result.startTime).toLocaleString()}
              </div>
              {result.endTime && (
                <div style={{ fontSize: 12, color: '#666' }}>
                  结束: {new Date(result.endTime).toLocaleString()}
                </div>
              )}
              {result.duration && (
                <div style={{ fontSize: 12, color: '#666' }}>
                  耗时: {formatDuration(Math.round(result.duration / 1000))}
                </div>
              )}
              {result.error && (
                <Alert
                  message={result.error}
                  type="error"
                  size="small"
                  style={{ marginTop: 8 }}
                />
              )}
            </div>
          </Timeline.Item>
        ))}
      </Timeline>
    );
  };

  return (
    <div style={{ height }} className="execution-monitor">
      <div style={{ marginBottom: 16 }}>
        <Row justify="space-between" align="middle">
          <Col>
            <Title level={4} style={{ margin: 0 }}>
              执行监控
            </Title>
          </Col>
          <Col>
            <Space>
              <Button
                icon={<ReloadOutlined />}
                onClick={refreshExecutions}
              >
                刷新
              </Button>
            </Space>
          </Col>
        </Row>
      </div>

      <Tabs activeKey={activeTab} onChange={setActiveTab}>
        <TabPane tab="概览" key="overview">
          {renderStatistics()}
          {renderExecutionList()}
        </TabPane>

        {showDetails && (
          <TabPane tab="详情" key="details">
            {selectedExecution ? (
              <div>
                <Card title="执行信息" size="small" style={{ marginBottom: 16 }}>
                  <Row gutter={16}>
                    <Col span={8}>
                      <Text strong>执行ID: </Text>
                      <Text code>{selectedExecution.executionId}</Text>
                    </Col>
                    <Col span={8}>
                      <Text strong>状态: </Text>
                      <Tag color={getStatusColor(selectedExecution.status)}>
                        {selectedExecution.status}
                      </Tag>
                    </Col>
                    <Col span={8}>
                      <Text strong>进度: </Text>
                      <Progress
                        percent={calculateProgress(selectedExecution)}
                        size="small"
                        style={{ width: 100, display: 'inline-block' }}
                      />
                    </Col>
                  </Row>
                  <Row gutter={16} style={{ marginTop: 12 }}>
                    <Col span={8}>
                      <Text strong>开始时间: </Text>
                      <Text>{new Date(selectedExecution.startTime).toLocaleString()}</Text>
                    </Col>
                    <Col span={8}>
                      <Text strong>持续时间: </Text>
                      <Text>{formatDuration(getDuration(selectedExecution))}</Text>
                    </Col>
                    <Col span={8}>
                      <Text strong>节点数量: </Text>
                      <Text>{Object.keys(selectedExecution.nodeResults).length}</Text>
                    </Col>
                  </Row>
                  {selectedExecution.error && (
                    <Alert
                      message="执行错误"
                      description={selectedExecution.error}
                      type="error"
                      style={{ marginTop: 12 }}
                    />
                  )}
                </Card>

                <Card title="节点执行时间线" size="small">
                  {renderNodeTimeline(selectedExecution)}
                </Card>
              </div>
            ) : (
              <Empty description="选择一个执行查看详情" />
            )}
          </TabPane>
        )}
      </Tabs>

      {/* 详情模态框 */}
      <Modal
        title="执行详情"
        open={detailsModalVisible}
        onCancel={() => setDetailsModalVisible(false)}
        width={800}
        footer={null}
      >
        {selectedExecution && (
          <div>
            <Card title="基本信息" size="small" style={{ marginBottom: 16 }}>
              <Row gutter={16}>
                <Col span={12}>
                  <Text strong>执行ID: </Text>
                  <Text code>{selectedExecution.executionId}</Text>
                </Col>
                <Col span={12}>
                  <Text strong>工作流ID: </Text>
                  <Text code>{selectedExecution.workflowId}</Text>
                </Col>
              </Row>
              <Row gutter={16} style={{ marginTop: 12 }}>
                <Col span={12}>
                  <Text strong>状态: </Text>
                  <Tag color={getStatusColor(selectedExecution.status)}>
                    {selectedExecution.status}
                  </Tag>
                </Col>
                <Col span={12}>
                  <Text strong>进度: </Text>
                  <Progress
                    percent={calculateProgress(selectedExecution)}
                    size="small"
                    style={{ width: 100, display: 'inline-block' }}
                  />
                </Col>
              </Row>
            </Card>

            <Card title="节点执行详情" size="small">
              {renderNodeTimeline(selectedExecution)}
            </Card>
          </div>
        )}
      </Modal>
    </div>
  );
};

export default ExecutionMonitor;