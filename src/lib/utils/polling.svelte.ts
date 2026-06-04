/**
 * 通用轮询管理工具
 * 用于 Dashboard 和 Server 页面的状态轮询
 */

export interface PollerOptions {
  /** 轮询执行函数 */
  pollFn: () => Promise<void>;
  /** 活跃状态判断函数（如服务器运行中） */
  isActive: () => boolean;
  /** 活跃时轮询间隔（毫秒） */
  activeInterval?: number;
  /** 空闲时轮询间隔（毫秒） */
  idleInterval?: number;
  /** 页面隐藏时轮询间隔（毫秒） */
  hiddenInterval?: number;
}

export function createPoller(options: PollerOptions) {
  const {
    pollFn,
    isActive,
    activeInterval = 2000,
    idleInterval = 5000,
    hiddenInterval = 10000,
  } = options;

  let pollToken = 0;
  let pollTimer: ReturnType<typeof setTimeout> | undefined;

  function nextPollDelay(): number {
    if (typeof document !== "undefined" && document.hidden) return hiddenInterval;
    if (isActive()) return activeInterval;
    return idleInterval;
  }

  async function pollLoop(token = pollToken) {
    if (token !== pollToken) return;
    try {
      await pollFn();
    } catch (e) {
      console.error("轮询失败:", e);
    }
    if (token !== pollToken) return;
    pollTimer = setTimeout(() => pollLoop(token), nextPollDelay());
  }

  function start() {
    pollToken += 1;
    if (pollTimer) clearTimeout(pollTimer);
    pollLoop();
  }

  function stop() {
    pollToken += 1;
    if (pollTimer) {
      clearTimeout(pollTimer);
      pollTimer = undefined;
    }
  }

  function setupVisibilityListener() {
    const onVisibilityChange = () => {
      if (typeof document !== "undefined" && !document.hidden) {
        // 页面恢复可见时，重置定时器以使用更短的活跃间隔，但不立即触发轮询
        if (pollTimer) clearTimeout(pollTimer);
        pollTimer = setTimeout(() => pollLoop(pollToken), nextPollDelay());
      }
    };
    document.addEventListener("visibilitychange", onVisibilityChange);
    return () => {
      document.removeEventListener("visibilitychange", onVisibilityChange);
      stop();
    };
  }

  return { start, stop, setupVisibilityListener };
}
