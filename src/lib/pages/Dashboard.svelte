<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { formatBytes, formatUptime, copyToClipboard } from "$lib/utils";
  import { appState, uiPreferences, setSelectedSaveId, ensureSelectedSaveId, sharedSaves, loadSharedSaves, sharedSettings, loadSharedSettings, toggleAutoUpdateHosting, serverInfo, serverState, serverLogs, runningServers, refreshServerStatus, clearServerLogs, setServerLoading, syncSelectedServerRuntime, serverView, selectRunningServer, getServerInfoForSave, resetServerInfoForSave, serverStatesBySave, serverInfoBySave } from "$lib/stores.svelte";
  import { toastStore } from "../stores/toast.svelte";
  import { createPoller } from "../utils/polling.svelte";
  import SaveSelector from "../components/SaveSelector.svelte";

  // 使用共享的服务器状态
  let status = $derived(serverState.status);
  let uptime = $derived(serverState.uptime);
  let pid = $derived(serverState.pid);
  let loading = $derived(serverState.loading);
  let runningCount = $derived(runningServers.length);

  let selectedSaveId = $derived(uiPreferences.selectedSaveId);
  let selectedRunningSaveId = $derived(serverView.selectedRunningSaveId);
  let viewingSaveId = $derived(selectedRunningSaveId || selectedSaveId);
  let selectedRunningServer = $derived(runningServers.find((server) => server.save_id === selectedRunningSaveId));
  const emptyServerInfo = { serverCode: "", port: 0, portLoading: false, codeParsed: false };
  let viewingServerInfo = $derived(serverInfoBySave[viewingSaveId || "__default__"] ?? emptyServerInfo);
  let launchSaveRuntime = $derived(serverStatesBySave[selectedSaveId || "__default__"]);
  let launchSaveLoading = $derived(launchSaveRuntime?.loading ?? "");
  let selectedSaveRunning = $derived(runningServers.some((server) => server.save_id === selectedSaveId));
  let launchModeLocked = $derived(selectedSaveRunning || launchSaveLoading !== "");
  let autoUpdateSaving = $state(false);
  let autoUpdateMessage = $state("");

  let cpuUsage = $state(0);
  let memUsed = $state(0);
  let memTotal = $state(0);
  let memPercent = $state(0);
  let netDownRate = $state(0);
  let netUpRate = $state(0);
  let totalDown = $state(0);
  let totalUp = $state(0);

  let parsedExistingLogsForCodeBySave = $state<Record<string, boolean>>({});

  // Non-reactive variables for internal state tracking
  let prevNetDown = 0;
  let prevNetUp = 0;
  let lastPollTime = 0;
  let firstNetPoll = true;
  let polling = false;

  // 轮询管理器
  const poller = createPoller({
    pollFn: pollAll,
    isActive: () => serverState.loading !== "" || serverState.status === "运行中" || runningServers.length > 0,
  });

  function formatRate(bytesPerSec: number): string {
    if (bytesPerSec < 0) return "0 B/s";
    if (bytesPerSec < 1024) return bytesPerSec.toFixed(0) + " B/s";
    if (bytesPerSec < 1048576) return (bytesPerSec / 1024).toFixed(1) + " KB/s";
    return (bytesPerSec / 1048576).toFixed(2) + " MB/s";
  }

  function getStatusColor(percent: number): string {
    if (percent >= 85) return "var(--danger)";
    if (percent >= 60) return "var(--warning)";
    return "var(--success)";
  }

  async function loadSaves() {
    await loadSharedSaves();
    ensureSelectedSaveId(sharedSaves);
  }

  function handleSelectedSaveChange(value: string) {
    setSelectedSaveId(value);
    pollAll();
  }

  async function handleToggleAutoUpdate() {
    autoUpdateSaving = true;
    autoUpdateMessage = "";
    const result = await toggleAutoUpdateHosting(selectedSaveId || null);
    autoUpdateMessage = result.message;
    if (!result.success) {
      toastStore.error(result.message);
    }
    autoUpdateSaving = false;
  }

  async function fetchPublicIp() {
    if (serverInfo.publicIp || serverInfo.ipLoading) return;
    serverInfo.ipLoading = true;
    try {
      serverInfo.publicIp = await invoke("get_public_ip");
    } catch {
      serverInfo.publicIp = "获取失败";
    }
    serverInfo.ipLoading = false;
  }

  async function fetchServerPort(saveId = viewingSaveId) {
    const info = getServerInfoForSave(saveId);
    if (info.port || info.portLoading) return;
    info.portLoading = true;
    try {
      info.port = await invoke("get_server_port", { saveId: saveId || null });
    } catch {
      info.port = 27015;
    }
    info.portLoading = false;
  }

  function parseServerCode(lines: string[], saveId = viewingSaveId) {
    const info = getServerInfoForSave(saveId);
    for (const line of lines) {
      const match = line.match(/Server Code:\s*(\d+)/);
      if (match) {
        info.serverCode = match[1];
        info.codeParsed = true;
        break;
      }
    }
  }

  function getSaveName(saveId: string): string {
    if (!saveId) return "";
    const save = sharedSaves.find((s: any) => s.id === saveId);
    return save ? (save.name ? `${save.id} - ${save.name}` : save.id) : saveId;
  }

  function formatLaunchMode(mode: string): string {
    return mode === "lan" ? "局域网" : "互联网";
  }

  async function handleRunningServerSelect(saveId: string) {
    if (saveId === selectedRunningSaveId) return;
    selectRunningServer(saveId);
    parsedExistingLogsForCodeBySave[saveId] = false;
    await refreshStatus();
  }

  async function handleCopyToClipboard(text: string) {
    const success = await copyToClipboard(text);
    if (success) {
      toastStore.success("已复制到剪贴板");
    } else {
      toastStore.error("复制失败");
    }
  }

  async function refreshStatus() {
    try {
      const targetSaveId = viewingSaveId;
      const newLines = await refreshServerStatus(targetSaveId);
      const activeSaveId = serverView.selectedRunningSaveId || targetSaveId;
      const activeInfo = getServerInfoForSave(activeSaveId);

      if (serverState.status === "运行中") {
        fetchPublicIp();
        fetchServerPort(activeSaveId);
        if (!activeInfo.codeParsed) {
          parseServerCode(newLines, activeSaveId);
        }
        if (!activeInfo.codeParsed && !parsedExistingLogsForCodeBySave[activeSaveId]) {
          parsedExistingLogsForCodeBySave[activeSaveId] = true;
          parseServerCode(serverLogs.map((log) => log.text), activeSaveId);
        }
      } else if (serverState.status !== "启动中") {
        resetServerInfoForSave(targetSaveId);
      }
    } catch (e) { console.error("刷新服务器状态失败:", e); }
  }

  async function refreshSystemStats() {
    try {
      const s: any = await invoke("get_system_stats");
      cpuUsage = s.cpu_usage;
      memUsed = s.used_memory;
      memTotal = s.total_memory;
      memPercent = s.memory_percentage;
      totalDown = s.bytes_received;
      totalUp = s.bytes_transmitted;

      const now = Date.now();
      if (firstNetPoll) {
        // First poll: initialize without computing rate
        firstNetPoll = false;
        prevNetDown = s.bytes_received;
        prevNetUp = s.bytes_transmitted;
        lastPollTime = now;
        netDownRate = 0;
        netUpRate = 0;
      } else {
        const elapsed = (now - lastPollTime) / 1000;
        if (elapsed > 0) {
          netDownRate = (s.bytes_received - prevNetDown) / elapsed;
          netUpRate = (s.bytes_transmitted - prevNetUp) / elapsed;
        }
        prevNetDown = s.bytes_received;
        prevNetUp = s.bytes_transmitted;
        lastPollTime = now;
      }
    } catch (e) { console.error("刷新系统信息失败:", e); }
  }

  async function startServer() {
    setServerLoading(selectedSaveId, "starting");
    clearServerLogs(selectedSaveId);
    resetServerInfoForSave(selectedSaveId);
    parsedExistingLogsForCodeBySave[selectedSaveId] = false;
    try {
      await invoke("start_server", {
        saveId: selectedSaveId || null,
        launchMode: appState.launchMode,
      });
    } catch (e: any) {
      toastStore.error(`${e}`);
    }
    setServerLoading(selectedSaveId, "");
    await refreshStatus();
  }

  async function stopServer() {
    const targetSaveId = viewingSaveId;
    setServerLoading(targetSaveId, "stopping");
    try {
      await invoke("stop_server", { saveId: targetSaveId || null });
      resetServerInfoForSave(targetSaveId);
    } catch (e: any) {
      toastStore.error(`${e}`);
    }
    setServerLoading(targetSaveId, "");
    await refreshStatus();
  }

  async function restartServer() {
    const targetSaveId = viewingSaveId;
    setServerLoading(targetSaveId, "restarting");
    resetServerInfoForSave(targetSaveId);
    parsedExistingLogsForCodeBySave[targetSaveId] = false;
    try {
      await invoke("restart_server", {
        saveId: targetSaveId || null,
        launchMode: selectedRunningServer?.launch_mode || appState.launchMode,
      });
    } catch (e: any) {
      toastStore.error(`${e}`);
    }
    setServerLoading(targetSaveId, "");
    await refreshStatus();
  }

  async function pollAll() {
    if (polling) return;
    polling = true;
    try {
      // 服务器运行中时同时刷新状态和系统监控，否则只刷新状态
      if (serverState.loading || serverState.status === "运行中") {
        await Promise.all([refreshStatus(), refreshSystemStats()]);
      } else {
        await refreshStatus();
      }
    } finally {
      polling = false;
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

<div>
  <div class="flex flex-wrap items-center justify-between gap-3 mb-8">
    <div>
      <h1 class="text-2xl font-bold text-[var(--text-primary)]">仪表盘</h1>
      <p class="text-sm text-[var(--text-muted)] mt-1">服务器运行状态与系统资源</p>
    </div>
    <div class="flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
      <div class="w-2 h-2 rounded-full {status === '运行中' ? 'bg-[var(--success)] animate-pulse' : status === '错误' ? 'bg-[var(--danger)]' : 'bg-[var(--text-muted)]'}"></div>
      <span class="text-sm text-[var(--text-secondary)]">{status}</span>
      {#if runningCount > 0}
        <span class="text-xs text-[var(--text-muted)]">运行 {runningCount}</span>
      {/if}
    </div>
  </div>

  <div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3 md:gap-5 mb-8">
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 hover:border-[var(--accent)] transition-all duration-[var(--transition-normal)] group">
      <div class="flex items-center gap-3 mb-4">
        <div class="w-10 h-10 rounded-lg bg-[var(--accent-subtle)] flex items-center justify-center">
          <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <p class="text-xs text-[var(--text-muted)] uppercase tracking-wider">服务器状态</p>
      </div>
      <p class="text-2xl font-bold {status === '运行中' ? 'text-[var(--success)]' : status === '错误' ? 'text-[var(--danger)]' : 'text-[var(--text-primary)]'}">
        {status}
      </p>
    </div>

    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 hover:border-[var(--success)] transition-all duration-[var(--transition-normal)] group">
      <div class="flex items-center gap-3 mb-4">
        <div class="w-10 h-10 rounded-lg bg-[var(--success-glow)] flex items-center justify-center">
          <svg class="w-5 h-5 text-[var(--success)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <p class="text-xs text-[var(--text-muted)] uppercase tracking-wider">运行时间</p>
      </div>
      <p class="text-2xl font-bold text-[var(--text-primary)]">{uptime}</p>
    </div>

    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 hover:border-[var(--warning)] transition-all duration-[var(--transition-normal)] group">
      <div class="flex items-center gap-3 mb-4">
        <div class="w-10 h-10 rounded-lg bg-[var(--warning-glow)] flex items-center justify-center">
          <svg class="w-5 h-5 text-[var(--warning)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
          </svg>
        </div>
        <p class="text-xs text-[var(--text-muted)] uppercase tracking-wider">进程 PID</p>
      </div>
      <p class="text-2xl font-bold text-[var(--text-primary)] font-mono">{pid}</p>
    </div>
  </div>

  {#if runningCount > 0}
  <div class="mb-8">
    <h2 class="text-base font-semibold text-[var(--text-primary)] mb-5 flex items-center gap-2">
      <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      服务器信息
    </h2>
    <div class="mb-4 {runningCount > 6 ? 'max-h-[360px] overflow-y-auto pr-1' : ''}">
      <div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
        {#each runningServers as server (server.save_id)}
          <button
            type="button"
            onclick={() => handleRunningServerSelect(server.save_id)}
            title={getSaveName(server.save_id)}
            class="min-h-[132px] rounded-xl border p-4 text-left transition-all duration-[var(--transition-normal)] cursor-pointer {server.save_id === selectedRunningSaveId ? 'border-[var(--accent)] bg-[var(--accent-subtle)] shadow-[var(--shadow-md)]' : 'border-[var(--border)] bg-[var(--bg-card)] hover:border-[var(--border-hover)] hover:bg-[var(--bg-card-hover)]'}"
          >
            <div class="flex min-w-0 items-start justify-between gap-3">
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <span class="h-2.5 w-2.5 shrink-0 rounded-full bg-[var(--success)] animate-pulse"></span>
                  <p class="truncate text-sm font-semibold text-[var(--text-primary)]">{getSaveName(server.save_id)}</p>
                </div>
                <p class="mt-1 text-xs text-[var(--text-muted)]">{formatLaunchMode(server.launch_mode)}</p>
              </div>
              <span class="shrink-0 rounded-md bg-[var(--success-glow)] px-2 py-1 text-xs font-medium text-[var(--success)]">{server.state}</span>
            </div>
            <div class="mt-4 grid grid-cols-2 gap-2 text-xs">
              <div class="min-w-0 rounded-lg bg-[var(--bg-primary)] px-3 py-2">
                <p class="text-[var(--text-muted)]">PID</p>
                <p class="truncate font-mono font-semibold text-[var(--text-primary)]">{server.pid ?? '--'}</p>
              </div>
              <div class="min-w-0 rounded-lg bg-[var(--bg-primary)] px-3 py-2">
                <p class="text-[var(--text-muted)]">运行</p>
                <p class="truncate font-semibold text-[var(--text-primary)]">{formatUptime(server.uptime_secs)}</p>
              </div>
              <div class="min-w-0 rounded-lg bg-[var(--bg-primary)] px-3 py-2">
                <p class="text-[var(--text-muted)]">输出</p>
                <p class="truncate font-semibold text-[var(--text-primary)]">{server.output_count} 行</p>
              </div>
              <div class="min-w-0 rounded-lg bg-[var(--bg-primary)] px-3 py-2">
                <p class="text-[var(--text-muted)]">查看</p>
                <p class="truncate font-semibold {server.save_id === selectedRunningSaveId ? 'text-[var(--accent-light)]' : 'text-[var(--text-primary)]'}">{server.save_id === selectedRunningSaveId ? '当前' : '切换'}</p>
              </div>
            </div>
          </button>
        {/each}
      </div>
    </div>

    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
      <!-- 当前存档 -->
      <div class="flex items-center gap-3 mb-5 pb-5 border-b border-[var(--border)]">
        <div class="w-10 h-10 rounded-lg bg-[var(--accent-subtle)] flex items-center justify-center">
          <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </div>
        <div class="flex-1 min-w-0">
          <p class="text-xs text-[var(--text-muted)] mb-1">当前运行存档</p>
          <p class="text-lg font-bold text-[var(--text-primary)] truncate">{getSaveName(viewingSaveId) || '--'}</p>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-10 h-10 rounded-lg flex items-center justify-center {(selectedRunningServer?.launch_mode || appState.launchMode) === 'internet' ? 'bg-[var(--accent-subtle)]' : 'bg-[var(--success-glow)]'}">
            {#if (selectedRunningServer?.launch_mode || appState.launchMode) === 'internet'}
              <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
              </svg>
            {:else}
              <svg class="w-5 h-5 text-[var(--success)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.858 15.355-5.858 21.213 0" />
              </svg>
            {/if}
          </div>
          <div>
            <p class="text-xs text-[var(--text-muted)] mb-1">启动方式</p>
            <p class="text-sm font-bold {(selectedRunningServer?.launch_mode || appState.launchMode) === 'internet' ? 'text-[var(--accent-light)]' : 'text-[var(--success)]'}">
              {formatLaunchMode(selectedRunningServer?.launch_mode || appState.launchMode)}
            </p>
          </div>
        </div>
      </div>

      <!-- 公网IP + 端口 -->
      <div class="mb-5 pb-5 border-b border-[var(--border)]">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-xs text-[var(--text-muted)] mb-1">连接地址</p>
            <p class="text-lg font-mono font-bold text-[var(--text-primary)]">
              {serverInfo.ipLoading ? '获取中...' : (serverInfo.publicIp || '--')}
              {#if viewingServerInfo.port}<span class="text-[var(--text-muted)]">:{viewingServerInfo.port}</span>{/if}
            </p>
          </div>
          <button
            onclick={() => handleCopyToClipboard(`${serverInfo.publicIp}:${viewingServerInfo.port}`)}
            disabled={!serverInfo.publicIp || serverInfo.ipLoading || !viewingServerInfo.port}
            class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg hover:border-[var(--accent)] hover:text-[var(--accent-light)] transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
            复制
          </button>
        </div>
        <p class="text-xs text-[var(--text-muted)] mt-2">
          有公网 IP 可直连，否则需组网工具（ZeroTier / Radmin VPN）
        </p>
      </div>

      <!-- 联机码 -->
      <div class="mt-5 pt-5 border-t border-[var(--border)]">
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0">
            <p class="text-xs text-[var(--text-muted)] mb-1">联机码</p>
            <p class="text-lg font-mono font-bold text-[var(--text-primary)] truncate">
              {viewingServerInfo.serverCode || '等待服务器输出...'}
            </p>
          </div>
          <button
            onclick={() => handleCopyToClipboard(viewingServerInfo.serverCode)}
            disabled={!viewingServerInfo.serverCode}
            class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg hover:border-[var(--accent)] hover:text-[var(--accent-light)] transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed ml-4"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
            复制
          </button>
        </div>
      </div>

    </div>
  </div>
  {/if}

  <div class="mb-8">
    <h2 class="text-base font-semibold text-[var(--text-primary)] mb-5 flex items-center gap-2">
      <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      系统监控
    </h2>
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3 md:gap-5">
      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 hover:border-[var(--accent)] transition-all duration-[var(--transition-normal)]">
        <div class="flex items-center gap-3 mb-4">
          <div class="w-10 h-10 rounded-lg bg-[var(--accent-subtle)] flex items-center justify-center">
            <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
            </svg>
          </div>
          <p class="text-xs text-[var(--text-muted)] uppercase tracking-wider">CPU 使用率</p>
        </div>
        <p class="text-2xl font-bold mb-3" style="color: {getStatusColor(cpuUsage)}">{cpuUsage.toFixed(1)}%</p>
        <div class="w-full h-2 rounded-full bg-[var(--border)]">
          <div class="h-full rounded-full transition-all duration-500" style="width: {Math.min(cpuUsage, 100)}%; background-color: {getStatusColor(cpuUsage)}"></div>
        </div>
      </div>

      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 hover:border-[var(--action)] transition-all duration-[var(--transition-normal)]">
        <div class="flex items-center gap-3 mb-4">
          <div class="w-10 h-10 rounded-lg bg-blue-50 flex items-center justify-center">
            <svg class="w-5 h-5 text-[var(--action)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
          </div>
          <p class="text-xs text-[var(--text-muted)] uppercase tracking-wider">内存使用</p>
        </div>
        <p class="text-2xl font-bold mb-1" style="color: {getStatusColor(memPercent)}">{memPercent.toFixed(1)}%</p>
        <p class="text-xs text-[var(--text-muted)] mb-3">{formatBytes(memUsed)} / {formatBytes(memTotal)}</p>
        <div class="w-full h-2 rounded-full bg-[var(--border)]">
          <div class="h-full rounded-full bg-[var(--action)] transition-all duration-500" style="width: {Math.min(memPercent, 100)}%"></div>
        </div>
      </div>

      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 hover:border-[var(--accent)] transition-all duration-[var(--transition-normal)]">
        <div class="flex items-center gap-3 mb-4">
          <div class="w-10 h-10 rounded-lg bg-[var(--success-glow)] flex items-center justify-center">
            <svg class="w-5 h-5 text-[var(--success)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
            </svg>
          </div>
          <p class="text-xs text-[var(--text-muted)] uppercase tracking-wider">网络流量</p>
        </div>
        <div class="space-y-2 mb-3">
          <div class="flex items-center gap-2">
            <svg class="w-4 h-4 text-[var(--success)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
            </svg>
            <span class="text-lg font-bold text-[var(--success)]">{formatRate(netDownRate)}</span>
          </div>
          <div class="flex items-center gap-2">
            <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18" />
            </svg>
            <span class="text-lg font-bold text-[var(--accent-light)]">{formatRate(netUpRate)}</span>
          </div>
        </div>
        <div class="text-xs text-[var(--text-muted)] space-y-1">
          <p>总下载: {formatBytes(totalDown)}</p>
          <p>总上传: {formatBytes(totalUp)}</p>
        </div>
      </div>
    </div>
  </div>

  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
    <h2 class="text-base font-semibold text-[var(--text-primary)] mb-5 flex items-center gap-2">
      <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
      快捷操作
    </h2>

    <div class="flex flex-col gap-4 mb-5 pb-5 border-b border-[var(--border)] lg:flex-row lg:items-center">
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
        <span class="text-xs text-[var(--text-muted)]">存档:</span>
        <SaveSelector saves={sharedSaves} bind:value={uiPreferences.selectedSaveId} onChange={handleSelectedSaveChange} />
      </div>
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
        <span class="text-xs text-[var(--text-muted)]">模式:</span>
        <div class="flex rounded-lg overflow-hidden border border-[var(--border)]">
          <button
            class="px-3 py-1.5 text-xs font-medium transition-all disabled:cursor-not-allowed disabled:opacity-50 {launchModeLocked ? '' : 'cursor-pointer'} {appState.launchMode === 'internet' ? 'bg-[var(--accent)] text-[var(--text-primary)]' : 'bg-[var(--bg-primary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
            onclick={() => appState.launchMode = 'internet'}
            disabled={launchModeLocked}
          >互联网</button>
          <button
            class="px-3 py-1.5 text-xs font-medium transition-all disabled:cursor-not-allowed disabled:opacity-50 {launchModeLocked ? '' : 'cursor-pointer'} {appState.launchMode === 'lan' ? 'bg-[var(--accent)] text-[var(--text-primary)]' : 'bg-[var(--bg-primary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
            onclick={() => appState.launchMode = 'lan'}
            disabled={launchModeLocked}
          >局域网</button>
        </div>
      </div>
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
        <span class="text-xs text-[var(--text-muted)]">自动更新:</span>
        <button
          type="button"
          role="switch"
          aria-checked={sharedSettings.autoUpdateHosting}
          class="relative h-7 w-12 rounded-full border transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed {sharedSettings.autoUpdateHosting ? 'bg-[var(--success)] border-[var(--success)]' : 'bg-[var(--bg-primary)] border-[var(--border)]'}"
          onclick={handleToggleAutoUpdate}
          disabled={autoUpdateSaving}
          title="自动更新托管"
        >
          <span class="absolute top-0.5 h-5 w-5 rounded-full bg-white transition-all {sharedSettings.autoUpdateHosting ? 'left-6' : 'left-0.5'}"></span>
        </button>
        {#if autoUpdateMessage}
          <span class="text-xs {autoUpdateMessage.includes('失败') ? 'text-[var(--danger)]' : 'text-[var(--success)]'}">{autoUpdateMessage}</span>
        {/if}
      </div>
    </div>

    <div class="flex flex-col gap-3 lg:flex-row">
      <button
        class="flex-1 px-6 py-3 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center justify-center gap-2 shadow-lg shadow-[var(--success-glow)]"
        onclick={startServer}
        disabled={!selectedSaveId || selectedSaveRunning || launchSaveLoading !== ''}
      >
        {#if launchSaveLoading === 'starting'}
          <div role="status" aria-live="polite" class="flex items-center gap-2">
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" aria-hidden="true"></div>
            <span>启动中...</span>
            <span class="sr-only">正在启动服务器</span>
          </div>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          {selectedSaveRunning ? '已在运行' : '启动服务器'}
        {/if}
      </button>

      <button
        class="flex-1 px-6 py-3 bg-gradient-to-r from-[var(--danger)] to-rose-600 hover:from-rose-500 hover:to-[var(--danger)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center justify-center gap-2 shadow-lg shadow-[var(--danger-glow)]"
        onclick={stopServer}
        disabled={status !== '运行中' || loading !== ''}
      >
        {#if loading === 'stopping'}
          <div role="status" aria-live="polite" class="flex items-center gap-2">
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" aria-hidden="true"></div>
            <span>停止中...</span>
            <span class="sr-only">正在停止服务器</span>
          </div>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
          </svg>
          停止服务器
        {/if}
      </button>

      <button
        class="flex-1 px-6 py-3 bg-gradient-to-r from-[var(--warning)] to-amber-600 hover:from-amber-500 hover:to-[var(--warning)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center justify-center gap-2 shadow-lg shadow-[var(--warning-glow)]"
        onclick={restartServer}
        disabled={status !== '运行中' || loading !== ''}
      >
        {#if loading === 'restarting'}
          <div role="status" aria-live="polite" class="flex items-center gap-2">
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" aria-hidden="true"></div>
            <span>重启中...</span>
            <span class="sr-only">正在重启服务器</span>
          </div>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          重启服务器
        {/if}
      </button>
    </div>
  </div>
</div>


