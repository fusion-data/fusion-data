import React from 'react';
import { Form, Input, Button, Card, Typography } from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';

const { Title } = Typography;

const LoginPage: React.FC = () => {
  const navigate = useNavigate();
  const [form] = Form.useForm();

  const handleLogin = async (values: any) => {
    console.log('Login values:', values);
    // TODO: 实现登录逻辑
    // 暂时直接跳转到仪表板
    navigate('/dashboard');
  };

  return (
    <div className="flex-center" style={{ minHeight: '100vh', backgroundColor: 'var(--bg-secondary)' }}>
      <Card
        style={{ width: 400, boxShadow: 'var(--shadow-2)' }}
        bodyStyle={{ padding: '40px' }}
      >
        <div style={{ textAlign: 'center', marginBottom: '32px' }}>
          <Title level={2} style={{ margin: 0, color: 'var(--text-primary)' }}>
            Hetumind Web
          </Title>
          <p style={{ color: 'var(--text-secondary)', marginTop: '8px' }}>
            AI Agent 开发和工作流编排平台
          </p>
        </div>

        <Form
          form={form}
          name="login"
          onFinish={handleLogin}
          autoComplete="off"
          size="large"
        >
          <Form.Item
            name="username"
            rules={[{ required: true, message: '请输入用户名' }]}
          >
            <Input
              prefix={<UserOutlined />}
              placeholder="用户名"
            />
          </Form.Item>

          <Form.Item
            name="password"
            rules={[{ required: true, message: '请输入密码' }]}
          >
            <Input.Password
              prefix={<LockOutlined />}
              placeholder="密码"
            />
          </Form.Item>

          <Form.Item>
            <Button type="primary" htmlType="submit" block style={{ height: '40px' }}>
              登录
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
};

export default LoginPage;