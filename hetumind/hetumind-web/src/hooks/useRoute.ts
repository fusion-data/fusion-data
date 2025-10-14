import React from 'react';
import { useLocation, useNavigate, useParams } from 'react-router-dom';
import { getPageTitle, getBreadcrumbs, requiresAuth } from '@/router/utils';

/**
 * 路由相关的自定义 Hook
 */
export const useRoute = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const params = useParams();

  // 获取当前路径信息
  const pathname = location.pathname;
  const search = location.search;
  const hash = location.hash;
  const state = location.state;

  // 获取页面标题
  const title = getPageTitle(pathname);

  // 获取面包屑导航
  const breadcrumbs = getBreadcrumbs(pathname);

  // 检查是否需要认证
  const authRequired = requiresAuth(pathname);

  // 导航函数
  const goTo = (path: string, options?: { replace?: boolean; state?: any }) => {
    if (options?.replace) {
      navigate(path, { replace: true, state: options?.state });
    } else {
      navigate(path, { state: options?.state });
    }
  };

  // 返回上一页
  const goBack = () => {
    navigate(-1);
  };

  // 前进到下一页
  const goForward = () => {
    navigate(1);
  };

  // 刷新当前页面
  const refresh = () => {
    navigate(0);
  };

  // 推送新路由（但不导航）
  const push = (path: string, state?: any) => {
    window.history.pushState(state, '', path);
  };

  // 替换当前路由（但不导航）
  const replace = (path: string, state?: any) => {
    window.history.replaceState(state, '', path);
  };

  // 检查当前路径是否匹配指定模式
  const isPathMatch = (pattern: string): boolean => {
    if (pattern.includes(':')) {
      // 动态路由匹配
      const patternParts = pattern.split('/');
      const pathParts = pathname.split('/');

      if (patternParts.length !== pathParts.length) {
        return false;
      }

      return patternParts.every((part, index) => {
        return part.startsWith(':') || part === pathParts[index];
      });
    } else {
      return pathname === pattern;
    }
  };

  // 检查当前路径是否以指定前缀开始
  const isPathStartsWith = (prefix: string): boolean => {
    return pathname.startsWith(prefix);
  };

  // 获取查询参数
  const getQueryParams = (): Record<string, string> => {
    const params = new URLSearchParams(search);
    const result: Record<string, string> = {};

    params.forEach((value, key) => {
      result[key] = value;
    });

    return result;
  };

  // 获取单个查询参数
  const getQueryParam = (key: string): string | null => {
    const params = new URLSearchParams(search);
    return params.get(key);
  };

  // 设置查询参数
  const setQueryParams = (params: Record<string, string | number | boolean>) => {
    const searchParams = new URLSearchParams(search);

    Object.entries(params).forEach(([key, value]) => {
      if (value === null || value === undefined || value === '') {
        searchParams.delete(key);
      } else {
        searchParams.set(key, String(value));
      }
    });

    const newSearch = searchParams.toString();
    navigate(`${pathname}${newSearch ? `?${newSearch}` : ''}${hash}`, { replace: true });
  };

  // 删除查询参数
  const removeQueryParams = (keys: string[]) => {
    const searchParams = new URLSearchParams(search);

    keys.forEach(key => {
      searchParams.delete(key);
    });

    const newSearch = searchParams.toString();
    navigate(`${pathname}${newSearch ? `?${newSearch}` : ''}${hash}`, { replace: true });
  };

  // 清除所有查询参数
  const clearQueryParams = () => {
    navigate(`${pathname}${hash}`, { replace: true });
  };

  return {
    // 当前路由信息
    pathname,
    search,
    hash,
    state,
    params,

    // 计算属性
    title,
    breadcrumbs,
    authRequired,

    // 导航方法
    goTo,
    goBack,
    goForward,
    refresh,
    push,
    replace,

    // 路径匹配方法
    isPathMatch,
    isPathStartsWith,

    // 查询参数方法
    getQueryParams,
    getQueryParam,
    setQueryParams,
    removeQueryParams,
    clearQueryParams,
  };
};

/**
 * 路由守卫 Hook
 */
export const useRouteGuard = () => {
  const { authRequired } = useRoute();

  // TODO: 实现真实的认证逻辑
  // 这里暂时返回 true，后续会实现真实的认证检查
  const isAuthenticated = true;

  const canAccess = authRequired ? isAuthenticated : true;

  return {
    canAccess,
    isAuthenticated,
    authRequired,
  };
};

/**
 * 面包屑导航 Hook
 */
export const useBreadcrumbs = () => {
  const { breadcrumbs } = useRoute();
  return breadcrumbs;
};

/**
 * 页面标题 Hook
 */
export const usePageTitle = () => {
  const { title } = useRoute();

  // 自动设置页面标题
  React.useEffect(() => {
    document.title = title;
  }, [title]);

  return title;
};