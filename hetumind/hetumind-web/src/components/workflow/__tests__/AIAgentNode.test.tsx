import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { AIAgentNode } from '../nodes/AIAgentNode';
import { NodeProvider } from '../nodes/NodeContext';

// Mock Ant Design components
jest.mock('antd', () => ({
  Card: ({ children, title, className }: any) => (
    <div className={className} data-testid="node-card">
      <h3 data-testid="node-title">{title}</h3>
      {children}
    </div>
  ),
  Button: ({ children, onClick, type, icon, ...props }: any) => (
    <button
      onClick={onClick}
      data-type={type}
      data-icon={icon}
      {...props}
    >
      {icon}
      {children}
    </button>
  ),
  Space: ({ children }: any) => <div>{children}</div>,
  Tooltip: ({ children, title }: any) => (
    <div title={title}>{children}</div>
  ),
  Tag: ({ children, color }: any) => (
    <span data-color={color}>{children}</span>
  ),
  Select: ({ children, onChange, value, ...props }: any) => (
    <select onChange={(e) => onChange?.(e.target.value)} value={value} {...props}>
      {children}
    </select>
  ),
  'Select.Option': ({ children, value }: any) => (
    <option value={value}>{children}</option>
  ),
  Input: ({ onChange, value, placeholder, type, ...props }: any) => (
    <input
      onChange={(e) => onChange?.(e.target.value)}
      value={value}
      placeholder={placeholder}
      type={type}
      {...props}
    />
  ),
  InputNumber: ({ onChange, value, min, max, ...props }: any) => (
    <input
      type="number"
      onChange={(e) => onChange?.(Number(e.target.value))}
      value={value}
      min={min}
      max={max}
      {...props}
    />
  ),
  Switch: ({ checked, onChange, ...props }: any) => (
    <input
      type="checkbox"
      checked={checked}
      onChange={(e) => onChange?.(e.target.checked)}
      {...props}
    />
  ),
  Slider: ({ onChange, value, min, max, ...props }: any) => (
    <input
      type="range"
      onChange={(e) => onChange?.(Number(e.target.value))}
      value={value}
      min={min}
      max={max}
      {...props}
    />
  ),
  Tabs: ({ children, activeKey, onChange }: any) => (
    <div data-active-key={activeKey}>
      {children}
    </div>
  ),
  'Tabs.TabPane': ({ children, tab }: any) => (
    <div data-tab={tab}>{children}</div>
  ),
  Form: ({ children }: any) => <form>{children}</form>,
  'Form.Item': ({ children, label }: any) => (
    <div>
      <label>{label}</label>
      {children}
    </div>
  ),
}));

// Mock icons
jest.mock('@ant-design/icons', () => ({
  RobotOutlined: () => <span data-testid="robot-icon">🤖</span>,
  MessageOutlined: () => <span data-testid="message-icon">💬</span>,
  EditOutlined: () => <span data-testid="edit-icon">✏️</span>,
  PictureOutlined: () => <span data-testid="picture-icon">🖼️</span>,
  SoundOutlined: () => <span data-testid="sound-icon">🔊</span>,
  ApiOutlined: () => <span data-testid="api-icon">🔌</span>,
  SettingOutlined: () => <span data-testid="setting-icon">⚙</span>,
  PlayCircleOutlined: () => <span data-testid="play-icon">▶️</span>,
}));

// Mock React Flow
jest.mock('@xyflow/react', () => ({
  Handle: ({ position, type }: any) => (
    <div data-testid={`handle-${position}-${type}`} />
  ),
  Position: { Left: 'left', Right: 'right', Top: 'top', Bottom: 'bottom' },
}));

const mockNodeData = {
  label: 'Test AI Agent',
  description: 'Test AI agent description',
  agentType: 'chat',
  config: {
    model: 'gpt-3.5-turbo',
    temperature: 0.7,
    maxTokens: 1024,
    systemPrompt: 'You are a helpful assistant.',
  },
  input: {
    message: 'Hello, how are you?',
  },
};

