import { Page, PageResult } from '@fusion-data/modelsql/page';

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

export interface AgentCapabilities {
  max_concurrent_tasks: number;
  labels: Record<string, string>;
  metadata: Record<string, string>;
}

export interface AgentStatistics {
  success_tasks: number;
  failure_tasks: number;
  total_tasks: number;
  avg_response_ms: number;
  last_failure_ms: number;
  consecutive_failures: number;
}

export const AgentStatus = {
  // 空闲
  Idle: 10,
  // 忙碌
  Busy: 20,
  // 连接中
  Connecting: 30,
  // 断开连接中
  Disconnecting: 31,
  // 离线
  Offline: 90,
  // 错误状态
  Error: 99,
  // 在线
  Online: 100,
} as const;
export type AgentStatus = (typeof AgentStatus)[keyof typeof AgentStatus];
export const AgentStatusColor = {
  [AgentStatus.Idle]: 'green',
  [AgentStatus.Busy]: 'orange',
  [AgentStatus.Connecting]: 'blue',
  [AgentStatus.Disconnecting]: 'blue',
  [AgentStatus.Offline]: 'red',
  [AgentStatus.Error]: 'red',
  [AgentStatus.Online]: 'green',
} as const;
export const AgentStatusText = {
  [AgentStatus.Idle]: '空闲',
  [AgentStatus.Busy]: '忙碌',
  [AgentStatus.Connecting]: '连接中',
  [AgentStatus.Disconnecting]: '断开连接中',
  [AgentStatus.Offline]: '离线',
  [AgentStatus.Error]: '错误状态',
  [AgentStatus.Online]: '在线',
} as const;

export interface SchedAgent {
  id: string;
  name: string;
  description?: string;
  capabilities: AgentCapabilities;
  statistics: AgentStatistics;
  status: AgentStatus;
  created_at: string;
  last_heartbeat_at: string;
}

export const ExecuteCommand = {
  Bash: 'bash',
  Uv: 'uv',
  Python: 'python',
  Node: 'node',
  Npx: 'npx',
  Cargo: 'cargo',
  Java: 'java',
} as const;

export type ExecuteCommand = (typeof ExecuteCommand)[keyof typeof ExecuteCommand];

export interface ResourceLimits {
  /// 最大内存使用量 (MB)
  max_memory_mb?: number;
  /// 最大CPU使用率 (0.0-1.0)
  max_cpu_percent?: number;
  /// 最大执行时间 (秒)
  max_execution_time_secs?: number;
  /// 最大输出大小 (字节)
  max_output_size_bytes?: number;
}

export interface TaskConfig {
  timeout: number;
  max_retries: number;
  retry_interval: number;
  cmd: ExecuteCommand;
  args: string[];
  capture_output: boolean;
  max_output_size: number;
  labels?: Record<string, string>;
  resource_limits?: ResourceLimits;
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
  namespace_id: string;
  name: string;
  description?: string;
  environment?: Record<string, string | number>;
  config?: TaskConfig;
  status: JobStatus;
  created_at: string;
  updated_at: string;
}

export const ScheduleKind = {
  /// Cron 定时作业
  Cron: 1,
  /// 间隔定时作业。可以通过设置最大执行次数为 1 次来表达 Once 执行，可以通过设置 start_time 来设置定时执行时间
  Interval: 2,
  /// 守护进程作业
  Daemon: 3,
  /// 事件驱动作业
  Event: 4,
  /// 流程任务
  Flow: 5,
} as const;

export type ScheduleKind = (typeof ScheduleKind)[keyof typeof ScheduleKind];

export const ScheduleKindText = {
  [ScheduleKind.Cron]: 'Cron 定时作业',
  [ScheduleKind.Interval]: '间隔定时作业',
  [ScheduleKind.Daemon]: '守护进程作业',
  [ScheduleKind.Event]: '事件驱动作业',
  [ScheduleKind.Flow]: '流程任务',
} as const;

export const ScheduleStatus = {
  /// 已创建
  Created: 1,
  /// 调度已过期，不再生成有效
  Expired: 98,
  /// 已禁用
  Disabled: 99,
  /// 已启用
  Enabled: 100,
} as const;

export type ScheduleStatus = (typeof ScheduleStatus)[keyof typeof ScheduleStatus];

export const ScheduleStatusText = {
  [ScheduleStatus.Created]: '已创建',
  [ScheduleStatus.Expired]: '调度已过期',
  [ScheduleStatus.Disabled]: '已禁用',
  [ScheduleStatus.Enabled]: '已启用',
} as const;

export interface SchedSchedule {
  id: string;
  job_id: string;
  name?: string;
  description?: string;
  schedule_kind: ScheduleKind;
  start_time?: string;
  end_time?: string;
  status: ScheduleStatus;
  cron_expression?: string;
  interval_secs?: number;
  max_count?: number;
  next_run_at?: string;
  created_by: number;
  created_at: string;
  updated_by?: number;
  updated_at?: string;
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
export type PageResult_SchedSchedule = PageResult<SchedSchedule>;
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
