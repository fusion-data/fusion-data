/**
 * TypeScript type definitions for Hetuflow SDK WebAssembly bindings
 *
 * This file provides TypeScript types for the WASM-compiled Hetuflow SDK.
 * Generated types ensure type safety when using the SDK in TypeScript projects.
 */

// Base types and interfaces
export interface QueryParams {
  page?: number;
  limit?: number;
  sort?: string;
  order?: 'asc' | 'desc';
  [key: string]: any;
}

export interface QueryResult<T = any> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  pages: number;
}

export interface ApiResponse<T = any> {
  data: T;
  success: boolean;
  message?: string;
  errors?: string[];
}

// Error types
export class HetuflowError extends Error {
  constructor(message: string);
  static fromSdkError(message: string): HetuflowError;
  static networkError(message: string): HetuflowError;
  static validationError(message: string): HetuflowError;
  static configError(message: string): HetuflowError;
}

// Configuration
export class WasmConfig {
  constructor(base_url: string);

  // Authentication
  set auth_token(token: string | null);
  get auth_token(): string | null;

  // Headers
  setHeader(name: string, value: string): void;

  // Timeout
  set timeout(milliseconds: number);
  get timeout(): number;

  // Compression
  set compression(enabled: boolean);
  get compression(): boolean;
}

// Base API interface
interface BaseApi {
  query(params?: QueryParams): Promise<QueryResult>;
  get(id: string): Promise<any>;
  create(data: any): Promise<any>;
  update(id: string, data: any): Promise<any>;
  delete(id: string): Promise<void>;
}

// API Classes
export class WasmAgentsApi implements BaseApi {
  query(params?: QueryParams): Promise<QueryResult<Agent>>;
  get(id: string): Promise<Agent>;
  create(data: CreateAgentRequest): Promise<Agent>;
  update(id: string, data: UpdateAgentRequest): Promise<Agent>;
  delete(id: string): Promise<void>;
}

export class WasmJobsApi implements BaseApi {
  query(params?: QueryParams): Promise<QueryResult<Job>>;
  get(id: string): Promise<Job>;
  create(data: CreateJobRequest): Promise<Job>;
  update(id: string, data: UpdateJobRequest): Promise<Job>;
  delete(id: string): Promise<void>;

  // Job-specific methods
  start(id: string): Promise<Job>;
  stop(id: string): Promise<Job>;
  pause(id: string): Promise<Job>;
  resume(id: string): Promise<Job>;
}

export class WasmTasksApi implements BaseApi {
  query(params?: QueryParams): Promise<QueryResult<Task>>;
  get(id: string): Promise<Task>;
  create(data: CreateTaskRequest): Promise<Task>;
  update(id: string, data: UpdateTaskRequest): Promise<Task>;
  delete(id: string): Promise<void>;
}

export class WasmSchedulesApi implements BaseApi {
  query(params?: QueryParams): Promise<QueryResult<Schedule>>;
  get(id: string): Promise<Schedule>;
  create(data: CreateScheduleRequest): Promise<Schedule>;
  update(id: string, data: UpdateScheduleRequest): Promise<Schedule>;
  delete(id: string): Promise<void>;

  // Schedule-specific methods
  enable(id: string): Promise<Schedule>;
  disable(id: string): Promise<Schedule>;
}

export class WasmTaskInstancesApi implements BaseApi {
  query(params?: QueryParams): Promise<QueryResult<TaskInstance>>;
  get(id: string): Promise<TaskInstance>;
  // Task instances typically cannot be created/updated/deleted directly
  create(data: any): Promise<never>;
  update(id: string, data: any): Promise<never>;
  delete(id: string): Promise<never>;

  // Task instance specific methods
  retry(id: string): Promise<TaskInstance>;
  cancel(id: string): Promise<TaskInstance>;
  getLogs(id: string): Promise<string[]>;
}

export class WasmServersApi implements BaseApi {
  query(params?: QueryParams): Promise<QueryResult<Server>>;
  get(id: string): Promise<Server>;
  create(data: CreateServerRequest): Promise<Server>;
  update(id: string, data: UpdateServerRequest): Promise<Server>;
  delete(id: string): Promise<void>;

  // Server-specific methods
  connect(id: string): Promise<Server>;
  disconnect(id: string): Promise<Server>;
  getStatus(id: string): Promise<ServerStatus>;
}

export class WasmSystemApi {
  // System information
  info(): Promise<SystemInfo>;
  health(): Promise<HealthStatus>;
  metrics(): Promise<SystemMetrics>;
  version(): Promise<string>;

  // System operations
  shutdown(): Promise<void>;
  restart(): Promise<void>;
}

export class WasmGatewayApi {
  // Gateway operations
  route(request: GatewayRequest): Promise<GatewayResponse>;
  getRoutes(): Promise<Route[]>;
  addRoute(route: RouteConfig): Promise<Route>;
  removeRoute(routeId: string): Promise<void>;
}

export class WasmAuthApi {
  // Authentication
  login(credentials: LoginCredentials): Promise<AuthResult>;
  logout(): Promise<void>;
  refresh(): Promise<AuthResult>;
  verify(): Promise<boolean>;

  // Token management
  getToken(): Promise<string | null>;
  setToken(token: string): Promise<void>;

  // User management
  getCurrentUser(): Promise<User>;
  updateProfile(profile: UserProfile): Promise<User>;
}

