<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { tick } from "svelte";
  import { highlightText } from "$lib/utils";
  import { appState, uiPreferences, setSelectedSaveId, ensureSelectedSaveId, sharedSaves, loadSharedSaves, sharedSettings, loadSharedSettings, toggleAutoUpdateHosting, serverState, serverLogs, runningServers, refreshServerStatus, clearServerLogs, appendServerLogs, setServerLoading, syncSelectedServerRuntime, serverView, selectRunningServer, serverStatesBySave } from "$lib/stores.svelte";
  import { toastStore } from "../stores/toast.svelte";
  import { createPoller } from "../utils/polling.svelte";
  import { createAutoScroll } from "../utils/composables.svelte";
  import SaveSelector from "../components/SaveSelector.svelte";

  // 使用共享的服务器状态
  let status = $derived(serverState.status);
  let pid = $derived(serverState.pid);
  let uptime = $derived(serverState.uptime);
  let loading = $derived(serverState.loading);
  let logs = $derived(serverLogs);
  let runningCount = $derived(runningServers.length);

  let logSearchText = $state("");
  let normalizedLogSearch = $derived(logSearchText.trim().toLowerCase());
  let filteredLogs = $derived(
    normalizedLogSearch
      ? logs.filter((log) => log.text.toLowerCase().includes(normalizedLogSearch))
      : logs
  );

  // 使用自动滚动 composable
  const autoScroller = createAutoScroll();

  let firstLoadDone = false;
  let polling = false;

  let selectedSaveId = $derived(uiPreferences.selectedSaveId);
  let selectedRunningSaveId = $derived(serverView.selectedRunningSaveId);
  let viewingSaveId = $derived(selectedRunningSaveId || selectedSaveId);
  let selectedRunningServer = $derived(runningServers.find((server) => server.save_id === selectedRunningSaveId));
  let launchSaveRuntime = $derived(serverStatesBySave[selectedSaveId || "__default__"]);
  let launchSaveLoading = $derived(launchSaveRuntime?.loading ?? "");
  let selectedSaveRunning = $derived(runningServers.some((server) => server.save_id === selectedSaveId));
  let launchModeLocked = $derived(selectedSaveRunning || launchSaveLoading !== "");
  let autoUpdateSaving = $state(false);
  let autoUpdateMessage = $state("");
  let localCommand = $state("");
  let localCommandSending = $state(false);
  let canSendLocalCommand = $derived(
    !!viewingSaveId && status === "运行中" && loading === "" && localCommand.trim() !== "" && !localCommandSending
  );

  // 轮询管理器
  const poller = createPoller({
    pollFn: refreshStatus,
    isActive: () => serverState.loading !== "" || serverState.status === "运行中" || runningServers.length > 0,
  });

  async function loadSaves() {
    await loadSharedSaves();
    ensureSelectedSaveId(sharedSaves);
  }

  function handleSelectedSaveChange(value: string) {
    setSelectedSaveId(value);
    refreshStatus();
  }

  function getSaveName(saveId: string): string {
    if (!saveId) return "";
    const save = sharedSaves.find((s: any) => s.id === saveId);
    return save ? (save.name ? `${save.id} - ${save.name}` : save.id) : saveId;
  }

  async function handleRunningServerSelect(saveId: string) {
    if (saveId === selectedRunningSaveId) return;
    selectRunningServer(saveId);
    firstLoadDone = false;
    autoScroller.autoScroll = true;
    await scrollLogsToBottomAfterRender();
    await refreshStatus();
    await scrollLogsToBottomAfterRender();
  }

  async function scrollLogsToBottomAfterRender() {
    await tick();
    requestAnimationFrame(() => {
      autoScroller.scrollToBottom();
    });
  }

  async function handleToggleAutoUpdate() {
    autoUpdateSaving = true;
    autoUpdateMessage = "";
    const result = await toggleAutoUpdateHosting(selectedSaveId || null);
    autoUpdateMessage = result.message;
    if (result.success) {
      appendServerLogs([`[系统] ${result.message}`], selectedSaveId);
    } else {
      appendServerLogs([`[错误] ${result.message}`], selectedSaveId);
    }
    autoUpdateSaving = false;
  }

  async function refreshStatus() {
    if (polling) return;
    polling = true;
    try {
      const newLines = await refreshServerStatus(viewingSaveId);
      if (newLines.length > 0) {
        if (!firstLoadDone) {
          firstLoadDone = true;
          autoScroller.autoScroll = true;
          await scrollLogsToBottomAfterRender();
        } else if (!normalizedLogSearch && autoScroller.autoScroll) {
          await scrollLogsToBottomAfterRender();
        }
      }
    } catch {
    } finally {
      polling = false;
    }
  }

  async function startServer() {
    setServerLoading(selectedSaveId, "starting");
    clearServerLogs(selectedSaveId);
    firstLoadDone = false;
    autoScroller.autoScroll = true;
    try {
      await invoke("start_server", {
        saveId: selectedSaveId || null,
        launchMode: appState.launchMode,
      });
    } catch (e: any) {
      toastStore.error(`${e}`);
      appendServerLogs([`[错误] ${e}`], selectedSaveId);
    }
    setServerLoading(selectedSaveId, "");
    await refreshStatus();
  }

  async function stopServer() {
    const targetSaveId = viewingSaveId;
    setServerLoading(targetSaveId, "stopping");
    try {
      await invoke("stop_server", { saveId: targetSaveId || null });
    } catch (e: any) {
      appendServerLogs([`[错误] ${e}`], targetSaveId);
    }
    setServerLoading(targetSaveId, "");
    await refreshStatus();
  }

  async function restartServer() {
    const targetSaveId = viewingSaveId;
    setServerLoading(targetSaveId, "restarting");
    autoScroller.autoScroll = true;
    try {
      await invoke("restart_server", {
        saveId: targetSaveId || null,
        launchMode: selectedRunningServer?.launch_mode || appState.launchMode,
      });
    } catch (e: any) {
      appendServerLogs([`[错误] ${e}`], targetSaveId);
    }
    setServerLoading(targetSaveId, "");
    await refreshStatus();
  }

  async function forceStop() {
    if (!confirm("确定要强制停止服务器吗？未保存的数据可能丢失。")) return;
    const targetSaveId = viewingSaveId;
    try {
      await invoke("force_stop_server", { saveId: targetSaveId || null });
      await refreshStatus();
    } catch (e: any) {
      appendServerLogs([`[错误] ${e}`], targetSaveId);
    }
  }

  async function sendLocalCommand() {
    const command = localCommand.trim();
    if (!canSendLocalCommand || !command) return;

    localCommandSending = true;
    try {
      await invoke("send_server_command", { saveId: viewingSaveId || null, command });
      localCommand = "";
      autoScroller.autoScroll = true;
      await refreshStatus();
      await scrollLogsToBottomAfterRender();
    } catch (e: any) {
      appendServerLogs([`[错误] 本地命令发送失败: ${e}`], viewingSaveId);
      await scrollLogsToBottomAfterRender();
    } finally {
      localCommandSending = false;
    }
  }

  $effect(() => {
    loadSaves();
    loadSharedSettings();
    poller.start();
    const cleanup = poller.setupVisibilityListener();
    return () => {
      cleanup();
      poller.stop();
    };
  });

  $effect(() => {
    if (selectedSaveId && !selectedRunningSaveId) {
      syncSelectedServerRuntime(selectedSaveId);
    }
  });
