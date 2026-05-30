<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { rconLogs, addRconLog } from "$lib/stores.svelte";
  import { highlightText } from "$lib/utils";

  let connected = $state(false);
  let command = $state("");
  let connecting = $state(false);
  let logContainer: HTMLDivElement | undefined = $state();
  let logSearch = $state("");

  async function checkStatus() {
    try {
      const status = await invoke("rcon_status") as boolean;
      if (connected && !status) {
        addRconLog("连接已断开（服务器可能已关闭）", "error");
        scrollToBottom();
      }
      connected = status;
    } catch { connected = false; }
  }

  async function pollResponses() {
    if (!connected) return;
    try {
      const lines = await invoke("rcon_poll") as string[];
      if (lines.length > 0) {
        for (const line of lines) {
          addRconLog(line, "response");
        }
        scrollToBottom();
      }
    } catch {}
  }

  async function connect() {
    connecting = true;
    addRconLog("正在连接...", "system");
    try {
      const welcome = await invoke("rcon_connect") as string;
      connected = true;
      addRconLog(welcome, "info");
      addRconLog("RCON 连接成功", "system");
    } catch (e: any) {
      addRconLog(`连接失败: ${e}`, "error");
    }
    connecting = false;
    scrollToBottom();
  }

  async function disconnect() {
    clearTimeout(autoConnectTimer);
    await invoke("rcon_disconnect");
    connected = false;
    addRconLog("已断开连接", "system");
    scrollToBottom();
  }

  async function send() {
    const cmd = command.trim();
    if (!cmd) return;

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

  function scrollToBottom() {
    setTimeout(() => {
      if (logContainer) logContainer.scrollTop = logContainer.scrollHeight;
    }, 50);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") send();
  }

  let pollTimer: ReturnType<typeof setTimeout> | undefined;
  let statusTimer: ReturnType<typeof setTimeout> | undefined;
  let autoConnectTimer: ReturnType<typeof setTimeout> | undefined;
  let effectAlive = true;

  function schedulePoll() {
    pollTimer = setTimeout(async () => {
      await pollResponses();
      if (effectAlive) schedulePoll();
    }, 2000);
  }

  function scheduleStatusCheck() {
    statusTimer = setTimeout(async () => {
      await checkStatus();
      if (effectAlive) scheduleStatusCheck();
    }, 10000);
  }

  $effect(() => {
    effectAlive = true;
    checkStatus();
    schedulePoll();
    scheduleStatusCheck();

    // Auto-connect when a server starts
    const unlisten = listen("server-started", () => {
      addRconLog("检测到服务器启动，5 秒后自动连接 RCON...", "system");
      scrollToBottom();
      clearTimeout(autoConnectTimer);
      autoConnectTimer = setTimeout(async () => {
        if (effectAlive && !connected && !connecting) {
          await connect();
        }
      }, 5000);
    });

    return () => {
      effectAlive = false;
      clearTimeout(pollTimer);
      clearTimeout(statusTimer);
      clearTimeout(autoConnectTimer);
      unlisten.then(fn => fn());
    };
  });
</script>

<div class="flex flex-col h-full gap-5">
  <div>
    <h1 class="text-2xl font-bold text-[var(--text-primary)]">RCON 控制台</h1>
    <p class="text-sm text-[var(--text-muted)] mt-1">远程控制台管理</p>
  </div>

  <!-- Connection Status -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex-shrink-0">
    <div class="flex items-center gap-4 flex-wrap">
      <div class="flex items-center gap-3">
        <div class="w-3 h-3 rounded-full {connected ? 'bg-[var(--success)] animate-pulse' : 'bg-[var(--text-muted)]'}"></div>
        <span class="text-sm font-medium {connected ? 'text-[var(--success)]' : 'text-[var(--text-secondary)]'}">
          {connected ? "已连接" : "未连接"}
        </span>
      </div>

      <div class="flex gap-3 ml-auto">
        <button
          class="px-5 py-2.5 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={connect}
          disabled={connected || connecting}
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
  <div class="flex-1 bg-[var(--bg-card)] border border-[var(--border)] rounded-xl flex flex-col min-h-0">
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
            bind:value={logSearch}
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
    <div bind:this={logContainer} class="flex-1 overflow-y-auto p-4 font-mono text-xs leading-6">
      {#if rconLogs.length === 0}
        <div class="flex items-center justify-center h-full text-[var(--text-muted)]">
          <p class="italic">连接 RCON 后可发送命令...</p>
        </div>
      {:else}
        {@const filtered = logSearch ? rconLogs.filter(r => r.text.toLowerCase().includes(logSearch.toLowerCase())) : rconLogs}
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
          {#each filtered as r}
            <p class="py-0.5 {r.type === 'error' ? 'text-[var(--danger)]' : r.type === 'command' ? 'text-[var(--text-primary)] font-medium' : r.type === 'system' ? 'text-[var(--accent-light)]' : r.type === 'info' ? 'text-[var(--success)]' : 'text-[var(--text-secondary)]'}">
              {@html highlightText(r.text, logSearch)}
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
        disabled={!connected}
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
      disabled={!connected || !command.trim()}
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
      </svg>
      发送
    </button>
  </div>
</div>


