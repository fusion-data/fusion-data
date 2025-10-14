import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { WorkflowState, Workflow, WorkflowNode, WorkflowEdge } from './types';

interface WorkflowStore extends WorkflowState {
  // Basic actions
  setWorkflows: (workflows: Workflow[]) => void;
  setCurrentWorkflow: (workflow: Workflow | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Workflow management
  addWorkflow: (workflow: Workflow) => void;
  updateWorkflow: (id: string, updates: Partial<Workflow>) => void;
  deleteWorkflow: (id: string) => void;
  duplicateWorkflow: (id: string, name?: string) => void;

  // Node management
  addNode: (node: WorkflowNode) => void;
  updateNode: (id: string, updates: Partial<WorkflowNode>) => void;
  deleteNode: (id: string) => void;
  duplicateNode: (id: string) => void;
  moveNode: (id: string, position: { x: number; y: number }) => void;

  // Edge management
  addEdge: (edge: WorkflowEdge) => void;
  updateEdge: (id: string, updates: Partial<WorkflowEdge>) => void;
  deleteEdge: (id: string) => void;

  // Selection management
  selectNode: (id: string, multi?: boolean) => void;
  selectEdge: (id: string, multi?: boolean) => void;
  selectAll: () => void;
  clearSelection: () => void;
  selectNodesInRect: (rect: { x: number; y: number; width: number; height: number }) => void;

  // Clipboard operations
  copySelection: () => void;
  pasteSelection: (position?: { x: number; y: number }) => void;
  cutSelection: () => void;

  // History management (undo/redo)
  pushToHistory: (workflow: Workflow) => void;
  undo: () => void;
  redo: () => void;
  clearHistory: () => void;

  // View state management
  setZoom: (zoom: number) => void;
  setPan: (pan: { x: number; y: number }) => void;
  fitView: () => void;
  resetView: () => void;

  // Validation
  validateWorkflow: (workflow: Workflow) => { valid: boolean; errors: string[] };

  // API calls
  fetchWorkflows: () => Promise<void>;
  fetchWorkflow: (id: string) => Promise<void>;
  saveWorkflow: (workflow: Workflow) => Promise<void>;
  createWorkflow: (workflow: Partial<Workflow>) => Promise<Workflow>;
  updateWorkflowApi: (id: string, updates: Partial<Workflow>) => Promise<void>;
  deleteWorkflowApi: (id: string) => Promise<void>;

  // Utility actions
  reset: () => void;
}

const initialState: WorkflowState = {
  loading: false,
  error: null,
  lastUpdated: null,
  workflows: [],
  currentWorkflow: null,
  selectedNodes: [],
  selectedEdges: [],
  clipboard: {
    nodes: [],
    edges: [],
  },
  history: {
    past: [],
    present: null,
    future: [],
    maxStates: 50,
  },
  viewState: {
    zoom: 1,
    pan: { x: 0, y: 0 },
    fitView: false,
  },
};

export const useWorkflowStore = create<WorkflowStore>()(
  immer((set, get) => ({
    ...initialState,

    // Basic actions
    setWorkflows: (workflows) =>
      set((state) => {
        state.workflows = workflows;
        state.lastUpdated = Date.now();
      }),

    setCurrentWorkflow: (workflow) =>
      set((state) => {
        state.currentWorkflow = workflow;
        state.selectedNodes = [];
        state.selectedEdges = [];
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

    // Workflow management
    addWorkflow: (workflow) =>
      set((state) => {
        state.workflows.push(workflow);
        state.lastUpdated = Date.now();
      }),

    updateWorkflow: (id, updates) =>
      set((state) => {
        const index = state.workflows.findIndex((w) => w.id === id);
        if (index !== -1) {
          Object.assign(state.workflows[index], updates);
        }
        if (state.currentWorkflow?.id === id) {
          Object.assign(state.currentWorkflow, updates);
        }
        state.lastUpdated = Date.now();
      }),

    deleteWorkflow: (id) =>
      set((state) => {
        state.workflows = state.workflows.filter((w) => w.id !== id);
        if (state.currentWorkflow?.id === id) {
          state.currentWorkflow = null;
        }
        state.lastUpdated = Date.now();
      }),

    duplicateWorkflow: (id, name) =>
      set((state) => {
        const workflow = state.workflows.find((w) => w.id === id);
        if (workflow) {
          const duplicate: Workflow = {
            ...workflow,
            id: `workflow_${Date.now()}`,
            name: name || `${workflow.name} (副本)`,
            status: 'draft',
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
          };
          state.workflows.push(duplicate);
          state.lastUpdated = Date.now();
        }
      }),

    // Node management
    addNode: (node) =>
      set((state) => {
        if (state.currentWorkflow) {
          state.currentWorkflow.nodes.push(node);
          state.lastUpdated = Date.now();
        }
      }),

    updateNode: (id, updates) =>
      set((state) => {
        if (state.currentWorkflow) {
          const node = state.currentWorkflow.nodes.find((n) => n.id === id);
          if (node) {
            Object.assign(node, updates);
            state.lastUpdated = Date.now();
          }
        }
      }),

    deleteNode: (id) =>
      set((state) => {
        if (state.currentWorkflow) {
          state.currentWorkflow.nodes = state.currentWorkflow.nodes.filter((n) => n.id !== id);
          // Also delete connected edges
          state.currentWorkflow.edges = state.currentWorkflow.edges.filter(
            (e) => e.source !== id && e.target !== id
          );
          // Remove from selection
          state.selectedNodes = state.selectedNodes.filter((n) => n !== id);
          state.lastUpdated = Date.now();
        }
      }),

    duplicateNode: (id) =>
      set((state) => {
        if (state.currentWorkflow) {
          const node = state.currentWorkflow.nodes.find((n) => n.id === id);
          if (node) {
            const duplicate: WorkflowNode = {
              ...node,
              id: `node_${Date.now()}`,
              position: {
                x: node.position.x + 50,
                y: node.position.y + 50,
              },
            };
            state.currentWorkflow.nodes.push(duplicate);
            state.lastUpdated = Date.now();
          }
        }
      }),

    moveNode: (id, position) =>
      set((state) => {
        if (state.currentWorkflow) {
          const node = state.currentWorkflow.nodes.find((n) => n.id === id);
          if (node) {
            node.position = position;
            state.lastUpdated = Date.now();
          }
        }
      }),

    // Edge management
    addEdge: (edge) =>
      set((state) => {
        if (state.currentWorkflow) {
          state.currentWorkflow.edges.push(edge);
          state.lastUpdated = Date.now();
        }
      }),

    updateEdge: (id, updates) =>
      set((state) => {
        if (state.currentWorkflow) {
          const edge = state.currentWorkflow.edges.find((e) => e.id === id);
          if (edge) {
            Object.assign(edge, updates);
            state.lastUpdated = Date.now();
          }
        }
      }),

    deleteEdge: (id) =>
      set((state) => {
        if (state.currentWorkflow) {
          state.currentWorkflow.edges = state.currentWorkflow.edges.filter((e) => e.id !== id);
          // Remove from selection
          state.selectedEdges = state.selectedEdges.filter((e) => e !== id);
          state.lastUpdated = Date.now();
        }
      }),

    // Selection management
    selectNode: (id, multi = false) =>
      set((state) => {
        if (multi) {
          if (state.selectedNodes.includes(id)) {
            state.selectedNodes = state.selectedNodes.filter((n) => n !== id);
          } else {
            state.selectedNodes.push(id);
          }
        } else {
          state.selectedNodes = [id];
        }
        state.selectedEdges = [];
      }),

    selectEdge: (id, multi = false) =>
      set((state) => {
        if (multi) {
          if (state.selectedEdges.includes(id)) {
            state.selectedEdges = state.selectedEdges.filter((e) => e !== id);
          } else {
            state.selectedEdges.push(id);
          }
        } else {
          state.selectedEdges = [id];
        }
        state.selectedNodes = [];
      }),

    selectAll: () =>
      set((state) => {
        if (state.currentWorkflow) {
          state.selectedNodes = state.currentWorkflow.nodes.map((n) => n.id);
          state.selectedEdges = state.currentWorkflow.edges.map((e) => e.id);
        }
      }),

    clearSelection: () =>
      set((state) => {
        state.selectedNodes = [];
        state.selectedEdges = [];
      }),

    selectNodesInRect: (rect) =>
      set((state) => {
        if (state.currentWorkflow) {
          const selectedNodes = state.currentWorkflow.nodes
            .filter((node) => {
              const { x, y } = node.position;
              return (
                x >= rect.x &&
                x <= rect.x + rect.width &&
                y >= rect.y &&
                y <= rect.y + rect.height
              );
            })
            .map((node) => node.id);

          state.selectedNodes = selectedNodes;
          state.selectedEdges = [];
        }
      }),

    // Clipboard operations
    copySelection: () =>
      set((state) => {
        if (state.currentWorkflow) {
          const selectedNodes = state.currentWorkflow.nodes.filter((n) =>
            state.selectedNodes.includes(n.id)
          );
          const selectedEdges = state.currentWorkflow.edges.filter((e) =>
            state.selectedEdges.includes(e.id)
          );

          state.clipboard = {
            nodes: selectedNodes,
            edges: selectedEdges,
          };
        }
      }),

    pasteSelection: (position) =>
      set((state) => {
        if (state.currentWorkflow && state.clipboard.nodes.length > 0) {
          const now = Date.now();
          const nodeMap = new Map<string, string>();

          // Paste nodes with new IDs
          const pastedNodes = state.clipboard.nodes.map((node) => {
            const newId = `node_${now}_${Math.random().toString(36).substr(2, 9)}`;
            nodeMap.set(node.id, newId);

            return {
              ...node,
              id: newId,
              position: {
                x: position ? position.x : node.position.x + 50,
                y: position ? position.y : node.position.y + 50,
              },
            };
          });

          // Paste edges with updated node IDs
          const pastedEdges = state.clipboard.edges.map((edge) => ({
            ...edge,
            id: `edge_${now}_${Math.random().toString(36).substr(2, 9)}`,
            source: nodeMap.get(edge.source) || edge.source,
            target: nodeMap.get(edge.target) || edge.target,
          }));

          state.currentWorkflow.nodes.push(...pastedNodes);
          state.currentWorkflow.edges.push(...pastedEdges);
          state.lastUpdated = Date.now();
        }
      }),

    cutSelection: () =>
      set((state) => {
        if (state.currentWorkflow) {
          // Copy to clipboard first
          get().copySelection();

          // Delete selected nodes and edges
          state.currentWorkflow.nodes = state.currentWorkflow.nodes.filter(
            (n) => !state.selectedNodes.includes(n.id)
          );
          state.currentWorkflow.edges = state.currentWorkflow.edges.filter(
            (e) => !state.selectedEdges.includes(e.id)
          );

          // Clear selection
          state.selectedNodes = [];
          state.selectedEdges = [];
          state.lastUpdated = Date.now();
        }
      }),

    // History management
    pushToHistory: (workflow) =>
      set((state) => {
        state.history.past.push(JSON.parse(JSON.stringify(workflow)));
        state.history.present = workflow;
        state.history.future = [];

        // Limit history size
        if (state.history.past.length > state.history.maxStates) {
          state.history.past.shift();
        }
      }),

    undo: () =>
      set((state) => {
        if (state.history.past.length > 0) {
          const previous = state.history.past.pop()!;
          state.history.future.push(state.history.present!);
          state.history.present = previous;
          if (state.currentWorkflow) {
            state.currentWorkflow = previous;
          }
        }
      }),

    redo: () =>
      set((state) => {
        if (state.history.future.length > 0) {
          const next = state.history.future.pop()!;
          state.history.past.push(state.history.present!);
          state.history.present = next;
          if (state.currentWorkflow) {
            state.currentWorkflow = next;
          }
        }
      }),

    clearHistory: () =>
      set((state) => {
        state.history.past = [];
        state.history.future = [];
        if (state.currentWorkflow) {
          state.history.present = state.currentWorkflow;
        }
      }),

    // View state management
    setZoom: (zoom) =>
      set((state) => {
        state.viewState.zoom = Math.max(0.1, Math.min(3, zoom));
      }),

    setPan: (pan) =>
      set((state) => {
        state.viewState.pan = pan;
      }),

    fitView: () =>
      set((state) => {
        state.viewState.fitView = true;
        setTimeout(() => {
          set((s) => {
            s.viewState.fitView = false;
          });
        }, 100);
      }),

    resetView: () =>
      set((state) => {
        state.viewState.zoom = 1;
        state.viewState.pan = { x: 0, y: 0 };
        state.viewState.fitView = false;
      }),

    // Validation
    validateWorkflow: (workflow) => {
      const errors: string[] = [];

      // Check if workflow has nodes
      if (workflow.nodes.length === 0) {
        errors.push('工作流至少需要一个节点');
      }

      // Check for duplicate node IDs
      const nodeIds = workflow.nodes.map((n) => n.id);
      const duplicateNodeIds = nodeIds.filter((id, index) => nodeIds.indexOf(id) !== index);
      if (duplicateNodeIds.length > 0) {
        errors.push(`发现重复的节点ID: ${duplicateNodeIds.join(', ')}`);
      }

      // Check for orphaned edges
      workflow.edges.forEach((edge) => {
        if (!nodeIds.includes(edge.source)) {
          errors.push(`边 ${edge.id} 的源节点 ${edge.source} 不存在`);
        }
        if (!nodeIds.includes(edge.target)) {
          errors.push(`边 ${edge.id} 的目标节点 ${edge.target} 不存在`);
        }
      });

      // Check for disconnected nodes (except start nodes)
      const connectedNodes = new Set<string>();
      workflow.edges.forEach((edge) => {
        connectedNodes.add(edge.source);
        connectedNodes.add(edge.target);
      });

      const disconnectedNodes = workflow.nodes.filter(
        (node) => !connectedNodes.has(node.id) && node.type !== 'trigger'
      );
      if (disconnectedNodes.length > 0) {
        errors.push(`发现未连接的节点: ${disconnectedNodes.map((n) => n.data.label || n.id).join(', ')}`);
      }

      return {
        valid: errors.length === 0,
        errors,
      };
    },

    // API calls (placeholder implementations)
    fetchWorkflows: async () => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // TODO: 实现真实的 API 调用
        // const workflows = await workflowApi.getWorkflows();

        // 模拟 API 响应
        await new Promise((resolve) => setTimeout(resolve, 1000));

        set((state) => {
          state.workflows = [];
          state.loading = false;
          state.lastUpdated = Date.now();
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : '获取工作流失败';
          state.loading = false;
        });
      }
    },

    fetchWorkflow: async (_id) => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // TODO: 实现真实的 API 调用
        // const workflow = await workflowApi.getWorkflow(id);

        // 模拟 API 响应
        await new Promise((resolve) => setTimeout(resolve, 1000));

        set((state) => {
          state.currentWorkflow = null;
          state.loading = false;
          state.lastUpdated = Date.now();
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : '获取工作流详情失败';
          state.loading = false;
        });
      }
    },

    saveWorkflow: async (_workflow) => {
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // TODO: 实现真实的 API 调用
        // await workflowApi.saveWorkflow(workflow);

        // 模拟 API 响应
        await new Promise((resolve) => setTimeout(resolve, 1000));

        set((state) => {
          state.loading = false;
          state.lastUpdated = Date.now();
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : '保存工作流失败';
          state.loading = false;
        });
      }
    },

    createWorkflow: async (workflow) => {
      // TODO: 实现真实的 API 调用
      // return await workflowApi.createWorkflow(workflow);

      // 模拟 API 响应
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const newWorkflow: Workflow = {
        id: `workflow_${Date.now()}`,
        name: workflow.name || '新工作流',
        description: workflow.description || '',
        version: '1.0.0',
        status: 'draft',
        nodes: workflow.nodes || [],
        edges: workflow.edges || [],
        variables: workflow.variables || [],
        settings: workflow.settings || {},
        metadata: workflow.metadata || { tags: [] },
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        createdBy: 'current_user',
        updatedBy: 'current_user',
      };

      set((state) => {
        state.workflows.push(newWorkflow);
        state.currentWorkflow = newWorkflow;
        state.lastUpdated = Date.now();
      });

      return newWorkflow;
    },

    updateWorkflowApi: async (id, updates) => {
      // TODO: 实现真实的 API 调用
      // await workflowApi.updateWorkflow(id, updates);

      // 模拟 API 响应
      await new Promise((resolve) => setTimeout(resolve, 1000));

      get().updateWorkflow(id, updates);
    },

    deleteWorkflowApi: async (id) => {
      // TODO: 实现真实的 API 调用
      // await workflowApi.deleteWorkflow(id);

      // 模拟 API 响应
      await new Promise((resolve) => setTimeout(resolve, 1000));

      get().deleteWorkflow(id);
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
  useWorkflowStore.subscribe(
    (state) => ({
      workflowsCount: state.workflows.length,
      currentWorkflowId: state.currentWorkflow?.id,
      selectedNodesCount: state.selectedNodes.length,
      selectedEdgesCount: state.selectedEdges.length,
      loading: state.loading,
      error: state.error,
    })
  );
}