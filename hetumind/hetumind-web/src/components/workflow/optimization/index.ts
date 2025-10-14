export { default as PerformanceOptimizer } from './PerformanceOptimizer';
export { default as WorkflowEngineOptimizer } from './WorkflowEngineOptimizer';
export { default as CanvasOptimizer } from './CanvasOptimizer';
export { default as PerformanceHub } from './PerformanceHub';

// Types
export type {
  EngineOptimizationConfig,
  CanvasOptimizationConfig,
} from './types';

// Performance monitoring utilities
export class PerformanceMonitor {
  private static instance: PerformanceMonitor;
  private metrics: Map<string, number[]> = new Map();
  private observers: PerformanceObserver[] = [];

  static getInstance(): PerformanceMonitor {
    if (!PerformanceMonitor.instance) {
      PerformanceMonitor.instance = new PerformanceMonitor();
    }
    return PerformanceMonitor.instance;
  }

  startMonitoring(name: string): () => void {
    const startTime = performance.now();

    return () => {
      const endTime = performance.now();
      const duration = endTime - startTime;

      if (!this.metrics.has(name)) {
        this.metrics.set(name, []);
      }

      const values = this.metrics.get(name)!;
      values.push(duration);

      // Keep only last 100 measurements
      if (values.length > 100) {
        values.shift();
      }
    };
  }

  getMetrics(name: string): { avg: number; min: number; max: number; count: number } | null {
    const values = this.metrics.get(name);
    if (!values || values.length === 0) {
      return null;
    }

    const avg = values.reduce((sum, val) => sum + val, 0) / values.length;
    const min = Math.min(...values);
    const max = Math.max(...values);

    return { avg, min, max, count: values.length };
  }

  getAllMetrics(): Record<string, { avg: number; min: number; max: number; count: number }> {
    const result: Record<string, { avg: number; min: number; max: number; count: number }> = {};

    for (const [name] of this.metrics) {
      const metrics = this.getMetrics(name);
      if (metrics) {
        result[name] = metrics;
      }
    }

    return result;
  }

  clearMetrics(name?: string): void {
    if (name) {
      this.metrics.delete(name);
    } else {
      this.metrics.clear();
    }
  }

  // Memory monitoring
  getMemoryUsage(): { used: number; total: number; percentage: number } | null {
    if ('memory' in performance) {
      const memory = (performance as any).memory;
      const used = memory.usedJSHeapSize / 1024 / 1024; // MB
      const total = memory.totalJSHeapSize / 1024 / 1024; // MB
      const percentage = (used / total) * 100;

      return { used, total, percentage };
    }
    return null;
  }

  // FPS monitoring
  startFPSMonitoring(): () => void {
    let frameCount = 0;
    let lastTime = performance.now();
    let fps = 0;

    const measureFPS = () => {
      frameCount++;
      const currentTime = performance.now();
      const deltaTime = currentTime - lastTime;

      if (deltaTime >= 1000) {
        fps = (frameCount * 1000) / deltaTime;
        frameCount = 0;
        lastTime = currentTime;

        if (!this.metrics.has('fps')) {
          this.metrics.set('fps', []);
        }

        const values = this.metrics.get('fps')!;
        values.push(fps);

        if (values.length > 100) {
          values.shift();
        }
      }

      requestAnimationFrame(measureFPS);
    };

    const animationId = requestAnimationFrame(measureFPS);

    return () => {
      cancelAnimationFrame(animationId);
    };
  }
}

// Performance optimization utilities
export class PerformanceOptimizerUtils {
  // Debounce utility
  static debounce<T extends (...args: any[]) => any>(
    func: T,
    delay: number
  ): (...args: Parameters<T>) => void {
    let timeoutId: NodeJS.Timeout;
    return (...args: Parameters<T>) => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(() => func(...args), delay);
    };
  }

  // Throttle utility
  static throttle<T extends (...args: any[]) => any>(
    func: T,
    delay: number
  ): (...args: Parameters<T>) => void {
    let lastCall = 0;
    return (...args: Parameters<T>) => {
      const now = Date.now();
      if (now - lastCall >= delay) {
        lastCall = now;
        func(...args);
      }
    };
  }

  // Memoize utility
  static memoize<T extends (...args: any[]) => any>(
    func: T,
    getKey?: (...args: Parameters<T>) => string
  ): T {
    const cache = new Map<string, ReturnType<T>>();

    return ((...args: Parameters<T>) => {
      const key = getKey ? getKey(...args) : JSON.stringify(args);

      if (cache.has(key)) {
        return cache.get(key);
      }

      const result = func(...args);
      cache.set(key, result);

      // Limit cache size
      if (cache.size > 1000) {
        const firstKey = cache.keys().next().value;
        cache.delete(firstKey);
      }

      return result;
    }) as T;
  }

  // Lazy loading utility
  static lazyLoad<T>(
    loader: () => Promise<T>
  ): () => Promise<T> {
    let instance: T | null = null;
    let loading: Promise<T> | null = null;

    return () => {
      if (instance) {
        return Promise.resolve(instance);
      }

      if (loading) {
        return loading;
      }

      loading = loader().then(result => {
        instance = result;
        loading = null;
        return result;
      });

      return loading;
    };
  }

  // Batch updates utility
  static batchUpdates<T>(
    items: T[],
    batchSize: number,
    processor: (batch: T[]) => Promise<void>
  ): Promise<void> {
    return new Promise((resolve, reject) => {
      let index = 0;

      const processBatch = async () => {
        try {
          const batch = items.slice(index, index + batchSize);
          if (batch.length === 0) {
            resolve();
            return;
          }

          await processor(batch);
          index += batchSize;

          // Use setTimeout to yield control to the browser
          setTimeout(processBatch, 0);
        } catch (error) {
          reject(error);
        }
      };

      processBatch();
    });
  }

  // Virtual scrolling utility
  static calculateVisibleItems<T>(
    items: T[],
    containerHeight: number,
    itemHeight: number,
    scrollTop: number,
    buffer: number = 5
  ): { items: T[]; startIndex: number; endIndex: number } {
    const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - buffer);
    const endIndex = Math.min(
      items.length - 1,
      Math.ceil((scrollTop + containerHeight) / itemHeight) + buffer
    );

    return {
      items: items.slice(startIndex, endIndex + 1),
      startIndex,
      endIndex,
    };
  }
}

import { useState, useEffect } from 'react';

// React hooks for performance monitoring
export const usePerformanceMonitor = (name: string) => {
  const monitor = PerformanceMonitor.getInstance();

  return {
    startTiming: () => monitor.startMonitoring(name),
    getMetrics: () => monitor.getMetrics(name),
    clearMetrics: () => monitor.clearMetrics(name),
  };
};

export const useMemoryMonitor = () => {
  const [memoryUsage, setMemoryUsage] = useState(() =>
    PerformanceMonitor.getInstance().getMemoryUsage()
  );

  useEffect(() => {
    const interval = setInterval(() => {
      setMemoryUsage(PerformanceMonitor.getInstance().getMemoryUsage());
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return memoryUsage;
};

export const useFPSMonitor = () => {
  const [fps, setFps] = useState<number>(0);
  const monitor = PerformanceMonitor.getInstance();

  useEffect(() => {
    const stopMonitoring = monitor.startFPSMonitoring();

    const interval = setInterval(() => {
      const metrics = monitor.getMetrics('fps');
      if (metrics) {
        setFps(metrics.avg);
      }
    }, 1000);

    return () => {
      clearInterval(interval);
      stopMonitoring();
    };
  }, []);

  return fps;
};