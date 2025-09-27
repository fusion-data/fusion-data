import { Button, Card, Space, Typography, Row, Col, Divider, message, Tabs } from "antd";
import {
  SmileOutlined,
  HeartOutlined,
  StarOutlined,
  GithubOutlined,
  RocketOutlined,
  ApiOutlined,
  HolderOutlined,
} from "@ant-design/icons";
import { generateId } from "@fusion-data/fusion-core";
import { formatDate } from "@fusion-data/fusion-core/time";
import HetuflowDemo from "./components/HetuflowDemo";
import DragDropDemo from "./components/DragDropDemo";

const { Title, Paragraph, Text } = Typography;

function App() {
  const handleUtilsDemo = () => {
    const today = formatDate(new Date());
    const id = generateId();
    message.success(`今天是 ${today}，生成的ID: ${id}`);
  };

  const tabItems = [
    {
      key: "overview",
      label: (
        <span>
          <SmileOutlined />
          项目概览
        </span>
      ),
      children: (
        <>
          <Row gutter={[16, 16]}>
            <Col xs={24} sm={12} md={8}>
              <Card
                title="技术栈"
                hoverable
                actions={[<StarOutlined key="star" />, <HeartOutlined key="heart" />, <SmileOutlined key="smile" />]}
              >
                <ul>
                  <li>React 19</li>
                  <li>TypeScript</li>
                  <li>Ant Design v5</li>
                  <li>Vite</li>
                  <li>pnpm workspaces</li>
                </ul>
              </Card>
            </Col>

            <Col xs={24} sm={12} md={8}>
              <Card
                title="Fusion Data 特性"
                hoverable
                actions={[
                  <GithubOutlined key="github" />,
                  <RocketOutlined key="rocket" />,
                  <StarOutlined key="star" />,
                ]}
              >
                <ul>
                  <li>模块化 Rust 架构</li>
                  <li>AI Agent 编排</li>
                  <li>分布式任务调度</li>
                  <li>类型安全的数据库访问</li>
                  <li>高性能 Web 服务</li>
                </ul>
              </Card>
            </Col>

            <Col xs={24} sm={12} md={8}>
              <Card title="共享工具演示" hoverable>
                <Paragraph>
                  点击下面的按钮演示来自 <Text code>@fusion-data/fusion-core</Text> 包的工具函数：
                </Paragraph>
                <Button type="primary" onClick={handleUtilsDemo} icon={<SmileOutlined />}>
                  测试工具函数
                </Button>
              </Card>
            </Col>
          </Row>

          <Divider />

          <Space size="middle">
            <Button type="primary" size="large">
              主要操作
            </Button>
            <Button size="large">次要操作</Button>
            <Button type="dashed" size="large">
              虚线按钮
            </Button>
            <Button type="link" size="large">
              链接按钮
            </Button>
          </Space>
        </>
      ),
    },
    {
      key: "hetuflow",
      label: (
        <span>
          <ApiOutlined />
          Hetuflow SDK
        </span>
      ),
      children: <HetuflowDemo />,
    },
    {
      key: "dragdrop",
      label: (
        <span>
          <HolderOutlined />
          拖拽演示
        </span>
      ),
      children: <DragDropDemo />,
    },
  ];

  return (
    <div style={{ padding: "24px" }}>
      <Title level={1}>
        <RocketOutlined /> Fusion Data Demo App
      </Title>

      <Paragraph>
        这是一个基于 <Text strong>React 19</Text> + <Text strong>Ant Design v5</Text> + <Text strong>TypeScript</Text>{" "}
        的示例应用程序。
      </Paragraph>

      <Divider />

      <Tabs defaultActiveKey="overview" size="large" items={tabItems} />
    </div>
  );
}

export default App;
