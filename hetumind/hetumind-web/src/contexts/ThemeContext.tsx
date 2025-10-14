import React, { createContext, useContext, useEffect, useState } from 'react';
import { theme as antTheme } from 'antd';
import { LocalStorage } from '@/utils/storage';

export type ThemeMode = 'light' | 'dark' | 'system';
export type ColorScheme = 'blue' | 'purple' | 'green' | 'orange';

interface ThemeContextType {
  themeMode: ThemeMode;
  colorScheme: ColorScheme;
  setThemeMode: (mode: ThemeMode) => void;
  setColorScheme: (scheme: ColorScheme) => void;
  currentTheme: 'light' | 'dark';
  antdTheme: any;
  resetTheme: () => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  // 从本地存储加载设置
  const savedSettings = LocalStorage.getThemeSettings();
  const [themeMode, setThemeMode] = useState<ThemeMode>(savedSettings?.themeMode || 'system');
  const [colorScheme, setColorScheme] = useState<ColorScheme>(savedSettings?.colorScheme || 'blue');
  const [currentTheme, setCurrentTheme] = useState<'light' | 'dark'>('light');

  // 获取系统主题
  const getSystemTheme = (): 'light' | 'dark' => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  };

  // 获取主题颜色
  const getThemeColors = (_theme: 'light' | 'dark', scheme: ColorScheme) => {
    const baseColors = {
      blue: { primary: '#1890ff', success: '#52c41a', warning: '#faad14', error: '#ff4d4f' },
      purple: { primary: '#722ed1', success: '#52c41a', warning: '#faad14', error: '#ff4d4f' },
      green: { primary: '#52c41a', success: '#73d13d', warning: '#faad14', error: '#ff4d4f' },
      orange: { primary: '#fa8c16', success: '#52c41a', warning: '#faad14', error: '#ff4d4f' },
    };

    return baseColors[scheme];
  };

  // 保存主题设置到本地存储
  const saveThemeSettings = (mode: ThemeMode, scheme: ColorScheme) => {
    LocalStorage.setThemeSettings({ themeMode: mode, colorScheme: scheme });
  };

  // 重置主题到默认设置
  const resetTheme = () => {
    const defaultMode: ThemeMode = 'system';
    const defaultScheme: ColorScheme = 'blue';
    setThemeMode(defaultMode);
    setColorScheme(defaultScheme);
    saveThemeSettings(defaultMode, defaultScheme);
  };

  // 计算当前主题
  useEffect(() => {
    const currentThemeValue = themeMode === 'system' ? getSystemTheme() : themeMode;
    setCurrentTheme(currentThemeValue);
    applyTheme(currentThemeValue, colorScheme);
  }, [themeMode, colorScheme]);

  // 监听系统主题变化
  useEffect(() => {
    if (themeMode !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (e: MediaQueryListEvent) => {
      setCurrentTheme(e.matches ? 'dark' : 'light');
      applyTheme(e.matches ? 'dark' : 'light', colorScheme);
    };

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [themeMode, colorScheme]);

  // 应用主题到 DOM
  const applyTheme = (theme: 'light' | 'dark', scheme: ColorScheme) => {
    const root = document.documentElement;
    root.setAttribute('data-theme', theme);
    root.setAttribute('data-color-scheme', scheme);
  };

  // Ant Design 主题配置
  const antdTheme = React.useMemo(() => {
    const colors = getThemeColors(currentTheme, colorScheme);
    const algorithm = currentTheme === 'dark' ? antTheme.darkAlgorithm : antTheme.defaultAlgorithm;

    return {
      algorithm,
      token: {
        colorPrimary: colors.primary,
        colorSuccess: colors.success,
        colorWarning: colors.warning,
        colorError: colors.error,
        borderRadius: 8,
        wireframe: false,
      },
      components: {
        Layout: {
          siderBg: currentTheme === 'dark' ? '#001529' : '#fff',
          triggerBg: currentTheme === 'dark' ? '#002140' : '#f0f2f5',
        },
        Menu: {
          darkItemBg: '#001529',
          darkSubMenuItemBg: '#000c17',
          darkItemSelectedBg: colors.primary,
        },
      },
    };
  }, [currentTheme, colorScheme]);

  return (
    <ThemeContext.Provider
      value={{
        themeMode,
        colorScheme,
        setThemeMode: (mode: ThemeMode) => {
          setThemeMode(mode);
          saveThemeSettings(mode, colorScheme);
        },
        setColorScheme: (scheme: ColorScheme) => {
          setColorScheme(scheme);
          saveThemeSettings(themeMode, scheme);
        },
        currentTheme,
        antdTheme,
        resetTheme,
      }}
    >
      {children}
    </ThemeContext.Provider>
  );
};