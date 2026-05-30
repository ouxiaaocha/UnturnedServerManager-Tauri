<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let steamCmdPath = $state("");
  let serverRoot = $state("");
  let saving = $state(false);
  let message = $state("");
  let existingConfig: any = null;

  async function loadConfig() {
    try {
      const config: any = await invoke("get_config");
      if (config.servers && config.servers.length > 0) {
        const s = config.servers[0];
        existingConfig = s;
        steamCmdPath = s.steamCmdPath || "";
        serverRoot = s.serverRoot || "";
      }
    } catch {}
  }

  async function save() {
    saving = true;
    message = "";

    try {
      const server = {
        ...(existingConfig || {}),
        steamCmdPath,
        serverRoot,
      };
      await invoke("save_config", { servers: [server] });
      message = "保存成功";
    } catch (e: any) {
      message = `保存失败: ${e}`;
    }
    saving = false;
  }

  $effect(() => { loadConfig(); });
</script>

<div class="flex flex-col gap-5 h-full overflow-y-auto">
  <div>
    <h1 class="text-2xl font-bold text-[var(--text-primary)]">设置</h1>
    <p class="text-sm text-[var(--text-muted)] mt-1">配置 SteamCMD 与服务端路径</p>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-5">
    <!-- Server Configuration -->
    <div>
      <h2 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-3 flex items-center gap-2">
        <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
        </svg>
        路径配置
      </h2>
      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 space-y-4">
        <div>
          <span class="block text-xs text-[var(--text-muted)] mb-2">SteamCMD 路径</span>
          <input type="text" bind:value={steamCmdPath} placeholder="C:\SteamCMD\steamcmd.exe"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
          <p class="text-xs text-[var(--danger)] mt-1">⚠ 目录不能包含中文字符，否则可能导致服务器无法启动</p>
        </div>
        <div>
          <span class="block text-xs text-[var(--text-muted)] mb-2">服务端目录</span>
          <input type="text" bind:value={serverRoot} placeholder="C:\SteamCMD\steamapps\common\U3DS"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
          <p class="text-xs text-[var(--danger)] mt-1">⚠ 目录不能包含中文字符，否则可能导致服务器无法启动</p>
        </div>
      </div>
    </div>
  </div>

  <!-- Save Button -->
  <div class="flex items-center gap-4">
    <button
      class="px-8 py-3 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
      onclick={save}
      disabled={saving}
    >
      {#if saving}
        <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
        保存中...
      {:else}
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        保存设置
      {/if}
    </button>
    {#if message}
      <span class="text-sm {message.includes('失败') ? 'text-[var(--danger)]' : 'text-[var(--success)]'} flex items-center gap-1">
        {#if message.includes('失败')}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        {/if}
        {message}
      </span>
    {/if}
  </div>
</div>


