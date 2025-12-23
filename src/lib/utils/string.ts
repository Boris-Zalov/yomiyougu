/**
 * Strip punctuation and normalize whitespace from a string.
 * Useful for search normalization and text comparison.
 * 
 * @param str - The input string to process
 * @returns The string with punctuation removed and whitespace normalized
 */
export function stripPunctuation(str: string): string {
  return str.replace(/[^\w\s]|_/g, "").replace(/\s+/g, " ");
}
