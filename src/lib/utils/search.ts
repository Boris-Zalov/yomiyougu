import type Fuse from "fuse.js";

/**
 * Strips punctuation from a string for search normalization
 */
export function stripPunctuation(str: string): string {
  return str.replace(/[^\w\s]|_/g, "").replace(/\s+/g, " ");
}

/**
 * Shared Fuse.js configuration for consistent search behavior across the application
 */
export const fuseOptions: Fuse.IFuseOptions<unknown> = {
  threshold: 0.4,
  ignoreLocation: true,
  includeScore: true,
  getFn: (obj: object, path: string | string[]) => {
    const key = Array.isArray(path) ? path[0] : path;
    const value = (obj as Record<string, unknown>)[key];
    if (typeof value === "string") {
      return stripPunctuation(value);
    }
    return value as string;
  },
};
