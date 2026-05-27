<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { highlightText, formatUptime } from "$lib/utils";

  let status = $state("已停止");
  let pid = $state("--");
  let uptime = $state("--");
  let logs: Array<{text: string, level: string}> = $state([]);
  let loading = $state("");
  let outputIndex = $state(0);
  let logContainer: HTMLDivElement | undefined = $state();
  let logSearch = $state("");
  let isNearBottom = false;
  let firstLoadDone = false;
  let isStarting = false;
  let polling = false;

  let saves = $state<any[]>([]);
  let selectedSaveId = $state("");
  let launchMode = $state("internet");

  async function loadSaves() {
    try {
      saves = await invoke("list_server_saves");
      if (saves.length > 0 && !selectedSaveId) {
        selectedSaveId = saves[0].id;
      }
    } catch {}
  }

  function classifyLine(line: string): string {
    if (line.includes("[Error]") || line.includes("Exception")) return "error";
    if (line.includes("[Warning]")) return "warning";
    if (line.includes("[系统]")) return "system";
    if (line.includes("Loading level") || line.includes("registered")) return "info";
    return "normal";
  }

  async function refreshStatus() {
    if (polling) return;
    polling = true;
    try {
      const s: any = await invoke("get_server_status");
      status = s.state;
      pid = s.pid ? String(s.pid) : "--";
      uptime = formatUptime(s.uptime_secs);

      if (!isStarting && s.output_count > outputIndex) {
        const newLines: string[] = await invoke("get_server_output", { fromIndex: outputIndex });
        for (const line of newLines) {
          logs.push({ text: line, level: classifyLine(line) });
        }
        if (logs.length > 500) logs = logs.slice(-500);
        outputIndex = s.output_count;
        if (!firstLoadDone) {
          firstLoadDone = true;
          isNearBottom = true;
          setTimeout(() => {
            if (logContainer) logContainer.scrollTop = logContainer.scrollHeight;
          }, 50);
        } else if (isNearBottom) {
          setTimeout(() => {
            if (logContainer) logContainer.scrollTop = logContainer.scrollHeight;
          }, 50);
        }
      }
    } catch {}
    polling = false;
  }

  async function startServer() {
    loading = "starting";
    isStarting = true;
    try {
      await invoke("start_server", {
        saveId: selectedSaveId || null,
        launchMode: launchMode,
      });
      outputIndex = 0;
      logs = [];
      firstLoadDone = false;
      isNearBottom = true;
    } catch (e: any) {
      logs.push({ text: `[错误] ${e}`, level: "error" });
    }
    isStarting = false;
    loading = "";
  }

  async function stopServer() {
    loading = "stopping";
    try {
      await invoke("stop_server");
    } catch (e: any) {
      logs.push({ text: `[错误] ${e}`, level: "error" });
    }
    loading = "";
  }

  async function restartServer() {
    loading = "restarting";
    isStarting = true;
    try {
      await invoke("restart_server", {
        saveId: selectedSaveId || null,
        launchMode: launchMode,
      });
      outputIndex = 0;
      logs = [];
      firstLoadDone = false;
      isNearBottom = true;
    } catch (e: any) {
      logs.push({ text: `[错误] ${e}`, level: "error" });
    }
    isStarting = false;
    loading = "";
  }

  async function forceStop() {
    try {
      await invoke("force_stop_server");
    } catch (e: any) {
      logs.push({ text: `[错误] ${e}`, level: "error" });
    }
  }

  function onScroll() {
    if (!logContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = logContainer;
    isNearBottom = scrollHeight - scrollTop - clientHeight < 80;
  }

  $effect(() => {
    loadSaves();
    refreshStatus();
    const interval = setInterval(refreshStatus, 2000);
    return () => clearInterval(interval);
  });
</script>

<div class="flex flex-col h-full gap-5">
  <!-- Header -->
  <div class="flex items-center justify-between flex-shrink-0">
    <div>
      <h1 class="text-2xl font-bold text-white">服务器控制</h1>
      <p class="text-sm text-[var(--text-muted)] mt-1">管理 Unturned 服务器实例</p>
    </div>
    <div class="flex items-center gap-3 px-4 py-2 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
      <div class="w-2.5 h-2.5 rounded-full {status === '运行中' ? 'bg-[var(--success)] animate-pulse' : status === '错误' ? 'bg-[var(--danger)]' : 'bg-[var(--text-muted)]'}"></div>
      <span class="text-sm font-medium {status === '运行中' ? 'text-[var(--success)]' : status === '错误' ? 'text-[var(--danger)]' : 'text-[var(--text-secondary)]'}">{status}</span>
    </div>
  </div>

  <!-- Control Panel -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex-shrink-0">
    <div class="flex items-center justify-between flex-wrap gap-4">
      <div class="flex gap-3 flex-wrap">
        <button
          class="px-5 py-2.5 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-white text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={startServer}
          disabled={status === '运行中' || loading !== ''}
        >
          {#if loading === 'starting'}
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            </svg>
          {/if}
          {loading === 'starting' ? '启动中...' : '启动'}
        </button>

        <button
          class="px-5 py-2.5 bg-gradient-to-r from-[var(--danger)] to-rose-600 hover:from-rose-500 hover:to-[var(--danger)] text-white text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={stopServer}
          disabled={status !== '运行中' || loading !== ''}
        >
          {#if loading === 'stopping'}
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
            </svg>
          {/if}
          {loading === 'stopping' ? '停止中...' : '停止'}
        </button>

        <button
          class="px-5 py-2.5 bg-gradient-to-r from-[var(--warning)] to-amber-600 hover:from-amber-500 hover:to-[var(--warning)] text-white text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={restartServer}
          disabled={status !== '运行中' || loading !== ''}
        >
          {#if loading === 'restarting'}
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          {/if}
          {loading === 'restarting' ? '重启中...' : '重启'}
        </button>

        <button
          class="px-5 py-2.5 bg-[var(--bg-elevated)] border border-[var(--border)] hover:border-[var(--danger)] text-[var(--text-secondary)] hover:text-[var(--danger)] text-sm rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={forceStop}
          disabled={status === '已停止'}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
          </svg>
          强制停止
        </button>
      </div>

      <div class="flex items-center gap-3">
        <select
          bind:value={selectedSaveId}
          class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-3 py-1.5 text-xs text-white focus:outline-none focus:border-[var(--accent)] transition-colors cursor-pointer"
        >
          {#each saves as save}
            <option value={save.id}>{save.id}{save.name ? ` - ${save.name}` : ''}</option>
          {/each}
        </select>
        <div class="flex rounded-lg overflow-hidden border border-[var(--border)]">
          <button
            class="px-2 py-1 text-xs font-medium transition-all cursor-pointer {launchMode === 'internet' ? 'bg-[var(--accent)] text-white' : 'bg-[var(--bg-primary)] text-[var(--text-secondary)] hover:text-white'}"
            onclick={() => launchMode = 'internet'}
          >互联网</button>
          <button
            class="px-2 py-1 text-xs font-medium transition-all cursor-pointer {launchMode === 'lan' ? 'bg-[var(--accent)] text-white' : 'bg-[var(--bg-primary)] text-[var(--text-secondary)] hover:text-white'}"
            onclick={() => launchMode = 'lan'}
          >局域网</button>
        </div>
      </div>

      <div class="flex gap-6 text-sm">
        <div class="flex items-center gap-2">
          <span class="text-[var(--text-muted)]">PID</span>
          <span class="text-white font-mono font-semibold bg-[var(--bg-elevated)] px-2 py-1 rounded">{pid}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-[var(--text-muted)]">运行</span>
          <span class="text-white font-semibold">{uptime}</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Log Output -->
  <div class="flex-1 bg-[var(--bg-card)] border border-[var(--border)] rounded-xl flex flex-col min-h-0">
    <div class="flex items-center justify-between px-5 py-3 border-b border-[var(--border)] flex-shrink-0">
      <div class="flex items-center gap-2">
        <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <span class="text-sm font-medium text-white">服务器输出</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="relative">
          <svg class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            bind:value={logSearch}
            placeholder="搜索日志..."
            class="w-44 bg-[var(--bg-primary)] border border-[var(--border)] rounded-md pl-8 pr-2 py-1 text-xs text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>
        <button
          class="text-xs text-[var(--text-muted)] hover:text-white transition-colors duration-[var(--transition-fast)] px-2 py-1 rounded hover:bg-[var(--bg-card-hover)] cursor-pointer"
          onclick={() => { logs = []; outputIndex = 0; }}
        >
          清空
        </button>
      </div>
    </div>
    <div bind:this={logContainer} onscroll={onScroll} class="flex-1 overflow-y-auto p-4 font-mono text-xs leading-6">
      {#if logs.length === 0}
        <div class="flex items-center justify-center h-full text-[var(--text-muted)]">
          <p class="italic">等待服务器启动...</p>
        </div>
      {:else}
        {@const filtered = logSearch ? logs.filter(l => l.text.toLowerCase().includes(logSearch.toLowerCase())) : logs}
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
          {#each filtered as log}
            <p class="py-0.5 {log.level === 'error' ? 'text-[var(--danger)]' : log.level === 'warning' ? 'text-[var(--warning)]' : log.level === 'info' ? 'text-[var(--success)]' : log.level === 'system' ? 'text-[var(--accent-light)] font-medium' : 'text-[var(--text-secondary)]'}">
              {@html highlightText(log.text, logSearch)}
            </p>
          {/each}
        {/if}
      {/if}
    </div>
  </div>
</div>
