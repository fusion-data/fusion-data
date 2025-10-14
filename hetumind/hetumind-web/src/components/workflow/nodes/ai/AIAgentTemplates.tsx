import React, { useState, useCallback } from 'react';
import {
  Card,
  Button,
  Space,
  Typography,
  Tag,
  Row,
  Col,
  Modal,
  Form,
  Input,
  Select,
  Alert,
  Divider,
  Tooltip,
  Badge,
  Empty,
  Popconfirm,
} from 'antd';
import {
  RobotOutlined,
  StarOutlined,
  CopyOutlined,
  EditOutlined,
  DeleteOutlined,
  PlusOutlined,
  ThunderboltOutlined,
  BookOutlined,
  CodeOutlined,
  ExperimentOutlined,
  ApiOutlined,
  DatabaseOutlined,
} from '@ant-design/icons';

const { Text, Title, Paragraph } = Typography;
const { TextArea } = Input;

// AI Agent 模板接口
interface AIAgentTemplate {
  id: string;
  name: string;
  description: string;
  category: 'chatbot' | 'assistant' | 'analyzer' | 'generator' | 'translator' | 'custom';
  agentType: 'chat' | 'completion' | 'embedding' | 'image' | 'speech' | 'custom';
  model: string;
  systemPrompt: string;
  temperature: number;
  maxTokens: number;
  tags: string[];
  icon?: React.ReactNode;
  isPopular?: boolean;
  isCustom?: boolean;
  config?: Record<string, any>;
}

interface AIAgentTemplatesProps {
  onTemplateSelect?: (template: AIAgentTemplate) => void;
  onTemplateCreate?: (template: Partial<AIAgentTemplate>) => void;
  onTemplateEdit?: (template: AIAgentTemplate) => void;
  onTemplateDelete?: (templateId: string) => void;
  readOnly?: boolean;
}

