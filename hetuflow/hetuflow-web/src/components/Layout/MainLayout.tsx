import React, { useState, useEffect } from 'react';
import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import { Avatar, Dropdown, Space } from 'antd';
import {
  DashboardOutlined,
  CloudServerOutlined,
  RobotOutlined,
  ScheduleOutlined,
  UnorderedListOutlined,
  UserOutlined,
  LogoutOutlined,
} from '@ant-design/icons';
import { ProLayout, MenuDataItem } from '@ant-design/pro-components';
import ThemeSwitcher from '../ThemeSwitcher';
import { useTheme } from '../../contexts/ThemeContext';

/**
 * 主布局组件
 * 使用 ProLayout 实现顶部导航栏、左侧可收起侧边栏和主内容区域
 */
const MainLayout: React.FC = () => {
  const [collapsed, setCollapsed] = useState(() => {
    const savedCollapsed = localStorage.getItem('mainLayoutCollapsed');
    return savedCollapsed !== null ? JSON.parse(savedCollapsed) : false;
  });
  // 监听 collapsed 状态变化并保存到 localStorage
  useEffect(() => {
    localStorage.setItem('mainLayoutCollapsed', JSON.stringify(collapsed));
  }, [collapsed]);

  const navigate = useNavigate();
  const location = useLocation();
  const { currentTheme } = useTheme();

  // 菜单项配置
  const menuItems: MenuDataItem[] = [
    {
      path: '/dashboard',
      name: '仪表板',
      icon: <DashboardOutlined />,
    },
    {
      path: '/servers',
      name: '服务器管理',
      icon: <CloudServerOutlined />,
    },
    {
      path: '/agents',
      name: '执行代理管理',
      icon: <RobotOutlined />,
    },
    {
      path: '/jobs',
      name: '作业管理',
      icon: <ScheduleOutlined />,
    },
    {
      path: '/tasks',
      name: '任务管理',
      icon: <UnorderedListOutlined />,
    },
  ];

  // 用户下拉菜单
  const userMenuItems = [
    {
      key: 'profile',
      icon: <UserOutlined />,
      label: '个人资料',
    },
    {
      key: 'logout',
      icon: <LogoutOutlined />,
      label: '退出登录',
    },
  ];

  // 处理用户菜单点击
  const handleUserMenuClick = ({ key }: { key: string }) => {
    if (key === 'logout') {
      // TODO: 实现退出登录逻辑
      console.log('退出登录');
    } else if (key === 'profile') {
      // TODO: 跳转到个人资料页面
      console.log('个人资料');
    }
  };

  return (
    <ProLayout
      title="Hetuflow"
      logo={false}
      breakpoint={false}
      collapsed={collapsed}
      onCollapse={setCollapsed}
      location={{
        pathname: location.pathname,
      }}
      route={{
        routes: menuItems,
      }}
      menuItemRender={(item, dom) => (
        <div
          onClick={() => {
            if (item.path) {
              navigate(item.path);
            }
          }}
        >
          {dom}
        </div>
      )}
      headerRender={() => (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            width: '100%',
            padding: '0 24px',
          }}
        >
          <div />
          <Space>
            <ThemeSwitcher />
            <Dropdown
              menu={{
                items: userMenuItems,
                onClick: handleUserMenuClick,
              }}
              placement="bottomRight"
            >
              <Space style={{ cursor: 'pointer' }}>
                <Avatar icon={<UserOutlined />} />
                <span>管理员</span>
              </Space>
            </Dropdown>
          </Space>
        </div>
      )}
      layout="mix"
      navTheme={currentTheme === 'dark' ? 'realDark' : 'light'}
      colorPrimary="#1890ff"
      siderWidth={240}
      style={{
        minHeight: '100vh',
        height: '100vh',
        width: '100vw',
        overflow: 'hidden',
      }}
      contentStyle={{
        padding: 24,
        overflow: 'auto',
      }}
    >
      <Outlet />
    </ProLayout>
  );
};

export default MainLayout;