// Main client class
export class WasmHetuflowClient {
  constructor(base_url: string);
  static withConfig(config: WasmConfig): WasmHetuflowClient;

  // API accessors
  get agents(): WasmAgentsApi;
  get jobs(): WasmJobsApi;
  get tasks(): WasmTasksApi;
  get schedules(): WasmSchedulesApi;
  get task_instances(): WasmTaskInstancesApi;
  get servers(): WasmServersApi;
  get system(): WasmSystemApi;
  get gateway(): WasmGatewayApi;
  get auth(): WasmAuthApi;
}

// Utility functions
export class WasmUtils {
  static errorToString(error: Error): string;
  static isPromise(value: any): boolean;
  static timestampMs(): number;
  static safeJsonStringify(value: any): string;
}

// Model types (these would be generated from your Rust models)
export interface Agent {
  id: string;
  name: string;
  status: 'online' | 'offline' | 'busy';
  capabilities: string[];
  metadata: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface Job {
  id: string;
  name: string;
  description?: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  tasks: Task[];
  schedule?: Schedule;
  created_at: string;
  updated_at: string;
  started_at?: string;
  completed_at?: string;
}

export interface Task {
  id: string;
  job_id: string;
  name: string;
  type: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped';
  input: any;
  output?: any;
  dependencies: string[];
  created_at: string;
  updated_at: string;
  started_at?: string;
  completed_at?: string;
}

export interface TaskInstance {
  id: string;
  task_id: string;
  job_id: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  input: any;
  output?: any;
  error?: string;
  agent_id?: string;
  logs: string[];
  created_at: string;
  updated_at: string;
  started_at?: string;
  completed_at?: string;
}

export interface Schedule {
  id: string;
  name: string;
  job_id: string;
  cron_expression: string;
  enabled: boolean;
  timezone?: string;
  next_run?: string;
  last_run?: string;
  created_at: string;
  updated_at: string;
}

export interface Server {
  id: string;
  name: string;
  address: string;
  port: number;
  status: 'online' | 'offline' | 'error';
  version: string;
  capabilities: string[];
  metadata: Record<string, any>;
  created_at: string;
  updated_at: string;
  last_seen?: string;
}

// Request types
export interface CreateAgentRequest {
  name: string;
  capabilities: string[];
  metadata?: Record<string, any>;
}

export interface UpdateAgentRequest {
  name?: string;
  capabilities?: string[];
  metadata?: Record<string, any>;
}

export interface CreateJobRequest {
  name: string;
  description?: string;
  tasks: CreateTaskRequest[];
  schedule?: CreateScheduleRequest;
}

export interface UpdateJobRequest {
  name?: string;
  description?: string;
  tasks?: CreateTaskRequest[];
  schedule?: CreateScheduleRequest;
}

export interface CreateTaskRequest {
  name: string;
  type: string;
  input: any;
  dependencies?: string[];
}

export interface UpdateTaskRequest {
  name?: string;
  type?: string;
  input?: any;
  dependencies?: string[];
}

export interface CreateScheduleRequest {
  name: string;
  job_id: string;
  cron_expression: string;
  enabled?: boolean;
  timezone?: string;
}

export interface UpdateScheduleRequest {
  name?: string;
  cron_expression?: string;
  enabled?: boolean;
  timezone?: string;
}

export interface CreateServerRequest {
  name: string;
  address: string;
  port: number;
  capabilities?: string[];
  metadata?: Record<string, any>;
}

export interface UpdateServerRequest {
  name?: string;
  address?: string;
  port?: number;
  capabilities?: string[];
  metadata?: Record<string, any>;
}

// Additional system types
export interface SystemInfo {
  version: string;
  build_time: string;
  git_commit: string;
  uptime: number;
  memory_usage: number;
  cpu_usage: number;
}

export interface HealthStatus {
  status: 'healthy' | 'unhealthy' | 'degraded';
  checks: HealthCheck[];
}

export interface HealthCheck {
  name: string;
  status: 'pass' | 'fail' | 'warn';
  message?: string;
  duration_ms?: number;
}

export interface SystemMetrics {
  timestamp: string;
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
  network_io: NetworkIO;
  active_jobs: number;
  completed_jobs: number;
  failed_jobs: number;
}

export interface NetworkIO {
  bytes_sent: number;
  bytes_received: number;
  packets_sent: number;
  packets_received: number;
}

export interface ServerStatus {
  id: string;
  status: 'online' | 'offline' | 'error';
  last_seen?: string;
  error_message?: string;
  metrics?: SystemMetrics;
}

export interface GatewayRequest {
  path: string;
  method: string;
  headers?: Record<string, string>;
  body?: any;
}

export interface GatewayResponse {
  status: number;
  headers: Record<string, string>;
  body: any;
}

export interface Route {
  id: string;
  path: string;
  method: string;
  target: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface RouteConfig {
  path: string;
  method: string;
  target: string;
  enabled?: boolean;
}

export interface LoginCredentials {
  username: string;
  password: string;
}

export interface AuthResult {
  token: string;
  refresh_token?: string;
  expires_in: number;
  user: User;
}

export interface User {
  id: string;
  username: string;
  email?: string;
  roles: string[];
  permissions: string[];
  created_at: string;
  updated_at: string;
}

export interface UserProfile {
  email?: string;
  display_name?: string;
  avatar_url?: string;
  preferences?: Record<string, any>;
}