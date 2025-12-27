<!--
  SyncSettingsCategory - Special sync settings category with Google OAuth integration
-->
<script lang="ts">
  import { Card, Heading, Helper, Button, Spinner, Toggle, Label, Badge, Input } from "flowbite-svelte";
  import { CloudArrowUpOutline, GoogleSolid, CloseOutline } from "flowbite-svelte-icons";
  import type { SettingCategory as CategoryType, SettingValue } from "$lib/types/settings";
  import { authApi, type AuthStatus } from "$lib";

  interface Props {
    category: CategoryType;
    authStatus: AuthStatus;
    onchange?: (key: string, value: SettingValue) => void;
    onAuthChange?: () => void;
  }

  let { category, authStatus, onchange, onAuthChange }: Props = $props();
  
  let isLoggingIn = $state(false);
  let isLoggingOut = $state(false);
  let isExchangingCode = $state(false);
  let loginError = $state<string | null>(null);
  let showCodeInput = $state(false);
  let authCode = $state("");

  async function handleGoogleLogin() {
    isLoggingIn = true;
    loginError = null;
    
    try {
      await authApi.googleLogin();
      // Show the code input after opening browser
      showCodeInput = true;
    } catch (err) {
      console.error("Failed to open login:", err);
      loginError = typeof err === "string" ? err : "Failed to open Google login";
    } finally {
      isLoggingIn = false;
    }
  }

  function cancelLogin() {
    showCodeInput = false;
    authCode = "";
    loginError = null;
  }

  async function submitAuthCode() {
    if (!authCode.trim()) {
      loginError = "Please enter the authorization code";
      return;
    }
    
    isExchangingCode = true;
    loginError = null;
    
    try {
      // Extract code from URL if user pasted the full redirect URL
      let code = authCode.trim();
      const urlMatch = code.match(/[?&]code=([^&]+)/);
      if (urlMatch) {
        code = decodeURIComponent(urlMatch[1]);
      }
      
      // Extract state from URL if present
      const stateMatch = authCode.match(/[?&]state=([^&]+)/);
      const state = stateMatch ? decodeURIComponent(stateMatch[1]) : "";
      
      // Get stored pending state
      const pendingState = authApi.getPendingOAuthState();
      if (!pendingState) {
        throw new Error("No pending OAuth session. Please try signing in again.");
      }
      
      await authApi.handleOAuthCallback(code, state || pendingState.state);
      showCodeInput = false;
      authCode = "";
      onAuthChange?.();
    } catch (err) {
      console.error("Failed to exchange code:", err);
      loginError = typeof err === "string" ? err : (err as Error).message || "Failed to complete login";
    } finally {
      isExchangingCode = false;
    }
  }

  async function handleGoogleLogout() {
    isLoggingOut = true;
    
    try {
      await authApi.googleLogout();
      onAuthChange?.();
    } catch (err) {
      console.error("Failed to logout:", err);
    } finally {
      isLoggingOut = false;
    }
  }

  function handleToggle(key: string, checked: boolean) {
    if (!authStatus.isAuthenticated) return;
    onchange?.(key, checked);
  }
</script>

<Card class="p-0 overflow-hidden" size="xl">
  <div class="px-4 py-3">
    <div class="flex items-center gap-2">
      <CloudArrowUpOutline class="w-5 h-5 text-gray-500 dark:text-gray-400" />
      <Heading tag="h1" class="text-base font-semibold">{category.label}</Heading>
    </div>
    {#if category.description}
      <Helper class="mt-1">
        {category.description}
      </Helper>
    {/if}
  </div>

  <div class="divide-y divide-slate-100 dark:divide-slate-700/50 px-4">
    <!-- Google Account Status -->
    <div class="py-4">
      <div class="flex items-center justify-between gap-4">
        <div class="flex-1 min-w-0">
          <Label class="setting-label">Google Account</Label>
          {#if authStatus.isAuthenticated}
            <div class="flex items-center gap-2 mt-1">
              <Badge color="green">Connected</Badge>
              {#if authStatus.email}
                <span class="text-sm text-gray-600 dark:text-gray-400">{authStatus.email}</span>
              {/if}
            </div>
          {:else if showCodeInput}
            <Helper class="setting-description mt-0.5">
              Complete sign-in by entering the code from the browser
            </Helper>
          {:else}
            <Helper class="setting-description mt-0.5">
              Connect your Google account to enable cloud sync
            </Helper>
          {/if}
        </div>
        <div class="shrink-0">
          {#if authStatus.isAuthenticated}
            <Button 
              size="sm" 
              color="alternative" 
              onclick={handleGoogleLogout}
              disabled={isLoggingOut}
            >
              {#if isLoggingOut}
                <Spinner size="4" class="me-2" />
              {/if}
              Disconnect
            </Button>
          {:else if !showCodeInput}
            <Button 
              size="sm" 
              color="light" 
              onclick={handleGoogleLogin}
              disabled={isLoggingIn}
            >
              {#if isLoggingIn}
                <Spinner size="4" class="me-2" />
              {:else}
                <GoogleSolid class="w-4 h-4 me-2" />
              {/if}
              Sign in with Google
            </Button>
          {/if}
        </div>
      </div>
      
      <!-- Code Input for OAuth callback -->
      {#if showCodeInput && !authStatus.isAuthenticated}
        <div class="mt-4 p-4 bg-slate-50 dark:bg-slate-800/50 rounded-lg">
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-3">
            After signing in with Google, you'll be redirected to a page. 
            Copy the <strong>entire URL</strong> from your browser's address bar and paste it below:
          </p>
          <div class="flex gap-2">
            <Input
              type="text"
              bind:value={authCode}
              placeholder="Paste the redirect URL here..."
              class="flex-1"
              disabled={isExchangingCode}
            />
            <Button 
              size="sm" 
              color="primary" 
              onclick={submitAuthCode}
              disabled={isExchangingCode || !authCode.trim()}
            >
              {#if isExchangingCode}
                <Spinner size="4" class="me-2" />
              {/if}
              Connect
            </Button>
            <Button 
              size="sm" 
              color="alternative" 
              onclick={cancelLogin}
              disabled={isExchangingCode}
            >
              <CloseOutline class="w-4 h-4" />
            </Button>
          </div>
        </div>
      {/if}
      
      {#if loginError}
        <Helper class="text-xs text-red-600 dark:text-red-400 mt-2">
          {loginError}
        </Helper>
      {/if}
    </div>

    <!-- Sync Settings (disabled unless authenticated) -->
    {#each category.settings as setting (setting.key)}
      <div class="setting-group py-3" class:opacity-50={!authStatus.isAuthenticated}>
        <div class="flex items-center justify-between gap-4">
          <div class="flex-1 min-w-0">
            <Label class="setting-label">{setting.label}</Label>
            {#if setting.description}
              <Helper class="setting-description mt-0.5">{setting.description}</Helper>
            {/if}
            {#if !authStatus.isAuthenticated}
              <Helper class="text-xs text-amber-600 dark:text-amber-400 mt-0.5">
                Sign in with Google to enable this option
              </Helper>
            {/if}
          </div>
          <div class="shrink-0">
            <Toggle
              checked={setting.value as boolean}
              disabled={!authStatus.isAuthenticated}
              onchange={(e) => handleToggle(setting.key, (e.target as HTMLInputElement).checked)}
            />
          </div>
        </div>
      </div>
    {/each}
  </div>
</Card>
