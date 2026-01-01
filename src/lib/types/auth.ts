/**
 * TypeScript types for authentication
 * These mirror the structures in src-tauri/src/auth/types.rs
 */

/** Authentication status returned by get_auth_status */
export interface AuthStatus {
  isAuthenticated: boolean;
  /** Whether the access token needs refreshing (but we have a refresh token) */
  needsRefresh: boolean;
  email?: string;
  displayName?: string;
}
