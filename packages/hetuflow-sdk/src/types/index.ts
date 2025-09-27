import { Page, Paged, PageResult } from "@fusion-data/modelsql/page";

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
  last_heartbeat?: Record<string, string>;
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

export interface JobForQuery {
  page?: number;
  page_size?: number;
  name?: string;
  agent_id?: string;
}

export interface SchedJob {
  id: string;
  name: string;
  description?: string;
  cron_expr?: string;
  agent_id: string;
  config?: Record<string, any>;
  enabled: boolean;
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

export interface TaskForQuery {
  page?: number;
  page_size?: number;
  name?: string;
  job_id?: string;
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

export interface TaskInstanceForQuery {
  page?: number;
  page_size?: number;
  task_id?: string;
  status?: TaskInstanceStatus;
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
  status: "healthy" | "unhealthy";
  checks: Record<string, any>;
}

export type PageResult_SchedAgent = PageResult<SchedAgent>;
export type PageResult_SchedJob = PageResult<SchedJob>;
export type PageResult_SchedTask = PageResult<SchedTask>;
export type PageResult_SchedTaskInstance = PageResult<SchedTaskInstance>;

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

export enum TaskStatus {
  PENDING = "pending",
  RUNNING = "running",
  SUCCESS = "success",
  FAILED = "failed",
  CANCELLED = "cancelled",
}

export enum TaskInstanceStatus {
  PENDING = "pending",
  RUNNING = "running",
  SUCCESS = "success",
  FAILED = "failed",
  CANCELLED = "cancelled",
}
