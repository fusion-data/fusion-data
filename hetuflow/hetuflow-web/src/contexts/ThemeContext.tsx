import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { theme as antTheme } from 'antd';

// 主题类型定义
export type ThemeMode = 'light' | 'dark' | 'system';

// 主题上下文接口
interface ThemeContextType {
  themeMode: ThemeMode;
  setThemeMode: (mode: ThemeMode) => void;
  currentTheme: 'light' | 'dark';
  isInitialized: boolean;
}

// 创建主题上下文
const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

// 获取系统主题偏好
const getSystemTheme = (): 'light' | 'dark' => {
  if (typeof window !== 'undefined') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return 'dark'; // 默认深色主题
};

// 从本地存储获取主题设置
const getStoredTheme = (): ThemeMode => {
  if (typeof window !== 'undefined') {
    try {
      const stored = localStorage.getItem('hetuflow-theme');
      if (stored === 'light' || stored === 'dark' || stored === 'system') {
        return stored;
      }
    } catch (error) {
      console.warn('Failed to read theme from localStorage:', error);
    }
  }
  return 'system'; // 默认使用系统主题
};

// 保存主题设置到本地存储
const storeTheme = (mode: ThemeMode) => {
  if (typeof window !== 'undefined') {
    try {
      localStorage.setItem('hetuflow-theme', mode);
    } catch (error) {
      console.warn('Failed to save theme to localStorage:', error);
    }
  }
};

// 计算当前应该应用的主题
const calculateCurrentTheme = (mode: ThemeMode): 'light' | 'dark' => {
  if (mode === 'system') {
    return getSystemTheme();
  }
  return mode;
};

// 应用主题到 DOM
const applyThemeToDOM = (theme: 'light' | 'dark') => {
  if (typeof document !== 'undefined') {
    try {
      document.documentElement.setAttribute('data-theme', theme);
      document.documentElement.setAttribute('data-color-mode', theme);

      // 应用 CSS 变量
      const root = document.documentElement;
      if (theme === 'dark') {
        root.style.setProperty('--bg-primary', '#000000');
        root.style.setProperty('--bg-secondary', '#141414');
        root.style.setProperty('--bg-tertiary', '#1f1f1f');
        root.style.setProperty('--text-primary', '#ffffff');
        root.style.setProperty('--text-secondary', 'rgba(255, 255, 255, 0.85)');
        root.style.setProperty('--text-tertiary', 'rgba(255, 255, 255, 0.65)');
        root.style.setProperty('--border-color', '#303030');
        root.style.setProperty('--primary-color', '#1890ff');
      } else {
        root.style.setProperty('--bg-primary', '#ffffff');
        root.style.setProperty('--bg-secondary', '#f5f5f5');
        root.style.setProperty('--bg-tertiary', '#fafafa');
        root.style.setProperty('--text-primary', '#000000');
        root.style.setProperty('--text-secondary', 'rgba(0, 0, 0, 0.85)');
        root.style.setProperty('--text-tertiary', 'rgba(0, 0, 0, 0.45)');
        root.style.setProperty('--border-color', '#f0f0f0');
        root.style.setProperty('--primary-color', '#1890ff');
      }
    } catch (error) {
      console.warn('Failed to apply theme to DOM:', error);
    }
  }
};

// 主题上下文 Provider 组件
export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  // 初始化状态：从 localStorage 读取，如果没有则使用系统主题
  const [themeMode, setThemeModeState] = useState<ThemeMode>('system');
  const [currentTheme, setCurrentTheme] = useState<'light' | 'dark'>('light');
  const [isInitialized, setIsInitialized] = useState(false);

  // 设置主题模式的函数，包含持久化逻辑
  const setThemeMode = useCallback((mode: ThemeMode) => {
    setThemeModeState(mode);
    storeTheme(mode);
  }, []);

  useEffect(() => {
    // 组件挂载时初始化主题
    const storedTheme = getStoredTheme();
    const initialTheme = calculateCurrentTheme(storedTheme);

    setThemeModeState(storedTheme);
    setCurrentTheme(initialTheme);
    applyThemeToDOM(initialTheme);
    setIsInitialized(true);
  }, []);

  useEffect(() => {
    if (!isInitialized) return;

    // 计算并更新当前主题
    const newCurrentTheme = calculateCurrentTheme(themeMode);
    setCurrentTheme(newCurrentTheme);
    applyThemeToDOM(newCurrentTheme);
  }, [themeMode, isInitialized]);

  useEffect(() => {
    if (!isInitialized) return;

    // 监听系统主题变化（仅在当前使用系统主题时响应）
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const handleChange = (e: MediaQueryListEvent) => {
      if (themeMode === 'system') {
        const newTheme = e.matches ? 'dark' : 'light';
        setCurrentTheme(newTheme);
        applyThemeToDOM(newTheme);
      }
    };

    // 添加事件监听器
    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handleChange);
    } else {
      // 兼容旧浏览器
      mediaQuery.addListener(handleChange);
    }

    return () => {
      // 清理事件监听器
      if (mediaQuery.removeEventListener) {
        mediaQuery.removeEventListener('change', handleChange);
      } else {
        mediaQuery.removeListener(handleChange);
      }
    };
  }, [themeMode, isInitialized]);

  const value: ThemeContextType = {
    themeMode,
    setThemeMode,
    currentTheme,
    isInitialized,
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
};

