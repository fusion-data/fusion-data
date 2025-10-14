export interface EngineOptimizationConfig {
  concurrency: {
    maxConcurrentNodes: number;
    maxConcurrentWorkflows: number;
    enableBatching: boolean;
    batchSize: number;
  };
  cache: {
    enableCache: boolean;
    maxCacheSize: number;
    cacheTimeout: number;
    strategy: 'lru' | 'fifo' | 'lfu';
  };
  memory: {
    enableMemoryOptimization: boolean;
    maxMemoryUsage: number;
    gcInterval: number;
    enableWeakReferences: boolean;
  };
  execution: {
    enablePreemption: boolean;
    timeoutStrategy: 'fail' | 'retry' | 'skip';
    retryAttempts: number;
    retryDelay: number;
  };
  monitoring: {
    enableMetrics: boolean;
    metricsInterval: number;
    enableProfiling: boolean;
    enableTracing: boolean;
  };
}

export interface CanvasOptimizationConfig {
  rendering: {
    enableVirtualization: boolean;
    virtualizationThreshold: number;
    enableMinimap: boolean;
    minimapPosition: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
    renderQuality: 'low' | 'medium' | 'high';
    enableSmoothPan: boolean;
    enableSmoothZoom: boolean;
  };
  performance: {
    enableCaching: boolean;
    cacheStrategy: 'node' | 'edge' | 'both';
    maxCacheSize: number;
    enableDebouncing: boolean;
    debounceDelay: number;
    enableBatchUpdates: boolean;
    batchSize: number;
  };
  interaction: {
    enableDragOptimization: boolean;
    enableConnectionOptimization: boolean;
    enableSelectionOptimization: boolean;
    maxSelectableNodes: number;
    enableHoverEffects: boolean;
    enableAnimation: boolean;
  };
  memory: {
    enableMemoryManagement: boolean;
    maxMemoryUsage: number;
    enableLazyLoading: boolean;
    enableNodePooling: boolean;
    cleanupInterval: number;
  };
}