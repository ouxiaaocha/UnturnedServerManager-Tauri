<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { highlightText } from "$lib/utils";

  let category = $state("app");
  let dates: string[] = $state([]);
  let selectedDate = $state("");
  let logLines: string[] = $state([]);
  let loading = $state(false);
  let logSearch = $state("");
  let loadGeneration = 0;

  const categories = [
    { id: "app", label: "软件日志", icon: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" },
    { id: "operation", label: "操作日志", icon: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z" },
    { id: "game", label: "游戏日志", icon: "M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" },
  ];

  async function loadDates() {
    try {
      dates = await invoke("list_log_dates", { category }) as string[];
      if (dates.length > 0 && !selectedDate) {
        selectedDate = dates[0];
      }
    } catch { dates = []; }
    await loadLog();
  }

  async function loadLog() {
    if (!selectedDate) { logLines = []; return; }
    const gen = ++loadGeneration;
    loading = true;
    try {
      const lines = await invoke("read_log_file", { category, date: selectedDate }) as string[];
      if (gen !== loadGeneration) return;
      logLines = lines;
    } catch (e: any) {
      if (gen !== loadGeneration) return;
      logLines = [`读取失败: ${e}`];
    }
    if (gen === loadGeneration) loading = false;
  }

  function classifyLogLine(line: string): string {
    if (line.includes("[ERROR]") || line.includes("异常") || line.includes("Exception")) return "error";
    if (line.includes("[Warning]") || line.includes("警告")) return "warning";
    if (line.includes("[系统]") || line.includes("启动") || line.includes("关闭")) return "system";
    return "normal";
  }

  $effect(() => { category; loadDates(); });
  $effect(() => { selectedDate; loadLog(); });
</script>

<div class="flex flex-col h-full gap-5">
  <div>
    <h1 class="text-2xl font-bold text-[var(--text-primary)]">日志中心</h1>
    <p class="text-sm text-[var(--text-muted)] mt-1">查看系统、操作和游戏日志</p>
  </div>

  <!-- Filter Bar -->
  <div class="flex items-center gap-3 flex-shrink-0 flex-wrap">
    <div class="flex gap-2">
      {#each categories as cat}
        <button
          class="px-4 py-2 text-sm rounded-lg transition-all duration-[var(--transition-normal)] cursor-pointer flex items-center gap-2 {category === cat.id
            ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]'
            : 'bg-[var(--bg-card)] border border-[var(--border)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[var(--accent)]'}"
          onclick={() => { category = cat.id; selectedDate = ""; loadDates(); }}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={cat.icon} />
          </svg>
          {cat.label}
        </button>
      {/each}
    </div>

    {#if dates.length > 0}
      <select
        bind:value={selectedDate}
        class="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)] cursor-pointer"
      >
        {#each dates as d}
          <option value={d}>{d}</option>
        {/each}
      </select>
    {/if}

    <button
      class="px-3 py-2 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] bg-[var(--bg-card)] border border-[var(--border)] rounded-lg hover:border-[var(--accent)] transition-all duration-[var(--transition-normal)] cursor-pointer flex items-center gap-1"
      onclick={loadLog}
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
      刷新
    </button>
  </div>

  <!-- Log Content -->
  <div class="flex-1 bg-[var(--bg-card)] border border-[var(--border)] rounded-xl flex flex-col min-h-0">
    <div class="flex flex-wrap items-center justify-between gap-3 px-5 py-3 border-b border-[var(--border)] flex-shrink-0">
      <span class="text-sm text-[var(--text-muted)]">
        {categories.find(c => c.id === category)?.label} — {selectedDate || '无'}
      </span>
      <div class="flex items-center gap-2">
        <div class="relative">
          <svg class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            bind:value={logSearch}
            placeholder="搜索日志..."
            class="w-full sm:w-44 bg-[var(--bg-primary)] border border-[var(--border)] rounded-md pl-8 pr-2 py-1 text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>
        <span class="text-xs text-[var(--text-muted)] bg-[var(--bg-elevated)] px-2 py-1 rounded">{logLines.length} 行</span>
      </div>
    </div>
    <div class="flex-1 overflow-y-auto p-4 font-mono text-xs leading-6">
      {#if loading}
        <div class="flex items-center justify-center h-full">
          <div class="flex items-center gap-2 text-[var(--text-muted)]">
            <div class="w-4 h-4 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
            <span class="italic">加载中...</span>
          </div>
        </div>
      {:else if logLines.length === 0}
        <div class="flex items-center justify-center h-full text-[var(--text-muted)]">
          <p class="italic">暂无日志</p>
        </div>
      {:else}
        {@const filtered = logSearch ? logLines.filter(l => l.toLowerCase().includes(logSearch.toLowerCase())) : logLines}
        {#if logSearch && filtered.length === 0}
          <div class="flex items-center justify-center h-full text-[var(--text-muted)]">
            <p class="italic">未找到匹配内容</p>
          </div>
        {:else}
          {#if logSearch}
            <div class="pb-2 mb-2 border-b border-[var(--border)] text-[var(--text-muted)]">
              找到 {filtered.length} 条匹配
            </div>
          {/if}
          {#each filtered as line}
            {@const level = classifyLogLine(line)}
            <p class="py-0.5 {level === 'error' ? 'text-[var(--danger)]' : level === 'warning' ? 'text-[var(--warning)]' : level === 'system' ? 'text-[var(--accent-light)]' : 'text-[var(--text-secondary)]'}">
              {@html highlightText(line, logSearch)}
            </p>
          {/each}
        {/if}
      {/if}
    </div>
  </div>
</div>