// 使用主题上下文的 Hook
export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

// Ant Design 主题配置
export const getAntdTheme = (currentTheme: 'light' | 'dark') => {
  const baseTheme = {
    token: {
      colorPrimary: '#1890ff',
      borderRadius: 6,
    },
    components: {
      Button: {
        colorPrimary: '#1890ff',
        algorithm: true, // 启用组件算法
      },
      Card: {
        colorBgContainer: currentTheme === 'dark' ? '#1f1f1f' : '#ffffff',
      },
      Table: {
        colorBgContainer: currentTheme === 'dark' ? '#141414' : '#ffffff',
        rowHoverBg: currentTheme === 'dark' ? '#262626' : '#f5f5f5',
      },
    },
  };

  if (currentTheme === 'dark') {
    return {
      ...baseTheme,
      algorithm: antTheme.darkAlgorithm, // Ant Design v5 内置的深色算法
      token: {
        ...baseTheme.token,
        colorBgContainer: '#141414',
        colorBgElevated: '#1f1f1f',
        colorBgLayout: '#000000',
        colorText: '#ffffff',
        colorTextSecondary: 'rgba(255, 255, 255, 0.85)',
        colorTextTertiary: 'rgba(255, 255, 255, 0.65)',
        colorTextQuaternary: 'rgba(255, 255, 255, 0.45)',
        colorBorder: '#303030',
        colorSplit: '#303030',
      },
      components: {
        ...baseTheme.components,
        Input: {
          colorBgContainer: '#1f1f1f',
          colorBorder: '#434343',
          colorBgHover: '#262626',
          colorBgActive: '#1890ff',
        },
        Select: {
          colorBgContainer: '#1f1f1f',
          colorBorder: '#434343',
          optionSelectedBg: '#111b26',
        },
        Modal: {
          contentBg: '#1f1f1f',
          headerBg: '#141414',
        },
        Dropdown: {
          colorBgElevated: '#1f1f1f',
        },
        Menu: {
          colorBgContainer: '#141414',
          subMenuItemBg: '#000000',
          itemHoverBg: '#1f1f1f',
          itemSelectedBg: '#111b26',
        },
      },
    };
  }

  return {
    ...baseTheme,
    algorithm: antTheme.defaultAlgorithm, // Ant Design v5 内置的浅色算法
    token: {
      ...baseTheme.token,
      colorBgContainer: '#ffffff',
      colorBgElevated: '#ffffff',
      colorBgLayout: '#f0f2f5',
      colorText: '#000000',
      colorTextSecondary: 'rgba(0, 0, 0, 0.85)',
      colorTextTertiary: 'rgba(0, 0, 0, 0.45)',
      colorTextQuaternary: 'rgba(0, 0, 0, 0.25)',
      colorBorder: '#f0f0f0',
      colorSplit: '#f0f0f0',
    },
    components: {
      ...baseTheme.components,
      Input: {
        colorBgContainer: '#ffffff',
        colorBorder: '#d9d9d9',
        colorBgHover: '#f5f5f5',
        colorBgActive: '#e6f7ff',
      },
      Select: {
        colorBgContainer: '#ffffff',
        colorBorder: '#d9d9d9',
        optionSelectedBg: '#f5f5f5',
      },
      Modal: {
        contentBg: '#ffffff',
        headerBg: '#fafafa',
      },
      Dropdown: {
        colorBgElevated: '#ffffff',
      },
      Menu: {
        colorBgContainer: '#ffffff',
        subMenuItemBg: '#fafafa',
        itemHoverBg: '#f5f5f5',
        itemSelectedBg: '#e6f7ff',
      },
    },
  };
};
