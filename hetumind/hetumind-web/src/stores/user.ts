import { create } from 'zustand';
import { persist, subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import { UserState, UserPreferences } from './types';

interface UserStore extends UserState {
  // Actions
  setUser: (user: UserState['user']) => void;
  updateUser: (updates: Partial<UserState['user']>) => void;
  setPreferences: (preferences: Partial<UserPreferences>) => void;
  updatePreferences: (updates: Partial<UserPreferences>) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  login: (credentials: { username: string; password: string }) => Promise<void>;
  logout: () => void;
  refreshToken: () => Promise<void>;
  hasPermission: (permission: string) => boolean;
  hasRole: (role: string) => boolean;
  reset: () => void;
}

const defaultPreferences: UserPreferences = {
  theme: {
    mode: 'system',
    colorScheme: 'blue',
  },
  language: 'zh-CN',
  autoSave: true,
  sidebarCollapsed: false,
  editorSettings: {
    fontSize: 14,
    tabSize: 2,
    wordWrap: true,
    minimap: true,
    theme: 'vs-dark',
  },
};

export const useUserStore = create<UserStore>()(
  subscribeWithSelector(
    persist(
      immer((set, get) => ({
        // Initial state
        loading: false,
        error: null,
        lastUpdated: null,
        user: null,
        isAuthenticated: false,

        // Actions
        setUser: (user) =>
          set((state) => {
            state.user = user;
            state.isAuthenticated = !!user;
            state.lastUpdated = Date.now();
            state.error = null;
          }),

        updateUser: (updates) =>
          set((state) => {
            if (state.user) {
              Object.assign(state.user, updates);
              state.lastUpdated = Date.now();
            }
          }),

        setPreferences: (preferences) =>
          set((state) => {
            if (state.user) {
              state.user.preferences = {
                ...state.user.preferences,
                ...preferences,
              };
              state.lastUpdated = Date.now();
            }
          }),

        updatePreferences: (updates) =>
          set((state) => {
            if (state.user) {
              state.user.preferences = {
                ...state.user.preferences,
                ...updates,
              };
              state.lastUpdated = Date.now();
            }
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

        login: async (credentials) => {
          set((state) => {
            state.loading = true;
            state.error = null;
          });

          try {
            // TODO: 实现真实的登录 API 调用
            // const response = await authApi.login(credentials);

            // 模拟登录成功
            const mockUser = {
              id: '1',
              username: credentials.username,
              email: `${credentials.username}@example.com`,
              avatar: undefined,
              roles: ['user', 'admin'],
              permissions: ['workflow:read', 'workflow:write', 'agent:read', 'agent:write'],
              preferences: defaultPreferences,
            };

            set((state) => {
              state.user = mockUser;
              state.isAuthenticated = true;
              state.loading = false;
              state.lastUpdated = Date.now();
            });
          } catch (error) {
            set((state) => {
              state.error = error instanceof Error ? error.message : '登录失败';
              state.loading = false;
            });
            throw error;
          }
        },

        logout: () =>
          set((state) => {
            state.user = null;
            state.isAuthenticated = false;
            state.error = null;
            state.lastUpdated = Date.now();
          }),

        refreshToken: async () => {
          try {
            // TODO: 实现真实的 token 刷新 API 调用
            // await authApi.refreshToken();

            set((state) => {
              state.lastUpdated = Date.now();
            });
          } catch (error) {
            // Token 刷新失败，退出登录
            get().logout();
            throw error;
          }
        },

        hasPermission: (permission) => {
          const { user } = get();
          return user?.permissions.includes(permission) ?? false;
        },

        hasRole: (role) => {
          const { user } = get();
          return user?.roles.includes(role) ?? false;
        },

        reset: () =>
          set((state) => {
            state.user = null;
            state.isAuthenticated = false;
            state.error = null;
            state.loading = false;
            state.lastUpdated = null;
          }),
      })),
      {
        name: 'user-store',
        partialize: (state) => ({
          user: state.user,
          isAuthenticated: state.isAuthenticated,
          lastUpdated: state.lastUpdated,
        }),
      }
    )
  )
);

// 订阅状态变化，用于调试
if (process.env.NODE_ENV === 'development') {
  useUserStore.subscribe(
    (state) => ({
      user: state.user,
      isAuthenticated: state.isAuthenticated,
      loading: state.loading,
      error: state.error,
    }),
    (state) => {
      console.log('User Store changed:', state);
    }
  );
}