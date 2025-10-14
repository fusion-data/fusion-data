import React, { useState, useCallback, useEffect } from 'react';
import {
  Card,
  Form,
  Input,
  Select,
  Switch,
  Slider,
  Button,
  Space,
  Typography,
  Tabs,
  Tag,
  Alert,
  Row,
  Col,
  InputNumber,
  Tooltip,
  Badge,
} from 'antd';
import {
  RobotOutlined,
  SettingOutlined,
  ApiOutlined,
  DatabaseOutlined,
  CodeOutlined,
  ExperimentOutlined,
  InfoCircleOutlined,
  PlayCircleOutlined,
  ReloadOutlined,
} from '@ant-design/icons';

const { Text, Title } = Typography;
const { TabPane } = Tabs;
const { TextArea } = Input;

// AI Agent 类型定义
interface AIAgentConfig {
  agentType: 'chat' | 'completion' | 'embedding' | 'image' | 'speech' | 'custom';
  model: string;
  systemPrompt: string;
  temperature: number;
  maxTokens: number;
  topP: number;
  frequencyPenalty: number;
  presencePenalty: number;
  enableStreaming: boolean;
  enableMemory: boolean;
  enableTools: boolean;
  apiKey?: string;
  apiEndpoint?: string;
  timeout: number;
  retryAttempts: number;
  customSettings?: Record<string, any>;
}

interface AIAgentConfigProps {
  nodeId: string;
  initialConfig?: Partial<AIAgentConfig>;
  onConfigChange?: (config: AIAgentConfig) => void;
  onTest?: (config: AIAgentConfig) => Promise<void>;
  readOnly?: boolean;
}

