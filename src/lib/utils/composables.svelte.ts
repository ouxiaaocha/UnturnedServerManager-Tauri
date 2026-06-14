/**
 * 日志过滤 Composable
 * 提供搜索和过滤日志的响应式功能
 */
export function createLogFilter<T extends { text: string }>(logs: T[]) {
  let searchText = $state("");

  let normalizedSearch = $derived(searchText.trim().toLowerCase());

  let filteredLogs = $derived(
    normalizedSearch
      ? logs.filter((log) => log.text.toLowerCase().includes(normalizedSearch))
      : logs
  );

  return {
    get searchText() {
      return searchText;
    },
    set searchText(val: string) {
      searchText = val;
    },
    get filteredLogs() {
      return filteredLogs;
    },
  };
}

/**
 * 自动滚动 Composable
 * 管理日志容器的自动滚动行为
 */
export function createAutoScroll() {
  let autoScroll = $state(true);
  let container: HTMLElement | undefined = $state();

  /**
   * 检查容器是否接近底部
   */
  function isNearBottom(threshold = 100): boolean {
    if (!container) return false;
    const { scrollTop, scrollHeight, clientHeight } = container;
    return scrollHeight - scrollTop - clientHeight < threshold;
  }

  /**
   * 滚动到底部
   */
  function scrollToBottom() {
    if (!container) return;
    container.scrollTop = container.scrollHeight;
  }

  /**
   * 处理滚动事件，更新自动滚动状态
   */
  function handleScroll() {
    if (!container) return;
    autoScroll = isNearBottom();
  }

  /**
   * 在新日志添加后自动滚动（如果启用）
   */
  function scrollIfEnabled() {
    if (autoScroll) {
      requestAnimationFrame(() => scrollToBottom());
    }
  }

  return {
    get autoScroll() {
      return autoScroll;
    },
    set autoScroll(val: boolean) {
      autoScroll = val;
    },
    get container() {
      return container;
    },
    set container(val: HTMLElement | undefined) {
      container = val;
    },
    isNearBottom,
    scrollToBottom,
    handleScroll,
    scrollIfEnabled,
  };
}
