/**
 * Escape HTML special characters to prevent XSS.
 * 任何用于 {@html} 的内容都应先经过此函数转义，再做后续格式化替换。
 */
export function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

/**
 * Escape HTML special characters to prevent XSS, then highlight search matches.
 */
export function highlightText(text: string, query: string): string {
  const escaped = escapeHtml(text);
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

/**
 * 日志行分类函数
 * 根据日志内容判断日志级别
 */
export function classifyLogLevel(line: string): string {
  if (line.includes("[Error]") || line.includes("Exception") || line.includes("错误")) return "error";
  if (line.includes("[Warning]") || line.includes("警告")) return "warning";
  if (line.includes("[系统]")) return "system";
  if (line.includes("Loading level") || line.includes("registered") || line.includes("成功")) return "info";
  return "normal";
}

/**
 * 滚动容器到底部
 */
export function scrollToBottom(element: HTMLElement | null): void {
  if (!element) return;
  requestAnimationFrame(() => {
    element.scrollTop = element.scrollHeight;
  });
}

/**
 * 复制文本到剪贴板
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}
