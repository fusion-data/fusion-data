import React from 'react';
import { Row, Col, Card, Statistic } from 'antd';
import { RobotOutlined, PlayCircleOutlined, CheckCircleOutlined, ClockCircleOutlined } from '@ant-design/icons';

const DashboardPage: React.FC = () => {
  return (
    <div style={{ padding: '24px' }}>
      <h2 style={{ marginBottom: '24px', color: 'var(--text-primary)' }}>
        仪表板
      </h2>

      <Row gutter={[16, 16]}>
        {/* 统计卡片 */}
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="AI 智能体"
              value={0}
              prefix={<RobotOutlined />}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="活跃智能体"
              value={0}
              prefix={<CheckCircleOutlined />}
              valueStyle={{ color: '#52c41a' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="工作流"
              value={0}
              prefix={<PlayCircleOutlined />}
              valueStyle={{ color: '#fa8c16' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="24小时执行"
              value={0}
              prefix={<ClockCircleOutlined />}
              valueStyle={{ color: '#722ed1' }}
            />
          </Card>
        </Col>
      </Row>

      <Row gutter={[16, 16]} style={{ marginTop: '24px' }}>
        <Col xs={24} lg={12}>
          <Card title="最近活动" extra={<a>查看全部</a>}>
            <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
              暂无活动记录
            </div>
          </Card>
        </Col>

        <Col xs={24} lg={12}>
          <Card title="热门工作流" extra={<a>查看全部</a>}>
            <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
              暂无工作流
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  );
};

export default DashboardPage;