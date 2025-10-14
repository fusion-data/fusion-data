import { ComponentType } from 'react';

/**
 * 基础路由配置接口
 */
export interface RouteConfig {
  path: string;
  title?: string;
  description?: string;
  icon?: React.ReactNode;
  hidden?: boolean;
  requireAuth?: boolean;
  roles?: string[];
  keepAlive?: boolean;
  meta?: Record<string, any>;
}

/**
 * 静态路由配置接口
 */
export interface StaticRouteConfig extends RouteConfig {
  component: ComponentType | React.LazyExoticComponent<ComponentType>;
  children?: StaticRouteConfig[];
  index?: boolean;
}

/**
 * 动态路由配置接口
 */
export interface DynamicRouteConfig extends RouteConfig {
  componentLoader: () => Promise<ComponentType>;
  children?: DynamicRouteConfig[];
}

/**
 * 菜单项配置接口
 */
export interface MenuItemConfig {
  key: string;
  label: string;
  icon?: React.ReactNode;
  path?: string;
  children?: MenuItemConfig[];
  hidden?: boolean;
  badge?: string | number;
  disabled?: boolean;
  external?: boolean;
  target?: '_blank' | '_self' | '_parent' | '_top';
}

/**
 * 面包屑项配置接口
 */
export interface BreadcrumbConfig {
  title: string;
  path?: string;
  icon?: React.ReactNode;
}

/**
 * 路由元数据接口
 */
export interface RouteMeta {
  title: string;
  description?: string;
  keywords?: string[];
  keepAlive?: boolean;
  affix?: boolean;
  noCache?: boolean;
  hidden?: boolean;
  icon?: React.ReactNode;
  breadcrumb?: boolean;
  activeMenu?: string;
  noTagsView?: boolean;
  followAuth?: string;
  showParent?: boolean;
  frameSrc?: string;
  frameBlank?: boolean;
  hiddenHeaderContent?: boolean;
  order?: number;
  roles?: string[];
  permissions?: string[];
}

/**
 * 路由上下文接口
 */
export interface RouteContextType {
  currentRoute: RouteConfig | null;
  breadcrumbs: BreadcrumbConfig[];
  menuItems: MenuItemConfig[];
  addRoute: (route: RouteConfig) => void;
  removeRoute: (path: string) => void;
  updateRoute: (path: string, route: Partial<RouteConfig>) => void;
  navigateTo: (path: string, state?: any) => void;
  goBack: () => void;
  goForward: () => void;
  refresh: () => void;
}

/**
 * 权限检查接口
 */
export interface PermissionGuard {
  roles?: string[];
  permissions?: string[];
  mode?: 'all' | 'any'; // 需要 all 所有权限或 any 任一权限
  fallback?: React.ReactNode;
  onUnauthorized?: () => void;
}

/**
 * 路由守卫接口
 */
export interface RouteGuard {
  beforeEnter?: (to: string, from: string, next: () => void) => boolean | void;
  afterEnter?: (to: string, from: string) => void;
  onLeave?: (to: string, from: string) => void;
}

/**
 * 滚动行为配置接口
 */
export interface ScrollBehavior {
  x?: number;
  y?: number;
  top?: boolean;
  left?: boolean;
  behavior?: 'auto' | 'smooth' | 'instant';
}

/**
 * 路由错误边界接口
 */
export interface RouteErrorBoundaryState {
  hasError: boolean;
  error?: Error;
  errorInfo?: React.ErrorInfo;
}

/**
 * 路由性能监控接口
 */
export interface RoutePerformanceMetrics {
  path: string;
  loadTime: number;
  renderTime: number;
  componentLoadTime?: number;
  timestamp: number;
}

/**
 * 路由缓存配置接口
 */
export interface RouteCacheConfig {
  max: number;
  ttl?: number; // 生存时间（毫秒）
  exclude?: string[]; // 排除的路径
  include?: string[]; // 包含的路径
}

// 类型已经在声明时导出，无需重复导出