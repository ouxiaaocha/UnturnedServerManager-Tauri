<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let updating = $state(false);
  let output: string[] = $state([]);
  let unlisten: (() => void) | undefined = undefined;
  const outputLimit = 1000;

  function appendOutput(line: string) {
    output = [...output.slice(-(outputLimit - 1)), line];
  }

  async function startUpdate() {
    updating = true;
    output = [];

    // Listen for real-time progress events
    unlisten = await listen<string>("update-output", (event) => {
      appendOutput(event.payload);
    });

    try {
      await invoke("run_update");
    } catch (e: any) {
      appendOutput(`[错误] ${e}`);
    }

    if (unlisten) {
      unlisten();
      unlisten = undefined;
    }
    updating = false;
  }
</script>

<div class="flex flex-col gap-5 h-full overflow-y-auto">
  <div>
    <h1 class="text-2xl font-bold text-[var(--text-primary)]">服务端更新</h1>
    <p class="text-sm text-[var(--text-muted)] mt-1">通过 SteamCMD 更新到最新版本</p>
  </div>

  <!-- Update Action Card -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6 flex-shrink-0">
    <div class="flex flex-col gap-4 sm:flex-row sm:items-start">
      <div class="w-12 h-12 rounded-xl bg-[var(--accent-subtle)] flex items-center justify-center flex-shrink-0">
        <svg class="w-6 h-6 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </div>
      <div class="flex-1">
        <h2 class="text-base font-semibold text-[var(--text-primary)] mb-1">检查更新</h2>
        <p class="text-sm text-[var(--text-secondary)] mb-4">通过 SteamCMD 将 Unturned 服务端更新至最新版本。更新前请先停止服务器。</p>
        <button
          class="px-6 py-2.5 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={startUpdate}
          disabled={updating}
        >
          {#if updating}
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
            更新中...
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            开始更新
          {/if}
        </button>
      </div>
    </div>
  </div>

  <!-- Output Area -->
  {#if output.length > 0}
    <div class="flex-1 bg-[var(--bg-card)] border border-[var(--border)] rounded-xl flex flex-col min-h-0">
      <div class="flex flex-wrap items-center justify-between gap-3 px-5 py-3 border-b border-[var(--border)] flex-shrink-0">
        <div class="flex items-center gap-2">
          <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          <span class="text-sm font-medium text-[var(--text-primary)]">SteamCMD 输出</span>
        </div>
        <button
          class="text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors duration-[var(--transition-fast)] px-2 py-1 rounded hover:bg-[var(--bg-card-hover)] cursor-pointer"
          onclick={() => output = []}
        >
          清空
        </button>
      </div>
      <div class="flex-1 overflow-y-auto p-4 font-mono text-xs leading-6">
        {#each output as line}
          <p class="py-0.5 {line.includes('[错误]') ? 'text-[var(--danger)]' : line.includes('[系统]') ? 'text-[var(--accent-light)]' : line.includes('完成') ? 'text-[var(--success)]' : 'text-[var(--text-secondary)]'}">
            {line}
          </p>
        {/each}
      </div>
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center">
      <div class="text-center text-[var(--text-muted)]">
        <svg class="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
        <p class="text-sm">点击上方按钮开始更新</p>
      </div>
    </div>
  {/if}
</div>


