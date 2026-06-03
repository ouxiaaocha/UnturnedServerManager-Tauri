/**
 * Escape HTML special characters to prevent XSS, then highlight search matches.
 */
export function highlightText(text: string, query: string): string {
  const escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
  if (!query) return escaped;
  const q = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return escaped.replace(new RegExp(`(${q})`, 'gi'), '<mark class="bg-yellow-500/30 text-yellow-200 rounded px-0.5">$1</mark>');
}

/**
 * Generate a cryptographically random password using rejection sampling to avoid modulo bias.
 */
export function generatePassword(length = 16): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
  const limit = Math.floor(0x100000000 / chars.length) * chars.length;
  let result = "";
  while (result.length < length) {
    const arr = new Uint32Array(length - result.length);
    crypto.getRandomValues(arr);
    for (const v of arr) {
      if (v < limit) {
        result += chars[v % chars.length];
        if (result.length >= length) break;
      }
    }
  }
  return result;
}

/**
 * Format bytes into a human-readable string.
 */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
  if (bytes < 1073741824) return (bytes / 1048576).toFixed(1) + " MB";
  return (bytes / 1073741824).toFixed(2) + " GB";
}

/**
 * Format seconds into a human-readable uptime string.
 */
export function formatUptime(secs: number): string {
  if (secs <= 0) return "--";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  return h > 0 ? `${h}时${m}分${s}秒` : m > 0 ? `${m}分${s}秒` : `${s}秒`;
}
