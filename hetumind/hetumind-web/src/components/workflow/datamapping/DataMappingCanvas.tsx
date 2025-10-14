import React, { useState, useCallback, useRef, useEffect } from 'react';
import {
  Card,
  Row,
  Col,
  Button,
  Space,
  Typography,
  Select,
  Input,
  Switch,
  Divider,
  Alert,
  Tooltip,
  Badge,
  Empty,
  Tag,
} from 'antd';
import {
  ArrowRightOutlined,
  PlusOutlined,
  DeleteOutlined,
  SettingOutlined,
  SaveOutlined,
  CloseOutlined,
  CopyOutlined,
  EyeOutlined,
  EyeInvisibleOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons';

const { Text, Title } = Typography;
const { Option } = Select;

// 数据字段接口
interface DataField {
  id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  path: string;
  description?: string;
  required?: boolean;
  example?: any;
}

// 映射规则接口
interface MappingRule {
  id: string;
  sourceField: string;
  targetField: string;
  transformType: 'direct' | 'function' | 'expression' | 'conditional';
  transformFunction?: string;
  expression?: string;
  defaultValue?: any;
  condition?: {
    field: string;
    operator: 'equals' | 'not_equals' | 'contains' | 'greater_than' | 'less_than';
    value: any;
  };
  enabled: boolean;
}

// 数据映射配置接口
interface DataMappingConfig {
  id: string;
  name: string;
  description?: string;
  sourceType: 'json' | 'xml' | 'csv' | 'database' | 'api';
  targetType: 'json' | 'xml' | 'csv' | 'database' | 'api';
  sourceFields: DataField[];
  targetFields: DataField[];
  mappings: MappingRule[];
}

interface DataMappingCanvasProps {
  config?: Partial<DataMappingConfig>;
  onConfigChange?: (config: DataMappingConfig) => void;
  onPreview?: (config: DataMappingConfig) => void;
  readOnly?: boolean;
  height?: number;
}

export const DataMappingCanvas: React.FC<DataMappingCanvasProps> = ({
  config,
  onConfigChange,
  onPreview,
  readOnly = false,
  height = 600,
}) => {
  const [activeTab, setActiveTab] = useState('mapping');
  const [currentConfig, setCurrentConfig] = useState<DataMappingConfig>({
    id: 'mapping_1',
    name: '数据映射配置',
    sourceType: 'json',
    targetType: 'json',
    sourceFields: [],
    targetFields: [],
    mappings: [],
    ...config,
  });

  // 示例数据字段
  const sampleSourceFields: DataField[] = [
    { id: '1', name: '用户ID', type: 'string', path: 'user.id', required: true, example: 'user_123' },
    { id: '2', name: '用户名', type: 'string', path: 'user.username', required: true, example: 'john_doe' },
    { id: '3', name: '年龄', type: 'number', path: 'user.age', required: false, example: 25 },
    { id: '4', name: '邮箱', type: 'string', path: 'user.email', required: true, example: 'john@example.com' },
    { id: '5', name: '注册时间', type: 'string', path: 'user.created_at', required: false, example: '2024-01-15T10:30:00Z' },
    { id: '6', name: '个人资料', type: 'object', path: 'user.profile', required: false },
    { id: '7', name: '订单列表', type: 'array', path: 'user.orders', required: false },
  ];

  const sampleTargetFields: DataField[] = [
    { id: '1', name: 'customer_id', type: 'string', path: 'customer.id', required: true },
    { id: '2', name: 'customer_name', type: 'string', path: 'customer.name', required: true },
    { id: '3', name: 'customer_age', type: 'number', path: 'customer.age', required: false },
    { id: '4', name: 'customer_email', type: 'string', path: 'customer.email', required: true },
    { id: '5', name: 'registration_date', type: 'string', path: 'customer.registration_date', required: false },
    { id: '6', name: 'profile_data', type: 'object', path: 'customer.profile', required: false },
    { id: '7', name: 'order_history', type: 'array', path: 'customer.orders', required: false },
  ];

  // 转换函数选项
  const transformFunctions = [
    'toUpperCase', 'toLowerCase', 'trim', 'parseInt', 'parseFloat',
    'formatDate', 'formatCurrency', 'encrypt', 'decrypt',
    'validateEmail', 'validatePhone', 'generateUUID'
  ];

  // 配置变化处理
  const handleConfigChange = useCallback((updates: Partial<DataMappingConfig>) => {
    const newConfig = { ...currentConfig, ...updates };
    setCurrentConfig(newConfig);
    if (onConfigChange) {
      onConfigChange(newConfig);
    }
  }, [currentConfig, onConfigChange]);

  // 添加映射规则
  const addMapping = useCallback(() => {
    const newMapping: MappingRule = {
      id: `mapping_${Date.now()}`,
      sourceField: currentConfig.sourceFields[0]?.id || '',
      targetField: currentConfig.targetFields[0]?.id || '',
      transformType: 'direct',
      enabled: true,
    };

    handleConfigChange({
      mappings: [...currentConfig.mappings, newMapping],
    });
  }, [currentConfig, handleConfigChange]);

  // 更新映射规则
  const updateMapping = useCallback((id: string, updates: Partial<MappingRule>) => {
    handleConfigChange({
      mappings: currentConfig.mappings.map(m =>
        m.id === id ? { ...m, ...updates } : m
      ),
    });
  }, [currentConfig, handleConfigChange]);

  // 删除映射规则
  const deleteMapping = useCallback((id: string) => {
    handleConfigChange({
      mappings: currentConfig.mappings.filter(m => m.id !== id),
    });
  }, [currentConfig, handleConfigChange]);

  // 复制映射规则
  const duplicateMapping = useCallback((mapping: MappingRule) => {
    const newMapping: MappingRule = {
      ...mapping,
      id: `mapping_${Date.now()}`,
      targetField: `${mapping.targetField}_copy`,
    };
    handleConfigChange({
      mappings: [...currentConfig.mappings, newMapping],
    });
  }, [currentConfig, handleConfigChange]);

  // 渲染映射规则行
  const renderMappingRow = (mapping: MappingRule, index: number) => {
    const sourceField = currentConfig.sourceFields.find(f => f.id === mapping.sourceField);
    const targetField = currentConfig.targetFields.find(f => f.id === mapping.targetField);

    return (
      <Card
        key={mapping.id}
        size="small"
        style={{
          marginBottom: 8,
          border: mapping.enabled ? '1px solid #d9d9d9' : '1px dashed #d9d9d9',
          opacity: mapping.enabled ? 1 : 0.6,
        }}
      >
        <Row gutter={16} align="middle">
          <Col span={2}>
            <Switch
              checked={mapping.enabled}
              onChange={(checked) => updateMapping(mapping.id, { enabled: checked })}
              disabled={readOnly}
              size="small"
            />
          </Col>
          <Col span={4}>
            <Select
              value={mapping.sourceField}
              onChange={(value) => updateMapping(mapping.id, { sourceField: value })}
              disabled={readOnly}
              placeholder="源字段"
              size="small"
              style={{ width: '100%' }}
            >
              {currentConfig.sourceFields.map(field => (
                <Option key={field.id} value={field.id}>
                  {field.name}
                  <Text type="secondary" style={{ marginLeft: 4 }}>
                    ({field.type})
                  </Text>
                </Option>
              ))}
            </Select>
          </Col>
          <Col span={1}>
            <ArrowRightOutlined style={{ textAlign: 'center' }} />
          </Col>
          <Col span={4}>
            <Select
              value={mapping.targetField}
              onChange={(value) => updateMapping(mapping.id, { targetField: value })}
              disabled={readOnly}
              placeholder="目标字段"
              size="small"
              style={{ width: '100%' }}
            >
              {currentConfig.targetFields.map(field => (
                <Option key={field.id} value={field.id}>
                  {field.name}
                  <Text type="secondary" style={{ marginLeft: 4 }}>
                    ({field.type})
                  </Text>
                </Option>
              ))}
            </Select>
          </Col>
          <Col span={4}>
            <Select
              value={mapping.transformType}
              onChange={(value) => updateMapping(mapping.id, { transformType: value })}
              disabled={readOnly}
              size="small"
              style={{ width: '100%' }}
            >
              <Option value="direct">直接映射</Option>
              <Option value="function">函数转换</Option>
              <Option value="expression">表达式</Option>
              <Option value="conditional">条件映射</Option>
            </Select>
          </Col>
          <Col span={5}>
            {mapping.transformType === 'function' && (
              <Select
                value={mapping.transformFunction}
                onChange={(value) => updateMapping(mapping.id, { transformFunction: value })}
                disabled={readOnly}
                placeholder="选择转换函数"
                size="small"
                style={{ width: '100%' }}
              >
                {transformFunctions.map(func => (
                  <Option key={func} value={func}>{func}</Option>
                ))}
              </Select>
            )}
            {mapping.transformType === 'expression' && (
              <Input
                value={mapping.expression}
                onChange={(e) => updateMapping(mapping.id, { expression: e.target.value })}
                placeholder="输入转换表达式"
                disabled={readOnly}
                size="small"
              />
            )}
            {mapping.transformType === 'conditional' && (
              <Space.Compact size="small">
                <Input
                  value={mapping.condition?.field || ''}
                  onChange={(e) => updateMapping(mapping.id, {
                    condition: { ...mapping.condition, field: e.target.value }
                  })}
                  placeholder="条件字段"
                  size="small"
                />
                <Select
                  value={mapping.condition?.operator}
                  onChange={(value) => updateMapping(mapping.id, {
                    condition: { ...mapping.condition, operator: value }
                  })}
                  size="small"
                  style={{ width: 80 }}
                >
                  <Option value="equals">等于</Option>
                  <Option value="not_equals">不等于</Option>
                  <Option value="contains">包含</Option>
                  <Option value="greater_than">大于</Option>
                  <Option value="less_than">小于</Option>
                </Select>
                <Input
                  value={mapping.condition?.value || ''}
                  onChange={(e) => updateMapping(mapping.id, {
                    condition: { ...mapping.condition, value: e.target.value }
                  })}
                  placeholder="条件值"
                  size="small"
                />
              </Space.Compact>
            )}
          </Col>
          <Col span={2}>
            <Space>
              {!readOnly && (
                <>
                  <Tooltip title="复制">
                    <Button
                      type="text"
                      icon={<CopyOutlined />}
                      onClick={() => duplicateMapping(mapping)}
                      size="small"
                    />
                  </Tooltip>
                  <Tooltip title="删除">
                    <Button
                      type="text"
                      icon={<DeleteOutlined />}
                      onClick={() => deleteMapping(mapping.id)}
                      size="small"
                      danger
                    />
                  </Tooltip>
                </>
              )}
            </Space>
          </Col>
        </Row>

        {/* 字段信息提示 */}
        {(sourceField || targetField) && (
          <div style={{ marginTop: 8, fontSize: 12, color: '#666' }}>
            <Space wrap size="small">
              {sourceField && (
                <Tag color="blue">
                  源: {sourceField.path}
                  {sourceField.example && ` (示例: ${JSON.stringify(sourceField.example)})`}
                </Tag>
              )}
              {targetField && (
                <Tag color="green">
                  目标: {targetField.path}
                  {targetField.required && <Tag color="red">必填</Tag>}
                </Tag>
              )}
            </Space>
          </div>
        )}
      </Card>
    );
  };

  return (
    <div style={{ height }} className="data-mapping-canvas">
      {/* 配置头部 */}
      <Card size="small" style={{ marginBottom: 16 }}>
        <Row justify="space-between" align="middle">
          <Col>
            <Title level={5} style={{ margin: 0 }}>
              数据映射配置
            </Title>
          </Col>
          <Col>
            <Space>
              <Button
                type="primary"
                icon={<SaveOutlined />}
                onClick={() => console.log('保存配置:', currentConfig)}
                disabled={readOnly}
                size="small"
              >
                保存配置
              </Button>
              <Button
                icon={<ThunderboltOutlined />}
                onClick={() => onPreview && onPreview(currentConfig)}
                disabled={readOnly}
                size="small"
              >
                预览
              </Button>
            </Space>
          </Col>
        </Row>

        <Row style={{ marginTop: 12 }}>
          <Col span={8}>
            <Space>
              <Text strong>源数据类型:</Text>
              <Select
                value={currentConfig.sourceType}
                onChange={(value) => handleConfigChange({ sourceType: value })}
                disabled={readOnly}
                size="small"
              >
                <Option value="json">JSON</Option>
                <Option value="xml">XML</Option>
                <Option value="csv">CSV</Option>
                <Option value="database">数据库</Option>
                <Option value="api">API</Option>
              </Select>
            </Space>
          </Col>
          <Col span={8}>
            <Space>
              <Text strong>目标数据类型:</Text>
              <Select
                value={currentConfig.targetType}
                onChange={(value) => handleConfigChange({ targetType: value })}
                disabled={readOnly}
                size="small"
              >
                <Option value="json">JSON</Option>
                <Option value="xml">XML</Option>
                <Option value="csv">CSV</Option>
                <Option value="database">数据库</Option>
                <Option value="api">API</Option>
              </Select>
            </Space>
          </Col>
        </Row>
      </Card>

      {/* 字段预览 */}
      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={12}>
          <Card title="源字段" size="small">
            <Alert
              message="字段信息"
              description="这些是从源数据结构中提取的字段，用于映射到目标结构。"
              type="info"
              showIcon
              style={{ marginBottom: 12 }}
            />
            <div style={{ maxHeight: 200, overflowY: 'auto' }}>
              {sampleSourceFields.length > 0 ? (
                sampleSourceFields.map(field => (
                  <div
                    key={field.id}
                    style={{
                      padding: '4px 8px',
                      margin: '4px 0',
                      background: '#f5f5f5',
                      borderRadius: 4,
                      fontSize: 12,
                    }}
                  >
                    <Space>
                      <Text strong>{field.name}</Text>
                      <Tag size="small">{field.type}</Tag>
                      {field.required && <Tag color="red" size="small">必填</Tag>}
                    </Space>
                    <div style={{ fontSize: 11, color: '#666', marginTop: 4 }}>
                      路径: {field.path}
                      {field.example && (
                        <span> | 示例: {JSON.stringify(field.example)}</span>
                      )}
                    </div>
                  </div>
                ))
              ) : (
                <Empty description="暂无字段数据" image={Empty.PRESENTED_IMAGE_SIMPLE} />
              )}
            </div>
          </Card>
        </Col>
        <Col span={12}>
          <Card title="目标字段" size="small">
            <Alert
              message="字段信息"
              description="这些是目标数据结构中需要填充的字段。"
              type="info"
              showIcon
              style={{ marginBottom: 12 }}
            />
            <div style={{ maxHeight: 200, overflowY: 'auto' }}>
              {sampleTargetFields.length > 0 ? (
                sampleTargetFields.map(field => (
                  <div
                    key={field.id}
                    style={{
                      padding: '4px 8px',
                      margin: '4px 0',
                      background: '#f5f5f5',
                      borderRadius: 4,
                      fontSize: 12,
                    }}
                  >
                    <Space>
                      <Text strong>{field.name}</Text>
                      <Tag size="small">{field.type}</Tag>
                      {field.required && <Tag color="red" size="small">必填</Tag>}
                    </Space>
                    <div style={{ fontSize: 11, color: '#666', marginTop: 4 }}>
                      路径: {field.path}
                    </div>
                  </div>
                ))
              ) : (
                <Empty description="暂无字段数据" image={Empty.PRESENTED_IMAGE_SIMPLE} />
              )}
            </div>
          </Card>
        </Col>
      </Row>

      {/* 映射规则列表 */}
      <Card
        title={
          <Space>
            <span>映射规则</span>
            <Badge count={currentConfig.mappings.length} />
            {!readOnly && (
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={addMapping}
                size="small"
              >
                添加映射
              </Button>
            )}
          </Space>
        }
        size="small"
        style={{ marginBottom: 16 }}
      >
        {currentConfig.mappings.length > 0 ? (
          <div style={{ maxHeight: 300, overflowY: 'auto' }}>
            {currentConfig.mappings.map((mapping, index) =>
              renderMappingRow(mapping, index)
            )}
          </div>
        ) : (
          <Empty
            description="暂无映射规则"
            image={Empty.PRESENTED_IMAGE_SIMPLE}
          >
            {!readOnly && (
              <Button type="primary" icon={<PlusOutlined />} onClick={addMapping}>
                添加第一条映射规则
              </Button>
            )}
          </Empty>
        )}
      </Card>

      {/* 配置统计 */}
      <Card title="映射统计" size="small">
        <Row gutter={16}>
          <Col span={6}>
            <Statistic
              title="总映射数"
              value={currentConfig.mappings.length}
              valueStyle={{ color: '#1890ff' }}
            />
          </Col>
          <Col span={6}>
            <Statistic
              title="启用映射"
              value={currentConfig.mappings.filter(m => m.enabled).length}
              valueStyle={{ color: '#52c41a' }}
            />
          </Col>
          <Col span={6}>
            <Statistic
              title="函数转换"
              value={currentConfig.mappings.filter(m => m.transformType === 'function').length}
              valueStyle={{ color: '#722ed1' }}
            />
          </Col>
          <Col span={6}>
            <Statistic
              title="条件映射"
              value={currentConfig.mappings.filter(m => m.transformType === 'conditional').length}
              valueStyle={{ color: '#fa8c16' }}
            />
          </Col>
        </Row>
      </Card>
    </div>
  );
};

export default DataMappingCanvas;