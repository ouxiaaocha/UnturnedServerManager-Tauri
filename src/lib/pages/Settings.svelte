<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let steamCmdPath = $state("");
  let serverRoot = $state("");
  let serverId = $state("");
  let rconPort = $state(27115);
  let rconPassword = $state("");
  let rconTimeout = $state(0);
  let saving = $state(false);
  let message = $state("");
  let showPassword = $state(false);

  function generatePassword(): string {
    const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    const arr = new Uint32Array(16);
    crypto.getRandomValues(arr);
    return Array.from(arr, v => chars[v % chars.length]).join("");
  }

  async function loadConfig() {
    try {
      const config: any = await invoke("get_config");
      if (config.servers && config.servers.length > 0) {
        const s = config.servers[0];
        steamCmdPath = s.steamCmdPath || "";
        serverRoot = s.serverRoot || "";
        serverId = s.id || "";
        rconPort = s.rcon?.port || 27115;
        rconPassword = s.rcon?.password || "";
      }
    } catch {}
  }

  function validateServerId(id: string): string | null {
    if (!id.trim()) return "服务器 ID 不能为空";
    if (/[\/\\.]/.test(id)) return "服务器 ID 不能包含路径分隔符";
    if (id.length > 64) return "服务器 ID 过长";
    return null;
  }

  async function save() {
    saving = true;
    message = "";

    const idError = validateServerId(serverId);
    if (idError) {
      message = idError;
      saving = false;
      return;
    }

    try {
      const config = {
        servers: [{
          id: serverId,
          name: `${serverId}服务器`,
          steamCmdPath,
          serverRoot,
          serverEntry: `+InternetServer/${serverId}`,
          rcon: {
            enabled: true,
            host: "127.0.0.1",
            port: rconPort,
            password: rconPassword,
          }
        }]
      };
      await invoke("save_config", { servers: config });
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
    <h1 class="text-2xl font-bold text-white">设置</h1>
    <p class="text-sm text-[var(--text-muted)] mt-1">配置服务器连接参数</p>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-5">
    <!-- Server Configuration -->
    <div>
      <h2 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-3 flex items-center gap-2">
        <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
        </svg>
        服务器配置
      </h2>
      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 space-y-4">
        <div>
          <label class="block text-xs text-[var(--text-muted)] mb-2">SteamCMD 路径</label>
          <input type="text" bind:value={steamCmdPath} placeholder="C:\SteamCMD\steamcmd.exe"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
          <p class="text-xs text-[var(--danger)] mt-1">⚠ 目录不能包含中文字符，否则可能导致服务器无法启动</p>
        </div>
        <div>
          <label class="block text-xs text-[var(--text-muted)] mb-2">服务端目录</label>
          <input type="text" bind:value={serverRoot} placeholder="C:\SteamCMD\steamapps\common\U3DS"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
          <p class="text-xs text-[var(--danger)] mt-1">⚠ 目录不能包含中文字符，否则可能导致服务器无法启动</p>
        </div>
        <div>
          <label class="block text-xs text-[var(--text-muted)] mb-2">服务器 ID</label>
          <input type="text" bind:value={serverId} placeholder="PEI"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
        </div>
      </div>
    </div>

    <!-- RCON Configuration -->
    <div>
      <h2 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-3 flex items-center gap-2">
        <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        RCON 配置
      </h2>
      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 space-y-4">
        <div>
          <label class="block text-xs text-[var(--text-muted)] mb-2">端口</label>
          <input type="number" bind:value={rconPort} min="1024" max="65535"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
        </div>
        <div>
          <label class="block text-xs text-[var(--text-muted)] mb-2">密码</label>
          <div class="relative">
            <input type={showPassword ? "text" : "password"} bind:value={rconPassword} placeholder="输入 RCON 密码"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 pr-20 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
            <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-0.5">
              <button type="button"
                class="p-1.5 rounded hover:bg-[var(--bg-card-hover)] text-[var(--text-muted)] hover:text-white transition-colors cursor-pointer"
                onclick={() => showPassword = !showPassword}
                title={showPassword ? "隐藏密码" : "显示密码"}
              >
                {#if showPassword}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" /></svg>
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
                {/if}
              </button>
              <button type="button"
                class="p-1.5 rounded hover:bg-[var(--bg-card-hover)] text-[var(--text-muted)] hover:text-[var(--accent-light)] transition-colors cursor-pointer"
                onclick={() => rconPassword = generatePassword()}
                title="自动生成密码"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
              </button>
            </div>
          </div>
        </div>
        <div class="flex items-start gap-2 px-3 py-2 rounded-lg bg-[var(--warning-glow)]">
          <svg class="w-4 h-4 text-[var(--warning)] flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
          <span class="text-xs text-[var(--warning)]">密码已加密存储在配置文件中</span>
        </div>
      </div>
    </div>
  </div>

  <!-- RCON Connection -->
  <div>
    <h2 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-3 flex items-center gap-2">
      <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
      </svg>
      RCON 连接
    </h2>
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 space-y-4">
      <div>
        <label class="block text-xs text-[var(--text-muted)] mb-2">自动断开时间（分钟，0 = 不自动断开）</label>
        <input type="number" bind:value={rconTimeout} min="0" max="1440"
          class="w-[200px] bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
      </div>
      <p class="text-xs text-[var(--text-muted)]">设置为 0 表示保持连接直到手动断开。建议设置 30-60 分钟。</p>
    </div>
  </div>

  <!-- Save Button -->
  <div class="flex items-center gap-4">
    <button
      class="px-8 py-3 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-white text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
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