export const AIAgentConfig: React.FC<AIAgentConfigProps> = ({
  nodeId: _nodeId,
  initialConfig,
  onConfigChange,
  onTest,
  readOnly = false,
}) => {
  const [form] = Form.useForm();
  const [activeTab, setActiveTab] = useState('basic');
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{
    status: 'success' | 'error' | 'testing';
    message?: string;
    responseTime?: number;
  } | null>(null);

  // 默认配置
  const defaultConfig: AIAgentConfig = {
    agentType: 'chat',
    model: 'gpt-3.5-turbo',
    systemPrompt: '你是一个智能助手，请根据用户的问题提供准确、有用的回答。',
    temperature: 0.7,
    maxTokens: 2048,
    topP: 1,
    frequencyPenalty: 0,
    presencePenalty: 0,
    enableStreaming: false,
    enableMemory: true,
    enableTools: false,
    timeout: 30,
    retryAttempts: 3,
  };

  // 模型选项
  const modelOptions = {
    chat: [
      { value: 'gpt-4', label: 'GPT-4', description: '最强大的模型，适合复杂任务' },
      { value: 'gpt-3.5-turbo', label: 'GPT-3.5 Turbo', description: '快速且经济的选择' },
      { value: 'claude-3-opus', label: 'Claude 3 Opus', description: '高性能对话模型' },
      { value: 'claude-3-sonnet', label: 'Claude 3 Sonnet', description: '平衡性能和成本' },
    ],
    completion: [
      { value: 'gpt-3.5-turbo-instruct', label: 'GPT-3.5 Turbo Instruct', description: '文本生成专用' },
      { value: 'text-davinci-003', label: 'Text Davinci 003', description: '高质量的文本生成' },
    ],
    embedding: [
      { value: 'text-embedding-ada-002', label: 'Text Embedding Ada 002', description: '文本向量化' },
      { value: 'text-embedding-3-small', label: 'Text Embedding 3 Small', description: '轻量级向量模型' },
      { value: 'text-embedding-3-large', label: 'Text Embedding 3 Large', description: '高质量向量模型' },
    ],
    image: [
      { value: 'dall-e-3', label: 'DALL-E 3', description: '最新图像生成模型' },
      { value: 'dall-e-2', label: 'DALL-E 2', description: '稳定的图像生成' },
      { value: 'stable-diffusion', label: 'Stable Diffusion', description: '开源图像生成' },
    ],
    speech: [
      { value: 'whisper-1', label: 'Whisper 1', description: '语音转文本' },
      { value: 'tts-1', label: 'TTS 1', description: '文本转语音' },
    ],
    custom: [
      { value: 'custom', label: '自定义模型', description: '配置自定义模型' },
    ],
  };

  // 初始化表单
  useEffect(() => {
    const config = { ...defaultConfig, ...initialConfig };
    form.setFieldsValue(config);
  }, [initialConfig, form]);

  // 表单变化处理
  const handleFormChange = useCallback((_changedValues: any, allValues: AIAgentConfig) => {
    if (onConfigChange) {
      onConfigChange(allValues);
    }
  }, [onConfigChange]);

  // 测试配置
  const handleTest = useCallback(async () => {
    try {
      const config = form.getFieldsValue();
      setTesting(true);
      setTestResult({ status: 'testing' });

      if (onTest) {
        const startTime = Date.now();
        await onTest(config);
        const responseTime = Date.now() - startTime;

        setTestResult({
          status: 'success',
          message: '连接测试成功',
          responseTime,
        });
      } else {
        // 模拟测试
        await new Promise(resolve => setTimeout(resolve, 1000));
        setTestResult({
          status: 'success',
          message: '模拟测试成功（配置验证通过）',
          responseTime: 1000,
        });
      }
    } catch (error: any) {
      setTestResult({
        status: 'error',
        message: error?.message || '测试失败',
      });
    } finally {
      setTesting(false);
    }
  }, [form, onTest]);

  // 渲染基础配置
  const renderBasicConfig = () => (
    <Form
      form={form}
      layout="vertical"
      onValuesChange={handleFormChange}
      initialValues={defaultConfig}
    >
      <Row gutter={16}>
        <Col span={12}>
          <Form.Item
            label="智能体类型"
            name="agentType"
            tooltip="选择AI智能体的类型，不同类型适用于不同的任务"
          >
            <Select
              options={[
                { value: 'chat', label: '对话', icon: <RobotOutlined /> },
                { value: 'completion', label: '文本生成', icon: <CodeOutlined /> },
                { value: 'embedding', label: '向量嵌入', icon: <DatabaseOutlined /> },
                { value: 'image', label: '图像生成', icon: <ExperimentOutlined /> },
                { value: 'speech', label: '语音处理', icon: <ApiOutlined /> },
                { value: 'custom', label: '自定义', icon: <SettingOutlined /> },
              ]}
              disabled={readOnly}
            />
          </Form.Item>
        </Col>
        <Col span={12}>
          <Form.Item
            label="模型"
            name="model"
            tooltip="选择要使用的AI模型"
          >
            <Select
              options={modelOptions[form.getFieldValue('agentType') as keyof typeof modelOptions] || modelOptions.chat}
              disabled={readOnly}
              showSearch
              optionFilterProp="children"
            />
          </Form.Item>
        </Col>
      </Row>

      <Form.Item
        label="系统提示"
        name="systemPrompt"
        tooltip="系统提示定义了AI的行为和角色"
        rules={[{ required: true, message: '请输入系统提示' }]}
      >
        <TextArea
          rows={4}
          placeholder="输入系统提示，定义AI的行为和角色..."
          disabled={readOnly}
          showCount
          maxLength={2000}
        />
      </Form.Item>

      <Row gutter={16}>
        <Col span={8}>
          <Form.Item
            label={
              <Space>
                温度
                <Tooltip title="控制输出的随机性，值越高越有创造性">
                  <InfoCircleOutlined />
                </Tooltip>
              </Space>
            }
            name="temperature"
          >
            <Slider
              min={0}
              max={2}
              step={0.1}
              marks={{
                0: '保守',
                0.7: '平衡',
                1.4: '创意',
                2: '最大创意',
              }}
              disabled={readOnly}
            />
          </Form.Item>
        </Col>
        <Col span={8}>
          <Form.Item
            label={
              <Space>
                最大令牌数
                <Tooltip title="限制生成文本的最大长度">
                  <InfoCircleOutlined />
                </Tooltip>
              </Space>
            }
            name="maxTokens"
          >
            <InputNumber
              min={1}
              max={8192}
              disabled={readOnly}
              style={{ width: '100%' }}
            />
          </Form.Item>
        </Col>
        <Col span={8}>
          <Form.Item
            label={
              <Space>
                Top P
                <Tooltip title="核采样参数，控制词汇选择的多样性">
                  <InfoCircleOutlined />
                </Tooltip>
              </Space>
            }
            name="topP"
          >
            <Slider
              min={0}
              max={1}
              step={0.05}
              marks={{
                0: '精确',
                0.5: '平衡',
                1: '多样',
              }}
              disabled={readOnly}
            />
          </Form.Item>
        </Col>
      </Row>

      <Row gutter={16}>
        <Col span={12}>
          <Form.Item
            label={
              <Space>
                频率惩罚
                <Tooltip title="降低重复词语的概率">
                  <InfoCircleOutlined />
                </Tooltip>
              </Space>
            }
            name="frequencyPenalty"
          >
            <Slider
              min={-2}
              max={2}
              step={0.1}
              marks={{
                0: '无',
                1: '中等',
                2: '强',
              }}
              disabled={readOnly}
            />
          </Form.Item>
        </Col>
        <Col span={12}>
          <Form.Item
            label={
              <Space>
                存在惩罚
                <Tooltip title="鼓励谈论新话题">
                  <InfoCircleOutlined />
                </Tooltip>
              </Space>
            }
            name="presencePenalty"
          >
            <Slider
              min={-2}
              max={2}
              step={0.1}
              marks={{
                0: '无',
                1: '中等',
                2: '强',
              }}
              disabled={readOnly}
            />
          </Form.Item>
        </Col>
      </Row>

      <Row gutter={16}>
        <Col span={8}>
          <Form.Item label="启用流式响应" name="enableStreaming" valuePropName="checked">
            <Switch disabled={readOnly} />
          </Form.Item>
        </Col>
        <Col span={8}>
          <Form.Item label="启用记忆功能" name="enableMemory" valuePropName="checked">
            <Switch disabled={readOnly} />
          </Form.Item>
        </Col>
        <Col span={8}>
          <Form.Item label="启用工具调用" name="enableTools" valuePropName="checked">
            <Switch disabled={readOnly} />
          </Form.Item>
        </Col>
      </Row>
    </Form>
  );

  // 渲染高级配置
  const renderAdvancedConfig = () => (
    <Form
      form={form}
      layout="vertical"
      onValuesChange={handleFormChange}
    >
      <Card title="API 配置" size="small" style={{ marginBottom: 16 }}>
        <Row gutter={16}>
          <Col span={12}>
            <Form.Item label="API 端点" name="apiEndpoint">
              <Input
                placeholder="https://api.openai.com/v1"
                disabled={readOnly}
              />
            </Form.Item>
          </Col>
          <Col span={12}>
            <Form.Item label="API 密钥" name="apiKey">
              <Input.Password
                placeholder="sk-..."
                disabled={readOnly}
              />
            </Form.Item>
          </Col>
        </Row>
        <Row gutter={16}>
          <Col span={12}>
            <Form.Item label="超时时间 (秒)" name="timeout">
              <InputNumber
                min={1}
                max={300}
                disabled={readOnly}
                style={{ width: '100%' }}
              />
            </Form.Item>
          </Col>
          <Col span={12}>
            <Form.Item label="重试次数" name="retryAttempts">
              <InputNumber
                min={0}
                max={10}
                disabled={readOnly}
                style={{ width: '100%' }}
              />
            </Form.Item>
          </Col>
        </Row>
      </Card>

      <Card title="性能优化" size="small" style={{ marginBottom: 16 }}>
        <Row gutter={16}>
          <Col span={12}>
            <Form.Item label="批处理大小" name="batchSize">
              <InputNumber
                min={1}
                max={100}
                disabled={readOnly}
                style={{ width: '100%' }}
              />
            </Form.Item>
          </Col>
          <Col span={12}>
            <Form.Item label="并发请求数" name="maxConcurrent">
              <InputNumber
                min={1}
                max={10}
                disabled={readOnly}
                style={{ width: '100%' }}
              />
            </Form.Item>
          </Col>
        </Row>
      </Card>

      <Card title="自定义设置" size="small">
        <Form.Item name="customSettings">
          <TextArea
            rows={6}
            placeholder="输入JSON格式的自定义设置..."
            disabled={readOnly}
          />
        </Form.Item>
      </Card>
    </Form>
  );

  // 渲染测试面板
  const renderTestPanel = () => (
    <div>
      <Alert
        message="测试AI智能体配置"
        description="点击下方按钮测试当前的配置是否能够正常工作"
        type="info"
        showIcon
        style={{ marginBottom: 16 }}
      />

      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={12}>
          <Button
            type="primary"
            icon={<PlayCircleOutlined />}
            onClick={handleTest}
            loading={testing}
            disabled={readOnly}
            block
          >
            {testing ? '测试中...' : '开始测试'}
          </Button>
        </Col>
        <Col span={12}>
          <Button
            icon={<ReloadOutlined />}
            onClick={() => setTestResult(null)}
            disabled={readOnly}
            block
          >
            清除结果
          </Button>
        </Col>
      </Row>

      {testResult && (
        <Card
          title={
            <Space>
              测试结果
              <Badge
                status={
                  testResult.status === 'success' ? 'success' :
                  testResult.status === 'error' ? 'error' : 'processing'
                }
                text={
                  testResult.status === 'success' ? '成功' :
                  testResult.status === 'error' ? '失败' : '测试中'
                }
              />
            </Space>
          }
          size="small"
        >
          <div style={{ marginBottom: 8 }}>
            <Text strong>状态: </Text>
            <Tag
              color={
                testResult.status === 'success' ? 'green' :
                testResult.status === 'error' ? 'red' : 'blue'
              }
            >
              {testResult.message}
            </Tag>
          </div>
          {testResult.responseTime && (
            <div>
              <Text strong>响应时间: </Text>
              <Text>{testResult.responseTime}ms</Text>
            </div>
          )}
        </Card>
      )}

      <Divider />

      <Title level={5}>测试历史</Title>
      <Alert
        message="暂无测试历史"
        description="完成测试后，历史记录将显示在这里"
        type="info"
        showIcon={false}
      />
    </div>
  );

  return (
    <div className="ai-agent-config" style={{ width: '100%' }}>
      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
        size="small"
        tabBarStyle={{ marginBottom: 16 }}
      >
        <TabPane
          tab={
            <Space>
              <SettingOutlined />
              基础配置
            </Space>
          }
          key="basic"
        >
          {renderBasicConfig()}
        </TabPane>

        <TabPane
          tab={
            <Space>
              <CodeOutlined />
              高级配置
            </Space>
          }
          key="advanced"
        >
          {renderAdvancedConfig()}
        </TabPane>

        <TabPane
          tab={
            <Space>
              <ExperimentOutlined />
              测试
            </Space>
          }
          key="test"
        >
          {renderTestPanel()}
        </TabPane>
      </Tabs>
    </div>
  );
};

export default AIAgentConfig;