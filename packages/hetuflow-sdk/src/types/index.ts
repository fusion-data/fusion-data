import { Page, Paged, PageResult } from '@fusion-data/modelsql/page';

export interface AgentForCreate {
  name: string;
  description?: string;
  config?: Record<string, any>;
}

export interface AgentForUpdate {
  name?: string;
  description?: string;
  config?: Record<string, any>;
}

export interface AgentFilter {
  id?: Record<string, string>;
  status?: Record<string, number>;
  address?: Record<string, string>;
  last_heartbeat_at?: Record<string, string>;
  created_at?: Record<string, string>;
  updated_at?: Record<string, string>;
}

export interface AgentForQuery {
  page: Page;
  filter: AgentFilter;
}

export interface SchedAgent {
  id: string;
  name: string;
  description?: string;
  config?: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface JobForCreate {
  name: string;
  description?: string;
  cron_expr?: string;
  agent_id: string;
  config?: Record<string, any>;
}

export interface JobForUpdate {
  name?: string;
  description?: string;
  cron_expr?: string;
  agent_id?: string;
  config?: Record<string, any>;
}

export interface JobFilter {
  id?: Record<string, string | undefined>;
  name?: Record<string, string | undefined>;
  namespace_id?: Record<string, string | undefined>;
  status?: Record<string, number | undefined>;
  created_at?: Record<string, string | undefined>;
  last_heartbeat_at?: Record<string, string | undefined>;
}

export interface JobForQuery {
  page: Page;
  filter: JobFilter;
}

export const JobStatus = {
  CREATED: 1,
  EXPIRED: 98,
  DISABLED: 99,
  ENABLED: 100,
} as const;

export type JobStatus = (typeof JobStatus)[keyof typeof JobStatus];

export interface SchedJob {
  id: string;
  name: string;
  description?: string;
  cron_expr?: string;
  agent_id: string;
  config?: Record<string, any>;
  status: JobStatus;
  created_at: string;
  updated_at: string;
}

export interface TaskForCreate {
  name: string;
  description?: string;
  job_id: string;
  config?: Record<string, any>;
}

export interface TaskForUpdate {
  name?: string;
  description?: string;
  config?: Record<string, any>;
}

export interface TaskFilter {
  id?: Record<string, string | undefined>;
  job_id?: Record<string, string | undefined>;
  schedule_id?: Record<string, string | undefined>;
  namespace_id?: Record<string, string | undefined>;
  task_config?: Record<string, string | undefined>;
  status?: Record<string, number | undefined>;
  scheduled_at?: Record<string, string | undefined>;
  locked_at?: Record<string, string | undefined>;
  created_at?: Record<string, string | undefined>;
  updated_at?: Record<string, string | undefined>;
}

export interface TaskForQuery {
  page: Page;
  filter: TaskFilter;
}

export interface SchedTask {
  id: string;
  name: string;
  description?: string;
  job_id: string;
  config?: Record<string, any>;
  status: TaskStatus;
  created_at: string;
  updated_at: string;
}

export interface TaskInstanceForCreate {
  task_id: string;
  config?: Record<string, any>;
}

export interface TaskInstanceForUpdate {
  config?: Record<string, any>;
  status?: TaskInstanceStatus;
}

export interface TaskInstanceFilter {
  id?: Record<string, string | undefined>;
  task_id?: Record<string, string | undefined>;
  agent_id?: Record<string, string | undefined>;
  status?: Record<string, number | undefined>;
  started_at?: Record<string, string | undefined>;
  completed_at?: Record<string, string | undefined>;
  created_at?: Record<string, string | undefined>;
  updated_at?: Record<string, string | undefined>;
}

export interface TaskInstanceForQuery {
  page: Page;
  filter: TaskInstanceFilter;
}

export interface SchedTaskInstance {
  id: string;
  task_id: string;
  config?: Record<string, any>;
  status: TaskInstanceStatus;
  result?: Record<string, any>;
  started_at?: string;
  finished_at?: string;
  created_at: string;
  updated_at: string;
}

// Server 相关类型定义
export interface ServerForUpdate {
  name?: string;
  address?: string;
  bind_namespaces?: string[];
  status?: ServerStatus;
  description?: string;
}

export interface ServerFilter {
  id?: Record<string, string>;
  name?: Record<string, string>;
  bind_namespaces?: Record<string, string>;
  status?: Record<string, number>;
  address?: Record<string, string>;
  created_at?: Record<string, string>;
  updated_at?: Record<string, string>;
}

export interface ServerForQuery {
  page: Page;
  filter: ServerFilter;
}

export interface SchedServer {
  id: string;
  name: string;
  address: string;
  bind_namespaces: string[];
  status: ServerStatus;
  description?: string;
  last_heartbeat_at: string;
  created_at: string;
}

export interface GenerateTokenRequest {
  subject: string;
  expires_in?: number;
}

export interface GenerateTokenResponse {
  token: string;
  expires_at: string;
}

export interface GatewayCommandRequest {
  agent_id: string;
  command: string;
  args?: Record<string, any>;
}

export interface HealthStatus {
  status: 'healthy' | 'unhealthy';
  checks: Record<string, any>;
}

export type PageResult_SchedAgent = PageResult<SchedAgent>;
export type PageResult_SchedJob = PageResult<SchedJob>;
export type PageResult_SchedTask = PageResult<SchedTask>;
export type PageResult_SchedTaskInstance = PageResult<SchedTaskInstance>;
export type PageResult_SchedServer = PageResult<SchedServer>;

export interface IdStringResult {
  id: string;
}

export interface IdUuidResult {
  id: string;
}

export interface WebError {
  code: string;
  message: string;
  details?: Record<string, any>;
}

export const TaskStatus = {
  /// 等待分发
  Pending: 1,
  /// 进行中，具体情况见最新的 SchedTaskInstance
  Doing: 10,
  /// 错误
  Failed: 90,
  /// 取消完成
  Cancelled: 99,
  /// 成功完成
  Succeeded: 100,
} as const;

export type TaskStatus = (typeof TaskStatus)[keyof typeof TaskStatus];

export const TaskInstanceStatus = {
  // 等待分发
  Pending: 1,
  // 已分发
  Dispatched: 5,
  // 运行中
  Running: 10,
  // 超时
  Timeout: 20,
  // 暂停
  Paused: 30,
  // 跳过
  Skipped: 40,
  // 失败
  Failed: 90,
  // 取消
  Cancelled: 99,
  // 成功
  Succeeded: 100,
} as const;

export type TaskInstanceStatus = (typeof TaskInstanceStatus)[keyof typeof TaskInstanceStatus];

export const ServerStatus = {
  // 非活跃
  Inactive: 99,
  // 活跃
  Active: 100,
} as const;

export type ServerStatus = (typeof ServerStatus)[keyof typeof ServerStatus];
