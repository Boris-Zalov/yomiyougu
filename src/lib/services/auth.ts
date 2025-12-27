/**
 * Authentication API service
 * Implements Google OAuth 2.0 with PKCE for desktop and mobile
 */

import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { AuthStatus } from "$lib/types/auth";

// Google OAuth configuration from environment
const GOOGLE_CLIENT_ID = import.meta.env.VITE_GOOGLE_CLIENT_ID;
const GOOGLE_CLIENT_SECRET = import.meta.env.VITE_GOOGLE_CLIENT_SECRET;
const GOOGLE_OAUTH_SCOPE = import.meta.env.VITE_GOOGLE_OAUTH_SCOPE;

const REDIRECT_URI = "http://localhost:8085";

/** OAuth config returned from backend */
interface OAuthConfig {
  authUrl: string;
  state: string;
  codeVerifier: string;
}

// Store pending OAuth state for verification
let pendingOAuthState: { state: string; codeVerifier: string } | null = null;

/**
 * Get the current authentication status from backend
 */
export async function getAuthStatus(): Promise<AuthStatus> {
  return invoke<AuthStatus>("get_auth_status");
}

/**
 * Initiate Google OAuth login flow
 * Opens browser for OAuth consent, user must manually enter the code
 * (For production, implement a local HTTP server to catch the callback)
 */
export async function googleLogin(): Promise<AuthStatus> {
  const config = await invoke<OAuthConfig>("get_google_auth_url", {
    clientId: GOOGLE_CLIENT_ID,
    redirectUri: REDIRECT_URI,
    scope: GOOGLE_OAUTH_SCOPE,
  });

  // Store state for verification
  pendingOAuthState = {
    state: config.state,
    codeVerifier: config.codeVerifier,
  };

  // Open the OAuth URL in the default browser
  await openUrl(config.authUrl);

  // Return current status - the actual login will complete via handleOAuthCallback
  return getAuthStatus();
}

/**
 * Handle OAuth callback with authorization code
 * Call this when the OAuth callback is received
 */
export async function handleOAuthCallback(
  code: string,
  state: string
): Promise<AuthStatus> {
  // Verify state matches
  if (!pendingOAuthState || pendingOAuthState.state !== state) {
    throw new Error("Invalid OAuth state - possible CSRF attack");
  }

  const codeVerifier = pendingOAuthState.codeVerifier;
  pendingOAuthState = null;

  // Exchange code for tokens
  return invoke<AuthStatus>("exchange_google_code", {
    code,
    codeVerifier,
    clientId: GOOGLE_CLIENT_ID,
    clientSecret: GOOGLE_CLIENT_SECRET,
    redirectUri: REDIRECT_URI,
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
  pendingOAuthState = null;
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

/**
 * Get the pending OAuth state (for callback handling)
 */
export function getPendingOAuthState() {
  return pendingOAuthState;
}

/**
 * Set pending OAuth state (for restoring from storage after app restart)
 */
export function setPendingOAuthState(state: string, codeVerifier: string) {
  pendingOAuthState = { state, codeVerifier };
}
