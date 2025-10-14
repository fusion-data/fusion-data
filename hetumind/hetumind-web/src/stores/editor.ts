import { create } from 'zustand';
import { persist, subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import { EditorState, EditorTab } from './types';

interface EditorStore extends EditorState {
  // Tab management
  addTab: (tab: EditorTab) => void;
  closeTab: (tabId: string) => void;
  updateTab: (tabId: string, updates: Partial<EditorTab>) => void;
  setActiveTab: (tabId: string) => void;
  closeOtherTabs: (tabId: string) => void;
  closeTabsToRight: (tabId: string) => void;
  closeAllTabs: () => void;

  // Clipboard operations
  setClipboard: (data: any) => void;
  clearClipboard: () => void;

  // History management
  pushToHistory: (data: any) => void;
  undo: () => void;
  redo: () => void;
  clearHistory: () => void;

  // View state management
  setZoom: (zoom: number) => void;
  setPan: (pan: { x: number; y: number }) => void;
  fitView: () => void;
  setShowGrid: (show: boolean) => void;
  setSnapToGrid: (snap: boolean) => void;

  // Tool management
  setSelectedTool: (tool: string) => void;
  setToolConfig: (tool: string, config: any) => void;

  // Utility actions
  reset: () => void;
}

const initialState: EditorState = {
  activeTab: '',
  tabs: [],
  clipboard: null,
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
    showGrid: true,
    snapToGrid: false,
  },
  tools: {
    selected: 'select',
    config: {},
  },
};

export const useEditorStore = create<EditorStore>()(
  subscribeWithSelector(
    persist(
      immer((set) => ({
        ...initialState,

        // Tab management
        addTab: (tab) =>
          set((state) => {
            const existingTab = state.tabs.find((t) => t.id === tab.id);
            if (!existingTab) {
              state.tabs.push(tab);
            }
            state.activeTab = tab.id;
          }),

        closeTab: (tabId) =>
          set((state) => {
            const index = state.tabs.findIndex((t) => t.id === tabId);
            if (index !== -1) {
              state.tabs.splice(index, 1);

              // 如果关闭的是当前活动标签，切换到其他标签
              if (state.activeTab === tabId) {
                if (state.tabs.length > 0) {
                  state.activeTab = state.tabs[Math.max(0, index - 1)].id;
                } else {
                  state.activeTab = '';
                }
              }
            }
          }),

        updateTab: (tabId, updates) =>
          set((state) => {
            const tab = state.tabs.find((t) => t.id === tabId);
            if (tab) {
              Object.assign(tab, updates);
            }
          }),

        setActiveTab: (tabId) =>
          set((state) => {
            const tab = state.tabs.find((t) => t.id === tabId);
            if (tab) {
              state.activeTab = tabId;
            }
          }),

        closeOtherTabs: (tabId) =>
          set((state) => {
            state.tabs = state.tabs.filter((t) => t.id === tabId);
            state.activeTab = tabId;
          }),

        closeTabsToRight: (tabId) =>
          set((state) => {
            const index = state.tabs.findIndex((t) => t.id === tabId);
            if (index !== -1) {
              state.tabs = state.tabs.slice(0, index + 1);
            }
          }),

        closeAllTabs: () =>
          set((state) => {
            state.tabs = [];
            state.activeTab = '';
          }),

        // Clipboard operations
        setClipboard: (data) =>
          set((state) => {
            state.clipboard = data;
          }),

        clearClipboard: () =>
          set((state) => {
            state.clipboard = null;
          }),

        // History management
        pushToHistory: (data) =>
          set((state) => {
            state.history.past.push(JSON.parse(JSON.stringify(data)));
            state.history.present = data;
            state.history.future = [];

            // 限制历史记录大小
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
            }
          }),

        redo: () =>
          set((state) => {
            if (state.history.future.length > 0) {
              const next = state.history.future.pop()!;
              state.history.past.push(state.history.present!);
              state.history.present = next;
            }
          }),

        clearHistory: () =>
          set((state) => {
            state.history.past = [];
            state.history.future = [];
            state.history.present = null;
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

        setShowGrid: (show) =>
          set((state) => {
            state.viewState.showGrid = show;
          }),

        setSnapToGrid: (snap) =>
          set((state) => {
            state.viewState.snapToGrid = snap;
          }),

        // Tool management
        setSelectedTool: (tool) =>
          set((state) => {
            state.tools.selected = tool;
          }),

        setToolConfig: (tool, config) =>
          set((state) => {
            state.tools.config[tool] = config;
          }),

        // Utility actions
        reset: () =>
          set((state) => {
            Object.assign(state, initialState);
          }),
      })),
      {
        name: 'editor-store',
        partialize: (state) => ({
          viewState: state.viewState,
          tools: state.tools,
        }),
      }
    )
  )
);

// 订阅状态变化，用于调试
if (process.env.NODE_ENV === 'development') {
  useEditorStore.subscribe(
    (state) => ({
      activeTab: state.activeTab,
      tabsCount: state.tabs.length,
      selectedTool: state.tools.selected,
      zoom: state.viewState.zoom,
      showGrid: state.viewState.showGrid,
      snapToGrid: state.viewState.snapToGrid,
    }),
    (state) => {
      console.log('Editor Store changed:', state);
    }
  );
}