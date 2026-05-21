<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let activeTab = $state("save");
  let saves = $state<any[]>([]);
  let selectedSaveId = $state("");
  let loading = $state(false);
  let saving = $state(false);
  let message = $state("");
  let pluginsLoading = $state(false);

  // Commands.dat fields
  let cmdName = $state("");
  let cmdMap = $state("");
  let cmdPort = $state(27015);
  let cmdMaxPlayers = $state(24);
  let cmdPassword = $state("");
  let cmdOwner = $state("");
  let cmdCheats = $state(false);
  let cmdPve = $state(false);
  let cmdPerspective = $state("Both");
  let cmdGslt = $state("");

  // RCON config
  let rconPort = $state(27115);
  let rconPassword = $state("");
  let showRconPassword = $state(false);

  // Plugins
  let plugins = $state<any[]>([]);
  let pluginNotes = $state<Record<string, string>>({});

  // Race condition guard: discard stale responses when switching saves
  let loadGeneration = 0;
  let msgTimer: ReturnType<typeof setTimeout> | undefined;
  let noteSaveTimer: ReturnType<typeof setTimeout> | undefined;
  let lastRawLines: string[] = [];

  async function loadSaves() {
    try {
      saves = await invoke("list_server_saves");
      if (saves.length > 0 && !selectedSaveId) {
        selectedSaveId = saves[0].id;
        await loadCommandsDat();
      }
    } catch {}
  }

  async function loadCommandsDat() {
    if (!selectedSaveId) return;
    const gen = ++loadGeneration;
    loading = true;
    try {
      const info: any = await invoke("read_commands_dat", { saveId: selectedSaveId });
      if (gen !== loadGeneration) return; // stale response
      cmdName = info.name ?? "";
      cmdMap = info.map ?? "";
      cmdPort = info.port ?? 27015;
      cmdMaxPlayers = info.max_players ?? 24;
      cmdPassword = info.password ?? "";
      cmdOwner = info.owner ?? "";
      cmdCheats = info.cheats ?? false;
      cmdPve = info.pve ?? false;
      cmdPerspective = info.perspective ?? "Both";
      cmdGslt = info.gslt ?? "";
      lastRawLines = info.raw_lines ?? [];
    } catch {}
    // Load RCON config from Rocket.config.xml
    try {
      const rcon: any = await invoke("read_rocket_rcon_config", { saveId: selectedSaveId });
      if (gen !== loadGeneration) return;
      rconPort = rcon.port ?? 27115;
      rconPassword = rcon.password ?? "";
    } catch {}
    if (gen === loadGeneration) loading = false;
  }

  async function saveCommandsDat() {
    saving = true;
    try {
      await invoke("save_commands_dat", {
        saveId: selectedSaveId,
        info: {
          name: cmdName || null,
          map: cmdMap || null,
          port: cmdPort > 0 ? cmdPort : null,
          max_players: cmdMaxPlayers > 0 ? cmdMaxPlayers : null,
          password: cmdPassword || null,
          owner: cmdOwner || null,
          cheats: cmdCheats,
          pve: cmdPve,
          perspective: cmdPerspective || null,
          gslt: cmdGslt || null,
          raw_lines: lastRawLines,
        },
      });
      // Save RCON config to Rocket.config.xml
      await invoke("save_rocket_rcon_config", {
        saveId: selectedSaveId,
        port: rconPort,
        password: rconPassword,
      });
      message = "配置已保存";
      clearTimeout(msgTimer);
      msgTimer = setTimeout(() => message = "", 3000);
    } catch (e: any) {
      alert(e);
    }
    saving = false;
  }

  async function loadPlugins() {
    if (!selectedSaveId) return;
    const gen = loadGeneration;
    pluginsLoading = true;
    try {
      const [p, n] = await Promise.all([
        invoke("list_plugins", { saveId: selectedSaveId }),
        invoke("load_plugin_notes"),
      ]);
      if (gen !== loadGeneration) return;
      plugins = p as any[];
      pluginNotes = n as Record<string, string>;
    } catch {}
    if (gen === loadGeneration) pluginsLoading = false;
  }

  async function openPluginDir() {
    try {
      await invoke("open_plugin_config_dir", { saveId: selectedSaveId });
    } catch (e: any) {
      alert(e);
    }
  }

  function onPluginNoteBlur(pluginName: string, note: string) {
    pluginNotes = { ...pluginNotes, [pluginName]: note };
    clearTimeout(noteSaveTimer);
    noteSaveTimer = setTimeout(async () => {
      try {
        await invoke("save_plugin_notes", { notes: pluginNotes });
      } catch (e: any) {
        alert(e);
      }
    }, 500);
  }

  async function onSaveChange() {
    await loadCommandsDat();
    if (activeTab === "plugins") {
      await loadPlugins();
    }
  }

  async function onTabChange(tab: string) {
    activeTab = tab;
    if (tab === "plugins") {
      await loadPlugins();
    }
  }

  $effect(() => {
    loadSaves();
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-2xl font-bold text-white">存档管理</h1>
      <p class="text-sm text-[var(--text-muted)] mt-1">管理服务器存档配置与插件</p>
    </div>
    {#if message}
      <div class="px-4 py-2 rounded-lg bg-[var(--success-glow)] text-[var(--success)] text-sm">{message}</div>
    {/if}
  </div>

  <!-- Save Selector -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-4 mb-5">
    <div class="flex items-center gap-4">
      <label class="text-sm text-[var(--text-secondary)]">选择存档:</label>
      <select
        bind:value={selectedSaveId}
        onchange={onSaveChange}
        class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-white focus:outline-none focus:border-[var(--accent)] transition-colors cursor-pointer min-w-[200px]"
      >
        {#each saves as save}
          <option value={save.id}>{save.id}{save.name ? ` - ${save.name}` : ''}</option>
        {/each}
      </select>
      {#if saves.length === 0}
        <span class="text-sm text-[var(--text-muted)]">未找到存档目录</span>
      {/if}
    </div>
  </div>

  <!-- Tabs -->
  <div class="flex gap-2 mb-5">
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'save' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-white hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('save')}
    >
      存档配置
    </button>
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'plugins' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-white hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('plugins')}
    >
      插件管理
    </button>
  </div>

  {#if activeTab === 'save'}
    <!-- Commands.dat Editor -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
      <h2 class="text-base font-semibold text-white mb-5 flex items-center gap-2">
        <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
        </svg>
        Commands.dat 配置
      </h2>

      {#if loading}
        <div class="flex items-center justify-center py-10">
          <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else}
        <div class="grid grid-cols-2 gap-5">
          <!-- Name -->
          <div>
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">服务器名称</label>
            <input type="text" bind:value={cmdName} placeholder="My Unturned Server"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Map -->
          <div>
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">地图</label>
            <input type="text" bind:value={cmdMap} placeholder="PEI"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Port -->
          <div>
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">端口</label>
            <input type="number" bind:value={cmdPort} min="1024" max="65535"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- MaxPlayers -->
          <div>
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">最大玩家数</label>
            <input type="number" bind:value={cmdMaxPlayers} min="1" max="200"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Password -->
          <div>
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">服务器密码</label>
            <input type="text" bind:value={cmdPassword} placeholder="留空表示无密码"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Owner -->
          <div>
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">管理员 SteamID64</label>
            <input type="text" bind:value={cmdOwner} placeholder="76561198000000000"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Perspective -->
          <div>
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">视角</label>
            <select bind:value={cmdPerspective}
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white focus:outline-none focus:border-[var(--accent)] transition-colors cursor-pointer">
              <option value="First">第一人称</option>
              <option value="Third">第三人称</option>
              <option value="Both">两者皆可</option>
              <option value="Vehicle">载具</option>
            </select>
          </div>

          <!-- GSLT -->
          <div class="col-span-2">
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">GSLT (Game Server Login Token)</label>
            <input type="text" bind:value={cmdGslt} placeholder="可选，用于在服务器浏览器中显示"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>
        </div>

        <!-- Toggle switches -->
        <div class="flex gap-8 mt-5 pt-5 border-t border-[var(--border)]">
          <label class="flex items-center gap-3 cursor-pointer">
            <button
              class="w-10 h-6 rounded-full transition-colors cursor-pointer {cmdPve ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
              onclick={() => cmdPve = !cmdPve}
            >
              <div class="w-4 h-4 rounded-full bg-white transform transition-transform {cmdPve ? 'translate-x-5' : 'translate-x-1'}"></div>
            </button>
            <span class="text-sm text-[var(--text-secondary)]">PvE 模式</span>
          </label>

          <label class="flex items-center gap-3 cursor-pointer">
            <button
              class="w-10 h-6 rounded-full transition-colors cursor-pointer {cmdCheats ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
              onclick={() => cmdCheats = !cmdCheats}
            >
              <div class="w-4 h-4 rounded-full bg-white transform transition-transform {cmdCheats ? 'translate-x-5' : 'translate-x-1'}"></div>
            </button>
            <span class="text-sm text-[var(--text-secondary)]">启用作弊</span>
          </label>
        </div>

        <div class="mt-6 flex justify-end">
          <button
            class="px-6 py-2.5 bg-gradient-to-r from-[var(--accent)] to-blue-600 hover:from-blue-500 hover:to-[var(--accent)] text-white text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center gap-2 shadow-lg"
            onclick={saveCommandsDat}
            disabled={saving}
          >
            {#if saving}
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              保存中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              保存配置
            {/if}
          </button>
        </div>
      {/if}
    </div>

    <!-- RCON Config -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6 mt-5">
      <h2 class="text-base font-semibold text-white mb-5 flex items-center gap-2">
        <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
        </svg>
        RCON 配置
      </h2>
      {#if loading}
        <div class="flex items-center justify-center py-10">
          <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else}
        <div class="grid grid-cols-2 gap-5">
          <div>
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">RCON 端口</label>
            <input type="number" bind:value={rconPort} min="1024" max="65535"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-white focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>
          <div>
            <label class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">RCON 密码</label>
            <div class="relative">
              <input type={showRconPassword ? "text" : "password"} bind:value={rconPassword} placeholder="输入 RCON 密码"
                class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 pr-20 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              <div class="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
                <button type="button"
                  class="p-1.5 text-[var(--text-muted)] hover:text-white transition-colors cursor-pointer"
                  onclick={() => showRconPassword = !showRconPassword}
                >
                  {#if showRconPassword}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.542 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" /></svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
                  {/if}
                </button>
                <button type="button"
                  class="p-1.5 text-[var(--text-muted)] hover:text-white transition-colors cursor-pointer"
                  onclick={() => {
                    const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
                    const arr = new Uint32Array(16);
                    crypto.getRandomValues(arr);
                    rconPassword = Array.from(arr, v => chars[v % chars.length]).join("");
                  }}
                  title="生成随机密码"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" /></svg>
                </button>
              </div>
            </div>
          </div>
        </div>
        <div class="mt-4 px-4 py-3 rounded-lg bg-[var(--bg-primary)] border border-[var(--border)] text-xs text-[var(--text-muted)]">
          <svg class="w-4 h-4 inline mr-1 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
          每个存档有独立的 RCON 配置，修改后点击下方「保存配置」同步保存。
        </div>
      {/if}
    </div>

  {:else}
    <!-- Plugins Tab -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
      <div class="flex items-center justify-between mb-5">
        <h2 class="text-base font-semibold text-white flex items-center gap-2">
          <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          已安装插件
        </h2>
        <button
          class="px-4 py-2 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm text-[var(--text-secondary)] hover:text-white hover:border-[var(--accent)] transition-all cursor-pointer flex items-center gap-2"
          onclick={openPluginDir}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
          </svg>
          打开插件目录
        </button>
      </div>

      {#if pluginsLoading}
        <div class="flex items-center justify-center py-10">
          <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if plugins.length === 0}
        <div class="text-center py-10">
          <svg class="w-12 h-12 text-[var(--text-muted)] mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
          <p class="text-[var(--text-muted)] text-sm">未找到插件</p>
          <p class="text-[var(--text-muted)] text-xs mt-1">请确认 Rocket 插件已安装到 Rocket/Plugins 目录</p>
        </div>
      {:else}
        <div class="space-y-3">
          {#each plugins as plugin}
            <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4 hover:border-[var(--accent)] transition-all">
              <div class="flex items-center gap-3 mb-3">
                <div class="w-8 h-8 rounded-lg bg-[var(--accent-subtle)] flex items-center justify-center flex-shrink-0">
                  <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                  </svg>
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-medium text-white truncate">{plugin.name}</p>
                  <p class="text-xs text-[var(--text-muted)] truncate">{plugin.file_name}</p>
                </div>
              </div>
              <div>
                <label class="block text-xs text-[var(--text-muted)] mb-1">备注</label>
                <input
                  type="text"
                  value={pluginNotes[plugin.name] || ""}
                  onblur={(e) => onPluginNoteBlur(plugin.name, (e.target as HTMLInputElement).value)}
                  placeholder="添加中文备注..."
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                />
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