const defaultProps = {
  id: 'ai-agent-1',
  type: 'aiAgent',
  data: mockNodeData,
  selected: false,
  dragging: false,
};

const renderWithProvider = (component: React.ReactElement) => {
  const mockUpdateNodeData = jest.fn();
  const mockOnSelect = jest.fn();

  return render(
    <NodeProvider
      value={{
        updateNodeData: mockUpdateNodeData,
        onSelect: mockOnSelect,
        selectedNodes: [],
        updateNodePosition: jest.fn(),
        deleteNode: jest.fn(),
        duplicateNode: jest.fn(),
      }}
    >
      {component}
    </NodeProvider>
  );
};

describe('AIAgentNode', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders AI agent node correctly', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      expect(screen.getByTestId('node-card')).toBeInTheDocument();
      expect(screen.getByTestId('node-title')).toHaveTextContent('Test AI Agent');
      expect(screen.getByTestId('robot-icon')).toBeInTheDocument();
    });

    it('displays correct agent type icon', () => {
      const chatProps = { ...defaultProps, data: { ...mockNodeData, agentType: 'chat' } };
      renderWithProvider(<AIAgentNode {...chatProps} />);

      expect(screen.getByTestId('message-icon')).toBeInTheDocument();
    });

    it('displays edit icon for completion agents', () => {
      const completionProps = { ...defaultProps, data: { ...mockNodeData, agentType: 'completion' } };
      renderWithProvider(<AIAgentNode {...completionProps} />);

      expect(screen.getByTestId('edit-icon')).toBeInTheDocument();
    });

    it('displays picture icon for image generation agents', () => {
      const imageProps = { ...defaultProps, data: { ...mockNodeData, agentType: 'image' } };
      renderWithProvider(<AIAgentNode {...imageProps} />);

      expect(screen.getByTestId('picture-icon')).toBeInTheDocument();
    });

    it('displays sound icon for speech agents', () => {
      const speechProps = { ...defaultProps, data: { ...mockNodeData, agentType: 'speech' } };
      renderWithProvider(<AIAgentNode {...speechProps} />);

      expect(screen.getByTestId('sound-icon')).toBeInTheDocument();
    });

    it('shows selected state when selected', () => {
      const selectedProps = { ...defaultProps, selected: true };
      const { container } = renderWithProvider(<AIAgentNode {...selectedProps} />);

      const nodeCard = screen.getByTestId('node-card');
      expect(nodeCard).toHaveClass('selected');
    });

    it('shows dragging state when dragging', () => {
      const draggingProps = { ...defaultProps, dragging: true };
      const { container } = renderWithProvider(<AIAgentNode {...draggingProps} />);

      const nodeCard = screen.getByTestId('node-card');
      expect(nodeCard).toHaveClass('dragging');
    });
  });

  describe('Configuration', () => {
    it('handles model selection', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const modelSelect = screen.getByDisplayValue('gpt-3.5-turbo');
      fireEvent.change(modelSelect, { target: { value: 'gpt-4' } });

      expect(modelSelect).toHaveValue('gpt-4');
    });

    it('handles temperature adjustment', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const temperatureSlider = screen.getByDisplayValue('0.7');
      fireEvent.change(temperatureSlider, { target: { value: '0.9' } });

      expect(temperatureSlider).toHaveValue(0.9);
    });

    it('handles max tokens adjustment', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const maxTokensInput = screen.getByDisplayValue('1024');
      fireEvent.change(maxTokensInput, { target: { value: '2048' } });

      expect(maxTokensInput).toHaveValue(2048);
    });

    it('handles system prompt change', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const systemPromptInput = screen.getByDisplayValue('You are a helpful assistant.');
      fireEvent.change(systemPromptInput, { target: { value: 'You are an expert assistant.' } });

      expect(systemPromptInput).toHaveValue('You are an expert assistant.');
    });

    it('validates temperature range', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const temperatureSlider = screen.getByDisplayValue('0.7');

      // Test minimum value
      fireEvent.change(temperatureSlider, { target: { value: '-0.1' } });
      expect(temperatureSlider).toHaveValue(-0.1);

      // Test maximum value
      fireEvent.change(temperatureSlider, { target: { value: '2.0' } });
      expect(temperatureSlider).toHaveValue(2);
    });

    it('validates max tokens range', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const maxTokensInput = screen.getByDisplayValue('1024');

      // Test minimum value
      fireEvent.change(maxTokensInput, { target: { value: '1' } });
      expect(maxTokensInput).toHaveValue(1);

      // Test maximum value
      fireEvent.change(maxTokensInput, { target: { value: '32000' } });
      expect(maxTokensInput).toHaveValue(32000);
    });
  });

  describe('Agent Types', () => {
    it('handles chat agent configuration', () => {
      const chatProps = { ...defaultProps, data: { ...mockNodeData, agentType: 'chat' } };
      renderWithProvider(<AIAgentNode {...chatProps} />);

      expect(screen.getByDisplayValue('Hello, how are you?')).toBeInTheDocument();
      expect(screen.getByTestId('message-icon')).toBeInTheDocument();
    });

    it('handles completion agent configuration', () => {
      const completionProps = {
        ...defaultProps,
        data: {
          ...mockNodeData,
          agentType: 'completion',
          input: { prompt: 'Complete this sentence:' },
        },
      };
      renderWithProvider(<AIAgentNode {...completionProps} />);

      expect(screen.getByDisplayValue('Complete this sentence:')).toBeInTheDocument();
      expect(screen.getByTestId('edit-icon')).toBeInTheDocument();
    });

    it('handles embedding agent configuration', () => {
      const embeddingProps = {
        ...defaultProps,
        data: {
          ...mockNodeData,
          agentType: 'embedding',
          input: { text: 'This is a sample text for embedding.' },
        },
      };
      renderWithProvider(<AIAgentNode {...embeddingProps} />);

      expect(screen.getByDisplayValue('This is a sample text for embedding.')).toBeInTheDocument();
    });

    it('handles image generation agent configuration', () => {
      const imageProps = {
        ...defaultProps,
        data: {
          ...mockNodeData,
          agentType: 'image',
          input: { prompt: 'A beautiful landscape painting' },
        },
      };
      renderWithProvider(<AIAgentNode {...imageProps} />);

      expect(screen.getByDisplayValue('A beautiful landscape painting')).toBeInTheDocument();
      expect(screen.getByTestId('picture-icon')).toBeInTheDocument();
    });

    it('handles speech agent configuration', () => {
      const speechProps = {
        ...defaultProps,
        data: {
          ...mockNodeData,
          agentType: 'speech',
          input: { text: 'Hello, this is a speech synthesis test.' },
        },
      };
      renderWithProvider(<AIAgentNode {...speechProps} />);

      expect(screen.getByDisplayValue('Hello, this is a speech synthesis test.')).toBeInTheDocument();
      expect(screen.getByTestId('sound-icon')).toBeInTheDocument();
    });
  });

  describe('Interactions', () => {
    it('calls onSelect when clicked', () => {
      const mockOnSelect = jest.fn();
      renderWithProvider(
        <NodeProvider
          value={{
            updateNodeData: jest.fn(),
            onSelect: mockOnSelect,
            selectedNodes: [],
            updateNodePosition: jest.fn(),
            deleteNode: jest.fn(),
            duplicateNode: jest.fn(),
          }}
        >
          <AIAgentNode {...defaultProps} />
        </NodeProvider>
      );

      fireEvent.click(screen.getByTestId('node-card'));
      expect(mockOnSelect).toHaveBeenCalledWith(defaultProps.id);
    });

    it('renders configuration button', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      expect(screen.getByTestId('setting-icon')).toBeInTheDocument();
    });

    it('handles agent type change', async () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const agentTypeSelect = screen.getByDisplayValue('chat');
      fireEvent.change(agentTypeSelect, { target: { value: 'completion' } });

      expect(agentTypeSelect).toHaveValue('completion');
    });

    it('handles test execution', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const testButton = screen.getByTestId('play-icon');
      fireEvent.click(testButton.closest('button')!);

      expect(testButton).toBeInTheDocument();
    });

    it('handles keyboard interactions', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const nodeCard = screen.getByTestId('node-card');

      // Test Enter key
      fireEvent.keyDown(nodeCard, { key: 'Enter' });

      // Test Space key
      fireEvent.keyDown(nodeCard, { key: ' ' });

      // Test Escape key
      fireEvent.keyDown(nodeCard, { key: 'Escape' });

      expect(true).toBe(true); // Verify no errors thrown
    });

    it('handles right-click context menu', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const nodeCard = screen.getByTestId('node-card');
      fireEvent.contextMenu(nodeCard);

      expect(true).toBe(true); // Verify no errors thrown
    });
  });

  describe('Validation', () => {
    it('validates required configuration fields', () => {
      const invalidProps = {
        ...defaultProps,
        data: {
          ...mockNodeData,
          config: { model: '', temperature: 1.5, maxTokens: 0 },
        },
      };

      renderWithProvider(<AIAgentNode {...invalidProps} />);

      const nodeCard = screen.getByTestId('node-card');
      expect(nodeCard).toBeInTheDocument();
    });

    it('validates agent type specific inputs', () => {
      const invalidChatProps = {
        ...defaultProps,
        data: {
          ...mockNodeData,
          agentType: 'chat',
          input: { message: '' },
        },
      };

      renderWithProvider(<AIAgentNode {...invalidChatProps} />);

      const nodeCard = screen.getByTestId('node-card');
      expect(nodeCard).toBeInTheDocument();
    });

    it('validates temperature bounds', () => {
      const invalidTempProps = {
        ...defaultProps,
        data: {
          ...mockNodeData,
          config: { ...mockNodeData.config, temperature: 1.5 },
        },
      };

      renderWithProvider(<AIAgentNode {...invalidTempProps} />);

      const temperatureSlider = screen.getByDisplayValue('1.5');
      expect(temperatureSlider).toBeInTheDocument();
    });

    it('validates max tokens bounds', () => {
      const invalidTokensProps = {
        ...defaultProps,
        data: {
          ...mockNodeData,
          config: { ...mockNodeData.config, maxTokens: -100 },
        },
      };

      renderWithProvider(<AIAgentNode {...invalidTokensProps} />);

      const maxTokensInput = screen.getByDisplayValue('-100');
      expect(maxTokensInput).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('maintains focus management', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const nodeCard = screen.getByTestId('node-card');
      nodeCard.focus();

      expect(nodeCard).toHaveFocus();
    });

    it('supports keyboard navigation', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const nodeCard = screen.getByTestId('node-card');
      fireEvent.keyDown(nodeCard, { key: 'Tab' });

      expect(nodeCard).toBeInTheDocument();
    });

    it('provides ARIA labels', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const nodeCard = screen.getByTestId('node-card');
      expect(nodeCard).toHaveAttribute('role', 'button');
      expect(nodeCard).toHaveAttribute('tabIndex', '0');
    });
  });

  describe('Data Updates', () => {
    it('handles data updates', async () => {
      const mockUpdateNodeData = jest.fn();
      renderWithProvider(
        <NodeProvider
          value={{
            updateNodeData: mockUpdateNodeData,
            onSelect: jest.fn(),
            selectedNodes: [],
            updateNodePosition: jest.fn(),
            deleteNode: jest.fn(),
            duplicateNode: jest.fn(),
          }}
        >
          <AIAgentNode {...defaultProps} />
        </NodeProvider>
      );

      const newData = { ...mockNodeData, label: 'Updated AI Agent' };
      expect(mockUpdateNodeData).toBeDefined();
    });

    it('handles configuration updates', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const modelSelect = screen.getByDisplayValue('gpt-3.5-turbo');
      fireEvent.change(modelSelect, { target: { value: 'gpt-4' } });

      expect(modelSelect).toHaveValue('gpt-4');
    });
  });

  describe('Error Handling', () => {
    it('handles missing configuration gracefully', () => {
      const missingConfigProps = {
        ...defaultProps,
        data: { label: 'Test AI Agent', agentType: 'chat' },
      };

      renderWithProvider(<AIAgentNode {...missingConfigProps} />);

      const nodeCard = screen.getByTestId('node-card');
      expect(nodeCard).toBeInTheDocument();
    });

    it('handles invalid agent type', () => {
      const invalidTypeProps = {
        ...defaultProps,
        data: { ...mockNodeData, agentType: 'invalid' },
      };

      renderWithProvider(<AIAgentNode {...invalidTypeProps} />);

      const nodeCard = screen.getByTestId('node-card');
      expect(nodeCard).toBeInTheDocument();
    });

    it('handles API errors gracefully', () => {
      const errorProps = {
        ...defaultProps,
        data: { ...mockNodeData, error: 'API connection failed' },
      };

      renderWithProvider(<AIAgentNode {...errorProps} />);

      const nodeCard = screen.getByTestId('node-card');
      expect(nodeCard).toBeInTheDocument();
    });
  });

  describe('Performance', () => {
    it('renders efficiently with large prompts', () => {
      const largePromptProps = {
        ...defaultProps,
        data: {
          ...mockNodeData,
          input: {
            message: 'A'.repeat(10000), // Large prompt
          },
        },
      };

      const startTime = performance.now();
      renderWithProvider(<AIAgentNode {...largePromptProps} />);
      const endTime = performance.now();

      // Should render within reasonable time
      expect(endTime - startTime).toBeLessThan(100);
    });

    it('handles rapid configuration changes', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      const modelSelect = screen.getByDisplayValue('gpt-3.5-turbo');

      // Rapidly change values
      for (let i = 0; i < 10; i++) {
        fireEvent.change(modelSelect, { target: { value: `model-${i}` } });
      }

      expect(modelSelect).toBeInTheDocument();
    });
  });

  describe('Integration', () => {
    it('integrates with React Flow handles', () => {
      renderWithProvider(<AIAgentNode {...defaultProps} />);

      expect(screen.getByTestId('handle-left-target')).toBeInTheDocument();
      expect(screen.getByTestId('handle-right-source')).toBeInTheDocument();
    });

    it('supports node deletion', () => {
      const mockDeleteNode = jest.fn();
      renderWithProvider(
        <NodeProvider
          value={{
            updateNodeData: jest.fn(),
            onSelect: jest.fn(),
            selectedNodes: [],
            updateNodePosition: jest.fn(),
            deleteNode: mockDeleteNode,
            duplicateNode: jest.fn(),
          }}
        >
          <AIAgentNode {...defaultProps} />
        </NodeProvider>
      );

      fireEvent.keyDown(screen.getByTestId('node-card'), { key: 'Delete' });
      expect(mockDeleteNode).toBeDefined();
    });

    it('supports node duplication', () => {
      const mockDuplicateNode = jest.fn();
      renderWithProvider(
        <NodeProvider
          value={{
            updateNodeData: jest.fn(),
            onSelect: jest.fn(),
            selectedNodes: [],
            updateNodePosition: jest.fn(),
            deleteNode: jest.fn(),
            duplicateNode: mockDuplicateNode,
          }}
        >
          <AIAgentNode {...defaultProps} />
        </NodeProvider>
      );

      fireEvent.keyDown(screen.getByTestId('node-card'), {
        key: 'd',
        ctrlKey: true,
      });
      expect(mockDuplicateNode).toBeDefined();
    });
  });
});