import '@testing-library/jest-dom';

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: jest.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(), // deprecated
    removeListener: jest.fn(), // deprecated
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
});

// Mock ResizeObserver
global.ResizeObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));

// Mock IntersectionObserver
global.IntersectionObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));

// Mock requestAnimationFrame
global.requestAnimationFrame = jest.fn(cb => setTimeout(cb, 16));
global.cancelAnimationFrame = jest.fn(id => clearTimeout(id));

// Mock performance API
Object.defineProperty(global.performance, 'now', {
  writable: true,
  value: jest.fn(() => Date.now()),
});

// Mock performance.memory for memory testing
Object.defineProperty(global.performance, 'memory', {
  writable: true,
  value: {
    usedJSHeapSize: 50 * 1024 * 1024, // 50MB
    totalJSHeapSize: 100 * 1024 * 1024, // 100MB
    jsHeapSizeLimit: 2048 * 1024 * 1024, // 2GB
  },
});

// Mock console methods to reduce noise in tests
const originalError = console.error;
const originalWarn = console.warn;

beforeAll(() => {
  console.error = (...args: any[]) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('Warning: ReactDOM.render is deprecated')
    ) {
      return;
    }
    originalError.call(console, ...args);
  };

  console.warn = (...args: any[]) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('componentWillReceiveProps')
    ) {
      return;
    }
    originalWarn.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
  console.warn = originalWarn;
});

// Global test utilities
export const createMockNode = (id: string, type: string, position = { x: 0, y: 0 }) => ({
  id,
  type,
  position,
  data: {
    label: `${type}-${id}`,
    config: {},
  },
  inputs: [],
  outputs: [],
});

export const createMockEdge = (source: string, target: string) => ({
  id: `${source}-${target}`,
  source,
  target,
});

export const createMockWorkflow = (nodeCount = 2) => {
  const nodes = Array.from({ length: nodeCount }, (_, i) =>
    createMockNode(`node-${i}`, 'mock', { x: i * 100, y: 0 })
  );

  const edges = [];
  for (let i = 0; i < nodeCount - 1; i++) {
    edges.push(createMockEdge(`node-${i}`, `node-${i + 1}`));
  }

  return {
    id: 'test-workflow',
    name: 'Test Workflow',
    description: 'Test workflow for unit tests',
    nodes,
    edges,
    variables: {},
  };
};

export const createMockEngineConfig = () => ({
  maxConcurrentNodes: 3,
  timeout: 5000,
  retryAttempts: 2,
  enableLogging: false,
  enableMetrics: false,
});

export const mockFetch = (response: any, ok = true) => {
  global.fetch = jest.fn(() =>
    Promise.resolve({
      ok,
      json: () => Promise.resolve(response),
      text: () => Promise.resolve(JSON.stringify(response)),
    } as Response)
  );
};

export const createMockPerformanceMetrics = () => ({
  workflowEngine: {
    executionTime: Math.random() * 1000,
    memoryUsage: Math.random() * 100,
    cacheHitRate: Math.random() * 100,
    concurrentWorkflows: Math.floor(Math.random() * 10),
    throughput: Math.random() * 100,
  },
  canvas: {
    renderTime: Math.random() * 16,
    nodeCount: Math.floor(Math.random() * 200),
    edgeCount: Math.floor(Math.random() * 300),
    fps: Math.random() * 60,
    memoryUsage: Math.random() * 100,
  },
  dataProcessing: {
    processingTime: Math.random() * 500,
    throughput: Math.random() * 1000,
    queueSize: Math.floor(Math.random() * 100),
    errorRate: Math.random() * 10,
  },
  system: {
    cpuUsage: Math.random() * 100,
    memoryUsage: Math.random() * 100,
    networkLatency: Math.random() * 200,
    diskIO: Math.random() * 500,
  },
});

export const waitForCondition = async (
  condition: () => boolean,
  timeout = 5000,
  interval = 100
): Promise<void> => {
  const startTime = Date.now();

  while (!condition() && Date.now() - startTime < timeout) {
    await new Promise(resolve => setTimeout(resolve, interval));
  }

  if (!condition()) {
    throw new Error(`Condition not met within ${timeout}ms`);
  }
};

export const flushPromises = () => new Promise(resolve => setTimeout(resolve, 0));

export const suppressConsoleErrors = () => {
  const originalError = console.error;
  console.error = jest.fn();

  return () => {
    console.error = originalError;
  };
};

// Test constants
export const TEST_IDS = {
  NODE_CARD: 'node-card',
  NODE_TITLE: 'node-title',
  CANVAS: 'react-flow',
  TOOLBAR: 'workflow-toolbar',
  PROPERTY_PANEL: 'property-panel',
  PERFORMANCE_CARD: 'performance-card',
  EXECUTION_MONITOR: 'execution-monitor',
} as const;

export const TEST_TIMEOUTS = {
  SHORT: 100,
  MEDIUM: 1000,
  LONG: 5000,
  EXTRA_LONG: 10000,
} as const;