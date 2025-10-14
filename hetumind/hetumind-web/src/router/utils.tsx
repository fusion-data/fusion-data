import { RouteObject, Navigate } from 'react-router-dom';
import { publicRoutes, protectedRoutes, notFoundRoute, redirectRoutes, LazyWrapper, AuthGuard } from './index';

/**
 * 将路由配置转换为 React Router 的 RouteObject 格式
 */
export const generateRoutes = (): RouteObject[] => {
  const routes: RouteObject[] = [];

  // 添加公共路由
  publicRoutes.forEach(route => {
    routes.push({
      path: route.path,
      element: <LazyWrapper><route.component /></LazyWrapper>,
    });
  });

  // 添加受保护的路由
  routes.push({
    path: '/',
    element: <AuthGuard><div /></AuthGuard>,
    children: [
      // 重定向
      ...redirectRoutes.map(redirect => ({
        index: redirect.from === '/',
        path: redirect.from === '/' ? undefined : redirect.from.slice(1),
        element: <Navigate to={redirect.to} replace />
      })),

      // 受保护的子路由
      ...protectedRoutes.map(route => {
        if (route.children) {
          // 嵌套路由处理（如设置页面）
          return {
            path: route.path.slice(2), // 移除开头的 '/'
            element: <LazyWrapper><route.component /></LazyWrapper>,
            children: route.children.map(child => ({
              index: child.index || false,
              path: child.path.split('/').pop(), // 获取最后一部分路径
              element: <LazyWrapper><child.component /></LazyWrapper>,
            }))
          };
        } else {
          return {
            path: route.path.slice(2), // 移除开头的 '/'
            element: <LazyWrapper><route.component /></LazyWrapper>,
          };
        }
      }),
    ],
  });

  // 添加 404 路由
  routes.push({
    path: notFoundRoute.path,
    element: <LazyWrapper><notFoundRoute.component /></LazyWrapper>,
  });

  return routes;
};

/**
 * 根据路径获取页面标题
 */
export const getPageTitle = (pathname: string): string => {
  // 检查公共路由
  const publicRoute = publicRoutes.find(route => route.path === pathname);
  if (publicRoute) {
    return publicRoute.title || '登录';
  }

  // 检查受保护的路由
  const protectedRoute = protectedRoutes.find(route => {
    if (route.path.includes(':')) {
      // 动态路由匹配
      const routeParts = route.path.split('/');
      const pathParts = pathname.split('/');

      if (routeParts.length !== pathParts.length) {
        return false;
      }

      return routeParts.every((part, index) => {
        return part.startsWith(':') || part === pathParts[index];
      });
    } else {
      return route.path === pathname;
    }
  });

  if (protectedRoute) {
    return protectedRoute.title || '页面';
  }

  // 检查嵌套路由
  for (const route of protectedRoutes) {
    if (route.children) {
      const childRoute = route.children.find(child => child.path === pathname);
      if (childRoute) {
        return childRoute.title || '页面';
      }
    }
  }

  return 'Hetumind';
};

/**
 * 检查路径是否需要认证
 */
export const requiresAuth = (pathname: string): boolean => {
  return !publicRoutes.some(route => route.path === pathname);
};

/**
 * 获取面包屑导航数据
 */
export interface BreadcrumbItem {
  title: string;
  path?: string;
}

export const getBreadcrumbs = (pathname: string): BreadcrumbItem[] => {
  const breadcrumbs: BreadcrumbItem[] = [
    { title: '首页', path: '/dashboard' }
  ];

  const pathSegments = pathname.split('/').filter(Boolean);

  if (pathSegments.length === 0) {
    return breadcrumbs;
  }

  const firstSegment = pathSegments[0];

  // 主要页面映射
  const mainPageMap: Record<string, string> = {
    'dashboard': '仪表板',
    'workflows': '工作流',
    'agents': 'AI 智能体',
    'settings': '设置',
  };

  if (mainPageMap[firstSegment]) {
    breadcrumbs.push({
      title: mainPageMap[firstSegment],
      path: `/${firstSegment}`
    });
  }

  // 处理工作流相关页面
  if (firstSegment === 'workflows' && pathSegments.length > 1) {
    const secondSegment = pathSegments[1];

    if (secondSegment === 'new') {
      breadcrumbs.push({ title: '创建工作流' });
    } else if (pathSegments.length === 2) {
      // 工作流详情页
      breadcrumbs.push({ title: '工作流详情' });
    } else if (pathSegments.length === 3) {
      const thirdSegment = pathSegments[2];

      if (thirdSegment === 'executions') {
        breadcrumbs.push({ title: '执行记录' });
      } else if (thirdSegment === 'detail') {
        breadcrumbs.push({ title: '工作流详情' });
      }
    }
  }

  // 处理 AI 智能体相关页面
  if (firstSegment === 'agents' && pathSegments.length > 1) {
    const secondSegment = pathSegments[1];

    if (secondSegment === 'new') {
      breadcrumbs.push({ title: '创建智能体' });
    } else if (pathSegments.length === 2) {
      // 智能体详情页
      breadcrumbs.push({ title: '智能体详情' });
    } else if (pathSegments.length === 3) {
      const thirdSegment = pathSegments[2];

      if (thirdSegment === 'test') {
        breadcrumbs.push({ title: '测试智能体' });
      }
    }
  }

  // 处理设置页面
  if (firstSegment === 'settings' && pathSegments.length > 1) {
    const secondSegment = pathSegments[1];

    const settingsPageMap: Record<string, string> = {
      'models': '模型设置',
      'integrations': '集成设置',
      'credentials': '认证设置',
    };

    if (settingsPageMap[secondSegment]) {
      breadcrumbs.push({ title: settingsPageMap[secondSegment] });
    } else {
      breadcrumbs.push({ title: '通用设置' });
    }
  }

  return breadcrumbs;
};