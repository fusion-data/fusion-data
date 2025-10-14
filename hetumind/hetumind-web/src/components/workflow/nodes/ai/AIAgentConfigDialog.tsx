import React, { useState, useCallback, useEffect } from 'react';
import {
  Modal,
  Tabs,
  Space,
  Button,
  Typography,
  Alert,
  Row,
  Col,
  Card,
  Badge,
  Tooltip,
} from 'antd';
import {
  SettingOutlined,
  RobotOutlined,
  CodeOutlined,
  ExperimentOutlined,
  BarChartOutlined,
  ThunderboltOutlined,
  SaveOutlined,
  CloseOutlined,
} from '@ant-design/icons';
import AIAgentConfig from './AIAgentConfig';
import AIAgentTemplates from './AIAgentTemplates';
import AIAgentChat from './AIAgentChat';
import AIAgentMonitor from './AIAgentMonitor';

const { Title, Text } = Typography;
const { TabPane } = Tabs;

// AI Agent完整配置接口
interface CompleteAIAgentConfig {
  nodeId: string;
  name: string;
  description?: string;
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

interface AIAgentConfigDialogProps {
  visible: boolean;
  nodeId: string;
  initialConfig?: Partial<CompleteAIAgentConfig>;
  onClose: () => void;
  onSave?: (config: CompleteAIAgentConfig) => void;
  readOnly?: boolean;
}

export const AIAgentConfigDialog: React.FC<AIAgentConfigDialogProps> = ({
  visible,
  nodeId,
  initialConfig,
  onClose,
  onSave,
  readOnly = false,
}) => {
  const [activeTab, setActiveTab] = useState('config');
  const [config, setConfig] = useState<CompleteAIAgentConfig>({
    nodeId,
    name: 'AI Agent',
    agentType: 'chat',
    model: 'gpt-3.5-turbo',
    systemPrompt: '你是一个智能助手，请根据用户的问题提供准确、有用的回答。',
    temperature: 0.7,
    maxTokens: 2048,
    topP: 1,
    frequencyPenalty: 0,
    presencePenalty: 0,
    enableStreaming: true,
    enableMemory: true,
    enableTools: false,
    timeout: 30,
    retryAttempts: 3,
  });

  const [hasChanges, setHasChanges] = useState(false);
  const [testResult, setTestResult] = useState<{
    status: 'success' | 'error' | 'testing';
    message?: string;
  } | null>(null);

  // 初始化配置
  useEffect(() => {
    if (initialConfig) {
      setConfig(prev => ({ ...prev, ...initialConfig }));
    }
  }, [initialConfig]);

  // 配置变化处理
  const handleConfigChange = useCallback((newConfig: Partial<CompleteAIAgentConfig>) => {
    setConfig(prev => ({ ...prev, ...newConfig }));
    setHasChanges(true);
    setTestResult(null);
  }, []);

  // 模板选择处理
  const handleTemplateSelect = useCallback((template: any) => {
    setConfig(prev => ({
      ...prev,
      name: template.name,
      description: template.description,
      agentType: template.agentType,
      model: template.model,
      systemPrompt: template.systemPrompt,
      temperature: template.temperature,
      maxTokens: template.maxTokens,
    }));
    setHasChanges(true);
    setActiveTab('config');
  }, []);

  // 测试配置
  const handleTest = useCallback(async (testConfig: any) => {
    setTestResult({ status: 'testing' });

    try {
      // 模拟测试
      await new Promise(resolve => setTimeout(resolve, 1500));

      // 简单的验证逻辑
      if (!testConfig.systemPrompt || testConfig.systemPrompt.length < 10) {
        throw new Error('系统提示太短，请提供更详细的描述');
      }

      if (testConfig.temperature < 0 || testConfig.temperature > 2) {
        throw new Error('温度参数超出有效范围 (0-2)');
      }

      setTestResult({
        status: 'success',
        message: '配置验证成功！AI Agent 可以正常工作。',
      });
    } catch (error: any) {
      setTestResult({
        status: 'error',
        message: error?.message || '配置验证失败',
      });
    }
  }, []);

  // 保存配置
  const handleSave = useCallback(() => {
    if (onSave) {
      onSave(config);
    }
    onClose();
  }, [config, onSave, onClose]);

  // 关闭对话框
  const handleClose = useCallback(() => {
    if (hasChanges) {
      // 可以在这里添加未保存提醒
      console.log('存在未保存的更改');
    }
    onClose();
  }, [hasChanges, onClose]);

  return (
    <Modal
      title={
        <Space>
          <RobotOutlined />
          <span>AI Agent 配置</span>
          {hasChanges && (
            <Badge status="warning" text="有未保存的更改" />
          )}
        </Space>
      }
      open={visible}
      onCancel={handleClose}
      width={1200}
      footer={[
        <Button key="cancel" icon={<CloseOutlined />} onClick={handleClose}>
          取消
        </Button>,
        <Button
          key="save"
          type="primary"
          icon={<SaveOutlined />}
          onClick={handleSave}
          disabled={readOnly}
        >
          保存配置
        </Button>,
      ]}
      destroyOnClose
    >
      {testResult && (
        <Alert
          message={
            testResult.status === 'success' ? '测试成功' :
            testResult.status === 'error' ? '测试失败' : '测试中'
          }
          description={testResult.message}
          type={
            testResult.status === 'success' ? 'success' :
            testResult.status === 'error' ? 'error' : 'info'
          }
          showIcon
          style={{ marginBottom: 16 }}
          closable
          onClose={() => setTestResult(null)}
        />
      )}

      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
        size="small"
        type="card"
      >
        <TabPane
          tab={
            <Space>
              <SettingOutlined />
              基础配置
            </Space>
          }
          key="config"
        >
          <AIAgentConfig
            nodeId={nodeId}
            initialConfig={config}
            onConfigChange={handleConfigChange}
            onTest={handleTest}
            readOnly={readOnly}
          />
        </TabPane>

        <TabPane
          tab={
            <Space>
              <ThunderboltOutlined />
              模板库
            </Space>
          }
          key="templates"
        >
          <AIAgentTemplates
            onTemplateSelect={handleTemplateSelect}
            readOnly={readOnly}
          />
        </TabPane>

        <TabPane
          tab={
            <Space>
              <CodeOutlined />
              对话测试
            </Space>
          }
          key="chat"
        >
          <AIAgentChat
            initialConfig={{
              model: config.model,
              systemPrompt: config.systemPrompt,
              temperature: config.temperature,
              maxTokens: config.maxTokens,
              enableStreaming: config.enableStreaming,
              enableMemory: config.enableMemory,
            }}
            onConfigChange={(chatConfig) => {
              handleConfigChange({
                model: chatConfig.model,
                systemPrompt: chatConfig.systemPrompt,
                temperature: chatConfig.temperature,
                maxTokens: chatConfig.maxTokens,
                enableStreaming: chatConfig.enableStreaming,
                enableMemory: chatConfig.enableMemory,
              });
            }}
            readOnly={readOnly}
            height={500}
            showConfig={true}
          />
        </TabPane>

        <TabPane
          tab={
            <Space>
              <BarChartOutlined />
              性能监控
            </Space>
          }
          key="monitor"
        >
          <AIAgentMonitor
            nodeId={nodeId}
            agentName={config.name}
            height={500}
            showDetails={true}
          />
        </TabPane>

        <TabPane
          tab={
            <Space>
              <ExperimentOutlined />
              高级设置
            </Space>
          }
          key="advanced"
        >
          <div style={{ padding: 16 }}>
            <Alert
              message="高级设置"
              description="这里将包含更多高级配置选项，如工具集成、自定义函数、知识库连接等。此功能正在开发中。"
              type="info"
              showIcon
            />

            {/* 预留高级配置内容区域 */}
            <div style={{ marginTop: 16 }}>
              <Row gutter={16}>
                <Col span={12}>
                  <Card title="工具集成" size="small">
                    <Text type="secondary">工具集成功能开发中...</Text>
                  </Card>
                </Col>
                <Col span={12}>
                  <Card title="知识库连接" size="small">
                    <Text type="secondary">知识库连接功能开发中...</Text>
                  </Card>
                </Col>
              </Row>

              <Row gutter={16} style={{ marginTop: 16 }}>
                <Col span={12}>
                  <Card title="自定义函数" size="small">
                    <Text type="secondary">自定义函数功能开发中...</Text>
                  </Card>
                </Col>
                <Col span={12}>
                  <Card title="安全设置" size="small">
                    <Text type="secondary">安全设置功能开发中...</Text>
                  </Card>
                </Col>
              </Row>
            </div>
          </div>
        </TabPane>
      </Tabs>
    </Modal>
  );
};

export default AIAgentConfigDialog;