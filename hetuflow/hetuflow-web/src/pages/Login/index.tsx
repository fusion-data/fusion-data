import React from 'react';
import { Form, Input, Button, Card, Typography, Space } from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';

const { Title, Text } = Typography;

interface LoginForm {
  username: string;
  password: string;
}

/**
 * 登录页面组件
 * 提供用户身份验证功能
 */
const Login: React.FC = () => {
  const navigate = useNavigate();
  const [form] = Form.useForm();

  // 处理登录提交
  const handleLogin = async (values: LoginForm) => {
    try {
      // TODO: 集成实际的登录 API
      console.log('登录信息:', values);

      // 模拟登录成功，跳转到仪表板
      navigate('/dashboard');
    } catch (error) {
      console.error('登录失败:', error);
    }
  };

  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
      }}
    >
      <Card
        style={{
          width: 400,
          boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
        }}
      >
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          <div style={{ textAlign: 'center' }}>
            <Title level={2} style={{ color: '#1890ff', marginBottom: 8 }}>
              HetuFlow
            </Title>
            <Text type="secondary">分布式作业调度系统</Text>
          </div>

          <Form form={form} name="login" onFinish={handleLogin} autoComplete="off" size="large">
            <Form.Item
              name="username"
              rules={[
                {
                  required: true,
                  message: '请输入用户名！',
                },
              ]}
            >
              <Input prefix={<UserOutlined />} placeholder="用户名" />
            </Form.Item>

            <Form.Item
              name="password"
              rules={[
                {
                  required: true,
                  message: '请输入密码！',
                },
              ]}
            >
              <Input.Password prefix={<LockOutlined />} placeholder="密码" />
            </Form.Item>

            <Form.Item>
              <Button type="primary" htmlType="submit" style={{ width: '100%' }}>
                登录
              </Button>
            </Form.Item>
          </Form>
        </Space>
      </Card>
    </div>
  );
};

export default Login;
