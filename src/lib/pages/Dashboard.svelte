<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let status = $state("已停止");
  let uptime = $state("--");
  let pid = $state("--");
  let loading = $state("");

  let saves = $state<any[]>([]);
  let selectedSaveId = $state("");
  let launchMode = $state("internet");

  let cpuUsage = $state(0);
  let memUsed = $state(0);
  let memTotal = $state(0);
  let memPercent = $state(0);
  let netDownRate = $state(0);
  let netUpRate = $state(0);
  let totalDown = $state(0);
  let totalUp = $state(0);

  // Non-reactive variables for internal state tracking
  let prevNetDown = 0;
  let prevNetUp = 0;
  let lastPollTime = 0;
  let firstNetPoll = true;
  let polling = false;

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
    if (bytes < 1073741824) return (bytes / 1048576).toFixed(1) + " MB";
    return (bytes / 1073741824).toFixed(2) + " GB";
  }

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
    try {
      saves = await invoke("list_server_saves");
      if (saves.length > 0 && !selectedSaveId) {
        selectedSaveId = saves[0].id;
      }
    } catch {}
  }

  async function refreshStatus() {
    try {
      const s: any = await invoke("get_server_status");
      status = s.state;
      pid = s.pid ? String(s.pid) : "--";
      if (s.uptime_secs > 0) {
        const h = Math.floor(s.uptime_secs / 3600);
        const m = Math.floor((s.uptime_secs % 3600) / 60);
        const sec = Math.floor(s.uptime_secs % 60);
        uptime = h > 0 ? `${h}时${m}分${sec}秒` : m > 0 ? `${m}分${sec}秒` : `${sec}秒`;
      } else {
        uptime = "--";
      }
    } catch {}
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
    } catch {}
  }

  async function startServer() {
    loading = "starting";
    try {
      await invoke("start_server", {
        saveId: selectedSaveId || null,
        launchMode: launchMode,
      });
    } catch (e: any) {
      alert(e);
    }
    loading = "";
    await refreshStatus();
  }

  async function stopServer() {
    loading = "stopping";
    try {
      await invoke("stop_server");
    } catch (e: any) {
      alert(e);
    }
    loading = "";
    await refreshStatus();
  }

  async function restartServer() {
    loading = "restarting";
    try {
      await invoke("restart_server", {
        saveId: selectedSaveId || null,
        launchMode: launchMode,
      });
    } catch (e: any) {
      alert(e);
    }
    loading = "";
    await refreshStatus();
  }

  async function pollAll() {
    if (polling) return;
    polling = true;
    try {
      await Promise.all([refreshStatus(), refreshSystemStats()]);
    } finally {
      polling = false;
    }
  }

  $effect(() => {
    loadSaves();
    pollAll();
    const interval = setInterval(pollAll, 2000);
    return () => clearInterval(interval);
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="flex items-center justify-between mb-8">
    <div>
      <h1 class="text-2xl font-bold text-white">仪表盘</h1>
      <p class="text-sm text-[var(--text-muted)] mt-1">服务器状态概览与系统监控</p>
    </div>
    <div class="flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
      <div class="w-2 h-2 rounded-full {status === '运行中' ? 'bg-[var(--success)] animate-pulse' : status === '错误' ? 'bg-[var(--danger)]' : 'bg-[var(--text-muted)]'}"></div>
      <span class="text-sm text-[var(--text-secondary)]">{status}</span>
    </div>
  </div>

  <!-- Server Status Cards -->
  <div class="grid grid-cols-3 gap-5 mb-8">
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
      <p class="text-2xl font-bold text-white">{uptime}</p>
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
      <p class="text-2xl font-bold text-white font-mono">{pid}</p>
    </div>
  </div>

  <!-- System Monitoring Cards -->
  <div class="mb-8">
    <h2 class="text-base font-semibold text-white mb-5 flex items-center gap-2">
      <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      系统监控
    </h2>
    <div class="grid grid-cols-3 gap-5">
      <!-- CPU -->
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

      <!-- Memory -->
      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 hover:border-purple-500 transition-all duration-[var(--transition-normal)]">
        <div class="flex items-center gap-3 mb-4">
          <div class="w-10 h-10 rounded-lg bg-purple-500/10 flex items-center justify-center">
            <svg class="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
          </div>
          <p class="text-xs text-[var(--text-muted)] uppercase tracking-wider">内存使用</p>
        </div>
        <p class="text-2xl font-bold mb-1" style="color: {getStatusColor(memPercent)}">{memPercent.toFixed(1)}%</p>
        <p class="text-xs text-[var(--text-muted)] mb-3">{formatBytes(memUsed)} / {formatBytes(memTotal)}</p>
        <div class="w-full h-2 rounded-full bg-[var(--border)]">
          <div class="h-full rounded-full bg-purple-500 transition-all duration-500" style="width: {Math.min(memPercent, 100)}%"></div>
        </div>
      </div>

      <!-- Network -->
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

  <!-- Quick Actions -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
    <h2 class="text-base font-semibold text-white mb-5 flex items-center gap-2">
      <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
      快捷操作
    </h2>

    <!-- Save & Mode Selection -->
    <div class="flex items-center gap-4 mb-5 pb-5 border-b border-[var(--border)]">
      <div class="flex items-center gap-2">
        <label class="text-xs text-[var(--text-muted)]">存档:</label>
        <select
          bind:value={selectedSaveId}
          class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:border-[var(--accent)] transition-colors cursor-pointer"
        >
          {#each saves as save}
            <option value={save.id}>{save.id}{save.name ? ` - ${save.name}` : ''}</option>
          {/each}
        </select>
      </div>
      <div class="flex items-center gap-2">
        <label class="text-xs text-[var(--text-muted)]">模式:</label>
        <div class="flex rounded-lg overflow-hidden border border-[var(--border)]">
          <button
            class="px-3 py-1.5 text-xs font-medium transition-all cursor-pointer {launchMode === 'internet' ? 'bg-[var(--accent)] text-white' : 'bg-[var(--bg-primary)] text-[var(--text-secondary)] hover:text-white'}"
            onclick={() => launchMode = 'internet'}
          >互联网</button>
          <button
            class="px-3 py-1.5 text-xs font-medium transition-all cursor-pointer {launchMode === 'lan' ? 'bg-[var(--accent)] text-white' : 'bg-[var(--bg-primary)] text-[var(--text-secondary)] hover:text-white'}"
            onclick={() => launchMode = 'lan'}
          >局域网</button>
        </div>
      </div>
    </div>

    <div class="flex gap-4">
      <button
        class="flex-1 px-6 py-3 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-white text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center justify-center gap-2 shadow-lg shadow-[var(--success-glow)]"
        onclick={startServer}
        disabled={status === '运行中' || loading !== ''}
      >
        {#if loading === 'starting'}
          <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          启动中...
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          启动服务器
        {/if}
      </button>

      <button
        class="flex-1 px-6 py-3 bg-gradient-to-r from-[var(--danger)] to-rose-600 hover:from-rose-500 hover:to-[var(--danger)] text-white text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center justify-center gap-2 shadow-lg shadow-[var(--danger-glow)]"
        onclick={stopServer}
        disabled={status !== '运行中' || loading !== ''}
      >
        {#if loading === 'stopping'}
          <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          停止中...
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
          </svg>
          停止服务器
        {/if}
      </button>

      <button
        class="flex-1 px-6 py-3 bg-gradient-to-r from-[var(--warning)] to-amber-600 hover:from-amber-500 hover:to-[var(--warning)] text-white text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center justify-center gap-2 shadow-lg shadow-[var(--warning-glow)]"
        onclick={restartServer}
        disabled={status !== '运行中' || loading !== ''}
      >
        {#if loading === 'restarting'}
          <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          重启中...
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
