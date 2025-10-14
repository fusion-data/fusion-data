import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { ExecutionState, Execution, ExecutionLog, ExecutionMetrics } from './types';

interface ExecutionStore extends ExecutionState {
  // Basic actions
  setExecutions: (executions: Execution[]) => void;
  setCurrentExecution: (execution: Execution | null) => void;
  setLogs: (logs: ExecutionLog[]) => void;
  setMetrics: (metrics: ExecutionMetrics) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Execution management
  addExecution: (execution: Execution) => void;
  updateExecution: (id: string, updates: Partial<Execution>) => void;
  deleteExecution: (id: string) => void;

  // Log management
  addLog: (log: ExecutionLog) => void;
  addLogs: (logs: ExecutionLog[]) => void;
  clearLogs: () => void;

  // Filter and pagination
  setFilters: (filters: Record<string, any>) => void;
  updateFilter: (key: string, value: any) => void;
  clearFilters: () => void;
  setPagination: (pagination: Partial<ExecutionState['pagination']>) => void;
  setPage: (page: number) => void;
  setPageSize: (pageSize: number) => void;

  // Real-time updates
  startExecution: (execution: Execution) => void;
  updateExecutionStatus: (id: string, status: Execution['status']) => void;
  addExecutionLog: (log: ExecutionLog) => void;
  completeExecution: (id: string, result: { output?: any; metrics?: ExecutionMetrics }) => void;
  failExecution: (id: string, error: string) => void;

  // API calls
  fetchExecutions: () => Promise<void>;
  fetchExecution: (id: string) => Promise<void>;
  fetchLogs: (executionId: string) => Promise<void>;
  startWorkflowExecution: (workflowId: string, input?: any) => Promise<Execution>;
  cancelExecution: (id: string) => Promise<void>;
  retryExecution: (id: string) => Promise<void>;

  // Utility actions
  reset: () => void;
}

const initialState: ExecutionState = {
  loading: false,
  error: null,
  lastUpdated: null,
  executions: [],
  currentExecution: null,
  logs: [],
  metrics: {
    duration: 0,
    nodeExecutions: 0,
    successCount: 0,
    errorCount: 0,
    retryCount: 0,
  },
  filters: {},
  pagination: {
    current: 1,
    pageSize: 20,
    total: 0,
  },
};

