import { create } from 'zustand';
import { persist, subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import { AppState } from './types';

interface AppStore extends AppState {
  // Actions
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  showNotification: (notification: Omit<NonNullable<AppState['notification']>, 'timestamp'>) => void;
  hideNotification: () => void;
  setFeature: (feature: string, enabled: boolean) => void;
  reset: () => void;
}

export const useAppStore = create<AppStore>()(
  subscribeWithSelector(
    persist(
      immer((set) => ({
        // Initial state
        loading: false,
        error: null,
        notification: null,
        version: '0.1.1',
        buildTime: new Date().toISOString(),
        environment: process.env.NODE_ENV as 'development' | 'production' | 'test',
        features: {
          darkMode: true,
          workflowEditor: true,
          agentChat: true,
          codeEditor: true,
          debugMode: process.env.NODE_ENV === 'development',
          analytics: process.env.NODE_ENV === 'production',
        },

        // Actions
        setLoading: (loading) =>
          set((state) => {
            state.loading = loading;
          }),

        setError: (error) =>
          set((state) => {
            state.error = error;
          }),

        showNotification: (notification) =>
          set((state) => {
            state.notification = notification ? {
              ...notification,
              timestamp: Date.now(),
            } : null;
          }),

        hideNotification: () =>
          set((state) => {
            state.notification = null;
          }),

        setFeature: (feature, enabled) =>
          set((state) => {
            state.features[feature] = enabled;
          }),

        reset: () =>
          set((state) => {
            state.loading = false;
            state.error = null;
            state.notification = null;
          }),
      })),
      {
        name: 'app-store',
        partialize: (state) => ({
          features: state.features,
        }),
      }
    )
  )
);

// 订阅状态变化，用于调试
if (process.env.NODE_ENV === 'development') {
  useAppStore.subscribe(
    (state) => ({
      loading: state.loading,
      error: state.error,
      notification: state.notification,
    }),
    (state) => {
      console.log('App Store changed:', state);
    }
  );
}