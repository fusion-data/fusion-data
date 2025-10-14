import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { AgentState, Agent, AgentCategory, AgentTemplate, AgentTestResult } from './types';

interface AgentStore extends AgentState {
  // Basic actions
  setAgents: (agents: Agent[]) => void;
  setCurrentAgent: (agent: Agent | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Agent management
  addAgent: (agent: Agent) => void;
  updateAgent: (id: string, updates: Partial<Agent>) => void;
  deleteAgent: (id: string) => void;
  duplicateAgent: (id: string, name?: string) => void;

  // Category management
  setCategories: (categories: AgentCategory[]) => void;
  addCategory: (category: AgentCategory) => void;
  updateCategory: (id: string, updates: Partial<AgentCategory>) => void;
  deleteCategory: (id: string) => void;

  // Template management
  setTemplates: (templates: AgentTemplate[]) => void;
  addTemplate: (template: AgentTemplate) => void;
  updateTemplate: (id: string, updates: Partial<AgentTemplate>) => void;
  deleteTemplate: (id: string) => void;

  // Test management
  setTestResults: (results: AgentTestResult[]) => void;
  addTestResult: (result: AgentTestResult) => void;
  clearTestResults: () => void;

  // API calls
  fetchAgents: () => Promise<void>;
  fetchAgent: (id: string) => Promise<void>;
  createAgent: (agent: Partial<Agent>) => Promise<Agent>;
  updateAgentApi: (id: string, updates: Partial<Agent>) => Promise<void>;
  deleteAgentApi: (id: string) => Promise<void>;
  testAgent: (id: string, input: string) => Promise<AgentTestResult>;

  // Utility actions
  reset: () => void;
}

const initialState: AgentState = {
  loading: false,
  error: null,
  lastUpdated: null,
  agents: [],
  currentAgent: null,
  categories: [],
  templates: [],
  testResults: [],
};

export const useAgentStore = create<AgentStore>()(
  immer((set, get) => ({
    ...initialState,

    // Basic actions
    setAgents: (agents) =>
      set((state) => {
        state.agents = agents;
        state.lastUpdated = Date.now();
      }),

    setCurrentAgent: (agent) =>
      set((state) => {
        state.currentAgent = agent;
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

    // Agent management
    addAgent: (agent) =>
      set((state) => {
        state.agents.push(agent);
        state.lastUpdated = Date.now();
      }),

    updateAgent: (id, updates) =>
      set((state) => {
        const index = state.agents.findIndex((a) => a.id === id);
        if (index !== -1) {
          Object.assign(state.agents[index], updates);
        }
        if (state.currentAgent?.id === id) {
          Object.assign(state.currentAgent, updates);
        }
        state.lastUpdated = Date.now();
      }),

    deleteAgent: (_id) =>
      set((state) => {
        state.agents = state.agents.filter((a) => a.id !== _id);
        if (state.currentAgent?.id === _id) {
          state.currentAgent = null;
        }
        state.lastUpdated = Date.now();
      }),

    duplicateAgent: (id, name) =>
      set((state) => {
        const agent = state.agents.find((a) => a.id === id);
        if (agent) {
          const duplicate: Agent = {
            ...agent,
            id: `agent_${Date.now()}`,
            name: name || `${agent.name} (副本)`,
            status: 'inactive',
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
          };
          state.agents.push(duplicate);
          state.lastUpdated = Date.now();
        }
      }),

    // Category management
    setCategories: (categories) =>
      set((state) => {
        state.categories = categories;
        state.lastUpdated = Date.now();
      }),

    addCategory: (category) =>
      set((state) => {
        state.categories.push(category);
        state.lastUpdated = Date.now();
      }),

    updateCategory: (id, updates) =>
      set((state) => {
        const index = state.categories.findIndex((c) => c.id === id);
        if (index !== -1) {
          Object.assign(state.categories[index], updates);
        }
        state.lastUpdated = Date.now();
      }),

    deleteCategory: (_id) =>
      set((state) => {
        state.categories = state.categories.filter((c) => c.id !== _id);
        // Update agents that use this category
        state.agents.forEach((agent) => {
          if (agent.category === _id) {
            agent.category = 'default';
          }
        });
        state.lastUpdated = Date.now();
      }),

    // Template management
    setTemplates: (templates) =>
      set((state) => {
        state.templates = templates;
        state.lastUpdated = Date.now();
      }),

    addTemplate: (template) =>
      set((state) => {
        state.templates.push(template);
        state.lastUpdated = Date.now();
      }),

    updateTemplate: (id, updates) =>
      set((state) => {
        const index = state.templates.findIndex((t) => t.id === id);
        if (index !== -1) {
          Object.assign(state.templates[index], updates);
        }
        state.lastUpdated = Date.now();
      }),

    deleteTemplate: (id) =>
      set((state) => {
        state.templates = state.templates.filter((t) => t.id !== id);
        state.lastUpdated = Date.now();
      }),

    // Test management
    setTestResults: (results) =>
      set((state) => {
        state.testResults = results;
        state.lastUpdated = Date.now();
      }),

    addTestResult: (result) =>
      set((state) => {
        state.testResults.unshift(result); // Add to beginning (newest first)
        // Keep only last 100 test results
        if (state.testResults.length > 100) {
          state.testResults = state.testResults.slice(0, 100);
        }
        state.lastUpdated = Date.now();
      }),

    clearTestResults: () =>
      set((state) => {
        state.testResults = [];
        state.lastUpdated = Date.now();
      }),

    // API calls (placeholder implementations)
    fetchAgents: async () => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // TODO: 实现真实的 API 调用
        // const agents = await agentApi.getAgents();

        // 模拟 API 响应
        await new Promise((resolve) => setTimeout(resolve, 1000));

        set((state) => {
          state.agents = [];
          state.loading = false;
          state.lastUpdated = Date.now();
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : '获取智能体失败';
          state.loading = false;
        });
      }
    },

    fetchAgent: async (_id) => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // TODO: 实现真实的 API 调用
        // const agent = await agentApi.getAgent(id);

        // 模拟 API 响应
        await new Promise((resolve) => setTimeout(resolve, 1000));

        set((state) => {
          state.currentAgent = null;
          state.loading = false;
          state.lastUpdated = Date.now();
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : '获取智能体详情失败';
          state.loading = false;
        });
      }
    },

    createAgent: async (agent) => {
      // TODO: 实现真实的 API 调用
      // return await agentApi.createAgent(agent);

      // 模拟 API 响应
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const newAgent: Agent = {
        id: `agent_${Date.now()}`,
        name: agent.name || '新智能体',
        description: agent.description || '',
        avatar: agent.avatar,
        category: agent.category || 'default',
        type: agent.type || 'chat',
        model: agent.model || {
          provider: 'openai',
          model: 'gpt-3.5-turbo',
          temperature: 0.7,
          maxTokens: 2048,
        },
        config: agent.config || {
          systemPrompt: '你是一个有用的AI助手。',
          welcomeMessage: '你好！我是你的AI助手，有什么可以帮助你的吗？',
          instructions: [],
          constraints: [],
          examples: [],
          memory: {
            enabled: true,
            maxSize: 1000,
            retention: 7200,
          },
        },
        tools: agent.tools || [],
        knowledge: agent.knowledge || [],
        status: 'inactive',
        metrics: {
          totalConversations: 0,
          totalMessages: 0,
          averageResponseTime: 0,
          successRate: 0,
          tokenUsage: {
            total: 0,
            input: 0,
            output: 0,
          },
        },
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        createdBy: 'current_user',
        updatedBy: 'current_user',
      };

      set((state) => {
        state.agents.push(newAgent);
        state.currentAgent = newAgent;
        state.lastUpdated = Date.now();
      });

      return newAgent;
    },

    updateAgentApi: async (id, updates) => {
      // TODO: 实现真实的 API 调用
      // await agentApi.updateAgent(id, updates);

      // 模拟 API 响应
      await new Promise((resolve) => setTimeout(resolve, 1000));

      get().updateAgent(id, updates);
    },

    deleteAgentApi: async (id) => {
      // TODO: 实现真实的 API 调用
      // await agentApi.deleteAgent(id);

      // 模拟 API 响应
      await new Promise((resolve) => setTimeout(resolve, 1000));

      get().deleteAgent(id);
    },

    testAgent: async (id, input) => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // TODO: 实现真实的 API 调用
        // const result = await agentApi.testAgent(id, input);

        // 模拟 API 响应
        await new Promise((resolve) => setTimeout(resolve, 2000));

        const result: AgentTestResult = {
          id: `test_${Date.now()}`,
          agentId: id,
          input,
          output: `这是对"${input}"的模拟回复。`,
          status: 'success',
          metrics: {
            responseTime: Math.floor(Math.random() * 2000) + 500,
            tokenUsage: {
              input: Math.floor(Math.random() * 100) + 20,
              output: Math.floor(Math.random() * 200) + 50,
              total: 0,
            },
          },
          createdAt: new Date().toISOString(),
        };

        // 计算总 token 使用量
        result.metrics.tokenUsage.total =
          result.metrics.tokenUsage.input + result.metrics.tokenUsage.output;

        set((state) => {
          state.testResults.unshift(result);
          if (state.testResults.length > 100) {
            state.testResults = state.testResults.slice(0, 100);
          }
          state.loading = false;
          state.lastUpdated = Date.now();
        });

        return result;
      } catch (error) {
        const errorResult: AgentTestResult = {
          id: `test_${Date.now()}`,
          agentId: id,
          input,
          output: '',
          status: 'error',
          metrics: {
            responseTime: 0,
            tokenUsage: {
              input: 0,
              output: 0,
              total: 0,
            },
          },
          error: error instanceof Error ? error.message : '测试失败',
          createdAt: new Date().toISOString(),
        };

        set((state) => {
          state.testResults.unshift(errorResult);
          state.error = error instanceof Error ? error.message : '测试失败';
          state.loading = false;
          state.lastUpdated = Date.now();
        });

        return errorResult;
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
  useAgentStore.subscribe(
    (state) => ({
      agentsCount: state.agents.length,
      currentAgentId: state.currentAgent?.id,
      categoriesCount: state.categories.length,
      templatesCount: state.templates.length,
      testResultsCount: state.testResults.length,
      loading: state.loading,
      error: state.error,
    })
  );
}