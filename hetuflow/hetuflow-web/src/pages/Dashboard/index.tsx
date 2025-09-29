import React, { useEffect, useState } from 'react';
import { Row, Col, Card, Statistic, Typography, Space } from 'antd';
import {
  CloudServerOutlined,
  RobotOutlined,
  ProjectOutlined,
  PlayCircleOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
} from '@ant-design/icons';
import { apiService, TaskInstanceStatus } from '@/services/api';
import { useMessage } from '@/hooks/useMessage';

const { Title } = Typography;

/**
 * 仪表板页面组件
 * 显示系统概览和关键指标
 */
interface DashboardStats {
  servers: number;
  agents: number;
  totalJobs: number;
  runningTasks: number;
  completedTasks: number;
  failedTasks: number;
}

const Dashboard: React.FC = () => {
  const message = useMessage();
  const [stats, setStats] = useState<DashboardStats>({
    servers: 0,
    agents: 0,
    totalJobs: 0,
    runningTasks: 0,
    completedTasks: 0,
    failedTasks: 0,
  });
  const [loading, setLoading] = useState(true);

  /**
   * 获取仪表板统计数据
   */
  const fetchDashboardStats = async () => {
    try {
      setLoading(true);

      // 并行获取各种统计数据
      const [agentsResult, jobsResult, _tasksResult, _taskInstancesResult] = await Promise.all([
        // 获取代理列表
        apiService.agents.queryAgents({
          page: { page: 1, limit: 1 },
          filter: {},
        }),
        // 获取作业列表
        apiService.jobs.queryJobs({
          page: { page: 1, limit: 1 },
          filter: {},
        }),
        // 获取任务列表
        apiService.tasks.queryTasks({
          page: { page: 1, limit: 1 },
          filter: {},
        }),
        // 获取任务实例列表
        apiService.taskInstances.queryTaskInstances({
          page: { page: 1, limit: 1 },
          filter: {},
        }),
      ]);

      // 获取运行中的任务实例数量
      const runningTaskInstancesResult = await apiService.taskInstances.queryTaskInstances({
        page: { page: 1, limit: 1 },
        filter: { status: { $eq: TaskInstanceStatus.Running } },
      });

      // 获取已完成的任务实例数量
      const completedTaskInstancesResult = await apiService.taskInstances.queryTaskInstances({
        page: { page: 1, limit: 1 },
        filter: { status: { $eq: TaskInstanceStatus.Succeeded } },
      });

      // 获取失败的任务实例数量
      const failedTaskInstancesResult = await apiService.taskInstances.queryTaskInstances({
        page: { page: 1, limit: 1 },
        filter: { status: { $eq: TaskInstanceStatus.Failed } },
      });

      setStats({
        servers: 0, // 暂时没有服务器 API，使用默认值
        agents: agentsResult.page.total || 0,
        totalJobs: jobsResult.page.total || 0,
        runningTasks: runningTaskInstancesResult.page.total || 0,
        completedTasks: completedTaskInstancesResult.page.total || 0,
        failedTasks: failedTaskInstancesResult.page.total || 0,
      });
    } catch (error) {
      console.error('获取仪表板数据失败:', error);
      message.error('获取仪表板数据失败');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchDashboardStats();
  }, []);

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={2}>系统概览</Title>

      {/* 统计卡片 */}
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} md={8} lg={6}>
          <Card>
            <Statistic
              title="服务器数量"
              value={stats.servers}
              prefix={<CloudServerOutlined style={{ color: '#1890ff' }} />}
              loading={loading}
            />
          </Card>
        </Col>

        <Col xs={24} sm={12} md={8} lg={6}>
          <Card>
            <Statistic
              title="执行代理管理"
              value={stats.agents}
              prefix={<RobotOutlined style={{ color: '#52c41a' }} />}
              loading={loading}
            />
          </Card>
        </Col>

        <Col xs={24} sm={12} md={8} lg={6}>
          <Card>
            <Statistic
              title="作业总数"
              value={stats.totalJobs}
              prefix={<ProjectOutlined style={{ color: '#722ed1' }} />}
              loading={loading}
            />
          </Card>
        </Col>

        <Col xs={24} sm={12} md={8} lg={6}>
          <Card>
            <Statistic
              title="运行中任务"
              value={stats.runningTasks}
              prefix={<PlayCircleOutlined style={{ color: '#fa8c16' }} />}
              loading={loading}
            />
          </Card>
        </Col>
      </Row>

      {/* 任务执行统计 */}
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={8}>
          <Card>
            <Statistic
              title="已完成任务"
              value={stats.completedTasks}
              prefix={<CheckCircleOutlined style={{ color: '#52c41a' }} />}
              loading={loading}
            />
          </Card>
        </Col>

        <Col xs={24} sm={8}>
          <Card>
            <Statistic
              title="失败任务"
              value={stats.failedTasks}
              prefix={<CloseCircleOutlined style={{ color: '#f5222d' }} />}
              loading={loading}
            />
          </Card>
        </Col>

        <Col xs={24} sm={8}>
          <Card>
            <Statistic
              title="成功率"
              value={((stats.completedTasks / (stats.completedTasks + stats.failedTasks)) * 100).toFixed(1)}
              suffix="%"
              valueStyle={{ color: '#52c41a' }}
            />
          </Card>
        </Col>
      </Row>

      {/* TODO: 添加图表组件显示趋势数据 */}
      <Row gutter={[16, 16]}>
        <Col span={24}>
          <Card title="任务执行趋势" style={{ minHeight: 300 }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                height: 200,
                color: '#999',
              }}
            >
              图表组件待实现
            </div>
          </Card>
        </Col>
      </Row>
    </Space>
  );
};

export default Dashboard;
