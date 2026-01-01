/**
 * Platform-specific utilities
 */

declare global {
  interface Window {
    AndroidFullscreen?: {
      setFullscreen(enabled: boolean): void;
    };
  }
}

/**
 * Set fullscreen mode (Android only)
 * Hides the status bar and navigation bar
 */
export function setFullscreen(enabled: boolean): void {
  if (window.AndroidFullscreen) {
    window.AndroidFullscreen.setFullscreen(enabled);
  }
}
