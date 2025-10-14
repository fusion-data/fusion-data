export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  timestamp: string;
}

export interface PaginatedResponse<T = any> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface ErrorResponse {
  success: false;
  error: string;
  code?: string;
  details?: any;
  timestamp: string;
}

export interface LoginRequest {
  username: string;
  password: string;
  rememberMe?: boolean;
}

export interface LoginResponse {
  token: string;
  refreshToken: string;
  user: User;
  expiresIn: number;
}

export interface User {
  id: string;
  username: string;
  email: string;
  role: 'admin' | 'user' | 'viewer';
  permissions: string[];
  avatar?: string;
  createdAt: string;
  lastLoginAt?: string;
}

export interface CreateWorkflowRequest {
  name: string;
  description?: string;
  category?: string;
  tags?: string[];
}

export interface UpdateWorkflowRequest {
  name?: string;
  description?: string;
  nodes?: any[];
  edges?: any[];
  metadata?: any;
}

export interface RunWorkflowRequest {
  input?: any;
  options?: {
    debug?: boolean;
    timeout?: number;
  };
}

export interface CreateAgentRequest {
  name: string;
  description?: string;
  type: string;
  config: any;
}

export interface UpdateAgentRequest {
  name?: string;
  description?: string;
  config?: any;
  status?: string;
}

export interface ExecuteAgentRequest {
  input: any;
  options?: {
    stream?: boolean;
    timeout?: number;
  };
}

export interface SearchParams {
  query?: string;
  page?: number;
  pageSize?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  filters?: Record<string, any>;
}

export interface UploadResponse {
  url: string;
  filename: string;
  size: number;
  contentType: string;
}

export interface HealthCheckResponse {
  status: 'healthy' | 'unhealthy';
  timestamp: string;
  version: string;
  services: Record<string, 'healthy' | 'unhealthy'>;
}