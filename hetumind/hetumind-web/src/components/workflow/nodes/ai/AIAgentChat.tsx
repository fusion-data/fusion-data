import React, { useState, useCallback, useRef, useEffect } from 'react';
import {
  Card,
  Input,
  Button,
  Space,
  Typography,
  Avatar,
  message,
  Spin,
  Tag,
  Row,
  Col,
  Select,
  Slider,
  Switch,
  Tooltip,
  Badge,
  Empty,
} from 'antd';
import {
  SendOutlined,
  RobotOutlined,
  UserOutlined,
  ClearOutlined,
  PauseCircleOutlined,
  CopyOutlined,
  LikeOutlined,
  DislikeOutlined,
  ReloadOutlined,
  InfoCircleOutlined,
} from '@ant-design/icons';

const { Text, Paragraph } = Typography;
const { TextArea } = Input;

// 消息接口
interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  metadata?: {
    model?: string;
    temperature?: number;
    tokens?: number;
    responseTime?: number;
    cost?: number;
    rating?: 'up' | 'down';
  };
}

// AI Agent配置接口
interface ChatConfig {
  model: string;
  temperature: number;
  maxTokens: number;
  systemPrompt: string;
  enableStreaming: boolean;
  enableMemory: boolean;
  topP: number;
  frequencyPenalty: number;
  presencePenalty: number;
}

interface AIAgentChatProps {
  initialConfig?: Partial<ChatConfig>;
  onConfigChange?: (config: ChatConfig) => void;
  onMessage?: (message: ChatMessage) => void;
  readOnly?: boolean;
  height?: number;
  showConfig?: boolean;
}

