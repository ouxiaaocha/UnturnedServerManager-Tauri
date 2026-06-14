<script lang="ts">
  import { highlightText } from "$lib/utils";
  import { createLogFilter } from "$lib/utils/composables.svelte";

  interface LogEntry {
    text: string;
    level: string;
  }

  let {
    logs = [],
    searchable = true,
    emptyMessage = "暂无日志",
    maxHeight = "max-h-[50vh]",
    onClear,
  }: {
    logs?: LogEntry[];
    searchable?: boolean;
    emptyMessage?: string;
    maxHeight?: string;
    onClear?: () => void;
  } = $props();

  // 使用日志过滤 composable
  const logFilter = createLogFilter(logs);
  const filteredLogs = $derived(logFilter.filteredLogs);
</script>

<div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl flex flex-col {maxHeight}">
  <div class="flex flex-wrap items-center justify-between gap-3 px-5 py-3 border-b border-[var(--border)] flex-shrink-0">
    <div class="flex items-center gap-2">
      <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <span class="text-sm font-medium text-[var(--text-primary)]">日志输出</span>
      <span class="text-xs text-[var(--text-muted)]">({filteredLogs.length})</span>
    </div>
    <div class="flex items-center gap-2">
      {#if searchable}
        <div class="relative">
          <svg class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            bind:value={logFilter.searchText}
            placeholder="搜索日志..."
            class="w-full sm:w-44 bg-[var(--bg-primary)] border border-[var(--border)] rounded-md pl-8 pr-2 py-1 text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>
        {#if logFilter.searchText}
          <span class="text-xs text-[var(--text-muted)]">
            找到 {filteredLogs.length} 条匹配
          </span>
        {/if}
      {/if}
      {#if onClear}
        <button
          class="p-1.5 text-[var(--text-muted)] hover:text-[var(--danger)] transition-colors cursor-pointer"
          onclick={onClear}
          aria-label="清空日志"
          title="清空日志"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      {/if}
    </div>
  </div>

  <div class="flex-1 overflow-y-auto p-4 font-mono text-xs leading-6">
    {#if filteredLogs.length === 0}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-10 h-10 text-[var(--text-muted)] mx-auto mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-[var(--text-muted)]">
            {logFilter.searchText ? "未找到匹配内容" : emptyMessage}
          </p>
        </div>
      </div>
    {:else}
      {#each filteredLogs as log, i (i)}
        <div class="py-0.5 {log.level === 'error' ? 'text-[var(--danger)]' : log.level === 'warning' ? 'text-[var(--warning)]' : log.level === 'system' ? 'text-[var(--accent-light)]' : log.level === 'info' ? 'text-[var(--success)]' : 'text-[var(--text-secondary)]'}">
          {#if logFilter.searchText}
            {@html highlightText(log.text, logFilter.searchText)}
          {:else}
            {log.text}
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
