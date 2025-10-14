import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { RouteConfig, BreadcrumbConfig, RouteContextType } from '@/router/types';
import { getPageTitle, getBreadcrumbs } from '@/router/utils';

const RouteContext = createContext<RouteContextType | undefined>(undefined);

/**
 * 路由上下文 Provider
 */
export const RouteProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const location = useLocation();
  const navigate = useNavigate();

  const [currentRoute, setCurrentRoute] = useState<RouteConfig | null>(null);
  const [breadcrumbs, setBreadcrumbs] = useState<BreadcrumbConfig[]>([]);

  // 更新当前路由信息
  useEffect(() => {
    const title = getPageTitle(location.pathname);
    const newBreadcrumbs = getBreadcrumbs(location.pathname);

    const route: RouteConfig = {
      path: location.pathname,
      title,
    };

    setCurrentRoute(route);
    setBreadcrumbs(newBreadcrumbs);
  }, [location.pathname]);

  // 添加路由
  const addRoute = useCallback((route: RouteConfig) => {
    console.log('Adding route:', route);
    // TODO: 实现路由添加逻辑
  }, []);

  // 移除路由
  const removeRoute = useCallback((path: string) => {
    console.log('Removing route:', path);
    // TODO: 实现路由移除逻辑
  }, []);

  // 更新路由
  const updateRoute = useCallback((path: string, updates: Partial<RouteConfig>) => {
    console.log('Updating route:', path, updates);
    // TODO: 实现路由更新逻辑
  }, []);

  // 导航到指定路径
  const navigateTo = useCallback((path: string, state?: any) => {
    navigate(path, { state });
  }, [navigate]);

  // 返回上一页
  const goBack = useCallback(() => {
    navigate(-1);
  }, [navigate]);

  // 前进到下一页
  const goForward = useCallback(() => {
    navigate(1);
  }, [navigate]);

  // 刷新当前页面
  const refresh = useCallback(() => {
    window.location.reload();
  }, []);

  const value: RouteContextType = {
    currentRoute,
    breadcrumbs,
    menuItems: [], // TODO: 实现菜单项生成逻辑
    addRoute,
    removeRoute,
    updateRoute,
    navigateTo,
    goBack,
    goForward,
    refresh,
  };

  return (
    <RouteContext.Provider value={value}>
      {children}
    </RouteContext.Provider>
  );
};

/**
 * 使用路由上下文的 Hook
 */
export const useRouteContext = (): RouteContextType => {
  const context = useContext(RouteContext);
  if (context === undefined) {
    throw new Error('useRouteContext must be used within a RouteProvider');
  }
  return context;
};

export default RouteContext;