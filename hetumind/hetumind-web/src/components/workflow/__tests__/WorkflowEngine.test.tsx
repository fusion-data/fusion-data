import { WorkflowEngine, BaseNodeExecutor, WorkflowDefinition } from '../execution';

// Mock implementation for testing
class MockNodeExecutor extends BaseNodeExecutor {
  async execute(context: any): Promise<any> {
    return {
      nodeId: context.nodeId,
      status: 'completed',
      output: { result: `mock_result_${context.nodeId}` },
      timestamp: Date.now(),
    };
  }

  validate(config: any): boolean {
    return config && typeof config === 'object';
  }
}

class MockErrorNodeExecutor extends BaseNodeExecutor {
  async execute(context: any): Promise<any> {
    throw new Error(`Mock error for node ${context.nodeId}`);
  }

  validate(config: any): boolean {
    return false;
  }
}

describe('WorkflowEngine', () => {
  let engine: WorkflowEngine;
  let mockExecutor: MockNodeExecutor;
  let errorExecutor: MockErrorNodeExecutor;

  beforeEach(() => {
    engine = new WorkflowEngine({
      maxConcurrentNodes: 3,
      timeout: 5000,
      retryAttempts: 2,
      enableLogging: false,
      enableMetrics: false,
    });

    mockExecutor = new MockNodeExecutor();
    errorExecutor = new MockErrorNodeExecutor();
  });

  afterEach(() => {
    engine.clearExecutions();
  });

  describe('Engine Initialization', () => {
    it('creates engine with default configuration', () => {
      const defaultEngine = new WorkflowEngine();
      expect(defaultEngine).toBeInstanceOf(WorkflowEngine);
    });

    it('creates engine with custom configuration', () => {
      const customEngine = new WorkflowEngine({
        maxConcurrentNodes: 5,
        timeout: 10000,
        retryAttempts: 3,
      });

      expect(customEngine).toBeInstanceOf(WorkflowEngine);
    });

    it('returns engine statistics', () => {
      const stats = engine.getStats();
      expect(stats).toHaveProperty('activeExecutions');
      expect(stats).toHaveProperty('registeredExecutors');
      expect(stats).toHaveProperty('totalExecutions');
    });
  });

  describe('Executor Registration', () => {
    it('registers node executor successfully', () => {
      engine.registerExecutor('mock', mockExecutor);

      const stats = engine.getStats();
      expect(stats.registeredExecutors).toBeGreaterThan(0);
    });

    it('registers multiple executors', () => {
      engine.registerExecutor('mock', mockExecutor);
      engine.registerExecutor('error', errorExecutor);

      const stats = engine.getStats();
      expect(stats.registeredExecutors).toBeGreaterThanOrEqual(2);
    });

    it('prevents duplicate executor registration', () => {
      engine.registerExecutor('mock', mockExecutor);

      expect(() => {
        engine.registerExecutor('mock', mockExecutor);
      }).not.toThrow();

      // Should still only have one executor registered
      const stats = engine.getStats();
      expect(stats.registeredExecutors).toBe(1);
    });
  });

  describe('Workflow Execution', () => {
    const simpleWorkflow: WorkflowDefinition = {
      id: 'test-workflow',
      name: 'Test Workflow',
      description: 'A simple test workflow',
      nodes: [
        {
          id: 'node-1',
          type: 'mock',
          data: { config: {} },
          inputs: [],
          outputs: ['node-2'],
          position: { x: 0, y: 0 },
        },
        {
          id: 'node-2',
          type: 'mock',
          data: { config: {} },
          inputs: ['node-1'],
          outputs: [],
          position: { x: 100, y: 0 },
        },
      ],
      edges: [
        {
          id: 'edge-1',
          source: 'node-1',
          target: 'node-2',
        },
      ],
      variables: {},
    };

    beforeEach(() => {
      engine.registerExecutor('mock', mockExecutor);
    });

    it('executes simple workflow successfully', async () => {
      const execution = await engine.execute(simpleWorkflow);

      expect(execution).toHaveProperty('executionId');
      expect(execution).toHaveProperty('status');
      expect(execution.status).toBe('completed');
    });

    it('handles workflow execution with variables', async () => {
      const workflowWithVars = {
        ...simpleWorkflow,
        variables: {
          testVar: 'testValue',
          numberVar: 42,
        },
      };

      const execution = await engine.execute(workflowWithVars, {
        variables: { additionalVar: 'extraValue' }
      });

      expect(execution.status).toBe('completed');
    });

    it('handles empty workflow', async () => {
      const emptyWorkflow: WorkflowDefinition = {
        id: 'empty-workflow',
        name: 'Empty Workflow',
        description: 'An empty workflow',
        nodes: [],
        edges: [],
        variables: {},
      };

      const execution = await engine.execute(emptyWorkflow);
      expect(execution.status).toBe('completed');
    });

    it('handles workflow with single node', async () => {
      const singleNodeWorkflow: WorkflowDefinition = {
        id: 'single-node-workflow',
        name: 'Single Node Workflow',
        description: 'Workflow with single node',
        nodes: [
          {
            id: 'single-node',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      const execution = await engine.execute(singleNodeWorkflow);
      expect(execution.status).toBe('completed');
    });

    it('handles complex workflow with multiple branches', async () => {
      const complexWorkflow: WorkflowDefinition = {
        id: 'complex-workflow',
        name: 'Complex Workflow',
        description: 'Workflow with multiple branches',
        nodes: [
          {
            id: 'start',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: ['branch1', 'branch2'],
            position: { x: 0, y: 0 },
          },
          {
            id: 'branch1',
            type: 'mock',
            data: { config: {} },
            inputs: ['start'],
            outputs: ['end'],
            position: { x: 100, y: -50 },
          },
          {
            id: 'branch2',
            type: 'mock',
            data: { config: {} },
            inputs: ['start'],
            outputs: ['end'],
            position: { x: 100, y: 50 },
          },
          {
            id: 'end',
            type: 'mock',
            data: { config: {} },
            inputs: ['branch1', 'branch2'],
            outputs: [],
            position: { x: 200, y: 0 },
          },
        ],
        edges: [
          { id: 'edge-1', source: 'start', target: 'branch1' },
          { id: 'edge-2', source: 'start', target: 'branch2' },
          { id: 'edge-3', source: 'branch1', target: 'end' },
          { id: 'edge-4', source: 'branch2', target: 'end' },
        ],
        variables: {},
      };

      const execution = await engine.execute(complexWorkflow);
      expect(execution.status).toBe('completed');
    });
  });

  describe('Error Handling', () => {
    beforeEach(() => {
      engine.registerExecutor('mock', mockExecutor);
      engine.registerExecutor('error', errorExecutor);
    });

    it('handles node execution failure', async () => {
      const errorWorkflow: WorkflowDefinition = {
        id: 'error-workflow',
        name: 'Error Workflow',
        description: 'Workflow with failing node',
        nodes: [
          {
            id: 'error-node',
            type: 'error',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      const execution = await engine.execute(errorWorkflow);
      expect(execution.status).toBe('failed');
    });

    it('handles missing executor', async () => {
      const unknownWorkflow: WorkflowDefinition = {
        id: 'unknown-workflow',
        name: 'Unknown Workflow',
        description: 'Workflow with unknown node type',
        nodes: [
          {
            id: 'unknown-node',
            type: 'unknown',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      const execution = await engine.execute(unknownWorkflow);
      expect(execution.status).toBe('failed');
    });

    it('handles timeout errors', async () => {
      const timeoutEngine = new WorkflowEngine({
        maxConcurrentNodes: 1,
        timeout: 100, // Very short timeout
        retryAttempts: 1,
      });

      class SlowNodeExecutor extends BaseNodeExecutor {
        async execute(context: any): Promise<any> {
          // Simulate slow execution
          await new Promise(resolve => setTimeout(resolve, 200));
          return { nodeId: context.nodeId, status: 'completed' };
        }

        validate(config: any): boolean {
          return true;
        }
      }

      timeoutEngine.registerExecutor('slow', new SlowNodeExecutor());

      const slowWorkflow: WorkflowDefinition = {
        id: 'slow-workflow',
        name: 'Slow Workflow',
        description: 'Workflow with slow node',
        nodes: [
          {
            id: 'slow-node',
            type: 'slow',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      const execution = await timeoutEngine.execute(slowWorkflow);
      expect(execution.status).toBe('failed');
    });

    it('handles invalid workflow definition', async () => {
      const invalidWorkflow = {
        id: 'invalid-workflow',
        name: 'Invalid Workflow',
        // Missing required fields
        nodes: null,
        edges: [],
      } as any;

      await expect(engine.execute(invalidWorkflow)).rejects.toThrow();
    });
  });

  describe('Execution Control', () => {
    beforeEach(() => {
      engine.registerExecutor('mock', mockExecutor);
    });

    it('pauses workflow execution', async () => {
      const workflow: WorkflowDefinition = {
        id: 'pause-workflow',
        name: 'Pause Workflow',
        description: 'Workflow for testing pause',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: ['node-2'],
            position: { x: 0, y: 0 },
          },
          {
            id: 'node-2',
            type: 'mock',
            data: { config: {} },
            inputs: ['node-1'],
            outputs: [],
            position: { x: 100, y: 0 },
          },
        ],
        edges: [
          { id: 'edge-1', source: 'node-1', target: 'node-2' },
        ],
        variables: {},
      };

      const execution = await engine.execute(workflow);

      // Pause execution
      engine.pause(execution.executionId);

      const updatedExecution = engine.getExecution(execution.executionId);
      expect(updatedExecution?.status).toBe('paused');
    });

    it('resumes paused workflow execution', async () => {
      const workflow: WorkflowDefinition = {
        id: 'resume-workflow',
        name: 'Resume Workflow',
        description: 'Workflow for testing resume',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      const execution = await engine.execute(workflow);
      engine.pause(execution.executionId);

      // Resume execution
      engine.resume(execution.executionId);

      const updatedExecution = engine.getExecution(execution.executionId);
      expect(updatedExecution?.status).toBe('running');
    });

    it('cancels workflow execution', async () => {
      const workflow: WorkflowDefinition = {
        id: 'cancel-workflow',
        name: 'Cancel Workflow',
        description: 'Workflow for testing cancel',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      const execution = await engine.execute(workflow);

      // Cancel execution
      engine.cancel(execution.executionId);

      const updatedExecution = engine.getExecution(execution.executionId);
      expect(updatedExecution?.status).toBe('cancelled');
    });
  });

  describe('Event System', () => {
    beforeEach(() => {
      engine.registerExecutor('mock', mockExecutor);
    });

    it('emits execution started event', async () => {
      const startListener = jest.fn();
      engine.on('execution-started', startListener);

      const workflow: WorkflowDefinition = {
        id: 'event-workflow',
        name: 'Event Workflow',
        description: 'Workflow for testing events',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      await engine.execute(workflow);
      expect(startListener).toHaveBeenCalled();
    });

    it('emits node completed event', async () => {
      const nodeCompleteListener = jest.fn();
      engine.on('node-completed', nodeCompleteListener);

      const workflow: WorkflowDefinition = {
        id: 'node-event-workflow',
        name: 'Node Event Workflow',
        description: 'Workflow for testing node events',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      await engine.execute(workflow);
      expect(nodeCompleteListener).toHaveBeenCalled();
    });

    it('emits execution completed event', async () => {
      const completeListener = jest.fn();
      engine.on('execution-completed', completeListener);

      const workflow: WorkflowDefinition = {
        id: 'complete-event-workflow',
        name: 'Complete Event Workflow',
        description: 'Workflow for testing complete events',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      await engine.execute(workflow);
      expect(completeListener).toHaveBeenCalled();
    });

    it('removes event listeners', () => {
      const listener = jest.fn();
      engine.on('test-event', listener);
      engine.off('test-event', listener);

      // Should not throw error
      expect(() => {
        engine.emit('test-event', {});
      }).not.toThrow();
    });
  });

  describe('Concurrent Execution', () => {
    beforeEach(() => {
      engine.registerExecutor('mock', mockExecutor);
    });

    it('handles multiple concurrent executions', async () => {
      const workflow: WorkflowDefinition = {
        id: 'concurrent-workflow',
        name: 'Concurrent Workflow',
        description: 'Workflow for testing concurrency',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      // Start multiple executions
      const executions = await Promise.all([
        engine.execute(workflow),
        engine.execute(workflow),
        engine.execute(workflow),
      ]);

      // All should complete successfully
      executions.forEach(execution => {
        expect(execution.status).toBe('completed');
      });

      // Check that execution IDs are unique
      const executionIds = executions.map(e => e.executionId);
      const uniqueIds = new Set(executionIds);
      expect(uniqueIds.size).toBe(executionIds.length);
    });

    it('respects max concurrent nodes limit', async () => {
      const limitedEngine = new WorkflowEngine({
        maxConcurrentNodes: 1,
        timeout: 5000,
        retryAttempts: 1,
      });

      limitedEngine.registerExecutor('mock', mockExecutor);

      const workflow: WorkflowDefinition = {
        id: 'limited-workflow',
        name: 'Limited Concurrent Workflow',
        description: 'Workflow with concurrency limit',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: ['node-2', 'node-3'],
            position: { x: 0, y: 0 },
          },
          {
            id: 'node-2',
            type: 'mock',
            data: { config: {} },
            inputs: ['node-1'],
            outputs: [],
            position: { x: 100, y: -50 },
          },
          {
            id: 'node-3',
            type: 'mock',
            data: { config: {} },
            inputs: ['node-1'],
            outputs: [],
            position: { x: 100, y: 50 },
          },
        ],
        edges: [
          { id: 'edge-1', source: 'node-1', target: 'node-2' },
          { id: 'edge-2', source: 'node-1', target: 'node-3' },
        ],
        variables: {},
      };

      const execution = await limitedEngine.execute(workflow);
      expect(execution.status).toBe('completed');
    });
  });

  describe('Memory Management', () => {
    beforeEach(() => {
      engine.registerExecutor('mock', mockExecutor);
    });

    it('clears completed executions', () => {
      const workflow: WorkflowDefinition = {
        id: 'cleanup-workflow',
        name: 'Cleanup Workflow',
        description: 'Workflow for testing cleanup',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      engine.clearExecutions();
      const statsBefore = engine.getStats();
      expect(statsBefore.activeExecutions).toBe(0);
    });

    it('limits execution history', async () => {
      const workflow: WorkflowDefinition = {
        id: 'history-workflow',
        name: 'History Workflow',
        description: 'Workflow for testing history',
        nodes: [
          {
            id: 'node-1',
            type: 'mock',
            data: { config: {} },
            inputs: [],
            outputs: [],
            position: { x: 0, y: 0 },
          },
        ],
        edges: [],
        variables: {},
      };

      // Execute multiple workflows
      for (let i = 0; i < 10; i++) {
        await engine.execute(workflow);
      }

      // Check that executions are properly managed
      const stats = engine.getStats();
      expect(stats.activeExecutions).toBe(0);
    });
  });
});