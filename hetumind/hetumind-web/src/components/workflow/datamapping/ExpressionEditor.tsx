import React, { useState, useCallback, useMemo } from 'react';
import {
  Card,
  Form,
  Input,
  Select,
  Button,
  Space,
  Typography,
  Tag,
  Row,
  Col,
  Alert,
  Tooltip,
  Divider,
  Tree,
  AutoComplete,
  Badge,
  Tabs,
} from 'antd';
import {
  CodeOutlined,
  FunctionOutlined,
  PlayCircleOutlined,
  InfoCircleOutlined,
  CopyOutlined,
  BookOutlined,
  BulbOutlined,
  ThunderboltOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/cjs/styles/prism';

const { Text, Title, Paragraph } = Typography;
const { TextArea } = Input;
const { TabPane } = Tabs;
const { Option } = Select;

// 表达式函数接口
interface ExpressionFunction {
  name: string;
  category: 'string' | 'number' | 'date' | 'logic' | 'array' | 'object' | 'custom';
  description: string;
  syntax: string;
  examples: string[];
  parameters: Array<{
    name: string;
    type: string;
    description: string;
    required?: boolean;
  }>;
  returnType: string;
}

// 字段变量接口
interface FieldVariable {
  path: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  description?: string;
  example?: any;
  required?: boolean;
}

// 表达式测试结果接口
interface TestResult {
  success: boolean;
  result?: any;
  error?: string;
  executionTime?: number;
  tokensUsed?: number;
}

interface ExpressionEditorProps {
  value?: string;
  onChange?: (value: string) => void;
  variables?: FieldVariable[];
  onTest?: (expression: string, testData: any) => Promise<TestResult>;
  testData?: any;
  readOnly?: boolean;
  height?: number;
  placeholder?: string;
}

export const ExpressionEditor: React.FC<ExpressionEditorProps> = ({
  value = '',
  onChange,
  variables = [],
  onTest,
  testData = {},
  readOnly = false,
  height = 400,
  placeholder = '输入转换表达式...',
}) => {
  const [form] = Form.useForm();
  const [expression, setExpression] = useState(value);
  const [activeTab, setActiveTab] = useState('editor');
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [selectedFunction, setSelectedFunction] = useState<ExpressionFunction | null>(null);
  const [showExamples, setShowExamples] = useState(true);

  // 内置函数库
  const expressionFunctions: ExpressionFunction[] = [
    // 字符串函数
    {
      name: 'toUpperCase',
      category: 'string',
      description: '将字符串转换为大写',
      syntax: 'toUpperCase(string)',
      examples: ['toUpperCase("hello")', 'toUpperCase(user.name)'],
      parameters: [
        { name: 'str', type: 'string', description: '要转换的字符串', required: true },
      ],
      returnType: 'string',
    },
    {
      name: 'toLowerCase',
      category: 'string',
      description: '将字符串转换为小写',
      syntax: 'toLowerCase(string)',
      examples: ['toLowerCase("HELLO")', 'toLowerCase(user.email)'],
      parameters: [
        { name: 'str', type: 'string', description: '要转换的字符串', required: true },
      ],
      returnType: 'string',
    },
    {
      name: 'trim',
      category: 'string',
      description: '去除字符串两端的空白字符',
      syntax: 'trim(string)',
      examples: ['trim("  hello  ")', 'trim(user.input)'],
      parameters: [
        { name: 'str', type: 'string', description: '要处理的字符串', required: true },
      ],
      returnType: 'string',
    },
    {
      name: 'substring',
      category: 'string',
      description: '提取字符串的子串',
      syntax: 'substring(string, start, length?)',
      examples: ['substring("hello world", 0, 5)', 'substring(user.description, 10)'],
      parameters: [
        { name: 'str', type: 'string', description: '原字符串', required: true },
        { name: 'start', type: 'number', description: '开始位置', required: true },
        { name: 'length', type: 'number', description: '长度', required: false },
      ],
      returnType: 'string',
    },
    {
      name: 'replace',
      category: 'string',
      description: '替换字符串中的内容',
      syntax: 'replace(string, search, replace)',
      examples: ['replace("hello world", "world", "there")', 'replace(user.phone, "-", "")'],
      parameters: [
        { name: 'str', type: 'string', description: '原字符串', required: true },
        { name: 'search', type: 'string', description: '要查找的内容', required: true },
        { name: 'replace', type: 'string', description: '替换内容', required: true },
      ],
      returnType: 'string',
    },
    {
      name: 'split',
      category: 'string',
      description: '将字符串分割成数组',
      syntax: 'split(string, separator)',
      examples: ['split("a,b,c", ",")', 'split(user.tags, " ")'],
      parameters: [
        { name: 'str', type: 'string', description: '要分割的字符串', required: true },
        { name: 'separator', type: 'string', description: '分隔符', required: true },
      ],
      returnType: 'array',
    },
    {
      name: 'join',
      category: 'string',
      description: '将数组合并成字符串',
      syntax: 'join(array, separator)',
      examples: ['join(["a", "b", "c"], ",")', 'join(user.skills, ", ")'],
      parameters: [
        { name: 'array', type: 'array', description: '要合并的数组', required: true },
        { name: 'separator', type: 'string', description: '分隔符', required: true },
      ],
      returnType: 'string',
    },

    // 数字函数
    {
      name: 'parseInt',
      category: 'number',
      description: '将字符串转换为整数',
      syntax: 'parseInt(string)',
      examples: ['parseInt("123")', 'parseInt(user.age)'],
      parameters: [
        { name: 'str', type: 'string', description: '要转换的字符串', required: true },
      ],
      returnType: 'number',
    },
    {
      name: 'parseFloat',
      category: 'number',
      description: '将字符串转换为浮点数',
      syntax: 'parseFloat(string)',
      examples: ['parseFloat("123.45")', 'parseFloat(user.price)'],
      parameters: [
        { name: 'str', type: 'string', description: '要转换的字符串', required: true },
      ],
      returnType: 'number',
    },
    {
      name: 'round',
      category: 'number',
      description: '四舍五入到指定小数位',
      syntax: 'round(number, digits?)',
      examples: ['round(3.14159, 2)', 'round(user.amount, 0)'],
      parameters: [
        { name: 'num', type: 'number', description: '要舍入的数字', required: true },
        { name: 'digits', type: 'number', description: '小数位数', required: false },
      ],
      returnType: 'number',
    },
    {
      name: 'formatCurrency',
      category: 'number',
      description: '格式化货币显示',
      syntax: 'formatCurrency(number, currency?)',
      examples: ['formatCurrency(1234.56)', 'formatCurrency(user.price, "¥")'],
      parameters: [
        { name: 'num', type: 'number', description: '要格式化的数字', required: true },
        { name: 'currency', type: 'string', description: '货币符号', required: false },
      ],
      returnType: 'string',
    },

    // 日期函数
    {
      name: 'formatDate',
      category: 'date',
      description: '格式化日期显示',
      syntax: 'formatDate(date, format?)',
      examples: ['formatDate("2024-01-15")', 'formatDate(user.created_at, "YYYY-MM-DD")'],
      parameters: [
        { name: 'date', type: 'string', description: '日期字符串或对象', required: true },
        { name: 'format', type: 'string', description: '格式模板', required: false },
      ],
      returnType: 'string',
    },
    {
      name: 'now',
      category: 'date',
      description: '获取当前时间',
      syntax: 'now()',
      examples: ['now()', 'formatDate(now(), "YYYY-MM-DD")'],
      parameters: [],
      returnType: 'date',
    },
    {
      name: 'addDays',
      category: 'date',
      description: '日期加减天数',
      syntax: 'addDays(date, days)',
      examples: ['addDays("2024-01-15", 7)', 'addDays(user.expiry, -30)'],
      parameters: [
        { name: 'date', type: 'string', description: '日期字符串', required: true },
        { name: 'days', type: 'number', description: '天数', required: true },
      ],
      returnType: 'date',
    },

    // 逻辑函数
    {
      name: 'if',
      category: 'logic',
      description: '条件判断',
      syntax: 'if(condition, trueValue, falseValue)',
      examples: ['if(user.age > 18, "成年", "未成年")', 'if(item.price > 100, item.price * 0.9, item.price)'],
      parameters: [
        { name: 'condition', type: 'boolean', description: '条件表达式', required: true },
        { name: 'trueValue', type: 'any', description: '条件为真时的值', required: true },
        { name: 'falseValue', type: 'any', description: '条件为假时的值', required: true },
      ],
      returnType: 'any',
    },
    {
      name: 'equals',
      category: 'logic',
      description: '相等比较',
      syntax: 'equals(value1, value2)',
      examples: ['equals(user.status, "active")', 'equals(item.type, "premium")'],
      parameters: [
        { name: 'value1', type: 'any', description: '第一个值', required: true },
        { name: 'value2', type: 'any', description: '第二个值', required: true },
      ],
      returnType: 'boolean',
    },
    {
      name: 'contains',
      category: 'logic',
      description: '包含检查',
      syntax: 'contains(string, substring) 或 contains(array, item)',
      examples: ['contains(user.email, "@")', 'contains(user.roles, "admin")'],
      parameters: [
        { name: 'container', type: 'string|array', description: '容器', required: true },
        { name: 'item', type: 'any', description: '要查找的内容', required: true },
      ],
      returnType: 'boolean',
    },

    // 数组函数
    {
      name: 'length',
      category: 'array',
      description: '获取数组或字符串长度',
      syntax: 'length(array) 或 length(string)',
      examples: ['length(user.items)', 'length(user.description)'],
      parameters: [
        { name: 'collection', type: 'array|string', description: '集合', required: true },
      ],
      returnType: 'number',
    },
    {
      name: 'first',
      category: 'array',
      description: '获取数组第一个元素',
      syntax: 'first(array)',
      examples: ['first(user.items)', 'first(user.comments)'],
      parameters: [
        { name: 'array', type: 'array', description: '数组', required: true },
      ],
      returnType: 'any',
    },
    {
      name: 'last',
      category: 'array',
      description: '获取数组最后一个元素',
      syntax: 'last(array)',
      examples: ['last(user.items)', 'last(user.orders)'],
      parameters: [
        { name: 'array', type: 'array', description: '数组', required: true },
      ],
      returnType: 'any',
    },
    {
      name: 'filter',
      category: 'array',
      description: '过滤数组元素',
      syntax: 'filter(array, predicate)',
      examples: ['filter(user.items, item => item.price > 100)', 'filter(users, user => user.active)'],
      parameters: [
        { name: 'array', type: 'array', description: '原数组', required: true },
        { name: 'predicate', type: 'function', description: '过滤条件', required: true },
      ],
      returnType: 'array',
    },
    {
      name: 'map',
      category: 'array',
      description: '映射数组元素',
      syntax: 'map(array, transform)',
      examples: ['map(user.items, item => item.price)', 'map(users, user => user.name)'],
      parameters: [
        { name: 'array', type: 'array', description: '原数组', required: true },
        { name: 'transform', type: 'function', description: '转换函数', required: true },
      ],
      returnType: 'array',
    },

    // 对象函数
    {
      name: 'get',
      category: 'object',
      description: '获取对象属性值',
      syntax: 'get(object, path, defaultValue?)',
      examples: ['get(user, "profile.name")', 'get(settings, "theme", "default")'],
      parameters: [
        { name: 'object', type: 'object', description: '对象', required: true },
        { name: 'path', type: 'string', description: '属性路径', required: true },
        { name: 'defaultValue', type: 'any', description: '默认值', required: false },
      ],
      returnType: 'any',
    },
    {
      name: 'set',
      category: 'object',
      description: '设置对象属性值',
      syntax: 'set(object, path, value)',
      examples: ['set(user, "profile.age", 25)', 'set(settings, "theme", "dark")'],
      parameters: [
        { name: 'object', type: 'object', description: '对象', required: true },
        { name: 'path', type: 'string', description: '属性路径', required: true },
        { name: 'value', type: 'any', description: '要设置的值', required: true },
      ],
      returnType: 'object',
    },
    {
      name: 'keys',
      category: 'object',
      description: '获取对象的所有键',
      syntax: 'keys(object)',
      examples: ['keys(user.profile)', 'keys(settings)'],
      parameters: [
        { name: 'object', type: 'object', description: '对象', required: true },
      ],
      returnType: 'array',
    },
    {
      name: 'values',
      category: 'object',
      description: '获取对象的所有值',
      syntax: 'values(object)',
      examples: ['values(user.profile)', 'values(settings)'],
      parameters: [
        { name: 'object', type: 'object', description: '对象', required: true },
      ],
      returnType: 'array',
    },
  ];

  // 按类别分组函数
  const functionsByCategory = useMemo(() => {
    const categories = expressionFunctions.reduce((acc, func) => {
      if (!acc[func.category]) {
        acc[func.category] = [];
      }
      acc[func.category].push(func);
      return acc;
    }, {} as Record<string, ExpressionFunction[]>);

    return categories;
  }, []);

  // 类别显示名称
  const categoryNames = {
    string: '字符串函数',
    number: '数字函数',
    date: '日期函数',
    logic: '逻辑函数',
    array: '数组函数',
    object: '对象函数',
    custom: '自定义函数',
  };

  // 类别图标
  const categoryIcons = {
    string: <CodeOutlined />,
    number: <ThunderboltOutlined />,
    date: <BookOutlined />,
    logic: <BulbOutlined />,
    array: <FunctionOutlined />,
    object: <InfoCircleOutlined />,
    custom: <CodeOutlined />,
  };

  // 表达式变化处理
  const handleExpressionChange = useCallback((newExpression: string) => {
    setExpression(newExpression);
    if (onChange) {
      onChange(newExpression);
    }
    setTestResult(null); // 清除之前的测试结果
  }, [onChange]);

  // 插入变量
  const insertVariable = useCallback((variable: FieldVariable) => {
    const variableText = variable.path;
    const newExpression = expression + (expression && !expression.endsWith(' ') ? ' ' : '') + variableText;
    handleExpressionChange(newExpression);
  }, [expression, handleExpressionChange]);

  // 插入函数
  const insertFunction = useCallback((func: ExpressionFunction) => {
    const functionText = `${func.name}()`;
    const newExpression = expression + (expression && !expression.endsWith(' ') ? ' ' : '') + functionText;
    handleExpressionChange(newExpression);
    setSelectedFunction(func);
    setActiveTab('functions');
  }, [expression, handleExpressionChange]);

  // 测试表达式
  const handleTest = useCallback(async () => {
    if (!onTest || !expression.trim()) return;

    setTesting(true);
    setTestResult(null);

    try {
      const result = await onTest(expression, testData);
      setTestResult(result);
    } catch (error: any) {
      setTestResult({
        success: false,
        error: error?.message || '测试失败',
      });
    } finally {
      setTesting(false);
    }
  }, [expression, testData, onTest]);

  // 复制表达式
  const copyExpression = useCallback(() => {
    navigator.clipboard.writeText(expression);
    // message.success('表达式已复制到剪贴板');
  }, [expression]);

  // 格式化表达式
  const formatExpression = useCallback(() => {
    try {
      // 简单的格式化逻辑
      const formatted = expression
        .replace(/\s+/g, ' ')
        .replace(/,\s*/g, ', ')
        .replace(/\(\s+/g, '(')
        .replace(/\s+\)/g, ')');
      handleExpressionChange(formatted);
    } catch {
      // 格式化失败时保持原样
    }
  }, [expression, handleExpressionChange]);

  // 渲染变量树
  const renderVariableTree = () => {
    const treeData = variables.map(variable => ({
      title: (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span>
            <Text strong>{variable.name}</Text>
            <Text type="secondary" style={{ marginLeft: 8 }}>
              {variable.path}
            </Text>
          </span>
          <Tag color="blue" style={{ fontSize: 10 }}>
            {variable.type}
          </Tag>
        </div>
      ),
      key: variable.path,
      isLeaf: true,
      icon: <InfoCircleOutlined />,
      data: variable,
    }));

    return (
      <Tree
        showLine
        treeData={treeData}
        onSelect={(selectedKeys, info) => {
          if (info.node.data) {
            insertVariable(info.node.data);
          }
        }}
      />
    );
  };

  // 渲染函数列表
  const renderFunctionList = () => {
    return (
      <Tabs
        activeKey={selectedFunction?.category || 'string'}
        onChange={(key) => setSelectedFunction(null)}
        size="small"
        tabPosition="left"
        style={{ height: 300 }}
      >
        {Object.entries(functionsByCategory).map(([category, functions]) => (
          <TabPane
            tab={
              <span>
                {categoryIcons[category as keyof typeof categoryIcons]}
                {categoryNames[category as keyof typeof categoryNames]}
                <Badge count={functions.length} style={{ marginLeft: 4 }} />
              </span>
            }
            key={category}
          >
            <div style={{ height: 250, overflowY: 'auto' }}>
              {functions.map(func => (
                <Card
                  key={func.name}
                  size="small"
                  hoverable
                  style={{
                    marginBottom: 8,
                    border: selectedFunction?.name === func.name ? '2px solid #1890ff' : undefined,
                  }}
                  onClick={() => setSelectedFunction(func)}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <div>
                      <Text strong>{func.name}</Text>
                      <div style={{ fontSize: 12, color: '#666', marginTop: 2 }}>
                        {func.syntax}
                      </div>
                    </div>
                    <Button
                      type="text"
                      size="small"
                      icon={<ThunderboltOutlined />}
                      onClick={(e) => {
                        e.stopPropagation();
                        insertFunction(func);
                      }}
                    />
                  </div>
                </Card>
              ))}
            </div>
          </TabPane>
        ))}
      </Tabs>
    );
  };

  // 渲染函数详情
  const renderFunctionDetail = () => {
    if (!selectedFunction) {
      return (
        <Empty
          description="选择一个函数查看详情"
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      );
    }

    return (
      <div>
        <Card size="small" title={selectedFunction.name}>
          <Paragraph>{selectedFunction.description}</Paragraph>

          <div style={{ marginBottom: 16 }}>
            <Text strong>语法: </Text>
            <Tag color="blue">{selectedFunction.syntax}</Tag>
          </div>

          <div style={{ marginBottom: 16 }}>
            <Text strong>返回类型: </Text>
            <Tag color="green">{selectedFunction.returnType}</Tag>
          </div>

          {selectedFunction.parameters.length > 0 && (
            <div style={{ marginBottom: 16 }}>
              <Text strong>参数:</Text>
              <div style={{ marginTop: 8 }}>
                {selectedFunction.parameters.map((param, index) => (
                  <div key={index} style={{ marginBottom: 4 }}>
                    <Tag color="orange">{param.name}</Tag>
                    <Text type="secondary">: {param.type}</Text>
                    <Text style={{ marginLeft: 8 }}>{param.description}</Text>
                    {param.required && <Tag color="red" size="small" style={{ marginLeft: 4 }}>必填</Tag>}
                  </div>
                ))}
              </div>
            </div>
          )}

          <div>
            <Text strong>示例:</Text>
            <div style={{ marginTop: 8 }}>
              {selectedFunction.examples.map((example, index) => (
                <div key={index} style={{ marginBottom: 4 }}>
                  <Button
                    type="text"
                    size="small"
                    icon={<CopyOutlined />}
                    onClick={() => handleExpressionChange(example)}
                    style={{ padding: '0 4px', height: 20 }}
                  />
                  <code style={{
                    background: '#f5f5f5',
                    padding: '2px 4px',
                    borderRadius: 2,
                    fontSize: 12,
                  }}>
                    {example}
                  </code>
                </div>
              ))}
            </div>
          </div>
        </Card>
      </div>
    );
  };

  // 渲染测试结果
  const renderTestResult = () => {
    if (!testResult) return null;

    return (
      <Card
        size="small"
        title={
          <Space>
            <Text>测试结果</Text>
            <Tag color={testResult.success ? 'green' : 'red'}>
              {testResult.success ? '成功' : '失败'}
            </Tag>
          </Space>
        }
      >
        {testResult.success ? (
          <div>
            <div style={{ marginBottom: 8 }}>
              <Text strong>结果: </Text>
              <Tag color="green">
                {typeof testResult.result === 'object'
                  ? JSON.stringify(testResult.result)
                  : String(testResult.result)}
              </Tag>
            </div>
            {testResult.executionTime && (
              <div style={{ marginBottom: 8 }}>
                <Text strong>执行时间: </Text>
                <Text>{testResult.executionTime}ms</Text>
              </div>
            )}
            {testResult.tokensUsed && (
              <div>
                <Text strong>使用令牌: </Text>
                <Text>{testResult.tokensUsed}</Text>
              </div>
            )}
          </div>
        ) : (
          <Alert
            message="测试失败"
            description={testResult.error}
            type="error"
            showIcon
          />
        )}
      </Card>
    );
  };

  return (
    <div className="expression-editor" style={{ height }}>
      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
        size="small"
        tabBarStyle={{ marginBottom: 12 }}
      >
        <TabPane
          tab={
            <span>
              <CodeOutlined />
              编辑器
            </span>
          }
          key="editor"
        >
          <div style={{ height: 'calc(100% - 40px)' }}>
            <div style={{ marginBottom: 12 }}>
              <Space>
                <Button
                  icon={<PlayCircleOutlined />}
                  onClick={handleTest}
                  loading={testing}
                  disabled={!expression.trim()}
                  size="small"
                >
                  测试
                </Button>
                <Button
                  icon={<CopyOutlined />}
                  onClick={copyExpression}
                  disabled={!expression.trim()}
                  size="small"
                >
                  复制
                </Button>
                <Button
                  onClick={formatExpression}
                  disabled={!expression.trim()}
                  size="small"
                >
                  格式化
                </Button>
              </Space>
            </div>

            <TextArea
              value={expression}
              onChange={(e) => handleExpressionChange(e.target.value)}
              placeholder={placeholder}
              disabled={readOnly}
              rows={6}
              style={{ fontFamily: 'Monaco, Consolas, monospace', fontSize: 12 }}
            />

            {renderTestResult()}
          </div>
        </TabPane>

        <TabPane
          tab={
            <span>
              <InfoCircleOutlined />
              变量
            </span>
          }
          key="variables"
        >
          <div style={{ height: 300, overflow: 'auto' }}>
            {variables.length > 0 ? (
              renderVariableTree()
            ) : (
              <Empty description="暂无可用变量" />
            )}
          </div>
        </TabPane>

        <TabPane
          tab={
            <span>
              <FunctionOutlined />
              函数
            </span>
          }
          key="functions"
        >
          <div style={{ display: 'flex', height: 300 }}>
            <div style={{ flex: 1 }}>
              {renderFunctionList()}
            </div>
            {selectedFunction && (
              <Divider type="vertical" style={{ height: '100%' }} />
            )}
            <div style={{ flex: 1, paddingLeft: 16 }}>
              {renderFunctionDetail()}
            </div>
          </div>
        </TabPane>

        <TabPane
          tab={
            <span>
              <BookOutlined />
              示例
            </span>
          }
          key="examples"
        >
          <div style={{ height: 300, overflowY: 'auto' }}>
            <Alert
              message="表达式示例"
              description="点击下方示例快速应用到编辑器中"
              type="info"
              showIcon
              style={{ marginBottom: 16 }}
            />

            {[
              {
                category: '字符串处理',
                examples: [
                  'toUpperCase(user.name)',
                  'trim(user.email)',
                  'replace(user.phone, "-", "")',
                  'split(user.tags, ",")',
                ]
              },
              {
                category: '数字处理',
                examples: [
                  'parseInt(user.age)',
                  'round(user.price, 2)',
                  'formatCurrency(user.amount)',
                  'if(user.score > 80, user.score * 1.2, user.score)',
                ]
              },
              {
                category: '日期处理',
                examples: [
                  'formatDate(user.created_at, "YYYY-MM-DD")',
                  'addDays(user.expiry, 30)',
                  'formatDate(now(), "HH:mm:ss")',
                ]
              },
              {
                category: '逻辑判断',
                examples: [
                  'if(user.age > 18, "成年", "未成年")',
                  'equals(user.status, "active")',
                  'contains(user.email, "@")',
                ]
              },
              {
                category: '数组操作',
                examples: [
                  'length(user.items)',
                  'first(user.orders)',
                  'filter(user.items, item => item.price > 100)',
                ]
              },
              {
                category: '对象操作',
                examples: [
                  'get(user, "profile.name")',
                  'keys(user.settings)',
                  'set(user, "profile.age", 25)',
                ]
              },
            ].map((category, index) => (
              <div key={index} style={{ marginBottom: 16 }}>
                <Title level={5}>{category.category}</Title>
                <Space wrap>
                  {category.examples.map((example, exampleIndex) => (
                    <Tag
                      key={exampleIndex}
                      color="blue"
                      style={{
                        cursor: 'pointer',
                        fontFamily: 'Monaco, Consolas, monospace',
                        fontSize: 11,
                      }}
                      onClick={() => handleExpressionChange(example)}
                    >
                      {example}
                    </Tag>
                  ))}
                </Space>
              </div>
            ))}
          </div>
        </TabPane>
      </Tabs>
    </div>
  );
};

export default ExpressionEditor;