export const useExecutionStore = create<ExecutionStore>()(
  immer((set, get) => ({
    ...initialState,

    // Basic actions
    setExecutions: (executions) =>
      set((state) => {
        state.executions = executions;
        state.pagination.total = executions.length;
        state.lastUpdated = Date.now();
      }),

    setCurrentExecution: (execution) =>
      set((state) => {
        state.currentExecution = execution;
        state.lastUpdated = Date.now();
      }),

    setLogs: (logs) =>
      set((state) => {
        state.logs = logs;
        state.lastUpdated = Date.now();
      }),

    setMetrics: (metrics) =>
      set((state) => {
        state.metrics = metrics;
        state.lastUpdated = Date.now();
      }),

    setLoading: (loading) =>
      set((state) => {
        state.loading = loading;
      }),

    setError: (error) =>
      set((state) => {
        state.error = error;
        state.loading = false;
      }),

    // Execution management
    addExecution: (execution) =>
      set((state) => {
        state.executions.unshift(execution);
        state.pagination.total = state.executions.length;
        state.lastUpdated = Date.now();
      }),

    updateExecution: (id, updates) =>
      set((state) => {
        const index = state.executions.findIndex((e) => e.id === id);
        if (index !== -1) {
          Object.assign(state.executions[index], updates);
        }
        if (state.currentExecution?.id === id) {
          Object.assign(state.currentExecution, updates);
        }
        state.lastUpdated = Date.now();
      }),

    deleteExecution: (id) =>
      set((state) => {
        state.executions = state.executions.filter((e) => e.id !== id);
        state.pagination.total = state.executions.length;
        if (state.currentExecution?.id === id) {
          state.currentExecution = null;
        }
        state.lastUpdated = Date.now();
      }),

    // Log management
    addLog: (log) =>
      set((state) => {
        state.logs.push(log);
        state.lastUpdated = Date.now();
      }),

    addLogs: (logs) =>
      set((state) => {
        state.logs.push(...logs);
        state.lastUpdated = Date.now();
      }),

    clearLogs: () =>
      set((state) => {
        state.logs = [];
        state.lastUpdated = Date.now();
      }),

    // Filter and pagination
    setFilters: (filters) =>
      set((state) => {
        state.filters = filters;
        state.pagination.current = 1; // 重置到第一页
        state.lastUpdated = Date.now();
      }),

    updateFilter: (key, value) =>
      set((state) => {
        state.filters[key] = value;
        state.pagination.current = 1; // 重置到第一页
        state.lastUpdated = Date.now();
      }),

    clearFilters: () =>
      set((state) => {
        state.filters = {};
        state.pagination.current = 1; // 重置到第一页
        state.lastUpdated = Date.now();
      }),

    setPagination: (pagination) =>
      set((state) => {
        Object.assign(state.pagination, pagination);
        state.lastUpdated = Date.now();
      }),

    setPage: (page) =>
      set((state) => {
        state.pagination.current = page;
        state.lastUpdated = Date.now();
      }),

    setPageSize: (pageSize) =>
      set((state) => {
        state.pagination.pageSize = pageSize;
        state.pagination.current = 1; // 重置到第一页
        state.lastUpdated = Date.now();
      }),

    // Real-time updates
    startExecution: (execution) =>
      set((state) => {
        state.executions.unshift(execution);
        state.currentExecution = execution;
        state.logs = [];
        state.lastUpdated = Date.now();
      }),

    updateExecutionStatus: (id, status) =>
      set((state) => {
        const updateStatus = (execution: Execution) => {
          execution.status = status;
          if (status === 'completed' || status === 'failed' || status === 'cancelled') {
            execution.endTime = new Date().toISOString();
            if (execution.startTime) {
              execution.duration = new Date(execution.endTime).getTime() - new Date(execution.startTime).getTime();
            }
          }
        };

        const index = state.executions.findIndex((e) => e.id === id);
        if (index !== -1) {
          updateStatus(state.executions[index]);
        }
        if (state.currentExecution?.id === id) {
          updateStatus(state.currentExecution);
        }
        state.lastUpdated = Date.now();
      }),

    addExecutionLog: (log) =>
      set((state) => {
        if (state.currentExecution && log.executionId === state.currentExecution.id) {
          state.logs.push(log);
        }
        state.lastUpdated = Date.now();
      }),

    completeExecution: (id, result) =>
      set((state) => {
        const updateComplete = (execution: Execution) => {
          execution.status = 'completed';
          execution.endTime = new Date().toISOString();
          execution.output = result.output;
          if (result.metrics) {
            execution.metrics = result.metrics;
          }
          if (execution.startTime) {
            execution.duration = new Date(execution.endTime).getTime() - new Date(execution.startTime).getTime();
          }
        };

        const index = state.executions.findIndex((e) => e.id === id);
        if (index !== -1) {
          updateComplete(state.executions[index]);
        }
        if (state.currentExecution?.id === id) {
          updateComplete(state.currentExecution);
        }
        state.lastUpdated = Date.now();
      }),

    failExecution: (id, error) =>
      set((state) => {
        const updateFail = (execution: Execution) => {
          execution.status = 'failed';
          execution.endTime = new Date().toISOString();
          execution.error = error;
          if (execution.startTime) {
            execution.duration = new Date(execution.endTime).getTime() - new Date(execution.startTime).getTime();
          }
        };

        const index = state.executions.findIndex((e) => e.id === id);
        if (index !== -1) {
          updateFail(state.executions[index]);
        }
        if (state.currentExecution?.id === id) {
          updateFail(state.currentExecution);
        }
        state.lastUpdated = Date.now();
      }),

    // API calls (placeholder implementations)
    fetchExecutions: async () => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // TODO: 实现真实的 API 调用
        // const executions = await executionApi.getExecutions(filters);

        // 模拟 API 响应
        await new Promise((resolve) => setTimeout(resolve, 1000));

        set((state) => {
          state.executions = [];
          state.pagination.total = 0;
          state.loading = false;
          state.lastUpdated = Date.now();
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : '获取执行记录失败';
          state.loading = false;
        });
      }
    },

    fetchExecution: async (_id) => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // TODO: 实现真实的 API 调用
        // const execution = await executionApi.getExecution(id);

        // 模拟 API 响应
        await new Promise((resolve) => setTimeout(resolve, 1000));

        set((state) => {
          state.currentExecution = null;
          state.loading = false;
          state.lastUpdated = Date.now();
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : '获取执行详情失败';
          state.loading = false;
        });
      }
    },

    fetchLogs: async (_executionId) => {
      try {
        // TODO: 实现真实的 API 调用
        // const logs = await executionApi.getLogs(executionId);

        // 模拟 API 响应
        await new Promise((resolve) => setTimeout(resolve, 500));

        set((state) => {
          state.logs = [];
          state.lastUpdated = Date.now();
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : '获取执行日志失败';
        });
      }
    },

    startWorkflowExecution: async (workflowId, input) => {
      // TODO: 实现真实的 API 调用
      // return await executionApi.startWorkflow(workflowId, input);

      // 模拟 API 响应
      await new Promise((resolve) => setTimeout(resolve, 500));

      const execution: Execution = {
        id: `execution_${Date.now()}`,
        workflowId,
        workflowName: '工作流名称',
        status: 'running',
        input,
        startTime: new Date().toISOString(),
        metrics: {
          duration: 0,
          nodeExecutions: 0,
          successCount: 0,
          errorCount: 0,
          retryCount: 0,
        },
        logs: [],
        triggeredBy: 'manual',
        triggeredByUser: 'current_user',
      };

      get().addExecution(execution);
      get().setCurrentExecution(execution);

      // 模拟执行过程
      setTimeout(() => {
        get().completeExecution(execution.id, {
          output: { result: '执行成功' },
          metrics: {
            duration: 5000,
            nodeExecutions: 3,
            successCount: 3,
            errorCount: 0,
            retryCount: 0,
          },
        });
      }, 5000);

      return execution;
    },

    cancelExecution: async (id) => {
      // TODO: 实现真实的 API 调用
      // await executionApi.cancelExecution(id);

      // 模拟 API 响应
      await new Promise((resolve) => setTimeout(resolve, 500));

      get().updateExecutionStatus(id, 'cancelled');
    },

    retryExecution: async (id) => {
      // TODO: 实现真实的 API 调用
      // await executionApi.retryExecution(id);

      // 模拟 API 响应
      await new Promise((resolve) => setTimeout(resolve, 500));

      const execution = get().executions.find((e) => e.id === id);
      if (execution) {
        get().updateExecution(id, {
          status: 'running',
          startTime: new Date().toISOString(),
          endTime: undefined,
          duration: undefined,
          error: undefined,
        });
      }
    },

    // Utility actions
    reset: () =>
      set((state) => {
        Object.assign(state, initialState);
      }),
  }))
);

// 订阅状态变化，用于调试
if (process.env.NODE_ENV === 'development') {
  useExecutionStore.subscribe(
    (state) => ({
      executionsCount: state.executions.length,
      currentExecutionId: state.currentExecution?.id,
      currentExecutionStatus: state.currentExecution?.status,
      logsCount: state.logs.length,
      loading: state.loading,
      error: state.error,
    })
  );
}