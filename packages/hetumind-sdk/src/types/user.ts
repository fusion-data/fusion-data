/**
 * User related types based on backend API
 */

import { OpValBool, OpValDateTime, OpValString } from '@fusion-data/fusionsql';

/**
 * User entity
 */
export interface UserEntity {
  id: number;
  username: string;
  email?: string;
  display_name?: string;
  avatar_url?: string;
  is_active: boolean;
  is_admin: boolean;
  created_at: string;
  updated_at?: string;
  last_login_at?: string;
}

/**
 * Request for updating a user
 */
export interface UserForUpdate {
  username?: string;
  email?: string;
  display_name?: string;
  avatar_url?: string;
  is_active?: boolean;
  is_admin?: boolean;
}

export interface UserFilter {
  username?: OpValString;
  email?: OpValString;
  is_active?: OpValBool;
  is_admin?: OpValBool;
  created_at?: OpValDateTime;
}

/**
 * User query parameters
 */
export interface UserForPage {
  options: {
    page?: number;
    limit?: number;
    offset?: number;
    sort_by?: string;
    sort_order?: 'asc' | 'desc';
  };
  filters: {
    username?: { eq?: string; like?: string };
    email?: { eq?: string; like?: string };
    is_active?: { eq?: boolean };
    is_admin?: { eq?: boolean };
    created_at?: { from?: string; to?: string };
  }[];
}

/**
 * User password update request
 */
export interface UserPasswordUpdateRequest {
  old_password?: string;
  verification_code?: string;
  new_password: string;
}