</script>

<div class="flex flex-col gap-5">
  <!-- Header -->
  <div class="flex flex-wrap items-center justify-between gap-3 flex-shrink-0">
    <div>
      <h1 class="text-2xl font-bold text-[var(--text-primary)]">服务器控制</h1>
      <p class="text-sm text-[var(--text-muted)] mt-1">启动、停止、重启服务器</p>
    </div>
    <div class="flex items-center gap-3 px-4 py-2 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
      <div class="w-2.5 h-2.5 rounded-full {status === '运行中' ? 'bg-[var(--success)] animate-pulse' : status === '错误' ? 'bg-[var(--danger)]' : 'bg-[var(--text-muted)]'}"></div>
      <span class="text-sm font-medium {status === '运行中' ? 'text-[var(--success)]' : status === '错误' ? 'text-[var(--danger)]' : 'text-[var(--text-secondary)]'}">{status}</span>
      {#if runningCount > 0}
        <span class="text-xs text-[var(--text-muted)]">运行 {runningCount}</span>
      {/if}
    </div>
  </div>

  {#if runningCount > 0}
    <div class="flex-shrink-0">
      <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
        <h2 class="text-sm font-semibold text-[var(--text-primary)]">运行中的服务器</h2>
        <span class="text-xs text-[var(--text-muted)]">{runningCount} 个实例</span>
      </div>
      <div class="{runningCount > 8 ? 'max-h-[180px] overflow-y-auto pr-1' : ''}">
        <div class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {#each runningServers as server (server.save_id)}
            <button
              type="button"
              onclick={() => handleRunningServerSelect(server.save_id)}
              title={getSaveName(server.save_id)}
              class="flex min-h-[52px] items-center justify-between gap-3 rounded-lg border px-4 py-3 text-left transition-all duration-[var(--transition-normal)] cursor-pointer {server.save_id === selectedRunningSaveId ? 'border-[var(--accent)] bg-[var(--accent-subtle)] shadow-[var(--shadow-sm)]' : 'border-[var(--border)] bg-[var(--bg-card)] hover:border-[var(--border-hover)] hover:bg-[var(--bg-card-hover)]'}"
            >
              <div class="flex min-w-0 items-center gap-2">
                <span class="h-2.5 w-2.5 shrink-0 rounded-full bg-[var(--success)] animate-pulse"></span>
                <span class="truncate text-sm font-semibold text-[var(--text-primary)]">{getSaveName(server.save_id)}</span>
              </div>
              <span class="shrink-0 rounded-md px-2 py-1 text-xs font-medium {server.save_id === selectedRunningSaveId ? 'bg-[var(--accent)] text-white' : 'bg-[var(--success-glow)] text-[var(--success)]'}">
                {server.save_id === selectedRunningSaveId ? '当前' : '运行中'}
              </span>
            </button>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  <!-- Control Panel -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex-shrink-0">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div class="flex flex-wrap gap-3">
        <button
          class="px-5 py-2.5 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={startServer}
          disabled={!selectedSaveId || selectedSaveRunning || launchSaveLoading !== ''}
        >
          {#if launchSaveLoading === 'starting'}
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            </svg>
          {/if}
          {launchSaveLoading === 'starting' ? '启动中...' : selectedSaveRunning ? '已运行' : '启动'}
        </button>

        <button
          class="px-5 py-2.5 bg-gradient-to-r from-[var(--danger)] to-rose-600 hover:from-rose-500 hover:to-[var(--danger)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
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
          class="px-5 py-2.5 bg-gradient-to-r from-[var(--warning)] to-amber-600 hover:from-amber-500 hover:to-[var(--warning)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
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
          disabled={status !== '运行中' || loading !== ''}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
          </svg>
          强制停止
        </button>
      </div>

      <div class="flex flex-wrap items-center gap-3">
        <SaveSelector saves={sharedSaves} bind:value={uiPreferences.selectedSaveId} onChange={handleSelectedSaveChange} />
        <div class="flex rounded-lg overflow-hidden border border-[var(--border)]">
          <button
            class="px-2 py-1 text-xs font-medium transition-all disabled:cursor-not-allowed disabled:opacity-50 {launchModeLocked ? '' : 'cursor-pointer'} {appState.launchMode === 'internet' ? 'bg-[var(--accent)] text-[var(--text-primary)]' : 'bg-[var(--bg-primary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
            onclick={() => appState.launchMode = 'internet'}
            disabled={launchModeLocked}
          >互联网</button>
          <button
            class="px-2 py-1 text-xs font-medium transition-all disabled:cursor-not-allowed disabled:opacity-50 {launchModeLocked ? '' : 'cursor-pointer'} {appState.launchMode === 'lan' ? 'bg-[var(--accent)] text-[var(--text-primary)]' : 'bg-[var(--bg-primary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
            onclick={() => appState.launchMode = 'lan'}
            disabled={launchModeLocked}
          >局域网</button>
        </div>
        <div class="flex items-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-3 py-1.5">
          <span class="text-xs text-[var(--text-muted)]">自动更新托管</span>
          <button
            type="button"
            role="switch"
            aria-checked={sharedSettings.autoUpdateHosting}
            class="relative h-6 w-11 rounded-full border transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed {sharedSettings.autoUpdateHosting ? 'bg-[var(--success)] border-[var(--success)]' : 'bg-[var(--bg-elevated)] border-[var(--border)]'}"
            onclick={handleToggleAutoUpdate}
            disabled={autoUpdateSaving}
            title="自动更新托管"
          >
            <span class="absolute top-0.5 h-[18px] w-[18px] rounded-full bg-white transition-all {sharedSettings.autoUpdateHosting ? 'left-[21px]' : 'left-0.5'}"></span>
          </button>
        </div>
        {#if autoUpdateMessage}
          <span class="text-xs {autoUpdateMessage.includes('失败') ? 'text-[var(--danger)]' : 'text-[var(--success)]'}">{autoUpdateMessage}</span>
        {/if}
      </div>

      <div class="flex flex-wrap gap-4 sm:gap-6 text-sm">
        <div class="flex items-center gap-2">
          <span class="text-[var(--text-muted)]">PID</span>
          <span class="text-[var(--text-primary)] font-mono font-semibold bg-[var(--bg-elevated)] px-2 py-1 rounded">{pid}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-[var(--text-muted)]">运行</span>
          <span class="text-[var(--text-primary)] font-semibold">{uptime}</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Log Output -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl flex flex-col max-h-[50vh]">
    <div class="flex flex-wrap items-center justify-between gap-3 px-5 py-3 border-b border-[var(--border)] flex-shrink-0">
      <div class="flex items-center gap-2">
        <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <span class="text-sm font-medium text-[var(--text-primary)]">服务器输出</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="relative">
          <svg class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            bind:value={logSearchText}
            placeholder="搜索日志..."
            class="w-full sm:w-44 bg-[var(--bg-primary)] border border-[var(--border)] rounded-md pl-8 pr-2 py-1 text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>
        <button
          class="text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors duration-[var(--transition-fast)] px-2 py-1 rounded hover:bg-[var(--bg-card-hover)] cursor-pointer"
          onclick={() => clearServerLogs(viewingSaveId)}
        >
          清空
        </button>
      </div>
    </div>
    <div bind:this={autoScroller.container} onscroll={autoScroller.handleScroll} class="flex-1 overflow-y-auto p-4 font-mono text-xs leading-6">
      {#if logs.length === 0}
        <div class="flex items-center justify-center h-full text-[var(--text-muted)]">
          <p class="italic">等待服务器启动...</p>
        </div>
      {:else}
        {#if logSearchText && filteredLogs.length === 0}
          <div class="flex items-center justify-center h-full text-[var(--text-muted)]">
            <p class="italic">未找到匹配内容</p>
          </div>
        {:else}
          {#if logSearchText}
            <div class="pb-2 mb-2 border-b border-[var(--border)] text-[var(--text-muted)]">
              找到 {filteredLogs.length} 条匹配
            </div>
          {/if}
          {#each filteredLogs as log}
            <p class="py-0.5 {log.level === 'error' ? 'text-[var(--danger)]' : log.level === 'warning' ? 'text-[var(--warning)]' : log.level === 'info' ? 'text-[var(--success)]' : log.level === 'system' ? 'text-[var(--accent-light)] font-medium' : 'text-[var(--text-secondary)]'}">
              {@html highlightText(log.text, logSearchText)}
            </p>
          {/each}
        {/if}
      {/if}
    </div>
  </div>

  <!-- Local Command Input -->
  <form class="flex gap-3 flex-shrink-0" onsubmit={(e) => { e.preventDefault(); sendLocalCommand(); }}>
    <div class="flex-1 relative">
      <input
        type="text"
        bind:value={localCommand}
        placeholder="输入命令，如 Save 或 Say hello"
        class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-3 pr-11 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)] font-mono disabled:opacity-50 disabled:cursor-not-allowed"
        disabled={status !== '运行中' || loading !== ''}
        aria-label="本地服务器命令"
      />
      <div class="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
      </div>
    </div>
    <button
      type="submit"
      class="px-6 py-3 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
      disabled={!canSendLocalCommand}
      title="发送到本地服务器控制台"
    >
      {#if localCommandSending}
        <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
        发送中...
      {:else}
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
        </svg>
        发送
      {/if}
    </button>
  </form>
</div>
