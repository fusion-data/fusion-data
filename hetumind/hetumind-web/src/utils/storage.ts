import { ThemeMode, ColorScheme } from '@/contexts/ThemeContext';

export interface ThemeSettings {
  themeMode: ThemeMode;
  colorScheme: ColorScheme;
}

const STORAGE_KEY = 'hetumind-theme-settings';

/**
 * 本地存储工具类
 */
export class LocalStorage {
  /**
   * 获取主题设置
   */
  static getThemeSettings(): ThemeSettings | null {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        return JSON.parse(stored);
      }
    } catch (error) {
      console.warn('Failed to parse theme settings from localStorage:', error);
    }
    return null;
  }

  /**
   * 保存主题设置
   */
  static setThemeSettings(settings: ThemeSettings): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    } catch (error) {
      console.warn('Failed to save theme settings to localStorage:', error);
    }
  }

  /**
   * 清除主题设置
   */
  static clearThemeSettings(): void {
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch (error) {
      console.warn('Failed to clear theme settings from localStorage:', error);
    }
  }

  /**
   * 获取存储项
   */
  static getItem<T>(key: string): T | null {
    try {
      const item = localStorage.getItem(key);
      if (item) {
        return JSON.parse(item);
      }
    } catch (error) {
      console.warn(`Failed to parse item "${key}" from localStorage:`, error);
    }
    return null;
  }

  /**
   * 设置存储项
   */
  static setItem<T>(key: string, value: T): void {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch (error) {
      console.warn(`Failed to save item "${key}" to localStorage:`, error);
    }
  }

  /**
   * 删除存储项
   */
  static removeItem(key: string): void {
    try {
      localStorage.removeItem(key);
    } catch (error) {
      console.warn(`Failed to remove item "${key}" from localStorage:`, error);
    }
  }

  /**
   * 清空所有存储
   */
  static clear(): void {
    try {
      localStorage.clear();
    } catch (error) {
      console.warn('Failed to clear localStorage:', error);
    }
  }

  /**
   * 获取所有键
   */
  static keys(): string[] {
    try {
      return Object.keys(localStorage);
    } catch (error) {
      console.warn('Failed to get localStorage keys:', error);
      return [];
    }
  }

  /**
   * 检查是否支持 localStorage
   */
  static isSupported(): boolean {
    try {
      const test = '__test__';
      localStorage.setItem(test, test);
      localStorage.removeItem(test);
      return true;
    } catch {
      return false;
    }
  }
}

/**
 * 主题设置的 Hook
 */
export const useThemeStorage = () => {
  const saveThemeSettings = (themeMode: ThemeMode, colorScheme: ColorScheme) => {
    LocalStorage.setThemeSettings({ themeMode, colorScheme });
  };

  const loadThemeSettings = (): ThemeSettings | null => {
    return LocalStorage.getThemeSettings();
  };

  const clearThemeSettings = () => {
    LocalStorage.clearThemeSettings();
  };

  return {
    saveThemeSettings,
    loadThemeSettings,
    clearThemeSettings,
  };
};