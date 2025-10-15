import { PageResult } from '@fusion-data/fusionsql';
import { HetumindClient } from '../utils/client.js';
import {
  UserForUpdate,
  UserForPage,
  UserEntity,
  UserPasswordUpdateRequest,
} from '../types/index.js';

export class UserAPI {
  constructor(private client: HetumindClient) {}

  /**
   * Get user by ID
   */
  async getUserById(id: number): Promise<UserEntity | null> {
    return this.client.get<UserEntity | null>(`/api/v1/users/item/${id}`);
  }

  /**
   * Update user by ID
   */
  async updateUserById(id: number, data: UserForUpdate): Promise<void> {
    return this.client.put<void>(`/api/v1/users/item/${id}`, data);
  }

  /**
   * Update user password
   */
  async updateUserPassword(id: number, data: UserPasswordUpdateRequest): Promise<void> {
    return this.client.put<void>(`/api/v1/users/item/${id}/password`, data);
  }

  /**
   * Query users with filters
   */
  async queryUsers(query: UserForPage): Promise<PageResult<UserEntity>> {
    return this.client.post<PageResult<UserEntity>>('/api/v1/users/query', query);
  }
}
