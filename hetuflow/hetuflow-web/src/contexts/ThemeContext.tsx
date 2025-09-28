import React, { createContext, useContext, useEffect, useState } from 'react';
import { theme as antTheme } from 'antd';

// 主题类型定义
export type ThemeMode = 'light' | 'dark' | 'system';

// 主题上下文接口
interface ThemeContextType {
  themeMode: ThemeMode;
  setThemeMode: (mode: ThemeMode) => void;
  currentTheme: 'light' | 'dark';
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
    const stored = localStorage.getItem('hetuflow-theme');
    if (stored === 'light' || stored === 'dark' || stored === 'system') {
      return stored;
    }
  }
  return 'dark'; // 默认深色主题
};

// 保存主题设置到本地存储
const storeTheme = (mode: ThemeMode) => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('hetuflow-theme', mode);
  }
};

// 主题上下文 Provider 组件
export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [themeMode, setThemeMode] = useState<ThemeMode>('dark');
  const [currentTheme, setCurrentTheme] = useState<'light' | 'dark'>('dark');

  useEffect(() => {
    // 初始化主题设置
    const storedTheme = getStoredTheme();
    setThemeMode(storedTheme);

    // 监听系统主题变化
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (e: MediaQueryListEvent) => {
      if (themeMode === 'system') {
        setCurrentTheme(e.matches ? 'dark' : 'light');
      }
    };

    mediaQuery.addEventListener('change', handleChange);

    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }, []);

  useEffect(() => {
    // 根据主题模式更新当前主题
    if (themeMode === 'system') {
      setCurrentTheme(getSystemTheme());
    } else {
      setCurrentTheme(themeMode);
    }

    // 保存到本地存储
    storeTheme(themeMode);
  }, [themeMode]);

  useEffect(() => {
    // 应用主题到 document 元素
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', currentTheme);
      document.documentElement.style.setProperty('--primary-color', '#1890ff');
    }
  }, [currentTheme]);

  const value: ThemeContextType = {
    themeMode,
    setThemeMode,
    currentTheme,
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
        colorTextSecondary: 'rgba(255, 255, 255, 0.65)',
        colorBorder: '#303030',
      },
    };
  }

  return {
    ...baseTheme,
    token: {
      ...baseTheme.token,
      colorBgContainer: '#ffffff',
      colorBgElevated: '#ffffff',
      colorBgLayout: '#f0f2f5',
      colorText: '#000000',
      colorTextSecondary: 'rgba(0, 0, 0, 0.65)',
      colorBorder: '#f0f0f0',
    },
  };
};
