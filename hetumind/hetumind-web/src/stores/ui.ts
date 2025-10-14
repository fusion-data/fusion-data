import { create } from 'zustand';
import { persist, subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import { UIState } from './types';

interface UIStore extends UIState {
  // Actions
  setSidebarCollapsed: (collapsed: boolean) => void;
  toggleSidebar: () => void;
  setTheme: (theme: UIState['theme']) => void;
  setColorScheme: (colorScheme: UIState['colorScheme']) => void;
  setLanguage: (language: string) => void;
  openModal: (modal: string) => void;
  closeModal: (modal: string) => void;
  closeAllModals: () => void;
  openDrawer: (drawer: string) => void;
  closeDrawer: (drawer: string) => void;
  closeAllDrawers: () => void;
  showPanel: (panel: string, config?: { size?: number; position?: 'left' | 'right' | 'top' | 'bottom' }) => void;
  hidePanel: (panel: string) => void;
  togglePanel: (panel: string) => void;
  setLoading: (key: string, loading: boolean) => void;
  clearAllLoading: () => void;
  reset: () => void;
}

const initialState: UIState = {
  sidebarCollapsed: false,
  theme: 'system',
  colorScheme: 'blue',
  language: 'zh-CN',
  modals: {},
  drawers: {},
  panels: {},
  loading: {},
};

export const useUIStore = create<UIStore>()(
  subscribeWithSelector(
    persist(
      immer((set) => ({
        ...initialState,

        // Actions
        setSidebarCollapsed: (collapsed) =>
          set((state) => {
            state.sidebarCollapsed = collapsed;
          }),

        toggleSidebar: () =>
          set((state) => {
            state.sidebarCollapsed = !state.sidebarCollapsed;
          }),

        setTheme: (theme) =>
          set((state) => {
            state.theme = theme;
          }),

        setColorScheme: (colorScheme) =>
          set((state) => {
            state.colorScheme = colorScheme;
          }),

        setLanguage: (language) =>
          set((state) => {
            state.language = language;
          }),

        openModal: (modal) =>
          set((state) => {
            state.modals[modal] = true;
          }),

        closeModal: (modal) =>
          set((state) => {
            state.modals[modal] = false;
          }),

        closeAllModals: () =>
          set((state) => {
            Object.keys(state.modals).forEach((key) => {
              state.modals[key] = false;
            });
          }),

        openDrawer: (drawer) =>
          set((state) => {
            state.drawers[drawer] = true;
          }),

        closeDrawer: (drawer) =>
          set((state) => {
            state.drawers[drawer] = false;
          }),

        closeAllDrawers: () =>
          set((state) => {
            Object.keys(state.drawers).forEach((key) => {
              state.drawers[key] = false;
            });
          }),

        showPanel: (panel, config) =>
          set((state) => {
            state.panels[panel] = {
              visible: true,
              size: config?.size,
              position: config?.position || 'right',
            };
          }),

        hidePanel: (panel) =>
          set((state) => {
            if (state.panels[panel]) {
              state.panels[panel].visible = false;
            }
          }),

        togglePanel: (panel) =>
          set((state) => {
            if (state.panels[panel]) {
              state.panels[panel].visible = !state.panels[panel].visible;
            } else {
              state.panels[panel] = {
                visible: true,
                position: 'right',
              };
            }
          }),

        setLoading: (key, loading) =>
          set((state) => {
            state.loading[key] = loading;
          }),

        clearAllLoading: () =>
          set((state) => {
            state.loading = {};
          }),

        reset: () =>
          set((state) => {
            Object.assign(state, initialState);
          }),
      })),
      {
        name: 'ui-store',
        partialize: (state) => ({
          sidebarCollapsed: state.sidebarCollapsed,
          theme: state.theme,
          colorScheme: state.colorScheme,
          language: state.language,
        }),
      }
    )
  )
);

// 订阅状态变化，用于调试
if (process.env.NODE_ENV === 'development') {
  useUIStore.subscribe(
    (state) => ({
      sidebarCollapsed: state.sidebarCollapsed,
      theme: state.theme,
      colorScheme: state.colorScheme,
      language: state.language,
      modalsCount: Object.values(state.modals).filter(Boolean).length,
      drawersCount: Object.values(state.drawers).filter(Boolean).length,
      panelsCount: Object.values(state.panels).filter(p => p.visible).length,
    }),
    (state) => {
      console.log('UI Store changed:', state);
    }
  );
}