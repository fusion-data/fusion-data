import React, { useState, useCallback, useEffect } from 'react';
import {
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
  Collapse,
  Empty,
  Alert,
  Tooltip,
} from 'antd';
import {
  SettingOutlined,
  InfoCircleOutlined,
  DatabaseOutlined,
  DeleteOutlined,
  CopyOutlined,
  RobotOutlined,
} from '@ant-design/icons';
import { useWorkflowStore } from '@/stores';
import { useNodeContext } from '../nodes/NodeContext';
import { NodeConfig } from '../nodes/types';
import AIAgentConfigDialog from '../nodes/ai/AIAgentConfigDialog';

const { Text, Title } = Typography;
const { TabPane } = Tabs;
const { Panel } = Collapse;

interface PropertyPanelProps {
  nodeId?: string;
  onClose?: () => void;
}

export const PropertyPanel: React.FC<PropertyPanelProps> = ({ nodeId, onClose }) => {
  const { updateNode, deleteNode, selectedNodes } = useWorkflowStore();
  const { getNodeInfo, duplicateNode } = useNodeContext();
  const [form] = Form.useForm();
  const [activeTab, setActiveTab] = useState('basic');
  const [aiConfigDialogVisible, setAiConfigDialogVisible] = useState(false);

  const nodeInfo = nodeId ? getNodeInfo(nodeId) : null;
  const isSelected = nodeId && selectedNodes.includes(nodeId);

  // 表单值变化处理
  const handleFormChange = useCallback((_changedValues: any, allValues: any) => {
    if (!nodeId) return;

    const updates: Partial<NodeConfig> = {
      data: {
        ...nodeInfo?.data,
        ...allValues,
      },
    };

    updateNode(nodeId, updates);
  }, [nodeId, nodeInfo, updateNode]);

  // 删除节点
  const handleDelete = useCallback(() => {
    if (!nodeId) return;

    if (window.confirm('确定要删除这个节点吗？')) {
      deleteNode(nodeId);
      onClose?.();
    }
  }, [nodeId, deleteNode, onClose]);

  // 复制节点
  const handleDuplicate = useCallback(() => {
    if (!nodeId) return;

    const newNode = duplicateNode(nodeId);
    if (newNode) {
      // 选中复制的节点
      // TODO: 实现节点选择逻辑
    }
  }, [nodeId, duplicateNode]);

  // 重置表单
  const handleReset = useCallback(() => {
    form.resetFields();
    form.setFieldsValue(nodeInfo?.data);
  }, [form, nodeInfo]);

  // 当节点信息变化时更新表单
  useEffect(() => {
    if (nodeInfo) {
      form.setFieldsValue(nodeInfo.data);
    }
  }, [nodeInfo, form]);

  // 渲染基础属性表单
  const renderBasicProperties = () => {
    if (!nodeInfo) return null;

    return (
      <Form
        form={form}
        layout="vertical"
        onValuesChange={handleFormChange}
        initialValues={nodeInfo.data}
      >
        <Form.Item label="节点名称" name="label">
          <Input placeholder="输入节点名称" />
        </Form.Item>

        <Form.Item label="描述" name="description">
          <Input.TextArea
            rows={3}
            placeholder="输入节点描述"
          />
        </Form.Item>

        {/* 根据节点类型渲染特定配置 */}
        {renderNodeSpecificConfig()}

        <Form.Item label="启用" name="enabled" valuePropName="checked">
          <Switch />
        </Form.Item>
      </Form>
    );
  };

  // 根据节点类型渲染特定配置
  const renderNodeSpecificConfig = () => {
    if (!nodeInfo) return null;

    switch (nodeInfo.type) {
      case 'aiAgent':
        return (
          <>
            {/* AI Agent 高级配置选项 */}
            <Form.Item label="配置模式" name={['config', 'configMode']}>
              <Select
                placeholder="选择配置模式"
                onChange={(value) => {
                  // 切换配置模式时的处理逻辑
                  if (value === 'advanced') {
                    // 显示高级配置面板的逻辑
                  }
                }}
              >
                <Select.Option value="basic">基础配置</Select.Option>
                <Select.Option value="advanced">高级配置</Select.Option>
                <Select.Option value="template">模板选择</Select.Option>
              </Select>
            </Form.Item>

            {/* 简化版基础配置 */}
            <Form.Item label="智能体类型" name={['config', 'agentType']}>
              <Select>
                <Select.Option value="chat">对话</Select.Option>
                <Select.Option value="completion">文本生成</Select.Option>
                <Select.Option value="embedding">向量嵌入</Select.Option>
                <Select.Option value="image">图像生成</Select.Option>
                <Select.Option value="speech">语音处理</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item label="模型" name={['config', 'model']}>
              <Select placeholder="选择模型">
                <Select.Option value="gpt-4">GPT-4</Select.Option>
                <Select.Option value="gpt-3.5-turbo">GPT-3.5 Turbo</Select.Option>
                <Select.Option value="claude-3-opus">Claude 3 Opus</Select.Option>
                <Select.Option value="claude-3-sonnet">Claude 3 Sonnet</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item label="系统提示" name={['config', 'systemPrompt']}>
              <Input.TextArea
                rows={3}
                placeholder="定义AI的行为和角色..."
              />
            </Form.Item>

            <Form.Item label="温度" name={['config', 'temperature']}>
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
              />
            </Form.Item>

            <Form.Item label="最大令牌数" name={['config', 'maxTokens']}>
              <Input type="number" placeholder="默认: 2048" />
            </Form.Item>

            <Form.Item label="启用流式响应" name={['config', 'enableStreaming']} valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item label="启用记忆功能" name={['config', 'enableMemory']} valuePropName="checked">
              <Switch />
            </Form.Item>

            {/* 高级功能链接 */}
            <Alert
              message="更多功能"
              description="点击下方按钮打开完整的AI Agent配置界面，包含模板选择、对话测试、性能监控等高级功能。"
              type="info"
              showIcon
              style={{ marginTop: 16 }}
              action={
                <Button
                  size="small"
                  type="primary"
                  icon={<RobotOutlined />}
                  onClick={() => setAiConfigDialogVisible(true)}
                >
                  打开高级配置
                </Button>
              }
            />
          </>
        );

      case 'trigger':
        return (
          <>
            <Form.Item label="触发器类型" name="triggerType">
              <Select>
                <Select.Option value="manual">手动触发</Select.Option>
                <Select.Option value="schedule">定时触发</Select.Option>
                <Select.Option value="webhook">Webhook 触发</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item label="CRON 表达式" name={['config', 'cronExpression']}>
              <Input placeholder="0 0 * * *" />
            </Form.Item>
          </>
        );

      case 'action':
        return (
          <>
            <Form.Item label="动作类型" name="actionType">
              <Select>
                <Select.Option value="api">API 调用</Select.Option>
                <Select.Option value="code">代码执行</Select.Option>
                <Select.Option value="database">数据库操作</Select.Option>
                <Select.Option value="email">发送邮件</Select.Option>
                <Select.Option value="file">文件操作</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item label="请求方法" name={['config', 'method']}>
              <Select>
                <Select.Option value="GET">GET</Select.Option>
                <Select.Option value="POST">POST</Select.Option>
                <Select.Option value="PUT">PUT</Select.Option>
                <Select.Option value="DELETE">DELETE</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item label="请求URL" name={['config', 'url']}>
              <Input placeholder="https://api.example.com/endpoint" />
            </Form.Item>
          </>
        );

      case 'condition':
        return (
          <>
            <Form.Item label="条件类型" name="conditionType">
              <Select>
                <Select.Option value="if">IF 条件</Select.Option>
                <Select.Option value="switch">SWITCH 条件</Select.Option>
                <Select.Option value="custom">自定义条件</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item label="条件表达式" name={['config', 'expression']}>
              <Input.TextArea
                rows={3}
                placeholder="输入条件表达式"
              />
            </Form.Item>
          </>
        );

      case 'dataProcessor':
        return (
          <>
            <Form.Item label="处理器类型" name="processorType">
              <Select>
                <Select.Option value="mapper">数据映射</Select.Option>
                <Select.Option value="filter">数据过滤</Select.Option>
                <Select.Option value="aggregator">数据聚合</Select.Option>
                <Select.Option value="transformer">数据转换</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item label="转换脚本" name={['config', 'transformScript']}>
              <Input.TextArea
                rows={4}
                placeholder="输入转换脚本"
              />
            </Form.Item>
          </>
        );

      default:
        return null;
    }
  };

  // 渲染数据信息
  const renderDataInfo = () => {
    if (!nodeInfo) return null;

    return (
      <div>
        <Title level={5}>
          <DatabaseOutlined /> 数据信息
        </Title>

        <Collapse size="small" ghost>
          <Panel header="输入端口" key="inputs">
            {nodeInfo.inputs.length > 0 ? (
              <Space direction="vertical" style={{ width: '100%' }}>
                {nodeInfo.inputs.map((port: any) => (
                  <div key={port.id} style={{
                    padding: '8px',
                    background: 'var(--bg-secondary)',
                    borderRadius: '4px',
                  }}>
                    <div style={{
                      fontWeight: 500,
                      color: 'var(--text-primary)',
                      marginBottom: '4px',
                    }}>
                      {port.name}
                    </div>
                    <div style={{
                      fontSize: '12px',
                      color: 'var(--text-secondary)',
                    }}>
                      类型: {port.type}
                      {port.required && (
                        <Tag color="red" style={{ marginLeft: '4px', fontSize: '10px' }}>
                          必填
                        </Tag>
                      )}
                    </div>
                    {port.description && (
                      <div style={{
                        fontSize: '12px',
                        color: 'var(--text-tertiary)',
                        marginTop: '4px',
                      }}>
                        {port.description}
                      </div>
                    )}
                  </div>
                ))}
              </Space>
            ) : (
              <Text type="secondary">无输入端口</Text>
            )}
          </Panel>

          <Panel header="输出端口" key="outputs">
            {nodeInfo.outputs.length > 0 ? (
              <Space direction="vertical" style={{ width: '100%' }}>
                {nodeInfo.outputs.map((port: any) => (
                  <div key={port.id} style={{
                    padding: '8px',
                    background: 'var(--bg-secondary)',
                    borderRadius: '4px',
                  }}>
                    <div style={{
                      fontWeight: 500,
                      color: 'var(--text-primary)',
                      marginBottom: '4px',
                    }}>
                      {port.name}
                    </div>
                    <div style={{
                      fontSize: '12px',
                      color: 'var(--text-secondary)',
                    }}>
                      类型: {port.type}
                    </div>
                    {port.description && (
                      <div style={{
                        fontSize: '12px',
                        color: 'var(--text-tertiary)',
                        marginTop: '4px',
                      }}>
                        {port.description}
                      </div>
                    )}
                  </div>
                ))}
              </Space>
            ) : (
              <Text type="secondary">无输出端口</Text>
            )}
          </Panel>
        </Collapse>
      </div>
    );
  };

  // 渲染节点信息
  const renderNodeInfo = () => {
    if (!nodeInfo) return null;

    return (
      <div>
        <Title level={5}>
          <InfoCircleOutlined /> 节点信息
        </Title>

        <Space direction="vertical" style={{ width: '100%' }}>
          <div>
            <Text strong>节点类型:</Text>
            <Tag color="blue" style={{ marginLeft: '8px' }}>
              {nodeInfo.typeConfig?.displayName || nodeInfo.type}
            </Tag>
          </div>

          <div>
            <Text strong>节点ID:</Text>
            <Text copyable style={{ marginLeft: '8px', fontFamily: 'monospace' }}>
              {nodeInfo.id}
            </Text>
          </div>

          <div>
            <Text strong>状态:</Text>
            <Tag
              color={
                nodeInfo.status === 'success' ? 'green' :
                nodeInfo.status === 'error' ? 'red' :
                nodeInfo.status === 'running' ? 'blue' : 'default'
              }
              style={{ marginLeft: '8px' }}
            >
              {nodeInfo.status}
            </Tag>
          </div>

          {nodeInfo.typeConfig?.description && (
            <div>
              <Text strong>描述:</Text>
              <Text style={{ marginLeft: '8px' }}>
                {nodeInfo.typeConfig.description}
              </Text>
            </div>
          )}
        </Space>
      </div>
    );
  };

  // 面板头部
  const renderHeader = () => {
    if (!nodeInfo) {
      return (
        <div style={{
          padding: '16px',
          borderBottom: '1px solid var(--border-primary)',
        }}>
          <Empty
            description="请选择一个节点"
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            style={{ margin: 0 }}
          />
        </div>
      );
    }

    return (
      <div style={{
        padding: '16px',
        borderBottom: '1px solid var(--border-primary)',
      }}>
        <div style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}>
          <div>
            <Title level={5} style={{ margin: 0 }}>
              {nodeInfo.data.label}
            </Title>
            <Text type="secondary" style={{ fontSize: '12px' }}>
              {nodeInfo.typeConfig?.displayName}
            </Text>
          </div>

          <Space>
            <Tooltip title="复制节点">
              <Button
                type="text"
                size="small"
                icon={<CopyOutlined />}
                onClick={handleDuplicate}
              />
            </Tooltip>

            <Tooltip title="删除节点">
              <Button
                type="text"
                size="small"
                danger
                icon={<DeleteOutlined />}
                onClick={handleDelete}
              />
            </Tooltip>
          </Space>
        </div>

        {isSelected && (
          <Alert
            message="已选择此节点"
            type="info"
            style={{ marginTop: '12px', fontSize: '12px' }}
          />
        )}
      </div>
    );
  };

  return (
    <div className="property-panel" style={{
      width: '320px',
      height: '100%',
      background: 'var(--bg-primary)',
      borderLeft: '1px solid var(--border-primary)',
      display: 'flex',
      flexDirection: 'column',
    }}>
      {renderHeader()}

      {nodeInfo && (
        <div style={{ flex: 1, overflow: 'hidden' }}>
          <Tabs
            activeKey={activeTab}
            onChange={setActiveTab}
            size="small"
            style={{ flex: 1, display: 'flex', flexDirection: 'column' }}
            tabBarStyle={{ margin: 0, padding: '0 16px' }}
          >
            <TabPane
              tab={<span><SettingOutlined /> 属性</span>}
              key="basic"
            >
              <div style={{
                padding: '16px',
                overflow: 'auto',
                height: '100%',
              }}>
                {renderBasicProperties()}
              </div>
            </TabPane>

            <TabPane
              tab={<span><DatabaseOutlined /> 数据</span>}
              key="data"
            >
              <div style={{
                padding: '16px',
                overflow: 'auto',
                height: '100%',
              }}>
                {renderDataInfo()}
              </div>
            </TabPane>

            <TabPane
              tab={<span><InfoCircleOutlined /> 信息</span>}
              key="info"
            >
              <div style={{
                padding: '16px',
                overflow: 'auto',
                height: '100%',
              }}>
                {renderNodeInfo()}
              </div>
            </TabPane>
          </Tabs>
        </div>
      )}

      {/* 底部操作栏 */}
      {nodeInfo && (
        <div style={{
          padding: '12px 16px',
          borderTop: '1px solid var(--border-primary)',
          display: 'flex',
          justifyContent: 'space-between',
        }}>
          <Button size="small" onClick={handleReset}>
            重置
          </Button>
          <Button type="primary" size="small" onClick={() => form.submit()}>
            应用更改
          </Button>
        </div>
      )}

      {/* AI Agent 配置对话框 */}
      {nodeInfo?.type === 'aiAgent' && (
        <AIAgentConfigDialog
          visible={aiConfigDialogVisible}
          nodeId={nodeId || ''}
          initialConfig={nodeInfo?.data?.config}
          onClose={() => setAiConfigDialogVisible(false)}
          onSave={(config) => {
            // 保存AI Agent配置
            updateNode(nodeId!, {
              data: {
                ...nodeInfo?.data,
                config,
              },
            });
            setAiConfigDialogVisible(false);
          }}
        />
      )}
    </div>
  );
};

export default PropertyPanel;