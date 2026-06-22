<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    rconLogs,
    addRconLog,
    addRconLogs,
    runningServers,
    refreshRunningServers,
    serverView,
    sharedSaves,
    loadSharedSaves,
  } from "$lib/stores.svelte";
  import { highlightText } from "$lib/utils";
  import { createLogFilter, createAutoScroll } from "$lib/utils/composables.svelte";

  let connected = $state(false);
  let command = $state("");
  let connecting = $state(false);
  let selectedRconSaveId = $state("");
  let connectedSaveId = $state("");
  let runningCount = $derived(runningServers.length);
  let selectedRunningServer = $derived(runningServers.find((server) => server.save_id === selectedRconSaveId));

  // 使用日志过滤 composable
  const logFilter = createLogFilter(rconLogs);
  let filteredLogs = $derived(logFilter.filteredLogs);

  // 使用自动滚动 composable
  const autoScroller = createAutoScroll();

  function scrollToBottom() {
    autoScroller.scrollToBottom();
  }

  function getSaveName(saveId: string): string {
    if (!saveId) return "";
    const save = sharedSaves.find((s: any) => s.id === saveId);
    return save ? (save.name ? `${save.id} - ${save.name}` : save.id) : saveId;
  }

  function selectRconTarget(saveId: string) {
    if (connected || connecting) {
      addRconLog("请先断开当前 RCON 连接，再切换目标服务器", "error");
      scrollToBottom();
      return;
    }
    selectedRconSaveId = saveId;
  }

  function ensureRconTarget() {
    if (connected || connecting) return;
    if (selectedRconSaveId && runningServers.some((server) => server.save_id === selectedRconSaveId)) {
      return;
    }
    const preferred = serverView.selectedRunningSaveId;
    selectedRconSaveId = runningServers.some((server) => server.save_id === preferred)
      ? preferred
      : (runningServers[0]?.save_id ?? "");
  }

  async function checkStatus(token = pollGeneration) {
    try {
      const status = await invoke("rcon_status") as boolean;
      if (token !== pollGeneration) return;
      const target = status ? await invoke("rcon_connected_save_id") as string | null : "";
      if (token !== pollGeneration) return;
      if (connected && !status) {
        addRconLog("连接已断开（服务器可能已关闭）", "error");
        scrollToBottom();
      }
      if (!status) {
        connectedSaveId = "";
      }
      if (status && target) {
        connectedSaveId = target;
        if (!selectedRconSaveId) {
          selectedRconSaveId = target;
        }
      }
      connected = status;
    } catch {
      connected = false;
      connectedSaveId = "";
    }
  }

  async function pollResponses(token = pollGeneration) {
    if (!connected) return;
    try {
      const lines = await invoke("rcon_poll") as string[];
      if (token !== pollGeneration) return;
      if (lines.length > 0) {
        // Rocket RCON 服务器对每条命令发送 2 条响应（执行日志 + 结果），
        // 且通过 Broadcast 发送给所有客户端，导致每条消息出现两次。
        // 使用 Set 去重，保留首次出现的顺序。
        const seen = new Set<string>();
        const deduped = lines.filter(line => {
          if (seen.has(line)) return false;
          seen.add(line);
          return true;
        });
        addRconLogs(deduped, "response");
        scrollToBottom();
      }
    } catch (e) { console.error("RCON 轮询失败:", e); }
  }

  async function connect() {
    if (!selectedRconSaveId) {
      addRconLog("请选择要连接的运行服务器", "error");
      scrollToBottom();
      return;
    }
    if (!selectedRunningServer) {
      addRconLog("所选服务器未运行，无法连接 RCON", "error");
      scrollToBottom();
      return;
    }
    connecting = true;
    addRconLog(`正在连接 ${selectedRconSaveId}...`, "system");
    try {
      const welcome = await invoke("rcon_connect", { saveId: selectedRconSaveId }) as string;
      connected = true;
      connectedSaveId = selectedRconSaveId;
      addRconLog(welcome, "info");
      addRconLog(`RCON 连接成功: ${selectedRconSaveId}`, "system");
    } catch (e: any) {
      addRconLog(`连接失败: ${e}`, "error");
      connectedSaveId = "";
    }
    connecting = false;
    scrollToBottom();
  }

  async function disconnect() {
    try {
      await invoke("rcon_disconnect");
    } catch (e: any) {
      addRconLog(`断开时出错: ${e}`, "error");
    }
    connected = false;
    connectedSaveId = "";
    addRconLog("已断开连接", "system");
    scrollToBottom();
  }

  async function send() {
    const cmd = command.trim();
    if (!cmd) return;
    if (!connected || connectedSaveId !== selectedRconSaveId) {
      addRconLog("RCON 目标已变化，请重新连接后再发送命令", "error");
      connected = false;
      connectedSaveId = "";
      return;
    }

    addRconLog(`> ${cmd}`, "command");
    command = "";

    try {
      await invoke("rcon_send", { command: cmd });
    } catch (e: any) {
      addRconLog(`发送失败: ${e}`, "error");
      connected = false;
    }
    scrollToBottom();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") send();
  }

  let pollTimer: ReturnType<typeof setTimeout> | undefined;
  let statusTimer: ReturnType<typeof setTimeout> | undefined;
  let pollGeneration = 0;

  function schedulePoll(token: number) {
    pollTimer = setTimeout(async () => {
      if (token !== pollGeneration) return;
      await pollResponses(token);
      if (token === pollGeneration) schedulePoll(token);
    }, 2000);
  }

  function scheduleStatusCheck(token: number) {
    statusTimer = setTimeout(async () => {
      if (token !== pollGeneration) return;
      await refreshRunningServers();
      ensureRconTarget();
      await checkStatus(token);
      if (token === pollGeneration) scheduleStatusCheck(token);
    }, 10000);
  }

  $effect(() => {
    const token = ++pollGeneration;
    void loadSharedSaves();
    void refreshRunningServers().then(() => ensureRconTarget());
    checkStatus(token);
    schedulePoll(token);
    scheduleStatusCheck(token);

    return () => {
      pollGeneration += 1;
      clearTimeout(pollTimer);
      clearTimeout(statusTimer);
    };
  });
