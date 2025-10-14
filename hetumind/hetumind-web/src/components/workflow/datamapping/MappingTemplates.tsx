import React, { useState, useCallback, useMemo } from 'react';
import {
  Card,
  Row,
  Col,
  Button,
  Space,
  Typography,
  Tag,
  Tabs,
  Input,
  Select,
  Alert,
  Empty,
  Badge,
  Tooltip,
  Modal,
  Form,
  message,
  Divider,
} from 'antd';
import {
  AppstoreOutlined,
  FileTextOutlined,
  DatabaseOutlined,
  ApiOutlined,
  UserOutlined,
  ShopOutlined,
  BookOutlined,
  ThunderboltOutlined,
  CopyOutlined,
  EyeOutlined,
  EditOutlined,
  DeleteOutlined,
  PlusOutlined,
  SearchOutlined,
  FilterOutlined,
} from '@ant-design/icons';

const { Text, Title, Paragraph } = Typography;
const { Search } = Input;
const { Option } = Select;
const { TabPane } = Tabs;

// 映射模板接口
interface MappingTemplate {
  id: string;
  name: string;
  description: string;
  category: 'ecommerce' | 'crm' | 'finance' | 'analytics' | 'api' | 'database' | 'custom';
  sourceType: 'json' | 'csv' | 'xml' | 'database' | 'api';
  targetType: 'json' | 'csv' | 'xml' | 'database' | 'api';
  sourceFields: Array<{
    name: string;
    type: 'string' | 'number' | 'boolean' | 'object' | 'array';
    path: string;
    description?: string;
    required?: boolean;
  }>;
  targetFields: Array<{
    name: string;
    type: 'string' | 'number' | 'boolean' | 'object' | 'array';
    path: string;
    description?: string;
    required?: boolean;
  }>;
  mappings: Array<{
    sourceField: string;
    targetField: string;
    transformType: 'direct' | 'function' | 'expression' | 'conditional';
    transformFunction?: string;
    expression?: string;
    defaultValue?: any;
    condition?: {
      field: string;
      operator: string;
      value: any;
    };
    enabled: boolean;
  }>;
  tags: string[];
  popularity: number;
  usage: number;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

interface MappingTemplatesProps {
  onTemplateSelect?: (template: MappingTemplate) => void;
  onTemplateCreate?: (template: Omit<MappingTemplate, 'id' | 'createdAt' | 'updatedAt'>) => void;
  readOnly?: boolean;
  height?: number;
}

export const MappingTemplates: React.FC<MappingTemplatesProps> = ({
  onTemplateSelect,
  onTemplateCreate,
  readOnly = false,
  height = 600,
}) => {
  const [activeCategory, setActiveCategory] = useState<string>('all');
  const [searchText, setSearchText] = useState('');
  const [selectedTemplate, setSelectedTemplate] = useState<MappingTemplate | null>(null);
  const [templateModalVisible, setTemplateModalVisible] = useState(false);
  const [createModalVisible, setCreateModalVisible] = useState(false);
  const [form] = Form.useForm();

  // 预定义的映射模板
  const predefinedTemplates: MappingTemplate[] = [
    // 电商模板
    {
      id: 'ecommerce_product_sync',
      name: '电商商品同步',
      description: '将电商平台商品数据同步到本地数据库',
      category: 'ecommerce',
      sourceType: 'json',
      targetType: 'database',
      sourceFields: [
        { name: '商品ID', type: 'string', path: 'product_id', required: true },
        { name: '商品名称', type: 'string', path: 'name', required: true },
        { name: '价格', type: 'number', path: 'price', required: true },
        { name: '库存', type: 'number', path: 'stock', required: true },
        { name: '描述', type: 'string', path: 'description' },
        { name: '分类', type: 'string', path: 'category' },
        { name: '标签', type: 'array', path: 'tags' },
        { name: '状态', type: 'string', path: 'status' },
        { name: '创建时间', type: 'string', path: 'created_at' },
      ],
      targetFields: [
        { name: 'id', type: 'string', path: 'id', required: true },
        { name: 'title', type: 'string', path: 'title', required: true },
        { name: 'price', type: 'number', path: 'price', required: true },
        { name: 'inventory', type: 'number', path: 'inventory', required: true },
        { name: 'description', type: 'string', path: 'description' },
        { name: 'category_id', type: 'string', path: 'category_id' },
        { name: 'keywords', type: 'string', path: 'keywords' },
        { name: 'is_active', type: 'boolean', path: 'is_active' },
        { name: 'created_date', type: 'string', path: 'created_date' },
      ],
      mappings: [
        { sourceField: 'product_id', targetField: 'id', transformType: 'direct', enabled: true },
        { sourceField: 'name', targetField: 'title', transformType: 'direct', enabled: true },
        { sourceField: 'price', targetField: 'price', transformType: 'direct', enabled: true },
        { sourceField: 'stock', targetField: 'inventory', transformType: 'direct', enabled: true },
        { sourceField: 'description', targetField: 'description', transformType: 'direct', enabled: true },
        { sourceField: 'category', targetField: 'category_id', transformType: 'function', transformFunction: 'mapCategory', enabled: true },
        { sourceField: 'tags', targetField: 'keywords', transformType: 'function', transformFunction: 'join', enabled: true },
        { sourceField: 'status', targetField: 'is_active', transformType: 'conditional', condition: { field: 'status', operator: 'equals', value: 'active' }, enabled: true },
        { sourceField: 'created_at', targetField: 'created_date', transformType: 'function', transformFunction: 'formatDate', enabled: true },
      ],
      tags: ['电商', '商品', '同步'],
      popularity: 85,
      usage: 156,
      createdBy: '系统',
      createdAt: '2024-01-15',
      updatedAt: '2024-01-15',
    },
    {
      id: 'ecommerce_order_export',
      name: '订单导出',
      description: '将订单数据导出为CSV格式供财务使用',
      category: 'ecommerce',
      sourceType: 'database',
      targetType: 'csv',
      sourceFields: [
        { name: '订单ID', type: 'string', path: 'order_id', required: true },
        { name: '用户ID', type: 'string', path: 'user_id', required: true },
        { name: '订单状态', type: 'string', path: 'status' },
        { name: '总金额', type: 'number', path: 'total_amount', required: true },
        { name: '支付方式', type: 'string', path: 'payment_method' },
        { name: '收货地址', type: 'object', path: 'shipping_address' },
        { name: '商品列表', type: 'array', path: 'items' },
        { name: '下单时间', type: 'string', path: 'created_at' },
      ],
      targetFields: [
        { name: '订单编号', type: 'string', path: 'order_no', required: true },
        { name: '客户姓名', type: 'string', path: 'customer_name', required: true },
        { name: '订单状态', type: 'string', path: 'status' },
        { name: '订单金额', type: 'number', path: 'amount', required: true },
        { name: '支付方式', type: 'string', path: 'payment' },
        { name: '收货地址', type: 'string', path: 'address' },
        { name: '商品数量', type: 'number', path: 'item_count' },
        { name: '下单日期', type: 'string', path: 'order_date' },
      ],
      mappings: [
        { sourceField: 'order_id', targetField: 'order_no', transformType: 'direct', enabled: true },
        { sourceField: 'user_id', targetField: 'customer_name', transformType: 'function', transformFunction: 'getCustomerName', enabled: true },
        { sourceField: 'status', targetField: 'status', transformType: 'function', transformFunction: 'formatStatus', enabled: true },
        { sourceField: 'total_amount', targetField: 'amount', transformType: 'direct', enabled: true },
        { sourceField: 'payment_method', targetField: 'payment', transformType: 'direct', enabled: true },
        { sourceField: 'shipping_address', targetField: 'address', transformType: 'function', transformFunction: 'formatAddress', enabled: true },
        { sourceField: 'items', targetField: 'item_count', transformType: 'function', transformFunction: 'countItems', enabled: true },
        { sourceField: 'created_at', targetField: 'order_date', transformType: 'function', transformFunction: 'formatDate', enabled: true },
      ],
      tags: ['电商', '订单', '导出', '财务'],
      popularity: 78,
      usage: 89,
      createdBy: '系统',
      createdAt: '2024-01-20',
      updatedAt: '2024-01-20',
    },

    // CRM模板
    {
      id: 'crm_customer_import',
      name: '客户导入',
      description: '将外部客户数据导入到CRM系统',
      category: 'crm',
      sourceType: 'csv',
      targetType: 'database',
      sourceFields: [
        { name: '客户姓名', type: 'string', path: 'name', required: true },
        { name: '邮箱', type: 'string', path: 'email', required: true },
        { name: '电话', type: 'string', path: 'phone' },
        { name: '公司', type: 'string', path: 'company' },
        { name: '职位', type: 'string', path: 'position' },
        { name: '行业', type: 'string', path: 'industry' },
        { name: '地区', type: 'string', path: 'region' },
        { name: '客户来源', type: 'string', path: 'source' },
        { name: '备注', type: 'string', path: 'notes' },
      ],
      targetFields: [
        { name: 'id', type: 'string', path: 'id', required: true },
        { name: 'full_name', type: 'string', path: 'full_name', required: true },
        { name: 'email_address', type: 'string', path: 'email_address', required: true },
        { name: 'phone_number', type: 'string', path: 'phone_number' },
        { name: 'company_name', type: 'string', path: 'company_name' },
        { name: 'job_title', type: 'string', path: 'job_title' },
        { name: 'industry_type', type: 'string', path: 'industry_type' },
        { name: 'location', type: 'string', path: 'location' },
        { name: 'lead_source', type: 'string', path: 'lead_source' },
        { name: 'description', type: 'string', path: 'description' },
        { name: 'created_at', type: 'string', path: 'created_at' },
      ],
      mappings: [
        { sourceField: 'name', targetField: 'full_name', transformType: 'direct', enabled: true },
        { sourceField: 'email', targetField: 'email_address', transformType: 'function', transformFunction: 'validateEmail', enabled: true },
        { sourceField: 'phone', targetField: 'phone_number', transformType: 'function', transformFunction: 'formatPhone', enabled: true },
        { sourceField: 'company', targetField: 'company_name', transformType: 'direct', enabled: true },
        { sourceField: 'position', targetField: 'job_title', transformType: 'direct', enabled: true },
        { sourceField: 'industry', targetField: 'industry_type', transformType: 'function', transformFunction: 'mapIndustry', enabled: true },
        { sourceField: 'region', targetField: 'location', transformType: 'direct', enabled: true },
        { sourceField: 'source', targetField: 'lead_source', transformType: 'function', transformFunction: 'mapSource', enabled: true },
        { sourceField: 'notes', targetField: 'description', transformType: 'direct', enabled: true },
        { targetField: 'id', sourceField: '', targetField: 'id', transformType: 'function', transformFunction: 'generateUUID', enabled: true },
        { targetField: 'created_at', sourceField: '', targetField: 'created_at', transformType: 'function', transformFunction: 'now', enabled: true },
      ],
      tags: ['CRM', '客户', '导入'],
      popularity: 92,
      usage: 234,
      createdBy: '系统',
      createdAt: '2024-01-10',
      updatedAt: '2024-01-10',
    },

    // 财务模板
    {
      id: 'finance_report_generate',
      name: '财务报表生成',
      description: '从交易数据生成月度财务报表',
      category: 'finance',
      sourceType: 'database',
      targetType: 'json',
      sourceFields: [
        { name: '交易ID', type: 'string', path: 'transaction_id', required: true },
        { name: '交易类型', type: 'string', path: 'type', required: true },
        { name: '金额', type: 'number', path: 'amount', required: true },
        { name: '币种', type: 'string', path: 'currency' },
        { name: '分类', type: 'string', path: 'category' },
        { name: '描述', type: 'string', path: 'description' },
        { name: '交易时间', type: 'string', path: 'transaction_date' },
      ],
      targetFields: [
        { name: '报表期间', type: 'string', path: 'report_period', required: true },
        { name: '总收入', type: 'number', path: 'total_income', required: true },
        { name: '总支出', type: 'number', path: 'total_expense', required: true },
        { name: '净利润', type: 'number', path: 'net_profit', required: true },
        { name: '分类汇总', type: 'object', path: 'category_summary' },
        { name: '币种汇总', type: 'object', path: 'currency_summary' },
        { name: '生成时间', type: 'string', path: 'generated_at' },
      ],
      mappings: [
        { targetField: 'report_period', sourceField: '', targetField: 'report_period', transformType: 'function', transformFunction: 'generateReportPeriod', enabled: true },
        { targetField: 'total_income', sourceField: '', targetField: 'total_income', transformType: 'expression', expression: 'sum(transactions where type = "income")', enabled: true },
        { targetField: 'total_expense', sourceField: '', targetField: 'total_expense', transformType: 'expression', expression: 'sum(transactions where type = "expense")', enabled: true },
        { targetField: 'net_profit', sourceField: '', targetField: 'net_profit', transformType: 'expression', expression: 'total_income - total_expense', enabled: true },
        { targetField: 'category_summary', sourceField: '', targetField: 'category_summary', transformType: 'function', transformFunction: 'groupByCategory', enabled: true },
        { targetField: 'currency_summary', sourceField: '', targetField: 'currency_summary', transformType: 'function', transformFunction: 'groupByCurrency', enabled: true },
        { targetField: 'generated_at', sourceField: '', targetField: 'generated_at', transformType: 'function', transformFunction: 'now', enabled: true },
      ],
      tags: ['财务', '报表', '月度'],
      popularity: 88,
      usage: 167,
      createdBy: '系统',
      createdAt: '2024-01-25',
      updatedAt: '2024-01-25',
    },

    // API模板
    {
      id: 'api_data_transform',
      name: 'API数据转换',
      description: '将第三方API数据转换为内部格式',
      category: 'api',
      sourceType: 'api',
      targetType: 'json',
      sourceFields: [
        { name: '用户ID', type: 'string', path: 'user.id', required: true },
        { name: '用户名', type: 'string', path: 'user.username' },
        { name: '邮箱', type: 'string', path: 'user.email' },
        { name: '头像', type: 'string', path: 'user.avatar' },
        { name: '角色', type: 'array', path: 'user.roles' },
        { name: '权限', type: 'object', path: 'permissions' },
        { name: '最后登录', type: 'string', path: 'user.last_login' },
      ],
      targetFields: [
        { name: 'id', type: 'string', path: 'id', required: true },
        { name: 'username', type: 'string', path: 'username', required: true },
        { name: 'email', type: 'string', path: 'email' },
        { name: 'profile', type: 'object', path: 'profile' },
        { name: 'authorities', type: 'array', path: 'authorities' },
        { name: 'enabled', type: 'boolean', path: 'enabled' },
        { name: 'lastLoginAt', type: 'string', path: 'lastLoginAt' },
      ],
      mappings: [
        { sourceField: 'user.id', targetField: 'id', transformType: 'direct', enabled: true },
        { sourceField: 'user.username', targetField: 'username', transformType: 'direct', enabled: true },
        { sourceField: 'user.email', targetField: 'email', transformType: 'direct', enabled: true },
        { sourceField: 'user.avatar', targetField: 'profile.avatar', transformType: 'function', transformFunction: 'validateUrl', enabled: true },
        { sourceField: 'user.roles', targetField: 'authorities', transformType: 'function', transformFunction: 'mapRoles', enabled: true },
        { sourceField: 'permissions', targetField: 'profile.permissions', transformType: 'direct', enabled: true },
        { sourceField: 'user.last_login', targetField: 'lastLoginAt', transformType: 'function', transformFunction: 'formatDate', enabled: true },
        { targetField: 'enabled', sourceField: '', targetField: 'enabled', transformType: 'expression', expression: 'if(contains(user.roles, "admin"), true, false)', enabled: true },
      ],
      tags: ['API', '用户', '转换'],
      popularity: 95,
      usage: 312,
      createdBy: '系统',
      createdAt: '2024-01-18',
      updatedAt: '2024-01-18',
    },
  ];

  // 类别配置
  const categories = [
    { key: 'all', name: '全部', icon: <AppstoreOutlined /> },
    { key: 'ecommerce', name: '电商', icon: <ShopOutlined /> },
    { key: 'crm', name: 'CRM', icon: <UserOutlined /> },
    { key: 'finance', name: '财务', icon: <DatabaseOutlined /> },
    { key: 'analytics', name: '分析', icon: <ThunderboltOutlined /> },
    { key: 'api', name: 'API', icon: <ApiOutlined /> },
    { key: 'database', name: '数据库', icon: <DatabaseOutlined /> },
    { key: 'custom', name: '自定义', icon: <FileTextOutlined /> },
  ];

  // 过滤模板
  const filteredTemplates = useMemo(() => {
    return predefinedTemplates.filter(template => {
      const matchesCategory = activeCategory === 'all' || template.category === activeCategory;
      const matchesSearch = searchText === '' ||
        template.name.toLowerCase().includes(searchText.toLowerCase()) ||
        template.description.toLowerCase().includes(searchText.toLowerCase()) ||
        template.tags.some(tag => tag.toLowerCase().includes(searchText.toLowerCase()));

      return matchesCategory && matchesSearch;
    });
  }, [activeCategory, searchText]);

  // 选择模板
  const handleTemplateSelect = useCallback((template: MappingTemplate) => {
    setSelectedTemplate(template);
    setTemplateModalVisible(true);
  }, []);

  // 使用模板
  const handleUseTemplate = useCallback((template: MappingTemplate) => {
    if (onTemplateSelect) {
      onTemplateSelect(template);
    }
    setTemplateModalVisible(false);
    message.success('模板应用成功');
  }, [onTemplateSelect]);

  // 复制模板
  const handleCopyTemplate = useCallback((template: MappingTemplate) => {
    const copiedTemplate = {
      ...template,
      name: `${template.name} (副本)`,
      id: `copy_${Date.now()}`,
    };

    if (onTemplateCreate) {
      onTemplateCreate(copiedTemplate);
    }
    message.success('模板复制成功');
  }, [onTemplateCreate]);

  // 创建新模板
  const handleCreateTemplate = useCallback(async () => {
    try {
      const values = await form.validateFields();
      const newTemplate = {
        ...values,
        popularity: 0,
        usage: 0,
        createdBy: '当前用户',
      };

      if (onTemplateCreate) {
        onTemplateCreate(newTemplate);
      }
      setCreateModalVisible(false);
      form.resetFields();
      message.success('模板创建成功');
    } catch (error) {
      // 表单验证失败
    }
  }, [form, onTemplateCreate]);

  // 渲染模板卡片
  const renderTemplateCard = (template: MappingTemplate) => (
    <Card
      key={template.id}
      hoverable
      size="small"
      style={{
        height: '100%',
        cursor: 'pointer',
      }}
      actions={[
        <Tooltip title="查看详情">
          <EyeOutlined key="view" onClick={(e) => { e.stopPropagation(); handleTemplateSelect(template); }} />
        </Tooltip>,
        <Tooltip title="使用模板">
          <ThunderboltOutlined key="use" onClick={(e) => { e.stopPropagation(); handleUseTemplate(template); }} />
        </Tooltip>,
        <Tooltip title="复制模板">
          <CopyOutlined key="copy" onClick={(e) => { e.stopPropagation(); handleCopyTemplate(template); }} />
        </Tooltip>,
      ]}
      onClick={() => handleTemplateSelect(template)}
    >
      <div style={{ height: 200 }}>
        <div style={{ marginBottom: 8 }}>
          <Text strong style={{ fontSize: 14 }}>{template.name}</Text>
          <div style={{ float: 'right' }}>
            <Badge count={template.usage} showZero title="使用次数" />
          </div>
        </div>

        <Paragraph
          ellipsis={{ rows: 2 }}
          style={{ fontSize: 12, color: '#666', marginBottom: 8 }}
        >
          {template.description}
        </Paragraph>

        <div style={{ marginBottom: 8 }}>
          <Space wrap size="small">
            <Tag color="blue" size="small">{template.sourceType}</Tag>
            <span>→</span>
            <Tag color="green" size="small">{template.targetType}</Tag>
          </Space>
        </div>

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

        <div style={{ fontSize: 11, color: '#999' }}>
          <Space split={<span>•</span>}>
            <span>{template.sourceFields.length} 字段</span>
            <span>{template.mappings.length} 映射</span>
            <span>热度 {template.popularity}%</span>
          </Space>
        </div>
      </div>
    </Card>
  );

  // 渲染模板详情
  const renderTemplateDetail = (template: MappingTemplate) => (
    <div>
      <Title level={4}>{template.name}</Title>
      <Paragraph>{template.description}</Paragraph>

      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={8}>
          <Text strong>分类: </Text>
          <Tag color="blue">{categories.find(c => c.key === template.category)?.name}</Tag>
        </Col>
        <Col span={8}>
          <Text strong>数据类型: </Text>
          <Tag color="green">{template.sourceType} → {template.targetType}</Tag>
        </Col>
        <Col span={8}>
          <Text strong>热度: </Text>
          <Text>{template.popularity}%</Text>
        </Col>
      </Row>

      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={12}>
          <Text strong>源字段 ({template.sourceFields.length}):</Text>
          <div style={{ marginTop: 8, maxHeight: 200, overflowY: 'auto' }}>
            {template.sourceFields.map(field => (
              <div key={field.path} style={{ marginBottom: 4, fontSize: 12 }}>
                <Tag color="blue" size="small">{field.type}</Tag>
                <Text>{field.name}</Text>
                <Text type="secondary" style={{ marginLeft: 8 }}>{field.path}</Text>
                {field.required && <Tag color="red" size="small" style={{ marginLeft: 4 }}>必填</Tag>}
              </div>
            ))}
          </div>
        </Col>
        <Col span={12}>
          <Text strong>目标字段 ({template.targetFields.length}):</Text>
          <div style={{ marginTop: 8, maxHeight: 200, overflowY: 'auto' }}>
            {template.targetFields.map(field => (
              <div key={field.path} style={{ marginBottom: 4, fontSize: 12 }}>
                <Tag color="green" size="small">{field.type}</Tag>
                <Text>{field.name}</Text>
                <Text type="secondary" style={{ marginLeft: 8 }}>{field.path}</Text>
                {field.required && <Tag color="red" size="small" style={{ marginLeft: 4 }}>必填</Tag>}
              </div>
            ))}
          </div>
        </Col>
      </Row>

      <div style={{ marginBottom: 16 }}>
        <Text strong>映射规则 ({template.mappings.length}):</Text>
        <div style={{ marginTop: 8, maxHeight: 200, overflowY: 'auto' }}>
          {template.mappings.map((mapping, index) => (
            <div key={index} style={{ marginBottom: 4, fontSize: 12 }}>
              <Text code>{mapping.sourceField || '(生成)'}</Text>
              <span style={{ margin: '0 8px' }}>→</span>
              <Text code>{mapping.targetField}</Text>
              <Tag color="orange" size="small" style={{ marginLeft: 8 }}>
                {mapping.transformType}
              </Tag>
              {mapping.transformFunction && (
                <Tag size="small">{mapping.transformFunction}</Tag>
              )}
            </div>
          ))}
        </div>
      </div>

      <div style={{ marginBottom: 16 }}>
        <Text strong>标签: </Text>
        <Space wrap size="small">
          {template.tags.map(tag => (
            <Tag key={tag}>{tag}</Tag>
          ))}
        </Space>
      </div>

      <Alert
        message={`此模板已被使用 ${template.usage} 次`}
        type="info"
        showIcon
        style={{ marginBottom: 16 }}
      />
    </div>
  );

  return (
    <div className="mapping-templates" style={{ height }}>
      {/* 搜索和筛选 */}
      <div style={{ marginBottom: 16 }}>
        <Row gutter={16} align="middle">
          <Col span={12}>
            <Search
              placeholder="搜索模板名称、描述或标签"
              allowClear
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              prefix={<SearchOutlined />}
            />
          </Col>
          <Col span={12} style={{ textAlign: 'right' }}>
            <Space>
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={() => setCreateModalVisible(true)}
                disabled={readOnly}
              >
                创建模板
              </Button>
            </Space>
          </Col>
        </Row>
      </div>

      {/* 类别选择 */}
      <Tabs
        activeKey={activeCategory}
        onChange={setActiveCategory}
        size="small"
        style={{ marginBottom: 16 }}
      >
        {categories.map(category => (
          <TabPane
            tab={
              <span>
                {category.icon}
                {category.name}
              </span>
            }
            key={category.key}
          />
        ))}
      </Tabs>

      {/* 模板列表 */}
      <div style={{ height: 'calc(100% - 120px)', overflowY: 'auto' }}>
        {filteredTemplates.length > 0 ? (
          <Row gutter={[16, 16]}>
            {filteredTemplates.map(template => (
              <Col span={8} key={template.id}>
                {renderTemplateCard(template)}
              </Col>
            ))}
          </Row>
        ) : (
          <Empty
            description="没有找到匹配的模板"
            image={Empty.PRESENTED_IMAGE_SIMPLE}
          />
        )}
      </div>

      {/* 模板详情模态框 */}
      <Modal
        title="模板详情"
        open={templateModalVisible}
        onCancel={() => setTemplateModalVisible(false)}
        width={800}
        footer={[
          <Button key="close" onClick={() => setTemplateModalVisible(false)}>
            关闭
          </Button>,
          <Button
            key="copy"
            icon={<CopyOutlined />}
            onClick={() => selectedTemplate && handleCopyTemplate(selectedTemplate)}
          >
            复制模板
          </Button>,
          <Button
            key="use"
            type="primary"
            icon={<ThunderboltOutlined />}
            onClick={() => selectedTemplate && handleUseTemplate(selectedTemplate)}
          >
            使用模板
          </Button>,
        ]}
      >
        {selectedTemplate && renderTemplateDetail(selectedTemplate)}
      </Modal>

      {/* 创建模板模态框 */}
      <Modal
        title="创建新模板"
        open={createModalVisible}
        onCancel={() => setCreateModalVisible(false)}
        onOk={handleCreateTemplate}
        width={600}
      >
        <Form
          form={form}
          layout="vertical"
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
            <Input.TextArea rows={3} placeholder="描述模板的用途和特点" />
          </Form.Item>

          <Row gutter={16}>
            <Col span={12}>
              <Form.Item
                label="分类"
                name="category"
                rules={[{ required: true, message: '请选择模板分类' }]}
              >
                <Select placeholder="选择分类">
                  {categories.filter(c => c.key !== 'all').map(category => (
                    <Option key={category.key} value={category.key}>
                      {category.name}
                    </Option>
                  ))}
                </Select>
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item
                label="源数据类型"
                name="sourceType"
                rules={[{ required: true, message: '请选择源数据类型' }]}
              >
                <Select placeholder="选择数据类型">
                  <Option value="json">JSON</Option>
                  <Option value="csv">CSV</Option>
                  <Option value="xml">XML</Option>
                  <Option value="database">数据库</Option>
                  <Option value="api">API</Option>
                </Select>
              </Form.Item>
            </Col>
          </Row>

          <Form.Item
            label="目标数据类型"
            name="targetType"
            rules={[{ required: true, message: '请选择目标数据类型' }]}
          >
            <Select placeholder="选择数据类型">
              <Option value="json">JSON</Option>
              <Option value="csv">CSV</Option>
              <Option value="xml">XML</Option>
              <Option value="database">数据库</Option>
              <Option value="api">API</Option>
            </Select>
          </Form.Item>

          <Form.Item
            label="标签"
            name="tags"
          >
            <Select
              mode="tags"
              placeholder="输入标签，按回车添加"
              style={{ width: '100%' }}
            />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default MappingTemplates;