export const AIAgentTemplates: React.FC<AIAgentTemplatesProps> = ({
  onTemplateSelect,
  onTemplateCreate,
  onTemplateEdit,
  onTemplateDelete,
  readOnly = false,
}) => {
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [createModalVisible, setCreateModalVisible] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<AIAgentTemplate | null>(null);
  const [form] = Form.useForm();

  // 预定义模板
  const predefinedTemplates: AIAgentTemplate[] = [
    {
      id: 'customer-service',
      name: '客服助手',
      description: '专业的客户服务助手，友好、耐心地解决用户问题',
      category: 'chatbot',
      agentType: 'chat',
      model: 'gpt-3.5-turbo',
      systemPrompt: '你是一个专业的客服助手。请友好、耐心地回答用户的问题，提供准确、有帮助的信息。如果遇到无法解决的问题，请礼貌地建议用户联系人工客服。',
      temperature: 0.7,
      maxTokens: 1024,
      tags: ['客服', '对话', '友好'],
      icon: <RobotOutlined />,
      isPopular: true,
    },
    {
      id: 'code-assistant',
      name: '编程助手',
      description: '专业的编程助手，帮助编写、调试和优化代码',
      category: 'assistant',
      agentType: 'chat',
      model: 'gpt-4',
      systemPrompt: '你是一个专业的编程助手。请帮助用户编写、调试和优化代码。提供清晰的代码示例和详细的解释。遵循最佳实践和安全规范。',
      temperature: 0.3,
      maxTokens: 2048,
      tags: ['编程', '代码', '调试'],
      icon: <CodeOutlined />,
      isPopular: true,
    },
    {
      id: 'data-analyzer',
      name: '数据分析师',
      description: '专业的数据分析助手，帮助分析和解释数据',
      category: 'analyzer',
      agentType: 'completion',
      model: 'gpt-3.5-turbo',
      systemPrompt: '你是一个专业的数据分析师。请帮助用户分析数据、识别趋势、生成洞察。使用清晰的数据可视化建议和统计分析方法。',
      temperature: 0.5,
      maxTokens: 1536,
      tags: ['数据分析', '统计', '洞察'],
      icon: <DatabaseOutlined />,
    },
    {
      id: 'content-generator',
      name: '内容生成器',
      description: '创意内容生成助手，帮助创作各种类型的内容',
      category: 'generator',
      agentType: 'completion',
      model: 'gpt-3.5-turbo',
      systemPrompt: '你是一个专业的内容创作者。根据用户的需求创作高质量的内容，包括文章、营销文案、社交媒体帖子等。保持创意性和专业性。',
      temperature: 0.8,
      maxTokens: 1500,
      tags: ['内容创作', '创意', '写作'],
      icon: <ExperimentOutlined />,
    },
    {
      id: 'translator',
      name: '翻译专家',
      description: '专业的多语言翻译助手',
      category: 'translator',
      agentType: 'chat',
      model: 'gpt-4',
      systemPrompt: '你是一个专业的翻译专家。请提供准确、自然的翻译，保持原文的语气和含义。注意文化差异和语言习惯。',
      temperature: 0.4,
      maxTokens: 2048,
      tags: ['翻译', '多语言', '本地化'],
      icon: <ApiOutlined />,
    },
    {
      id: 'research-assistant',
      name: '研究助理',
      description: '学术研究助手，帮助文献检索和分析',
      category: 'assistant',
      agentType: 'chat',
      model: 'gpt-4',
      systemPrompt: '你是一个专业的研究助理。帮助用户进行文献检索、数据分析、论文写作等学术研究工作。保持学术严谨性和客观性。',
      temperature: 0.6,
      maxTokens: 2048,
      tags: ['研究', '学术', '文献'],
      icon: <BookOutlined />,
    },
  ];

  const [customTemplates, setCustomTemplates] = useState<AIAgentTemplate[]>([]);

  // 分类选项
  const categories = [
    { key: 'all', name: '全部模板', icon: <ThunderboltOutlined /> },
    { key: 'chatbot', name: '聊天机器人', icon: <RobotOutlined /> },
    { key: 'assistant', name: '智能助手', icon: <StarOutlined /> },
    { key: 'analyzer', name: '分析工具', icon: <DatabaseOutlined /> },
    { key: 'generator', name: '内容生成', icon: <ExperimentOutlined /> },
    { key: 'translator', name: '翻译工具', icon: <ApiOutlined /> },
    { key: 'custom', name: '自定义', icon: <CodeOutlined /> },
  ];

  // 获取分类图标
  const getCategoryIcon = (category: string) => {
    const cat = categories.find(c => c.key === category);
    return cat?.icon || <RobotOutlined />;
  };

  // 获取分类名称
  const getCategoryName = (category: string) => {
    const cat = categories.find(c => c.key === category);
    return cat?.name || category;
  };

  // 过滤模板
  const filteredTemplates = [
    ...predefinedTemplates,
    ...customTemplates,
  ].filter(template => {
    if (selectedCategory === 'all') return true;
    if (selectedCategory === 'custom') return template.isCustom;
    return template.category === selectedCategory;
  });

  // 处理模板选择
  const handleTemplateSelect = useCallback((template: AIAgentTemplate) => {
    if (onTemplateSelect) {
      onTemplateSelect(template);
    }
  }, [onTemplateSelect]);

  // 处理模板创建
  const handleCreateTemplate = useCallback(async (values: any) => {
    const newTemplate: AIAgentTemplate = {
      id: `custom_${Date.now()}`,
      name: values.name,
      description: values.description,
      category: values.category,
      agentType: values.agentType,
      model: values.model,
      systemPrompt: values.systemPrompt,
      temperature: values.temperature,
      maxTokens: values.maxTokens,
      tags: values.tags.split(',').map((tag: string) => tag.trim()).filter(Boolean),
      isCustom: true,
      config: values.config || {},
    };

    setCustomTemplates(prev => [...prev, newTemplate]);

    if (onTemplateCreate) {
      onTemplateCreate(newTemplate);
    }

    setCreateModalVisible(false);
    form.resetFields();
  }, [form, onTemplateCreate]);

  // 处理模板编辑
  const handleEditTemplate = useCallback((template: AIAgentTemplate) => {
    setEditingTemplate(template);
    form.setFieldsValue({
      name: template.name,
      description: template.description,
      category: template.category,
      agentType: template.agentType,
      model: template.model,
      systemPrompt: template.systemPrompt,
      temperature: template.temperature,
      maxTokens: template.maxTokens,
      tags: template.tags.join(', '),
    });
  }, [form]);

  // 处理模板更新
  const handleUpdateTemplate = useCallback(async (values: any) => {
    if (!editingTemplate) return;

    const updatedTemplate: AIAgentTemplate = {
      ...editingTemplate,
      name: values.name,
      description: values.description,
      category: values.category,
      agentType: values.agentType,
      model: values.model,
      systemPrompt: values.systemPrompt,
      temperature: values.temperature,
      maxTokens: values.maxTokens,
      tags: values.tags.split(',').map((tag: string) => tag.trim()).filter(Boolean),
    };

    setCustomTemplates(prev =>
      prev.map(t => t.id === editingTemplate.id ? updatedTemplate : t)
    );

    if (onTemplateEdit) {
      onTemplateEdit(updatedTemplate);
    }

    setEditingTemplate(null);
    form.resetFields();
  }, [editingTemplate, form, onTemplateEdit]);

  // 处理模板删除
  const handleDeleteTemplate = useCallback((templateId: string) => {
    setCustomTemplates(prev => prev.filter(t => t.id !== templateId));

    if (onTemplateDelete) {
      onTemplateDelete(templateId);
    }
  }, [onTemplateDelete]);

  // 处理模板复制
  const handleCopyTemplate = useCallback((template: AIAgentTemplate) => {
    const copiedTemplate: AIAgentTemplate = {
      ...template,
      id: `custom_${Date.now()}`,
      name: `${template.name} (副本)`,
      isCustom: true,
    };

    setCustomTemplates(prev => [...prev, copiedTemplate]);

    if (onTemplateCreate) {
      onTemplateCreate(copiedTemplate);
    }
  }, [onTemplateCreate]);

  return (
    <div className="ai-agent-templates">
      {/* 分类筛选 */}
      <div style={{ marginBottom: 16 }}>
        <Space wrap>
          {categories.map(category => (
            <Button
              key={category.key}
              type={selectedCategory === category.key ? 'primary' : 'default'}
              icon={category.icon}
              onClick={() => setSelectedCategory(category.key)}
              size="small"
            >
              {category.name}
            </Button>
          ))}
        </Space>
      </div>

      {/* 模板列表 */}
      <div>
        {filteredTemplates.length === 0 ? (
          <Empty
            description="暂无模板"
            image={Empty.PRESENTED_IMAGE_SIMPLE}
          />
        ) : (
          <Row gutter={[16, 16]}>
            {filteredTemplates.map(template => (
              <Col key={template.id} xs={24} sm={12} lg={8}>
                <Card
                  size="small"
                  hoverable
                  className="template-card"
                  actions={[
                    <Tooltip title="使用模板">
                      <ThunderboltOutlined
                        key="use"
                        onClick={() => handleTemplateSelect(template)}
                      />
                    </Tooltip>,
                    <Tooltip title="复制模板">
                      <CopyOutlined
                        key="copy"
                        onClick={() => handleCopyTemplate(template)}
                      />
                    </Tooltip>,
                    ...(template.isCustom ? [
                      <Tooltip title="编辑模板">
                        <EditOutlined
                          key="edit"
                          onClick={() => handleEditTemplate(template)}
                        />
                      </Tooltip>,
                      <Popconfirm
                        title="确定要删除这个模板吗？"
                        onConfirm={() => handleDeleteTemplate(template.id)}
                        key="delete"
                      >
                        <Tooltip title="删除模板">
                          <DeleteOutlined />
                        </Tooltip>
                      </Popconfirm>,
                    ] : []),
                  ]}
                >
                  <div style={{ marginBottom: 8 }}>
                    <Space>
                      <div style={{ fontSize: 16, color: '#1890ff' }}>
                        {template.icon || getCategoryIcon(template.category)}
                      </div>
                      <div style={{ flex: 1 }}>
                        <div style={{ fontWeight: 500, marginBottom: 2 }}>
                          {template.name}
                          {template.isPopular && (
                            <Tag color="gold" size="small" style={{ marginLeft: 4 }}>
                              热门
                            </Tag>
                          )}
                          {template.isCustom && (
                            <Tag color="blue" size="small" style={{ marginLeft: 4 }}>
                              自定义
                            </Tag>
                          )}
                        </div>
                        <Text type="secondary" style={{ fontSize: 12 }}>
                          {getCategoryName(template.category)}
                        </Text>
                      </div>
                    </Space>
                  </div>

                  <Paragraph
                    ellipsis={{ rows: 2, tooltip: template.description }}
                    style={{ fontSize: 12, marginBottom: 8 }}
                  >
                    {template.description}
                  </Paragraph>

                  <div style={{ marginBottom: 8 }}>
                    <Space wrap size="small">
                      {template.tags.slice(0, 3).map(tag => (
                        <Tag key={tag} size="small">{tag}</Tag>
                      ))}
                      {template.tags.length > 3 && (
                        <Tag size="small">+{template.tags.length - 3}</Tag>
                      )}
                    </Space>
                  </div>

                  <div style={{ fontSize: 11, color: '#666' }}>
                    <Space split={<span>•</span>}>
                      <span>{template.model}</span>
                      <span>温度: {template.temperature}</span>
                      <span>令牌: {template.maxTokens}</span>
                    </Space>
                  </div>
                </Card>
              </Col>
            ))}
          </Row>
        )}
      </div>

      {/* 创建模板按钮 */}
      {!readOnly && (
        <div style={{ marginTop: 16, textAlign: 'center' }}>
          <Button
            type="dashed"
            icon={<PlusOutlined />}
            onClick={() => setCreateModalVisible(true)}
          >
            创建自定义模板
          </Button>
        </div>
      )}

      {/* 创建/编辑模板模态框 */}
      <Modal
        title={editingTemplate ? '编辑模板' : '创建模板'}
        open={createModalVisible || !!editingTemplate}
        onCancel={() => {
          setCreateModalVisible(false);
          setEditingTemplate(null);
          form.resetFields();
        }}
        footer={null}
        width={600}
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={editingTemplate ? handleUpdateTemplate : handleCreateTemplate}
        >
          <Form.Item
            label="模板名称"
            name="name"
            rules={[{ required: true, message: '请输入模板名称' }]}
          >
            <Input placeholder="输入模板名称" />
          </Form.Item>

          <Form.Item
            label="模板描述"
            name="description"
            rules={[{ required: true, message: '请输入模板描述' }]}
          >
            <TextArea rows={2} placeholder="描述模板的用途和特点" />
          </Form.Item>

          <Row gutter={16}>
            <Col span={12}>
              <Form.Item
                label="分类"
                name="category"
                rules={[{ required: true, message: '请选择分类' }]}
              >
                <Select>
                  <Select.Option value="chatbot">聊天机器人</Select.Option>
                  <Select.Option value="assistant">智能助手</Select.Option>
                  <Select.Option value="analyzer">分析工具</Select.Option>
                  <Select.Option value="generator">内容生成</Select.Option>
                  <Select.Option value="translator">翻译工具</Select.Option>
                  <Select.Option value="custom">自定义</Select.Option>
                </Select>
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item
                label="智能体类型"
                name="agentType"
                rules={[{ required: true, message: '请选择智能体类型' }]}
              >
                <Select>
                  <Select.Option value="chat">对话</Select.Option>
                  <Select.Option value="completion">文本生成</Select.Option>
                  <Select.Option value="embedding">向量嵌入</Select.Option>
                  <Select.Option value="image">图像生成</Select.Option>
                  <Select.Option value="speech">语音处理</Select.Option>
                  <Select.Option value="custom">自定义</Select.Option>
                </Select>
              </Form.Item>
            </Col>
          </Row>

          <Form.Item
            label="模型"
            name="model"
            rules={[{ required: true, message: '请选择模型' }]}
          >
            <Select>
              <Select.Option value="gpt-4">GPT-4</Select.Option>
              <Select.Option value="gpt-3.5-turbo">GPT-3.5 Turbo</Select.Option>
              <Select.Option value="claude-3-opus">Claude 3 Opus</Select.Option>
              <Select.Option value="claude-3-sonnet">Claude 3 Sonnet</Select.Option>
            </Select>
          </Form.Item>

          <Form.Item
            label="系统提示"
            name="systemPrompt"
            rules={[{ required: true, message: '请输入系统提示' }]}
          >
            <TextArea rows={4} placeholder="定义AI的行为和角色" />
          </Form.Item>

          <Row gutter={16}>
            <Col span={12}>
              <Form.Item label="温度" name="temperature">
                <Input type="number" min={0} max={2} step={0.1} />
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item label="最大令牌数" name="maxTokens">
                <Input type="number" min={1} max={8192} />
              </Form.Item>
            </Col>
          </Row>

          <Form.Item label="标签" name="tags">
            <Input placeholder="用逗号分隔的标签，如：编程,助手,代码" />
          </Form.Item>

          <Form.Item style={{ marginBottom: 0, textAlign: 'right' }}>
            <Space>
              <Button
                onClick={() => {
                  setCreateModalVisible(false);
                  setEditingTemplate(null);
                  form.resetFields();
                }}
              >
                取消
              </Button>
              <Button type="primary" htmlType="submit">
                {editingTemplate ? '更新' : '创建'}
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default AIAgentTemplates;