</script>

<div class="flex flex-col gap-5">
  <div>
    <h1 class="text-2xl font-bold text-[var(--text-primary)]">RCON 控制台</h1>
    <p class="text-sm text-[var(--text-muted)] mt-1">远程服务器命令控制</p>
  </div>

  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex-shrink-0">
    <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
      <div>
        <h2 class="text-sm font-semibold text-[var(--text-primary)]">RCON 目标服务器</h2>
        <p class="mt-1 text-xs text-[var(--text-muted)]">连接后需先断开，才能切换目标</p>
      </div>
      <span class="text-xs text-[var(--text-muted)]">运行 {runningCount}</span>
    </div>

    {#if runningCount === 0}
      <div class="rounded-lg border border-dashed border-[var(--border)] bg-[var(--bg-primary)] px-4 py-5 text-center text-sm text-[var(--text-muted)]">
        当前没有运行中的服务器
      </div>
    {:else}
      <div class="{runningCount > 8 ? 'max-h-[180px] overflow-y-auto pr-1' : ''}">
        <div class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {#each runningServers as server (server.save_id)}
            <button
              type="button"
              onclick={() => selectRconTarget(server.save_id)}
              disabled={connected || connecting}
              title={getSaveName(server.save_id)}
              class="flex min-h-[56px] items-center justify-between gap-3 rounded-lg border px-4 py-3 text-left transition-all duration-[var(--transition-normal)] disabled:cursor-not-allowed disabled:opacity-70 {server.save_id === selectedRconSaveId ? 'border-[var(--accent)] bg-[var(--accent-subtle)] shadow-[var(--shadow-sm)]' : 'border-[var(--border)] bg-[var(--bg-primary)] hover:border-[var(--border-hover)] hover:bg-[var(--bg-card-hover)]'}"
            >
              <div class="flex min-w-0 items-center gap-2">
                <span class="h-2.5 w-2.5 shrink-0 rounded-full bg-[var(--success)] animate-pulse"></span>
                <span class="truncate text-sm font-semibold text-[var(--text-primary)]">{getSaveName(server.save_id)}</span>
              </div>
              <span class="shrink-0 rounded-md px-2 py-1 text-xs font-medium {server.save_id === selectedRconSaveId ? 'bg-[var(--accent)] text-white' : 'bg-[var(--success-glow)] text-[var(--success)]'}">
                {server.save_id === connectedSaveId && connected ? '已连接' : server.save_id === selectedRconSaveId ? '目标' : '运行中'}
              </span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Connection Status -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex-shrink-0">
    <div class="flex items-center gap-4 flex-wrap">
      <div class="flex items-center gap-3">
        <div class="w-3 h-3 rounded-full {connected ? 'bg-[var(--success)] animate-pulse' : 'bg-[var(--text-muted)]'}"></div>
        <span class="text-sm font-medium {connected ? 'text-[var(--success)]' : 'text-[var(--text-secondary)]'}">
          {connected ? "已连接" : "未连接"}
        </span>
        {#if selectedRconSaveId}
          <span class="rounded-md bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-muted)]">{getSaveName(selectedRconSaveId)}</span>
        {/if}
      </div>

      <div class="flex gap-3 ml-auto">
        <button
          class="px-5 py-2.5 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={connect}
          disabled={connected || connecting || !selectedRconSaveId || !selectedRunningServer}
        >
          {#if connecting}
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
            连接中...
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
            连接
          {/if}
        </button>

        <button
          class="px-5 py-2.5 bg-[var(--bg-elevated)] border border-[var(--border)] hover:border-[var(--danger)] text-[var(--text-secondary)] hover:text-[var(--danger)] text-sm rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={disconnect}
          disabled={!connected}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
          </svg>
          断开
        </button>
      </div>
    </div>
  </div>

  <!-- Response Area -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl flex flex-col max-h-[50vh]">
    <div class="flex flex-wrap items-center justify-between gap-3 px-5 py-3 border-b border-[var(--border)] flex-shrink-0">
      <div class="flex items-center gap-2">
        <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        <span class="text-sm font-medium text-[var(--text-primary)]">响应输出</span>
      </div>
      <div class="flex items-center gap-2">
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
        <button
          class="text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors duration-[var(--transition-fast)] px-2 py-1 rounded hover:bg-[var(--bg-card-hover)] cursor-pointer"
          onclick={() => { rconLogs.splice(0, rconLogs.length); }}
        >
          清空
        </button>
      </div>
    </div>
    <div bind:this={autoScroller.container} class="flex-1 overflow-y-auto p-4 font-mono text-xs leading-6">
      {#if rconLogs.length === 0}
        <div class="flex items-center justify-center h-full text-[var(--text-muted)]">
          <p class="italic">连接后发送命令...</p>
        </div>
      {:else}
        {#if logFilter.searchText && filteredLogs.length === 0}
          <div class="flex items-center justify-center h-full text-[var(--text-muted)]">
            <p class="italic">未找到匹配内容</p>
          </div>
        {:else}
          {#if logFilter.searchText}
            <div class="pb-2 mb-2 border-b border-[var(--border)] text-[var(--text-muted)]">
              找到 {filteredLogs.length} 条匹配
            </div>
          {/if}
          {#each filteredLogs as r}
            <p class="py-0.5 {r.type === 'error' ? 'text-[var(--danger)]' : r.type === 'command' ? 'text-[var(--text-primary)] font-medium' : r.type === 'system' ? 'text-[var(--accent-light)]' : r.type === 'info' ? 'text-[var(--success)]' : 'text-[var(--text-secondary)]'}">
              {@html highlightText(r.text, logFilter.searchText)}
            </p>
          {/each}
        {/if}
      {/if}
    </div>
  </div>

  <!-- Command Input -->
  <div class="flex gap-3 flex-shrink-0">
    <div class="flex-1 relative">
      <input
        type="text"
        bind:value={command}
        placeholder="输入 RCON 命令..."
        class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)] font-mono"
        disabled={!connected || connectedSaveId !== selectedRconSaveId}
        onkeydown={handleKeydown}
      />
      <div class="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 9l3 3m0 0l-3 3m3-3H8m13 0a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </div>
    </div>
    <button
      class="px-6 py-3 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
      onclick={send}
      disabled={!connected || connectedSaveId !== selectedRconSaveId || !command.trim()}
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
      </svg>
      发送
    </button>
  </div>
</div>


