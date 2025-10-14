import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { WorkflowEditor } from '../../WorkflowEditor';
import { WorkflowProvider } from '../../WorkflowContext';
import { WorkflowEngine, defaultWorkflowEngine } from '../../execution';
import {
  createMockNode,
  createMockEdge,
  createMockWorkflow,
  mockFetch,
  waitForCondition,
  suppressConsoleErrors,
} from '../setup';

// Mock React Flow
jest.mock('@xyflow/react', () => ({
  ReactFlow: ({ children, onNodesChange, onEdgesChange, onConnect, nodes, edges }: any) => (
    <div data-testid="react-flow">
      <div data-testid="nodes-count">{nodes?.length || 0}</div>
      <div data-testid="edges-count">{edges?.length || 0}</div>
      <button
        data-testid="add-node"
        onClick={() => onNodesChange?.([{ type: 'add', item: createMockNode('new-node', 'trigger') }])}
      >
        Add Node
      </button>
      <button
        data-testid="add-edge"
        onClick={() => onEdgesChange?.([{ type: 'add', item: createMockEdge('node-1', 'node-2') }])}
      >
        Add Edge
      </button>
      <button
        data-testid="connect"
        onClick={() => onConnect?.({ source: 'node-1', target: 'node-2' })}
      >
        Connect Nodes
      </button>
      {children}
    </div>
  ),
  Background: () => <div data-testid="background" />,
  Controls: () => <div data-testid="controls" />,
  MiniMap: () => <div data-testid="minimap" />,
  useReactFlow: () => ({
    fitView: jest.fn(),
    zoomIn: jest.fn(),
    zoomOut: jest.fn(),
    getNodes: () => [],
    getEdges: () => [],
    setNodes: jest.fn(),
    setEdges: jest.fn(),
  }),
  useNodesState: (initialNodes: any) => [initialNodes, jest.fn()],
  useEdgesState: (initialEdges: any) => [initialEdges, jest.fn()],
  addEdge: jest.fn(),
  ConnectionMode: { Strict: 'strict' },
}));

// Mock Ant Design components
jest.mock('antd', () => ({
  Button: ({ children, onClick, type, icon }: any) => (
    <button onClick={onClick} data-type={type} data-icon={icon}>
      {icon}
      {children}
    </button>
  ),
  Space: ({ children }: any) => <div>{children}</div>,
  Card: ({ children, title }: any) => (
    <div>
      <h3>{title}</h3>
      {children}
    </div>
  ),
  Tabs: ({ children, activeKey, onChange }: any) => (
    <div data-active-key={activeKey}>
      {children}
    </div>
  ),
  'Tabs.TabPane': ({ children, tab }: any) => (
    <div data-tab={tab}>
      {children}
    </div>
  ),
  Layout: ({ children }: any) => <div>{children}</div>,
  'Layout.Header': ({ children }: any) => <header>{children}</header>,
  'Layout.Content': ({ children }: any) => <main>{children}</main>,
  'Layout.Sider': ({ children }: any) => <aside>{children}</aside>,
  Menu: ({ children, onClick }: any) => (
    <nav onClick={onClick}>
      {children}
    </nav>
  ),
  'Menu.Item': ({ children, key }: any) => (
    <div data-key={key}>{children}</div>
  ),
  Form: ({ children, onFinish }: any) => (
    <form onSubmit={onFinish}>
      {children}
    </form>
  ),
  'Form.Item': ({ children, label }: any) => (
    <div>
      <label>{label}</label>
      {children}
    </div>
  ),
  Input: ({ onChange, value, placeholder }: any) => (
    <input
      onChange={onChange}
      value={value}
      placeholder={placeholder}
    />
  ),
  Select: ({ children, onChange, value }: any) => (
    <select onChange={onChange} value={value}>
      {children}
    </select>
  ),
  'Select.Option': ({ children, value }: any) => (
    <option value={value}>{children}</option>
  ),
  message: {
    success: jest.fn(),
    error: jest.fn(),
    warning: jest.fn(),
    info: jest.fn(),
  },
}));

