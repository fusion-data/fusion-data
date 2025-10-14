import React, { useState, useCallback } from 'react';
import {
  Card,
  Form,
  Input,
  Select,
  Button,
  Space,
  Typography,
  Upload,
  Row,
  Col,
  Alert,
  Steps,
  Table,
  Tag,
  Progress,
  Switch,
  InputNumber,
  Tabs,
  Divider,
  Tooltip,
  Badge,
  Empty,
  message,
} from 'antd';
import {
  UploadOutlined,
  DatabaseOutlined,
  FileTextOutlined,
  ApiOutlined,
  LinkOutlined,
  EyeOutlined,
  DownloadOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  InfoCircleOutlined,
  PlayCircleOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/cjs/styles/prism';

const { Text, Title, Paragraph } = Typography;
const { TextArea } = Input;
const { Option } = Select;
const { Step } = Steps;
const { TabPane } = Tabs;
const { Dragger } = Upload;

// 数据源类型
type DataSourceType = 'file' | 'database' | 'api' | 'url';

// 连接状态
type ConnectionStatus = 'idle' | 'connecting' | 'connected' | 'error';

// 数据字段接口
interface DataField {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  path: string;
  description?: string;
  required?: boolean;
  example?: any;
  sampleCount?: number;
}

// 数据预览接口
interface DataPreview {
  headers: string[];
  rows: any[][];
  totalRows: number;
  sampleSize: number;
  fields: DataField[];
}

// 连接配置接口
interface ConnectionConfig {
  type: DataSourceType;
  name: string;
  description?: string;
  config: {
    // 文件配置
    fileType?: 'json' | 'csv' | 'xml' | 'excel';
    encoding?: string;
    delimiter?: string;
    hasHeader?: boolean;
    sheetName?: string;

    // 数据库配置
    host?: string;
    port?: number;
    database?: string;
    username?: string;
    password?: string;
    table?: string;
    query?: string;

    // API配置
    url?: string;
    method?: 'GET' | 'POST';
    headers?: Record<string, string>;
    authType?: 'none' | 'bearer' | 'basic';
    authToken?: string;

    // URL配置
    urlAddress?: string;
    refreshInterval?: number;
  };
}

interface DataConnectorProps {
  onConnect?: (config: ConnectionConfig, preview: DataPreview) => void;
  onFieldsChange?: (fields: DataField[]) => void;
  readOnly?: boolean;
  height?: number;
}

export const DataConnector: React.FC<DataConnectorProps> = ({
  onConnect,
  onFieldsChange,
  readOnly = false,
  height = 600,
}) => {
  const [form] = Form.useForm();
  const [currentStep, setCurrentStep] = useState(0);
  const [dataSourceType, setDataSourceType] = useState<DataSourceType>('file');
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('idle');
  const [previewData, setPreviewData] = useState<DataPreview | null>(null);
  const [connectionConfig, setConnectionConfig] = useState<ConnectionConfig | null>(null);

  // 重置状态
  const resetState = useCallback(() => {
    setCurrentStep(0);
    setConnectionStatus('idle');
    setPreviewData(null);
    setConnectionConfig(null);
    form.resetFields();
  }, [form]);

  // 步骤变化处理
  const handleStepChange = useCallback(
    (step: number) => {
      if (step < currentStep) {
        setCurrentStep(step);
      }
    },
    [currentStep]
  );

  // 数据源类型选择
  const handleDataSourceTypeSelect = useCallback((type: DataSourceType) => {
    setDataSourceType(type);
    setCurrentStep(1);
    setConnectionStatus('idle');
    setPreviewData(null);
  }, []);

  // 测试连接
  const handleTestConnection = useCallback(async () => {
    try {
      const values = await form.validateFields();
      setConnectionStatus('connecting');

      // 构建连接配置
      const config: ConnectionConfig = {
        type: dataSourceType,
        name: values.name,
        description: values.description,
        config: values.config || {},
      };

      // 模拟连接测试
      await new Promise(resolve => setTimeout(resolve, 1500));

      // 生成模拟预览数据
      const mockPreview = generateMockPreview(dataSourceType, config);
      setPreviewData(mockPreview);
      setConnectionConfig(config);
      setConnectionStatus('connected');

      // 回调
      if (onConnect) {
        onConnect(config, mockPreview);
      }
      if (onFieldsChange) {
        onFieldsChange(mockPreview.fields);
      }

      setCurrentStep(2);
      message.success('连接成功！');
    } catch (error: any) {
      setConnectionStatus('error');
      message.error('连接失败: ' + (error?.message || '未知错误'));
    }
  }, [form, dataSourceType, onConnect, onFieldsChange]);

  // 生成模拟预览数据
  const generateMockPreview = useCallback((type: DataSourceType, config: ConnectionConfig): DataPreview => {
    const sampleData = generateSampleData(type, config);
    const fields = extractFieldsFromData(sampleData);

    return {
      headers: fields.map(f => f.name),
      rows: sampleData.slice(0, 5).map(row => fields.map(f => getNestedValue(row, f.path))),
      totalRows: sampleData.length,
      sampleSize: 5,
      fields,
    };
  }, []);

  // 生成示例数据
  const generateSampleData = useCallback((type: DataSourceType, config: ConnectionConfig): any[] => {
    const baseData = [
      {
        id: '1',
        name: '张三',
        email: 'zhangsan@example.com',
        age: 25,
        department: '技术部',
        salary: 8000,
        active: true,
        joinDate: '2023-01-15',
        skills: ['JavaScript', 'React', 'Node.js'],
        profile: {
          title: '前端工程师',
          level: '中级',
          location: '北京',
        },
      },
      {
        id: '2',
        name: '李四',
        email: 'lisi@example.com',
        age: 30,
        department: '产品部',
        salary: 12000,
        active: true,
        joinDate: '2022-06-20',
        skills: ['产品设计', '用户研究', '项目管理'],
        profile: {
          title: '产品经理',
          level: '高级',
          location: '上海',
        },
      },
      {
        id: '3',
        name: '王五',
        email: 'wangwu@example.com',
        age: 28,
        department: '设计部',
        salary: 9000,
        active: false,
        joinDate: '2023-03-10',
        skills: ['UI设计', 'UX设计', 'Figma'],
        profile: {
          title: 'UI设计师',
          level: '中级',
          location: '深圳',
        },
      },
      {
        id: '4',
        name: '赵六',
        email: 'zhaoliu@example.com',
        age: 32,
        department: '技术部',
        salary: 15000,
        active: true,
        joinDate: '2021-11-05',
        skills: ['Java', 'Spring', 'MySQL', 'Redis'],
        profile: {
          title: '后端工程师',
          level: '高级',
          location: '杭州',
        },
      },
      {
        id: '5',
        name: '孙七',
        email: 'sunqi@example.com',
        age: 26,
        department: '市场部',
        salary: 7000,
        active: true,
        joinDate: '2023-07-01',
        skills: ['市场营销', '内容策划', '数据分析'],
        profile: {
          title: '市场专员',
          level: '初级',
          location: '广州',
        },
      },
    ];

    // 根据数据源类型调整数据格式
    switch (type) {
      case 'csv':
        return baseData.map(item => ({
          ...item,
          skills: Array.isArray(item.skills) ? item.skills.join(';') : item.skills,
        }));
      case 'xml':
        return baseData;
      case 'api':
        return baseData;
      case 'database':
        return baseData;
      default:
        return baseData;
    }
  }, []);

  // 从数据中提取字段信息
  const extractFieldsFromData = useCallback((data: any[]): DataField[] => {
    if (!data || data.length === 0) return [];

    const firstItem = data[0];
    const fields: DataField[] = [];

    const extractFields = (obj: any, prefix: string = '') => {
      Object.entries(obj).forEach(([key, value]) => {
        const path = prefix ? `${prefix}.${key}` : key;
        const type = getValueType(value);

        fields.push({
          name: key,
          type,
          path,
          example: value,
          sampleCount: data.filter(item => getNestedValue(item, path) !== undefined).length,
        });

        // 如果是对象或数组，递归提取子字段
        if (type === 'object' && value !== null && typeof value === 'object') {
          extractFields(value, path);
        }
      });
    };

    extractFields(firstItem);
    return fields;
  }, []);

  // 获取值的类型
  const getValueType = (value: any): 'string' | 'number' | 'boolean' | 'object' | 'array' => {
    if (Array.isArray(value)) return 'array';
    if (typeof value === 'boolean') return 'boolean';
    if (typeof value === 'number') return 'number';
    if (typeof value === 'string') return 'string';
    if (typeof value === 'object' && value !== null) return 'object';
    return 'string';
  };

  // 获取嵌套值
  const getNestedValue = (obj: any, path: string): any => {
    return path.split('.').reduce((current, key) => current?.[key], obj);
  };

  // 渲染数据源类型选择
  const renderDataSourceTypeSelection = () => (
    <div>
      <Title level={4}>选择数据源类型</Title>
      <Paragraph>请选择您要连接的数据源类型：</Paragraph>

      <Row gutter={[16, 16]}>
        <Col span={6}>
          <Card
            hoverable
            onClick={() => handleDataSourceTypeSelect('file')}
            style={{
              textAlign: 'center',
              cursor: 'pointer',
              border: dataSourceType === 'file' ? '2px solid #1890ff' : undefined,
            }}
          >
            <UploadOutlined style={{ fontSize: 32, color: '#1890ff' }} />
            <div style={{ marginTop: 8 }}>
              <Text strong>文件上传</Text>
              <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>JSON, CSV, XML, Excel</div>
            </div>
          </Card>
        </Col>

        <Col span={6}>
          <Card
            hoverable
            onClick={() => handleDataSourceTypeSelect('database')}
            style={{
              textAlign: 'center',
              cursor: 'pointer',
              border: dataSourceType === 'database' ? '2px solid #1890ff' : undefined,
            }}
          >
            <DatabaseOutlined style={{ fontSize: 32, color: '#52c41a' }} />
            <div style={{ marginTop: 8 }}>
              <Text strong>数据库</Text>
              <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>MySQL, PostgreSQL, SQLite</div>
            </div>
          </Card>
        </Col>

        <Col span={6}>
          <Card
            hoverable
            onClick={() => handleDataSourceTypeSelect('api')}
            style={{
              textAlign: 'center',
              cursor: 'pointer',
              border: dataSourceType === 'api' ? '2px solid #1890ff' : undefined,
            }}
          >
            <ApiOutlined style={{ fontSize: 32, color: '#fa8c16' }} />
            <div style={{ marginTop: 8 }}>
              <Text strong>API 接口</Text>
              <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>REST API, GraphQL</div>
            </div>
          </Card>
        </Col>

        <Col span={6}>
          <Card
            hoverable
            onClick={() => handleDataSourceTypeSelect('url')}
            style={{
              textAlign: 'center',
              cursor: 'pointer',
              border: dataSourceType === 'url' ? '2px solid #1890ff' : undefined,
            }}
          >
            <LinkOutlined style={{ fontSize: 32, color: '#722ed1' }} />
            <div style={{ marginTop: 8 }}>
              <Text strong>URL 链接</Text>
              <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>公开数据源, RSS</div>
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  );

  // 渲染连接配置表单
  const renderConnectionConfig = () => (
    <div>
      <Title level={4}>配置数据源连接</Title>

      <Form
        form={form}
        layout="vertical"
        initialValues={{
          name: `${dataSourceType}_data_${Date.now()}`,
          config: getDefaultConfig(dataSourceType),
        }}
      >
        <Form.Item label="数据源名称" name="name" rules={[{ required: true, message: '请输入数据源名称' }]}>
          <Input placeholder="输入数据源名称" disabled={readOnly} />
        </Form.Item>

        <Form.Item label="描述" name="description">
          <TextArea rows={2} placeholder="输入数据源描述（可选）" disabled={readOnly} />
        </Form.Item>

        {renderDataSourceConfig()}

        <div style={{ marginTop: 24 }}>
          <Space>
            <Button
              type="primary"
              icon={<PlayCircleOutlined />}
              onClick={handleTestConnection}
              loading={connectionStatus === 'connecting'}
              disabled={readOnly}
            >
              {connectionStatus === 'connecting' ? '连接中...' : '测试连接'}
            </Button>

            <Button onClick={resetState} disabled={connectionStatus === 'connecting'}>
              重新选择
            </Button>
          </Space>

          {connectionStatus === 'connected' && (
            <Alert
              message="连接成功"
              description="数据源连接测试成功，可以预览数据了"
              type="success"
              showIcon
              style={{ marginTop: 12 }}
            />
          )}

          {connectionStatus === 'error' && (
            <Alert
              message="连接失败"
              description="请检查配置信息是否正确"
              type="error"
              showIcon
              style={{ marginTop: 12 }}
            />
          )}
        </div>
      </Form>
    </div>
  );

  // 渲染数据源特定配置
  const renderDataSourceConfig = () => {
    switch (dataSourceType) {
      case 'file':
        return (
          <>
            <Form.Item label={['config', 'fileType']} name={['config', 'fileType']}>
              <Select placeholder="选择文件类型">
                <Option value="json">JSON</Option>
                <Option value="csv">CSV</Option>
                <Option value="xml">XML</Option>
                <Option value="excel">Excel</Option>
              </Select>
            </Form.Item>

            <Form.Item label={['config', 'encoding']} name={['config', 'encoding']}>
              <Select placeholder="选择文件编码" defaultValue="utf-8">
                <Option value="utf-8">UTF-8</Option>
                <Option value="gbk">GBK</Option>
                <Option value="gb2312">GB2312</Option>
              </Select>
            </Form.Item>

            <Form.Item label={['config', 'hasHeader']} name={['config', 'hasHeader']} valuePropName="checked">
              <Switch checkedChildren="包含表头" unCheckedChildren="无表头" />
            </Form.Item>

            <Form.Item label={['config', 'delimiter']} name={['config', 'delimiter']} tooltip="CSV文件分隔符">
              <Input placeholder="默认为逗号" />
            </Form.Item>

            <Form.Item label="文件上传">
              <Dragger
                name="file"
                multiple={false}
                disabled={readOnly}
                beforeUpload={() => false}
                showUploadList={false}
              >
                <p className="ant-upload-drag-icon">
                  <UploadOutlined />
                </p>
                <p className="ant-upload-text">点击或拖拽文件到此区域上传</p>
                <p className="ant-upload-hint">支持单个文件上传，文件大小不超过 10MB</p>
              </Dragger>
            </Form.Item>
          </>
        );

      case 'database':
        return (
          <>
            <Row gutter={16}>
              <Col span={12}>
                <Form.Item label={['config', 'host']} name={['config', 'host']}>
                  <Input placeholder="数据库主机地址" />
                </Form.Item>
              </Col>
              <Col span={12}>
                <Form.Item label={['config', 'port']} name={['config', 'port']}>
                  <InputNumber placeholder="端口号" style={{ width: '100%' }} />
                </Form.Item>
              </Col>
            </Row>

            <Row gutter={16}>
              <Col span={12}>
                <Form.Item label={['config', 'database']} name={['config', 'database']}>
                  <Input placeholder="数据库名称" />
                </Form.Item>
              </Col>
              <Col span={12}>
                <Form.Item label={['config', 'table']} name={['config', 'table']}>
                  <Input placeholder="表名称" />
                </Form.Item>
              </Col>
            </Row>

            <Row gutter={16}>
              <Col span={12}>
                <Form.Item label={['config', 'username']} name={['config', 'username']}>
                  <Input placeholder="用户名" />
                </Form.Item>
              </Col>
              <Col span={12}>
                <Form.Item label={['config', 'password']} name={['config', 'password']}>
                  <Input.Password placeholder="密码" />
                </Form.Item>
              </Col>
            </Row>

            <Form.Item label={['config', 'query']} name={['config', 'query']}>
              <TextArea
                rows={4}
                placeholder="输入自定义 SQL 查询（可选）"
                style={{ fontFamily: 'Monaco, Consolas, monospace' }}
              />
            </Form.Item>
          </>
        );

      case 'api':
        return (
          <>
            <Form.Item label={['config', 'url']} name={['config', 'url']}>
              <Input placeholder="API 端点 URL" />
            </Form.Item>

            <Row gutter={16}>
              <Col span={8}>
                <Form.Item label={['config', 'method']} name={['config', 'method']}>
                  <Select placeholder="请求方法">
                    <Option value="GET">GET</Option>
                    <Option value="POST">POST</Option>
                  </Select>
                </Form.Item>
              </Col>
              <Col span={8}>
                <Form.Item label={['config', 'authType']} name={['config', 'authType']}>
                  <Select placeholder="认证类型">
                    <Option value="none">无认证</Option>
                    <Option value="bearer">Bearer Token</Option>
                    <Option value="basic">Basic Auth</Option>
                  </Select>
                </Form.Item>
              </Col>
              <Col span={8}>
                <Form.Item label={['config', 'authToken']} name={['config', 'authToken']}>
                  <Input placeholder="认证令牌" />
                </Form.Item>
              </Col>
            </Row>

            <Form.Item label={['config', 'headers']} name={['config', 'headers']}>
              <TextArea
                rows={4}
                placeholder='输入请求头，JSON 格式，例如：{"Content-Type": "application/json"}'
                style={{ fontFamily: 'Monaco, Consolas, monospace' }}
              />
            </Form.Item>
          </>
        );

      case 'url':
        return (
          <>
            <Form.Item label={['config', 'urlAddress']} name={['config', 'urlAddress']}>
              <Input placeholder="数据源 URL" />
            </Form.Item>

            <Form.Item label={['config', 'refreshInterval']} name={['config', 'refreshInterval']}>
              <InputNumber placeholder="刷新间隔（秒）" min={1} style={{ width: '100%' }} addonAfter="秒" />
            </Form.Item>
          </>
        );

      default:
        return null;
    }
  };

  // 获取默认配置
  const getDefaultConfig = (type: DataSourceType) => {
    switch (type) {
      case 'file':
        return {
          fileType: 'json',
          encoding: 'utf-8',
          hasHeader: true,
          delimiter: ',',
        };
      case 'database':
        return {
          host: 'localhost',
          port: 3306,
          database: '',
          table: '',
          username: '',
          password: '',
          query: '',
        };
      case 'api':
        return {
          url: '',
          method: 'GET',
          authType: 'none',
          authToken: '',
          headers: {},
        };
      case 'url':
        return {
          urlAddress: '',
          refreshInterval: 300,
        };
      default:
        return {};
    }
  };

  // 渲染数据预览
  const renderDataPreview = () => {
    if (!previewData) {
      return <Empty description="暂无预览数据" />;
    }

    const columns = previewData.fields.map(field => ({
      title: (
        <div>
          <Text strong>{field.name}</Text>
          <div style={{ fontSize: 11, color: '#666' }}>
            <Tag color="blue" style={{ fontSize: 10 }}>
              {field.type}
            </Tag>
            <span>
              {field.sampleCount}/{previewData.totalRows}
            </span>
          </div>
        </div>
      ),
      dataIndex: field.name,
      key: field.path,
      ellipsis: true,
      render: (value: any) => {
        if (value === null || value === undefined) {
          return <Text type="secondary">-</Text>;
        }
        if (typeof value === 'object') {
          return (
            <Tooltip title={JSON.stringify(value)}>
              <Text>Object</Text>
            </Tooltip>
          );
        }
        return <Text>{String(value)}</Text>;
      },
    }));

    const dataSource = previewData.rows.map((row, index) => {
      const item: any = { key: index };
      previewData.fields.forEach((field, fieldIndex) => {
        item[field.name] = row[fieldIndex];
      });
      return item;
    });

    return (
      <div>
        <div style={{ marginBottom: 16 }}>
          <Space>
            <Badge count={previewData.totalRows} showZero>
              <Text strong>总记录数</Text>
            </Badge>
            <Text type="secondary">|</Text>
            <Badge count={previewData.fields.length} showZero>
              <Text strong>字段数</Text>
            </Badge>
            <Text type="secondary">|</Text>
            <Text>预览前 {previewData.sampleSize} 条记录</Text>
          </Space>
        </div>

        <Table
          dataSource={dataSource}
          columns={columns}
          pagination={false}
          scroll={{ x: 'max-content', y: 300 }}
          size="small"
          bordered
        />

        <div style={{ marginTop: 16 }}>
          <Space>
            <Button icon={<EyeOutlined />} size="small">
              查看完整数据
            </Button>
            <Button icon={<DownloadOutlined />} size="small">
              导出数据
            </Button>
            <Button icon={<ReloadOutlined />} size="small" onClick={handleTestConnection}>
              刷新预览
            </Button>
          </Space>
        </div>
      </div>
    );
  };

  // 渲染字段信息
  const renderFieldInfo = () => {
    if (!previewData) {
      return <Empty description="暂无字段信息" />;
    }

    return (
      <div>
        <Title level={4}>字段信息</Title>
        <Table
          dataSource={previewData.fields}
          columns={[
            {
              title: '字段名',
              dataIndex: 'name',
              key: 'name',
              render: (name, record: DataField) => (
                <div>
                  <Text strong>{name}</Text>
                  <div style={{ fontSize: 11, color: '#666' }}>{record.path}</div>
                </div>
              ),
            },
            {
              title: '类型',
              dataIndex: 'type',
              key: 'type',
              render: (type: string) => <Tag color="blue">{type}</Tag>,
            },
            {
              title: '示例',
              dataIndex: 'example',
              key: 'example',
              render: (example: any) => {
                if (example === null || example === undefined) {
                  return <Text type="secondary">-</Text>;
                }
                if (typeof example === 'object') {
                  return (
                    <Tooltip title={JSON.stringify(example)}>
                      <Text>Object</Text>
                    </Tooltip>
                  );
                }
                return (
                  <Text ellipsis style={{ maxWidth: 200 }}>
                    {String(example)}
                  </Text>
                );
              },
            },
            {
              title: '非空率',
              dataIndex: 'sampleCount',
              key: 'sampleCount',
              render: (count: number, record: DataField) => {
                const rate = ((count / previewData.totalRows) * 100).toFixed(1);
                return (
                  <div>
                    <Progress percent={parseFloat(rate)} size="small" style={{ width: 80 }} format={() => `${rate}%`} />
                    <div style={{ fontSize: 11, color: '#666' }}>
                      {count}/{previewData.totalRows}
                    </div>
                  </div>
                );
              },
            },
          ]}
          pagination={false}
          size="small"
        />
      </div>
    );
  };

  // 步骤配置
  const steps = [
    {
      title: '选择类型',
      description: '选择数据源类型',
    },
    {
      title: '配置连接',
      description: '配置数据源参数',
    },
    {
      title: '预览数据',
      description: '预览和确认数据',
    },
  ];

  return (
    <div className="data-connector" style={{ height }}>
      <Steps current={currentStep} onChange={handleStepChange} items={steps} style={{ marginBottom: 24 }} />

      {currentStep === 0 && renderDataSourceTypeSelection()}

      {currentStep === 1 && renderConnectionConfig()}

      {currentStep === 2 && (
        <div>
          <Tabs defaultActiveKey="preview" size="small">
            <TabPane tab="数据预览" key="preview">
              {renderDataPreview()}
            </TabPane>
            <TabPane tab="字段信息" key="fields">
              {renderFieldInfo()}
            </TabPane>
            <TabPane tab="原始数据" key="raw">
              <div style={{ height: 400, overflow: 'auto' }}>
                <SyntaxHighlighter
                  language="json"
                  style={oneDark}
                  customStyle={{ margin: 0, borderRadius: 4 }}
                  PreTag="div"
                >
                  {JSON.stringify(previewData?.rows.slice(0, 3) || [], null, 2)}
                </SyntaxHighlighter>
              </div>
            </TabPane>
          </Tabs>

          <div style={{ marginTop: 24, paddingTop: 16, borderTop: '1px solid #f0f0f0' }}>
            <Space>
              <Button type="primary" icon={<CheckCircleOutlined />}>
                使用此数据源
              </Button>
              <Button icon={<ReloadOutlined />} onClick={handleTestConnection}>
                重新加载
              </Button>
              <Button onClick={resetState}>重新配置</Button>
            </Space>
          </div>
        </div>
      )}
    </div>
  );
};

export default DataConnector;
