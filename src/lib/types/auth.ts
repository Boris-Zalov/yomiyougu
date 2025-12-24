/**
 * TypeScript types for authentication
 * These mirror the structures in src-tauri/src/auth/types.rs
 */

/** Authentication status returned by get_auth_status */
export interface AuthStatus {
  isAuthenticated: boolean;
  email?: string;
  displayName?: string;
}