export const AIAgentChat: React.FC<AIAgentChatProps> = ({
  initialConfig,
  onConfigChange,
  onMessage,
  readOnly = false,
  height = 500,
  showConfig = true,
}) => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [streaming, setStreaming] = useState(false);
  const [config, setConfig] = useState<ChatConfig>({
    model: 'gpt-3.5-turbo',
    temperature: 0.7,
    maxTokens: 1024,
    systemPrompt: '你是一个智能助手，请根据用户的问题提供准确、有用的回答。',
    enableStreaming: true,
    enableMemory: true,
    topP: 1,
    frequencyPenalty: 0,
    presencePenalty: 0,
  });

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  // 初始化配置
  useEffect(() => {
    if (initialConfig) {
      setConfig(prev => ({ ...prev, ...initialConfig }));
    }
  }, [initialConfig]);

  // 滚动到底部
  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // 配置变化处理
  const handleConfigChange = useCallback((newConfig: Partial<ChatConfig>) => {
    const updatedConfig = { ...config, ...newConfig };
    setConfig(updatedConfig);
    if (onConfigChange) {
      onConfigChange(updatedConfig);
    }
  }, [config, onConfigChange]);

  // 发送消息
  const sendMessage = useCallback(async (content: string) => {
    if (loading || readOnly || !content.trim()) return;

    const userMessage: ChatMessage = {
      id: `user_${Date.now()}`,
      role: 'user',
      content: content.trim(),
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setLoading(true);

    try {
      // 构建消息历史
      const messageHistory = config.enableMemory
        ? [...messages.filter(m => m.role !== 'system'), userMessage]
        : [userMessage];

      // 模拟AI响应
      const startTime = Date.now();

      if (config.enableStreaming) {
        setStreaming(true);
        await simulateStreamingResponse(messageHistory, startTime);
      } else {
        await simulateNormalResponse(messageHistory, startTime);
      }

    } catch (error: any) {
      message.error('发送消息失败: ' + (error?.message || '未知错误'));

      const errorMessage: ChatMessage = {
        id: `error_${Date.now()}`,
        role: 'assistant',
        content: '抱歉，我遇到了一些问题，请稍后再试。',
        timestamp: new Date(),
      };

      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setLoading(false);
      setStreaming(false);
    }
  }, [loading, readOnly, messages, config, onMessage]);

  // 模拟普通响应
  const simulateNormalResponse = async (messageHistory: ChatMessage[], startTime: number) => {
    await new Promise(resolve => setTimeout(resolve, 1000 + Math.random() * 2000));

    const responseTime = Date.now() - startTime;
    const tokens = Math.floor(100 + Math.random() * 500);

    const assistantMessage: ChatMessage = {
      id: `assistant_${Date.now()}`,
      role: 'assistant',
      content: generateMockResponse(messageHistory[messageHistory.length - 1]?.content || ''),
      timestamp: new Date(),
      metadata: {
        model: config.model,
        temperature: config.temperature,
        tokens,
        responseTime,
        cost: (tokens / 1000) * 0.002, // 模拟成本计算
      },
    };

    setMessages(prev => [...prev, assistantMessage]);

    if (onMessage) {
      onMessage(assistantMessage);
    }
  };

  // 模拟流式响应
  const simulateStreamingResponse = async (messageHistory: ChatMessage[], startTime: number) => {
    const fullResponse = generateMockResponse(messageHistory[messageHistory.length - 1]?.content || '');
    const words = fullResponse.split(' ');

    let currentMessage = '';
    const messageId = `assistant_${Date.now()}`;

    for (let i = 0; i < words.length; i++) {
      if (!streaming) break; // 检查是否被中断

      currentMessage += (i > 0 ? ' ' : '') + words[i];

      setMessages(prev => {
        const existing = prev.findIndex(m => m.id === messageId);
        if (existing >= 0) {
          const updated = [...prev];
          updated[existing] = {
            ...updated[existing],
            content: currentMessage,
          };
          return updated;
        } else {
          return [...prev, {
            id: messageId,
            role: 'assistant',
            content: currentMessage,
            timestamp: new Date(),
          }];
        }
      });

      await new Promise(resolve => setTimeout(resolve, 50 + Math.random() * 100));
    }

    const responseTime = Date.now() - startTime;
    const tokens = Math.floor(100 + Math.random() * 500);

    // 更新最终消息的元数据
    setMessages(prev => prev.map(m =>
      m.id === messageId
        ? {
            ...m,
            metadata: {
              model: config.model,
              temperature: config.temperature,
              tokens,
              responseTime,
              cost: (tokens / 1000) * 0.002,
            },
          }
        : m
    ));
  };

  // 生成模拟响应
  const generateMockResponse = (userInput: string): string => {
    const responses = [
      `这是一个很好的问题。关于"${userInput}"，我认为需要从多个角度来考虑。首先，我们要理解问题的核心，然后分析可能的解决方案。`,
      `我理解您关于"${userInput}"的疑问。基于我的分析，我建议我们可以采取以下步骤来处理这个问题。`,
      `感谢您的提问。对于"${userInput}"这个话题，我有一些见解想与您分享。这个涉及到多个层面的考虑。`,
      `您提出的"${userInput}"很有意思。让我为您详细分析一下这个问题的关键点和可能的解决方案。`,
    ];

    return responses[Math.floor(Math.random() * responses.length)] +
      "\n\n具体来说，我们可以从以下几个方面来深入探讨：\n" +
      "1. 首先分析现状和挑战\n" +
      "2. 然后考虑可能的解决方案\n" +
      "3. 最后评估效果和优化建议\n\n" +
      "您觉得这个分析框架如何？如果您有其他想法，我很乐意进一步讨论。";
  };

  // 停止流式响应
  const stopStreaming = useCallback(() => {
    setStreaming(false);
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
  }, []);

  // 清空对话
  const clearMessages = useCallback(() => {
    setMessages([]);
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    setStreaming(false);
  }, []);

  // 复制消息
  const copyMessage = useCallback((content: string) => {
    navigator.clipboard.writeText(content);
    message.success('已复制到剪贴板');
  }, []);

  // 重新生成响应
  const regenerateResponse = useCallback(() => {
    if (messages.length === 0) return;

    const lastUserMessage = [...messages].reverse().find(m => m.role === 'user');
    if (lastUserMessage) {
      // 移除最后一个AI响应
      setMessages(prev => prev.slice(0, -1));
      // 重新发送用户消息
      sendMessage(lastUserMessage.content);
    }
  }, [messages, sendMessage]);

  // 评价响应
  const rateResponse = useCallback((messageId: string, rating: 'up' | 'down') => {
    setMessages(prev => prev.map(m =>
      m.id === messageId
        ? { ...m, metadata: { ...m.metadata, rating } }
        : m
    ));
    message.success(`感谢您的${rating === 'up' ? '好评' : '反馈'}`);
  }, []);

  // 渲染消息
  const renderMessage = (message: ChatMessage) => {
    const isUser = message.role === 'user';

    return (
      <div
        key={message.id}
        style={{
          display: 'flex',
          justifyContent: isUser ? 'flex-end' : 'flex-start',
          marginBottom: 16,
        }}
      >
        <div style={{ maxWidth: '70%' }}>
          <div style={{ display: 'flex', alignItems: 'flex-start', gap: 8 }}>
            {!isUser && (
              <Avatar
                size="small"
                icon={<RobotOutlined />}
                style={{ backgroundColor: '#1890ff', flexShrink: 0 }}
              />
            )}

            <div
              style={{
                backgroundColor: isUser ? '#1890ff' : '#f5f5f5',
                color: isUser ? 'white' : 'inherit',
                padding: '8px 12px',
                borderRadius: 12,
                borderBottomLeftRadius: isUser ? 12 : 4,
                borderBottomRightRadius: isUser ? 4 : 12,
                wordBreak: 'break-word',
              }}
            >
              <Paragraph style={{
                margin: 0,
                whiteSpace: 'pre-wrap',
                color: isUser ? 'white' : 'inherit',
              }}>
                {message.content}
              </Paragraph>

              {message.metadata && (
                <div style={{ marginTop: 8, fontSize: 11, opacity: 0.7 }}>
                  <Space wrap size="small">
                    {message.metadata.model && (
                      <Tag color={isUser ? 'blue' : 'default'} style={{ fontSize: 10 }}>
                        {message.metadata.model}
                      </Tag>
                    )}
                    {message.metadata.tokens && (
                      <span>{message.metadata.tokens} tokens</span>
                    )}
                    {message.metadata.responseTime && (
                      <span>{message.metadata.responseTime}ms</span>
                    )}
                    {message.metadata.cost && (
                      <span>${message.metadata.cost.toFixed(4)}</span>
                    )}
                  </Space>
                </div>
              )}
            </div>

            {isUser && (
              <Avatar
                size="small"
                icon={<UserOutlined />}
                style={{ backgroundColor: '#52c41a', flexShrink: 0 }}
              />
            )}
          </div>

          {/* 消息操作 */}
          {!isUser && message.content && (
            <div style={{
              marginTop: 4,
              marginLeft: 32,
              display: 'flex',
              gap: 8,
              alignItems: 'center',
            }}>
              <Button
                type="text"
                size="small"
                icon={<CopyOutlined />}
                onClick={() => copyMessage(message.content)}
                style={{ padding: '0 4px', height: 20 }}
              />
              <Button
                type="text"
                size="small"
                icon={<LikeOutlined />}
                onClick={() => rateResponse(message.id, 'up')}
                style={{
                  padding: '0 4px',
                  height: 20,
                }}
              />
              <Button
                type="text"
                size="small"
                icon={<DislikeOutlined />}
                onClick={() => rateResponse(message.id, 'down')}
                style={{
                  padding: '0 4px',
                  height: 20,
                }}
              />
              {messages[messages.length - 1]?.id === message.id && (
                <Button
                  type="text"
                  size="small"
                  icon={<ReloadOutlined />}
                  onClick={regenerateResponse}
                  style={{ padding: '0 4px', height: 20 }}
                />
              )}
            </div>
          )}
        </div>
      </div>
    );
  };

  // 渲染配置面板
  const renderConfigPanel = () => {
    if (!showConfig) return null;

    return (
      <Card size="small" title="对话配置" style={{ marginBottom: 16 }}>
        <Row gutter={16}>
          <Col span={12}>
            <div style={{ marginBottom: 8 }}>
              <Text strong>模型</Text>
              <Select
                value={config.model}
                onChange={(value) => handleConfigChange({ model: value })}
                style={{ width: '100%', marginTop: 4 }}
                size="small"
              >
                <Select.Option value="gpt-4">GPT-4</Select.Option>
                <Select.Option value="gpt-3.5-turbo">GPT-3.5 Turbo</Select.Option>
                <Select.Option value="claude-3-opus">Claude 3 Opus</Select.Option>
                <Select.Option value="claude-3-sonnet">Claude 3 Sonnet</Select.Option>
              </Select>
            </div>
          </Col>
          <Col span={12}>
            <div style={{ marginBottom: 8 }}>
              <Text strong>记忆功能</Text>
              <div style={{ marginTop: 4 }}>
                <Switch
                  checked={config.enableMemory}
                  onChange={(checked) => handleConfigChange({ enableMemory: checked })}
                  size="small"
                />
                <Text style={{ marginLeft: 8, fontSize: 12 }}>
                  {config.enableMemory ? '启用' : '禁用'}
                </Text>
              </div>
            </div>
          </Col>
        </Row>

        <div style={{ marginBottom: 8 }}>
          <Text strong>
            温度
            <Tooltip title="控制输出的随机性">
              <InfoCircleOutlined style={{ marginLeft: 4, fontSize: 12 }} />
            </Tooltip>
          </Text>
          <Slider
            value={config.temperature}
            onChange={(value) => handleConfigChange({ temperature: value })}
            min={0}
            max={2}
            step={0.1}
            style={{ marginTop: 4 }}
          />
        </div>

        <div>
          <Text strong>系统提示</Text>
          <TextArea
            value={config.systemPrompt}
            onChange={(e) => handleConfigChange({ systemPrompt: e.target.value })}
            rows={2}
            size="small"
            style={{ marginTop: 4 }}
          />
        </div>
      </Card>
    );
  };

  return (
    <div className="ai-agent-chat" style={{ height }}>
      {renderConfigPanel()}

      {/* 对话区域 */}
      <Card
        style={{ height: showConfig ? `calc(100% - ${showConfig ? 200 : 60}px)` : '100%' }}
        bodyStyle={{
          padding: 16,
          height: '100%',
          display: 'flex',
          flexDirection: 'column'
        }}
      >
        {/* 聊天头部 */}
        <div style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: 16,
          paddingBottom: 8,
          borderBottom: '1px solid #f0f0f0',
        }}>
          <Space>
            <RobotOutlined style={{ color: '#1890ff' }} />
            <Text strong>AI 助手对话</Text>
            {loading && (
              <Badge status="processing" text="处理中..." />
            )}
          </Space>

          <Space>
            <Button
              type="text"
              size="small"
              icon={<ClearOutlined />}
              onClick={clearMessages}
              disabled={messages.length === 0}
            >
              清空
            </Button>
          </Space>
        </div>

        {/* 消息列表 */}
        <div
          style={{
            flex: 1,
            overflowY: 'auto',
            marginBottom: 16,
            padding: '0 8px',
          }}
        >
          {messages.length === 0 ? (
            <Empty
              description="开始对话吧"
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              style={{ marginTop: 60 }}
            />
          ) : (
            messages.map(renderMessage)
          )}

          {streaming && (
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <Avatar size="small" icon={<RobotOutlined />} />
              <Space>
                <Spin size="small" />
                <Text type="secondary">正在输入...</Text>
                <Button
                  type="text"
                  size="small"
                  icon={<PauseCircleOutlined />}
                  onClick={stopStreaming}
                >
                  停止
                </Button>
              </Space>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* 输入区域 */}
        <div style={{ borderTop: '1px solid #f0f0f0', paddingTop: 12 }}>
          <div style={{ display: 'flex', gap: 8, alignItems: 'flex-end' }}>
            <TextArea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder={readOnly ? "只读模式" : "输入您的问题..."}
              autoSize={{ minRows: 1, maxRows: 4 }}
              disabled={readOnly || loading}
              onPressEnter={(e) => {
                if (!e.shiftKey) {
                  e.preventDefault();
                  sendMessage(input);
                }
              }}
              style={{ flex: 1 }}
            />
            <Button
              type="primary"
              icon={<SendOutlined />}
              onClick={() => sendMessage(input)}
              disabled={readOnly || loading || !input.trim()}
              loading={loading}
            >
              发送
            </Button>
          </div>

          <div style={{ marginTop: 8, fontSize: 12, color: '#666' }}>
            按 Enter 发送，Shift + Enter 换行
          </div>
        </div>
      </Card>
    </div>
  );
};

export default AIAgentChat;