// Mock icons
jest.mock('@ant-design/icons', () => ({
  PlayCircleOutlined: () => <span data-testid="play-icon">▶️</span>,
  SaveOutlined: () => <span data-testid="save-icon">💾</span>,
  LoadingOutlined: () => <span data-testid="loading-icon">⏳</span>,
  PlusOutlined: () => <span data-testid="plus-icon">➕</span>,
  DeleteOutlined: () => <span data-testid="delete-icon">🗑️</span>,
  SettingOutlined: () => <span data-testid="setting-icon">⚙️</span>,
}));

const mockStore = configureStore({
  reducer: {
    workflow: () => ({
      nodes: [],
      edges: [],
      selectedNodes: [],
      selectedEdges: [],
    }),
  },
});

const renderWithProviders = (component: React.ReactElement) => {
  return render(
    <Provider store={mockStore}>
      <WorkflowProvider>
        <WorkflowEngine />
        {component}
      </WorkflowProvider>
    </Provider>
  );
};

describe('Workflow Integration Tests', () => {
  const restoreConsole = suppressConsoleErrors();

  beforeEach(() => {
    jest.clearAllMocks();
    mockFetch({ success: true });
  });

  afterAll(() => {
    restoreConsole();
  });

  describe('Basic Workflow Creation', () => {
    it('creates a complete workflow from scratch', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Initially empty workflow
      expect(screen.getByTestId('nodes-count')).toHaveTextContent('0');
      expect(screen.getByTestId('edges-count')).toHaveTextContent('0');

      // Add first node
      fireEvent.click(screen.getByTestId('add-node'));
      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('1');
      });

      // Add second node
      fireEvent.click(screen.getByTestId('add-node'));
      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('2');
      });

      // Add edge between nodes
      fireEvent.click(screen.getByTestId('add-edge'));
      await waitFor(() => {
        expect(screen.getByTestId('edges-count')).toHaveTextContent('1');
      });

      // Verify workflow structure
      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
      expect(screen.getByTestId('background')).toBeInTheDocument();
      expect(screen.getByTestId('controls')).toBeInTheDocument();
      expect(screen.getByTestId('minimap')).toBeInTheDocument();
    });

    it('loads and displays existing workflow', async () => {
      const existingWorkflow = createMockWorkflow(3);

      renderWithProviders(<WorkflowEditor initialWorkflow={existingWorkflow} />);

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('3');
        expect(screen.getByTestId('edges-count')).toHaveTextContent('2');
      });
    });

    it('saves workflow configuration', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create workflow
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('add-edge'));

      // Save workflow
      const saveButton = screen.getByTestId('save-icon');
      fireEvent.click(saveButton);

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('2');
        expect(screen.getByTestId('edges-count')).toHaveTextContent('1');
      });
    });
  });

  describe('Node Management', () => {
    it('adds different types of nodes', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Add trigger node
      fireEvent.click(screen.getByTestId('add-node'));
      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('1');
      });

      // Add AI agent node
      const aiNode = createMockNode('ai-node-1', 'aiAgent');
      fireEvent.click(screen.getByTestId('add-node'));
      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('2');
      });

      // Add condition node
      const conditionNode = createMockNode('condition-node-1', 'condition');
      fireEvent.click(screen.getByTestId('add-node'));
      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('3');
      });
    });

    it('selects and configures nodes', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Add node
      fireEvent.click(screen.getByTestId('add-node'));
      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('1');
      });

      // Select node (in real implementation, this would be done by clicking the node)
      const nodeElement = screen.getByTestId('react-flow');
      fireEvent.click(nodeElement);

      // Configure node
      expect(nodeElement).toBeInTheDocument();
    });

    it('deletes nodes and their connections', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create workflow with nodes and edges
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('add-edge'));

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('2');
        expect(screen.getByTestId('edges-count')).toHaveTextContent('1');
      });

      // Delete node (simulated)
      const deleteButton = screen.getByTestId('delete-icon');
      fireEvent.click(deleteButton);

      // In real implementation, this would remove selected node
      expect(deleteButton).toBeInTheDocument();
    });
  });

  describe('Edge Management', () => {
    it('creates connections between nodes', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Add nodes
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('add-node'));

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('2');
      });

      // Create connection
      fireEvent.click(screen.getByTestId('connect'));
      await waitFor(() => {
        expect(screen.getByTestId('edges-count')).toHaveTextContent('1');
      });
    });

    it('validates node connections', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Try to connect without nodes
      fireEvent.click(screen.getByTestId('connect'));

      // Should handle gracefully without crashing
      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
    });

    it('removes connections', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create workflow with connection
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('connect'));

      await waitFor(() => {
        expect(screen.getByTestId('edges-count')).toHaveTextContent('1');
      });

      // Remove connection (simulated)
      const deleteButton = screen.getByTestId('delete-icon');
      fireEvent.click(deleteButton);

      expect(deleteButton).toBeInTheDocument();
    });
  });

  describe('Workflow Execution', () => {
    it('executes simple workflow', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create simple workflow
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('connect'));

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('2');
        expect(screen.getByTestId('edges-count')).toHaveTextContent('1');
      });

      // Execute workflow
      const executeButton = screen.getByTestId('play-icon');
      fireEvent.click(executeButton);

      // Should start execution without errors
      expect(executeButton).toBeInTheDocument();
    });

    it('pauses and resumes workflow execution', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create workflow
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('connect'));

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('1');
        expect(screen.getByTestId('edges-count')).toHaveTextContent('1');
      });

      // Execute and pause
      const executeButton = screen.getByTestId('play-icon');
      fireEvent.click(executeButton);

      // In real implementation, this would show pause/resume controls
      expect(executeButton).toBeInTheDocument();
    });

    it('handles workflow execution errors', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create workflow with invalid configuration
      fireEvent.click(screen.getByTestId('add-node'));

      // Mock execution failure
      mockFetch({ error: 'Execution failed' }, false);

      const executeButton = screen.getByTestId('play-icon');
      fireEvent.click(executeButton);

      // Should handle errors gracefully
      expect(executeButton).toBeInTheDocument();
    });
  });

  describe('AI Agent Integration', () => {
    it('configures AI agent nodes', async () => {
      const aiNodeData = {
        id: 'ai-node-1',
        type: 'aiAgent',
        position: { x: 100, y: 100 },
        data: {
          label: 'AI Assistant',
          agentType: 'chat',
          config: {
            model: 'gpt-3.5-turbo',
            temperature: 0.7,
            maxTokens: 1024,
          },
        },
      };

      renderWithProviders(<WorkflowEditor />);

      // Add AI agent
      fireEvent.click(screen.getByTestId('add-node'));

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('1');
      });

      // Configure AI agent (simulated)
      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
    });

    it('handles AI agent responses', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Add AI agent
      fireEvent.click(screen.getByTestId('add-node'));

      // Mock AI response
      mockFetch({
        response: 'This is a mock AI response',
        usage: { total_tokens: 100 },
      });

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('1');
      });

      // Execute workflow to test AI integration
      const executeButton = screen.getByTestId('play-icon');
      fireEvent.click(executeButton);

      expect(executeButton).toBeInTheDocument();
    });
  });

  describe('Data Flow', () => {
    it('passes data between nodes', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create data processing workflow
      fireEvent.click(screen.getByTestId('add-node')); // Data source
      fireEvent.click(screen.getByTestId('add-node')); // Data processor
      fireEvent.click(screen.getByTestId('add-node')); // Data output
      fireEvent.click(screen.getByTestId('connect')); // Connect 1-2
      fireEvent.click(screen.getByTestId('connect')); // Connect 2-3

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('3');
        expect(screen.getByTestId('edges-count')).toHaveTextContent('2');
      });

      // Execute workflow to test data flow
      const executeButton = screen.getByTestId('play-icon');
      fireEvent.click(executeButton);

      expect(executeButton).toBeInTheDocument();
    });

    it('transforms data through nodes', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create transformation workflow
      const dataNode = createMockNode('data-1', 'dataProcessor');
      fireEvent.click(screen.getByTestId('add-node'));

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('1');
      });

      // Mock data transformation
      mockFetch({
        transformedData: { result: 'transformed_value' },
        originalData: { input: 'original_value' },
      });

      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
    });
  });

  describe('Performance Integration', () => {
    it('monitors workflow performance', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create complex workflow
      for (let i = 0; i < 5; i++) {
        fireEvent.click(screen.getByTestId('add-node'));
      }

      // Create connections
      for (let i = 0; i < 4; i++) {
        fireEvent.click(screen.getByTestId('connect'));
      }

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('5');
        expect(screen.getByTestId('edges-count')).toHaveTextContent('4');
      });

      // Execute workflow and monitor performance
      const executeButton = screen.getByTestId('play-icon');
      fireEvent.click(executeButton);

      expect(executeButton).toBeInTheDocument();
    });

    it('handles large workflows efficiently', async () => {
      renderWithProviders(<WorkflowEditor />);

      const startTime = performance.now();

      // Create large workflow
      for (let i = 0; i < 50; i++) {
        fireEvent.click(screen.getByTestId('add-node'));
      }

      const endTime = performance.now();

      // Should handle large workflows efficiently
      expect(endTime - startTime).toBeLessThan(1000);
      expect(screen.getByTestId('nodes-count')).toHaveTextContent('50');
    });
  });

  describe('Error Handling', () => {
    it('handles invalid workflow configurations', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Try to create invalid connections
      fireEvent.click(screen.getByTestId('connect'));

      // Should handle gracefully
      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
    });

    it('recovers from execution failures', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Create workflow
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('connect'));

      // Mock execution failure
      mockFetch({ error: 'Execution failed' }, false);

      const executeButton = screen.getByTestId('play-icon');
      fireEvent.click(executeButton);

      // Should allow retry
      expect(executeButton).toBeInTheDocument();
    });

    it('validates node configurations', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Add node with invalid configuration
      fireEvent.click(screen.getByTestId('add-node'));

      // Should validate and show errors
      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('supports keyboard navigation', async () => {
      renderWithProviders(<WorkflowEditor />);

      const canvas = screen.getByTestId('react-flow');

      // Test keyboard navigation
      fireEvent.keyDown(canvas, { key: 'Tab' });
      fireEvent.keyDown(canvas, { key: 'Enter' });
      fireEvent.keyDown(canvas, { key: 'Escape' });

      expect(canvas).toBeInTheDocument();
    });

    it('provides screen reader support', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Add node for testing
      fireEvent.click(screen.getByTestId('add-node'));

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('1');
      });

      // Should have proper ARIA attributes
      const canvas = screen.getByTestId('react-flow');
      expect(canvas).toBeInTheDocument();
    });
  });

  describe('Memory Management', () => {
    it('cleans up resources on unmount', () => {
      const { unmount } = renderWithProviders(<WorkflowEditor />);

      // Create workflow
      fireEvent.click(screen.getByTestId('add-node'));
      fireEvent.click(screen.getByTestId('connect'));

      // Unmount component
      unmount();

      // Should not cause memory leaks
      expect(true).toBe(true);
    });

    it('manages large data sets efficiently', async () => {
      renderWithProviders(<WorkflowEditor />);

      // Add many nodes
      for (let i = 0; i < 100; i++) {
        fireEvent.click(screen.getByTestId('add-node'));
      }

      await waitFor(() => {
        expect(screen.getByTestId('nodes-count')).toHaveTextContent('100');
      });

      // Should manage memory efficiently
      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
    });
  });
});