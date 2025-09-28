import React, { useState, useEffect } from 'react';
import { Card, Button, Table, Space, Spin, Tag } from 'antd';
import { HetuflowSDK, type SchedAgent, type HealthStatus } from '@fusion-data/hetuflow-sdk';
import { useMessage } from '../hooks/useMessage';

// 创建 SDK 实例
const sdk = new HetuflowSDK({
  baseURL: '', // 使用空字符串，这样会使用当前域名，通过 Vite 代理转发到后端
  timeout: 30000,
});

const HetuflowDemo: React.FC = () => {
  const message = useMessage();
  const [loading, setLoading] = useState(false);
  const [agents, setAgents] = useState<SchedAgent[]>([]);
  const [health, setHealth] = useState<HealthStatus | null>(null);

  // 获取系统健康状态
  const fetchHealth = async () => {
    try {
      const healthStatus = await sdk.system.getHealth();
      setHealth(healthStatus);
    } catch (error) {
      console.error('Failed to fetch health:', error);
      message.error('获取系统状态失败');
    }
  };

  // 获取 Agent 列表
  const fetchAgents = async () => {
    setLoading(true);
    try {
      const result = await sdk.agents.queryAgents({
        page: {
          page: 1,
          limit: 10,
        },
        filter: {}, // 空过滤器，查询所有 agents
      });
      setAgents(result.result);
      message.success(`成功获取 ${result.result.length} 个 Agent`);
    } catch (error) {
      console.error('Failed to fetch agents:', error);
      message.error('获取 Agent 列表失败');
    } finally {
      setLoading(false);
    }
  };

  // 创建示例 Agent
  const createSampleAgent = async () => {
    setLoading(true);
    try {
      const result = await sdk.agents.createAgent({
        name: `Demo Agent ${Date.now()}`,
        description: '这是一个通过 SDK 创建的示例 Agent',
        config: {
          type: 'demo',
          created_by: 'hetuflow-sdk-demo',
        },
      });
      message.success(`Agent 创建成功: ${result.id}`);
      // 重新获取 Agent 列表
      await fetchAgents();
    } catch (error) {
      console.error('Failed to create agent:', error);
      message.error('创建 Agent 失败');
    } finally {
      setLoading(false);
    }
  };

  // 删除 Agent
  const deleteAgent = async (id: string) => {
    try {
      await sdk.agents.deleteAgent(id);
      message.success('Agent 删除成功');
      // 重新获取 Agent 列表
      await fetchAgents();
    } catch (error) {
      console.error('Failed to delete agent:', error);
      message.error('删除 Agent 失败');
    }
  };

  useEffect(() => {
    fetchHealth();
    fetchAgents();
  }, []);

  const columns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
      width: 200,
      render: (text: string) => <code style={{ fontSize: '12px' }}>{text.slice(0, 8)}...</code>,
    },
    {
      title: '远程地址',
      dataIndex: 'address',
      key: 'address',
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
    },
    {
      title: '最后心跳时间',
      dataIndex: 'last_heartbeat_at',
      key: 'last_heartbeat_at',
      render: (text: string) => new Date(text).toLocaleString('zh-CN'),
    },
    {
      title: '描述',
      dataIndex: 'description',
      key: 'description',
    },
    {
      title: '操作',
      key: 'actions',
      render: (_: any, record: SchedAgent) => (
        <Space>
          <Button type="link" danger size="small" onClick={() => deleteAgent(record.id)}>
            删除
          </Button>
        </Space>
      ),
    },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Card
        title="Hetuflow SDK 演示"
        extra={
          <Space>
            {health && <Tag color={health.status === 'healthy' ? 'green' : 'red'}>系统状态: {health.status}</Tag>}
          </Space>
        }
      >
        <Space direction="vertical" style={{ width: '100%' }} size="large">
          <Card size="small" title="操作">
            <Space>
              <Button type="primary" onClick={fetchAgents} loading={loading}>
                刷新 Agent 列表
              </Button>
              <Button onClick={createSampleAgent} loading={loading}>
                创建示例 Agent
              </Button>
              <Button onClick={fetchHealth}>检查系统状态</Button>
            </Space>
          </Card>

          <Card size="small" title={`Agent 列表 (${agents.length})`}>
            <Spin spinning={loading}>
              <Table
                columns={columns}
                dataSource={agents}
                rowKey="id"
                size="small"
                pagination={false}
                locale={{
                  emptyText: '暂无数据，请先创建 Agent 或检查 Hetuflow 服务是否运行',
                }}
              />
            </Spin>
          </Card>

          <Card size="small" title="SDK 信息">
            <div>
              <p>
                <strong>基础URL:</strong> 通过 Vite 代理转发到 http://localhost:9500
              </p>
              <p>
                <strong>SDK版本:</strong> @fusion-data/hetuflow-sdk v1.0.0
              </p>
              <p>
                <strong>功能模块:</strong>
              </p>
              <ul>
                <li>✅ Agent 管理 (agents)</li>
                <li>✅ Job 管理 (jobs)</li>
                <li>✅ Task 管理 (tasks)</li>
                <li>✅ TaskInstance 管理 (taskInstances)</li>
                <li>✅ 认证管理 (auth)</li>
                <li>✅ 网关操作 (gateway)</li>
                <li>✅ 系统监控 (system)</li>
              </ul>
            </div>
          </Card>
        </Space>
      </Card>
    </div>
  );
};

export default HetuflowDemo;
