/**
 * Authentication API service
 * Implements Google OAuth 2.0 with PKCE for desktop and mobile
 * Uses local HTTP server on port 8085 for OAuth callback on all platforms
 */

import { invoke } from "@tauri-apps/api/core";
import type { AuthStatus } from "$lib/types/auth";

// Google OAuth configuration from environment
const GOOGLE_CLIENT_ID = import.meta.env.VITE_GOOGLE_CLIENT_ID;
const GOOGLE_CLIENT_SECRET = import.meta.env.VITE_GOOGLE_CLIENT_SECRET;
const GOOGLE_OAUTH_SCOPE = import.meta.env.VITE_GOOGLE_OAUTH_SCOPE;

/**
 * Get the current authentication status from backend
 */
export async function getAuthStatus(): Promise<AuthStatus> {
  return invoke<AuthStatus>("get_auth_status");
}

/**
 * Initiate Google OAuth login flow
 * Uses local HTTP server on port 8085 to receive callback
 */
export async function googleLogin(): Promise<AuthStatus> {
  return invoke<AuthStatus>("google_sign_in", {
    clientId: GOOGLE_CLIENT_ID,
    clientSecret: GOOGLE_CLIENT_SECRET,
    scope: GOOGLE_OAUTH_SCOPE,
  });
}

/**
 * Refresh the access token using stored refresh token
 */
export async function refreshToken(): Promise<AuthStatus> {
  return invoke<AuthStatus>("refresh_google_token", {
    clientId: GOOGLE_CLIENT_ID,
    clientSecret: GOOGLE_CLIENT_SECRET,
  });
}

/**
 * Logout from Google (clear stored tokens)
 */
export async function googleLogout(): Promise<AuthStatus> {
  return invoke<AuthStatus>("google_logout");
}

/**
 * Set auth token manually (for development/testing)
 */
export async function setAuthToken(
  accessToken: string,
  refreshToken?: string,
  email?: string,
  displayName?: string
): Promise<AuthStatus> {
  return invoke<AuthStatus>("set_auth_token", {
    accessToken,
    refreshToken,
    email,
    displayName,
  });
}
