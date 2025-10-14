import React, { useState, useCallback, useMemo } from 'react';
import {
  Card,
  Row,
  Col,
  Typography,
  Button,
  Space,
  Alert,
  Tag,
  Empty,
  Spin,
  Table,
  Tabs,
} from 'antd';
import {
  EyeOutlined,
  EyeInvisibleOutlined,
  PlayCircleOutlined,
  ReloadOutlined,
} from '@ant-design/icons';

import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/cjs/styles/prism';
import type { DataMappingConfig, MappingRule } from './DataMappingCanvas';

const { Text, Title } = Typography;
const { TabPane } = Tabs;

interface DataMappingPreviewProps {
  config: DataMappingConfig;
  testData?: any;
  onTest?: (config: DataMappingConfig, testData: any) => any;
  readOnly?: boolean;
  height?: number;
}

export const DataMappingPreview: React.FC<DataMappingPreviewProps> = ({
  config,
  testData,
  onTest,
  readOnly = false,
  height = 600,
}) => {
  const [activeTab, setActiveTab] = useState('preview');
  const [showRawData, setShowRawData] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{
    success: boolean;
    data?: any;
    error?: string;
    metrics?: {
      totalFields: number;
      mappedFields: number;
      unmappedFields: number;
      transformationCount: number;
    };
  } | null>(null);

  // 示例测试数据
  const defaultTestData = {
    user: {
      id: 'user_123',
      username: 'john_doe',
      age: 25,
      email: 'john@example.com',
      created_at: '2024-01-15T10:30:00Z',
      profile: {
        bio: 'Software Developer',
        location: 'San Francisco',
        interests: ['coding', 'music', 'travel'],
      },
      orders: [
        { id: 'order_1', amount: 99.99, status: 'completed' },
        { id: 'order_2', amount: 149.99, status: 'pending' },
      ],
    },
  };

  const currentTestData = testData || defaultTestData;

  // 执行数据转换
  const executeMapping = useCallback((data: any, mappingConfig: DataMappingConfig) => {
    const result: any = {};
    let transformationCount = 0;

    // 应用所有映射规则
    mappingConfig.mappings.forEach(mapping => {
      if (!mapping.enabled) return;

      let value = getNestedValue(data, mapping.sourceField);

      // 根据转换类型处理值
      if (value !== undefined && value !== null) {
        try {
          switch (mapping.transformType) {
            case 'direct':
              // 直接映射
              break;

            case 'function':
              // 函数转换
              if (mapping.transformFunction) {
                value = applyTransformFunction(value, mapping.transformFunction);
                transformationCount++;
              }
              break;

            case 'expression':
              // 表达式转换 (这里简化处理)
              if (mapping.expression) {
                value = evaluateExpression(value, mapping.expression);
                transformationCount++;
              }
              break;

            case 'conditional':
              // 条件映射
              if (mapping.condition) {
                const conditionValue = getNestedValue(data, mapping.condition.field);
                const shouldMap = evaluateCondition(
                  conditionValue,
                  mapping.condition.operator,
                  mapping.condition.value
                );
                if (shouldMap) {
                  // 使用默认值或原值
                  value = mapping.defaultValue !== undefined ? mapping.defaultValue : value;
                  transformationCount++;
                } else {
                  value = undefined;
                }
              }
              break;
          }

          // 设置目标字段值
          if (value !== undefined) {
            setNestedValue(result, mapping.targetField, value);
          }
        } catch (error) {
          console.error(`转换字段 ${mapping.targetField} 时出错:`, error);
          value = mapping.defaultValue !== undefined ? mapping.defaultValue : value;
          if (value !== undefined) {
            setNestedValue(result, mapping.targetField, value);
          }
        }
      } else if (mapping.defaultValue !== undefined) {
        // 使用默认值
        setNestedValue(result, mapping.targetField, mapping.defaultValue);
      }
    });

    return {
      data: result,
      metrics: {
        totalFields: mappingConfig.targetFields.length,
        mappedFields: Object.keys(result).length,
        unmappedFields: mappingConfig.targetFields.length - Object.keys(result).length,
        transformationCount,
      },
    };
  }, []);

  // 测试配置
  const handleTest = useCallback(async () => {
    setTesting(true);
    setTestResult(null);

    try {
      await new Promise(resolve => setTimeout(resolve, 1000)); // 模拟处理时间

      const result = executeMapping(currentTestData, config);

      setTestResult({
        success: true,
        data: result.data,
        metrics: result.metrics,
      });

      if (onTest) {
        onTest(config, currentTestData);
      }
    } catch (error: any) {
      setTestResult({
        success: false,
        error: error?.message || '映射执行失败',
      });
    } finally {
      setTesting(false);
    }
  }, [config, currentTestData, onTest]);

  // 计算统计信息
  const stats = useMemo(() => {
    if (!testResult?.metrics) {
      return {
        totalFields: config.targetFields.length,
        mappedFields: 0,
        unmappedFields: config.targetFields.length,
        transformationCount: 0,
        mappingSuccessRate: 0,
      };
    }

    return {
      totalFields: testResult.metrics.totalFields,
      mappedFields: testResult.metrics.mappedFields,
      unmappedFields: testResult.metrics.unmappedFields,
      transformationCount: testResult.metrics.transformationCount,
      mappingSuccessRate: (testResult.metrics.mappedFields / testResult.metrics.totalFields) * 100,
    };
  }, [testResult, config]);

  // 获取嵌套值
  const getNestedValue = (obj: any, path: string): any => {
    return path.split('.').reduce((current, key) => current?.[key], obj);
  };

  // 设置嵌套值
  const setNestedValue = (obj: any, path: string, value: any) => {
    const keys = path.split('.');
    const lastKey = keys.pop()!;
    const target = keys.reduce((current, key) => {
      if (!current[key]) current[key] = {};
      return current[key];
    }, obj);
    target[lastKey] = value;
  };

  // 应用转换函数
  const applyTransformFunction = (value: any, functionName: string): any => {
    switch (functionName) {
      case 'toUpperCase':
        return typeof value === 'string' ? value.toUpperCase() : value;
      case 'toLowerCase':
        return typeof value === 'string' ? value.toLowerCase() : value;
      case 'trim':
        return typeof value === 'string' ? value.trim() : value;
      case 'parseInt':
        return parseInt(value, 10);
      case 'parseFloat':
        return parseFloat(value);
      case 'formatDate':
        return typeof value === 'string' ? new Date(value).toISOString() : value;
      case 'formatCurrency':
        return typeof value === 'number' ? `$${value.toFixed(2)}` : value;
      default:
        return value;
    }
  };

  // 简化的表达式求值
  const evaluateExpression = (value: any, expression: string): any => {
    try {
      // 这里只支持简单的操作，实际项目中可以使用更强大的表达式引擎
      if (expression === 'value * 2') {
        return typeof value === 'number' ? value * 2 : value;
      }
      if (expression === 'value + 1') {
        return typeof value === 'number' ? value + 1 : value;
      }
      return value;
    } catch {
      return value;
    }
  };

  // 简化的条件判断
  const evaluateCondition = (value: any, operator: string, compareValue: any): boolean => {
    switch (operator) {
      case 'equals':
        return value === compareValue;
      case 'not_equals':
        return value !== compareValue;
      case 'contains':
        return typeof value === 'string' && value.includes(compareValue);
      case 'greater_than':
        return typeof value === 'number' && value > compareValue;
      case 'less_than':
        return typeof value === 'number' && value < compareValue;
      default:
        return false;
    }
  };

  // 映射规则表格列
  const mappingColumns = [
    {
      title: '源字段',
      dataIndex: 'sourceField',
      key: 'sourceField',
      render: (fieldId: string) => {
        const field = config.sourceFields.find(f => f.id === fieldId);
        return field ? field.name : fieldId;
      },
    },
    {
      title: '目标字段',
      dataIndex: 'targetField',
      key: 'targetField',
      render: (fieldId: string) => {
        const field = config.targetFields.find(f => f.id === fieldId);
        return field ? field.name : fieldId;
      },
    },
    {
      title: '转换类型',
      dataIndex: 'transformType',
      key: 'transformType',
      render: (type: string) => {
        const typeMap: Record<string, string> = {
          direct: '直接映射',
          function: '函数转换',
          expression: '表达式',
          conditional: '条件映射',
        };
        return typeMap[type] || type;
      },
    },
    {
      title: '转换参数',
      dataIndex: 'transformFunction',
      key: 'transformParams',
      render: (func: string, record: MappingRule) => {
        if (record.transformType === 'function' && func) {
          return func;
        }
        if (record.transformType === 'expression' && record.expression) {
          return record.expression;
        }
        if (record.transformType === 'conditional' && record.condition) {
          return `${record.condition.field} ${record.condition.operator} ${record.condition.value}`;
        }
        return '-';
      },
    },
    {
      title: '状态',
      dataIndex: 'enabled',
      key: 'enabled',
      render: (enabled: boolean) => (
        <Tag color={enabled ? 'green' : 'default'}>
          {enabled ? '启用' : '禁用'}
        </Tag>
      ),
    },
  ];

  return (
    <div style={{ height }} className="data-mapping-preview">
      {/* 预览头部 */}
      <Card size="small" style={{ marginBottom: 16 }}>
        <Row justify="space-between" align="middle">
          <Col>
            <Title level={5} style={{ margin: 0 }}>
              映射预览
            </Title>
          </Col>
          <Col>
            <Space>
              <Button
                icon={<PlayCircleOutlined />}
                onClick={handleTest}
                loading={testing}
                disabled={readOnly}
                size="small"
              >
                测试映射
              </Button>
              <Button
                icon={<ReloadOutlined />}
                onClick={() => setTestResult(null)}
                disabled={readOnly}
                size="small"
              >
                重置
              </Button>
            </Space>
          </Col>
        </Row>

        {/* 统计信息 */}
        {testResult && (
          <Row gutter={16} style={{ marginTop: 12 }}>
            <Col span={6}>
              <Text>映射成功率: </Text>
              <Text strong style={{ marginLeft: 8, color: stats.mappingSuccessRate > 80 ? '#52c41a' : stats.mappingSuccessRate > 60 ? '#faad14' : '#ff4d4f' }}>
                {stats.mappingSuccessRate.toFixed(1)}%
              </Text>
            </Col>
            <Col span={6}>
              <Text>转换次数: </Text>
              <Text strong style={{ marginLeft: 8, color: '#1890ff' }}>
                {stats.transformationCount}
              </Text>
            </Col>
            <Col span={6}>
              <Text>映射字段: </Text>
              <Text strong style={{ marginLeft: 8, color: '#52c41a' }}>
                {stats.mappedFields}/{stats.totalFields}
              </Text>
            </Col>
            <Col span={6}>
              <Text>状态: </Text>
              <Tag
                color={testResult.success ? 'success' : 'error'}
                style={{ marginLeft: 8 }}
              >
                {testResult.success ? '成功' : '失败'}
              </Tag>
            </Col>
          </Row>
        )}
      </Card>

      {/* 预览内容 */}
      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
        size="small"
        type="card"
      >
        <TabPane tab="结构化预览" key="preview">
          <Row gutter={16}>
            <Col span={12}>
              <Card title="源数据" size="small">
                <div style={{ position: 'relative' }}>
                  <Button
                    type="text"
                    icon={showRawData ? <EyeInvisibleOutlined /> : <EyeOutlined />}
                    onClick={() => setShowRawData(!showRawData)}
                    size="small"
                    style={{ position: 'absolute', top: 0, right: 0, zIndex: 1 }}
                  />
                  <div style={{ height: 400, overflow: 'auto' }}>
                    {showRawData ? (
                      <SyntaxHighlighter
                        language="json"
                        style={oneDark}
                        customStyle={{ margin: 0, borderRadius: 4 }}
                        PreTag="div"
                      >
                        {JSON.stringify(currentTestData, null, 2)}
                      </SyntaxHighlighter>
                    ) : (
                      <pre style={{ margin: 0, fontSize: 12, whiteSpace: 'pre-wrap' }}>
                        {JSON.stringify(currentTestData, null, 2)}
                      </pre>
                    )}
                  </div>
                </div>
              </Card>
            </Col>
            <Col span={12}>
              <Card title="映射结果" size="small">
                <Spin spinning={testing}>
                  {testResult ? (
                    <div style={{ height: 400, overflow: 'auto' }}>
                      {testResult.success ? (
                        <div>
                          <Alert
                            message="映射成功"
                            description={`成功映射了 ${testResult.metrics.mappedFields} 个字段，应用 ${testResult.metrics.transformationCount} 个转换`}
                            type="success"
                            showIcon
                            style={{ marginBottom: 12 }}
                          />
                          <div>
                            {testResult.data ? (
                              <pre style={{ margin: 0, fontSize: 12, whiteSpace: 'pre-wrap' }}>
                                {JSON.stringify(testResult.data, null, 2)}
                              </pre>
                            ) : (
                              <Empty description="无映射结果" />
                            )}
                          </div>
                        </div>
                      ) : (
                        <div>
                          <Alert
                            message="映射失败"
                            description={testResult.error || '未知错误'}
                            type="error"
                            showIcon
                          />
                        </div>
                      )}
                    </div>
                  ) : (
                    <div style={{ height: 400, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                      <Empty description="点击测试按钮查看映射结果" />
                    </div>
                  )}
                </Spin>
              </Card>
            </Col>
          </Row>
        </TabPane>

        <TabPane tab="映射规则" key="mappings">
          <Card size="small">
            <Table
              dataSource={config.mappings}
              columns={mappingColumns}
              pagination={false}
              size="small"
              scroll={{ y: 300 }}
              rowKey="id"
            />
          </Card>
        </TabPane>

        <TabPane tab="字段对比" key="comparison">
          <Row gutter={16}>
            <Col span={12}>
              <Card title="源字段" size="small">
                <Table
                  dataSource={config.sourceFields}
                  columns={[
                    { title: '字段名', dataIndex: 'name', key: 'name' },
                    { title: '类型', dataIndex: 'type', key: 'type' },
                    { title: '路径', dataIndex: 'path', key: 'path' },
                    { title: '必填', dataIndex: 'required', key: 'required', render: (req) => req ? '是' : '否' },
                  ]}
                  pagination={false}
                  size="small"
                  scroll={{ y: 300 }}
                />
              </Card>
            </Col>
            <Col span={12}>
              <Card title="目标字段" size="small">
                <Table
                  dataSource={config.targetFields}
                  columns={[
                    { title: '字段名', dataIndex: 'name', key: 'name' },
                    { title: '类型', dataIndex: 'type', key: 'type' },
                    { title: '路径', dataIndex: 'path', key: 'path' },
                    { title: '必填', dataIndex: 'required', key: 'required', render: (req) => req ? '是' : '否' },
                  ]}
                  pagination={false}
                  size="small"
                  scroll={{ y: 300 }}
                />
              </Card>
            </Col>
          </Row>
        </TabPane>
      </Tabs>
    </div>
  );
};

export default DataMappingPreview;