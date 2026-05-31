<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { generatePassword } from "$lib/utils";

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

  // Workshop config
  let workshopConfig = $state<any>(null);
  let workshopLoading = $state(false);
  let workshopSaving = $state(false);
  let workshopModNotes = $state<Record<string, string>>({});
  let newModId = $state("");
  let newModNote = $state("");
  let ignoreChildrenInput = $state("");

  // Save init state
  let showInitPanel = $state(false);
  let newSaveName = $state("Server");
  let initRunning = $state(false);
  let initDone = $state(false);
  let newSaveLogs = $state<string[]>([]);

  // Rocket status for selected save
  let saveHasRocket = $state<boolean | null>(null);
  let rocketInitRunning = $state(false);
  let rocketInitDone = $state(false);
  let rocketInitLogs = $state<string[]>([]);

  // Race condition guard: discard stale responses when switching saves
  let loadGeneration = 0;
  let msgTimer: ReturnType<typeof setTimeout> | undefined;
  let noteSaveTimer: ReturnType<typeof setTimeout> | undefined;
  let lastRawLines: string[] = [];

  async function loadSaves() {
    const gen = ++loadGeneration;
    try {
      saves = await invoke("list_server_saves");
      if (gen !== loadGeneration) return;
      if (saves.length > 0 && !selectedSaveId) {
        selectedSaveId = saves[0].id;
        await loadCommandsDat();
      }
      if (gen !== loadGeneration) return;
      await checkRocketStatus();
    } catch {}
  }

  async function checkRocketStatus() {
    if (!selectedSaveId) {
      saveHasRocket = null;
      return;
    }
    try {
      saveHasRocket = await invoke("check_save_rocket_status", { saveId: selectedSaveId }) as boolean;
    } catch {
      saveHasRocket = false;
    }
  }

  async function initNewSave() {
    if (!newSaveName.trim()) return;
    initRunning = true;
    initDone = false;
    newSaveLogs = [];
    const unlisten = await listen<string>("installer-progress", (event) => {
      const msg = event.payload;
      if (msg.startsWith("DONE:")) {
        initDone = true;
        initRunning = false;
        unlisten();
        loadSaves();
      } else if (msg.startsWith("ERROR:")) {
        alert(`初始化失败: ${msg.slice(6)}`);
        initRunning = false;
        unlisten();
      } else {
        newSaveLogs.push(msg);
        if (newSaveLogs.length > 200) newSaveLogs = newSaveLogs.slice(-100);
      }
    });
    try {
      await invoke("init_server_save", { saveName: newSaveName });
    } catch (e: any) {
      alert(`启动失败: ${e}`);
      initRunning = false;
      unlisten();
    }
  }

  async function initRocketForSave() {
    if (!selectedSaveId) return;
    rocketInitRunning = true;
    rocketInitDone = false;
    rocketInitLogs = [];
    const unlisten = await listen<string>("installer-progress", (event) => {
      const msg = event.payload;
      if (msg.startsWith("DONE:")) {
        rocketInitDone = true;
        rocketInitRunning = false;
        unlisten();
        checkRocketStatus();
      } else if (msg.startsWith("ERROR:")) {
        alert(`初始化失败: ${msg.slice(6)}`);
        rocketInitRunning = false;
        unlisten();
      } else {
        rocketInitLogs.push(msg);
        if (rocketInitLogs.length > 200) rocketInitLogs = rocketInitLogs.slice(-100);
      }
    });
    try {
      await invoke("init_server_save", { saveName: selectedSaveId });
    } catch (e: any) {
      alert(`启动失败: ${e}`);
      rocketInitRunning = false;
      unlisten();
    }
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
    const gen = ++loadGeneration;
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

  async function loadWorkshopConfig() {
    if (!selectedSaveId) return;
    const gen = ++loadGeneration;
    workshopLoading = true;
    try {
      const [wc, mn] = await Promise.all([
        invoke("read_workshop_config", { saveId: selectedSaveId }),
        invoke("load_workshop_mod_notes"),
      ]);
      if (gen !== loadGeneration) return;
      workshopConfig = wc;
      workshopModNotes = mn as Record<string, string>;
      ignoreChildrenInput = ((wc as any).ignore_children_file_ids ?? []).join(", ");
    } catch (e: any) {
      console.error("加载创意工坊配置失败:", e);
      workshopConfig = {
        file_ids: [],
        ignore_children_file_ids: [],
        query_cache_max_age_seconds: 600,
        max_query_retries: 2,
        use_cached_downloads: true,
        should_monitor_updates: true,
        shutdown_update_detected_timer: 600,
        shutdown_update_detected_message: "Workshop file update detected, shutdown in: {0}",
        shutdown_kick_message: "Shutdown for Workshop file update.",
      };
      ignoreChildrenInput = "";
    }
    if (gen === loadGeneration) workshopLoading = false;
  }

  function parseWorkshopIdList(value: string, label: string): number[] {
    if (!value.trim()) return [];
    const parts = value.split(/[,\s]+/).map((s: string) => s.trim()).filter(Boolean);
    const invalid = parts.find((s: string) => !/^\d+$/.test(s));
    if (invalid) {
      throw new Error(`${label} 包含无效 ID: ${invalid}`);
    }
    return Array.from(new Set(parts.map((s: string) => Number.parseInt(s, 10))));
  }

  async function saveWorkshopConfig() {
    if (!selectedSaveId || !workshopConfig) return;
    workshopSaving = true;
    try {
      const normalizedConfig = {
        ...workshopConfig,
        ignore_children_file_ids: parseWorkshopIdList(ignoreChildrenInput, "忽略子项的模组 ID"),
      };
      await invoke("save_workshop_config", {
        saveId: selectedSaveId,
        workshopConfig: normalizedConfig,
      });
      await invoke("save_workshop_mod_notes", { notes: workshopModNotes });
      workshopConfig = normalizedConfig;
      ignoreChildrenInput = normalizedConfig.ignore_children_file_ids.join(", ");
      message = "创意工坊配置已保存";
      clearTimeout(msgTimer);
      msgTimer = setTimeout(() => message = "", 3000);
    } catch (e: any) {
      alert(e);
    }
    workshopSaving = false;
  }

  function addWorkshopMod() {
    if (!newModId.trim() && !newModNote.trim()) {
      alert("请输入创意工坊 ID");
      return;
    }
    if (!newModId.trim()) {
      alert("请输入创意工坊 ID");
      return;
    }
    const ids = newModId.split(/[,\s]+/).map((s: string) => s.trim()).filter((s: string) => s && /^\d+$/.test(s));
    if (ids.length === 0) {
      alert("请输入有效的创意工坊 ID（纯数字）");
      return;
    }

    const currentIds: number[] = workshopConfig?.file_ids || [];
    const existingNotes = { ...workshopModNotes };

    for (const idStr of ids) {
      const id = parseInt(idStr, 10);
      if (!currentIds.includes(id)) {
        currentIds.push(id);
      }
      if (newModNote.trim()) {
        existingNotes[idStr] = newModNote.trim();
      }
    }

    workshopConfig = { ...workshopConfig, file_ids: currentIds };
    workshopModNotes = existingNotes;
    newModId = "";
    newModNote = "";
  }

  function removeWorkshopMod(modId: number) {
    if (!workshopConfig) return;
    workshopConfig = {
      ...workshopConfig,
      file_ids: workshopConfig.file_ids.filter((id: number) => id !== modId),
    };
  }

  function onWorkshopModNoteChange(modId: string, note: string) {
    workshopModNotes = { ...workshopModNotes, [modId]: note };
  }

  async function saveWorkshopModNotes() {
    try {
      await invoke("save_workshop_mod_notes", { notes: workshopModNotes });
    } catch (e: any) {
      alert(e);
    }
  }

  async function openWorkshopUrl() {
    try {
      await invoke("open_url", { url: "https://steamcommunity.com/app/304930/workshop/" });
    } catch (e: any) {
      alert(e);
    }
  }

  async function openModPage(modId: number) {
    try {
      await invoke("open_url", { url: `https://steamcommunity.com/sharedfiles/filedetails/?id=${modId}` });
    } catch (e: any) {
      alert(e);
    }
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
    rocketInitDone = false;
    rocketInitRunning = false;
    await loadCommandsDat();
    await checkRocketStatus();
    if (activeTab === "plugins") {
      await loadPlugins();
    }
    if (activeTab === "workshop") {
      await loadWorkshopConfig();
    }
  }

  async function onTabChange(tab: string) {
    activeTab = tab;
    if (tab === "plugins") {
      await loadPlugins();
    }
    if (tab === "workshop") {
      await loadWorkshopConfig();
    }
  }

  $effect(() => {
    loadSaves();
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="flex flex-wrap items-center justify-between gap-3 mb-6">
    <div>
      <h1 class="text-2xl font-bold text-[var(--text-primary)]">存档管理</h1>
      <p class="text-sm text-[var(--text-muted)] mt-1">管理服务器存档配置与插件</p>
    </div>
  </div>

  {#if message}
    <div class="fixed bottom-5 right-5 z-50 flex max-w-[calc(100vw-2rem)] items-center gap-3 rounded-lg border border-[var(--border-accent)] bg-[var(--bg-card)] px-4 py-3 text-sm text-[var(--success)] shadow-[var(--shadow-lg)]">
      <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--success-glow)]">
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
      </div>
      <div>
        <p class="font-medium">{message}</p>
        <p class="text-xs text-[var(--text-muted)]">Commands.dat 与 RCON 配置已同步</p>
      </div>
    </div>
  {/if}

  <!-- Save Selector -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-4 mb-5">
    <div class="flex items-center gap-4 flex-wrap">
      <span class="text-sm text-[var(--text-secondary)]">选择存档:</span>
      {#if saves.length === 0}
        <span class="text-sm text-[var(--text-muted)]">未找到存档</span>
      {:else}
        <select
          bind:value={selectedSaveId}
          onchange={onSaveChange}
          class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors cursor-pointer min-w-[200px]"
        >
          {#each saves as save}
            <option value={save.id}>{save.id}{save.name ? ` - ${save.name}` : ''}</option>
          {/each}
        </select>
      {/if}
      <button
        class="px-4 py-2 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center gap-2"
        onclick={() => showInitPanel = !showInitPanel}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        新建存档
      </button>
    </div>

    <!-- Init New Save Panel -->
    {#if showInitPanel}
      <div class="mt-4 pt-4 border-t border-[var(--border)]">
        <p class="text-xs text-[var(--text-muted)] mb-3">创建新的服务器存档。服务端会自动启动并生成世界数据和配置文件。</p>
        <div class="flex flex-col items-stretch gap-3 sm:flex-row sm:items-end">
          <div class="flex-1">
            <span class="block text-xs text-[var(--text-muted)] mb-1.5">存档名称</span>
            <input type="text" bind:value={newSaveName} placeholder="Server"
              disabled={initRunning || initDone}
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors disabled:opacity-50" />
            <p class="text-[10px] text-[var(--danger)] mt-1">不能包含中文字符和特殊符号</p>
          </div>
          <button
            class="px-5 py-2 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2 flex-shrink-0"
            onclick={initNewSave} disabled={initRunning || initDone || !newSaveName.trim()}
          >
            {#if initRunning}
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              初始化中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              开始初始化
            {/if}
          </button>
        </div>
        {#if initDone}
          <p class="text-xs text-[var(--success)] mt-2 flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            存档初始化成功！
          </p>
        {/if}
        {#if initRunning && newSaveLogs.length > 0}
          <div class="mt-3 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 max-h-32 overflow-y-auto">
            {#each newSaveLogs as log}
              <p class="text-xs text-[var(--text-secondary)] leading-5 font-mono">{log}</p>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Rocket Status Warning -->
  {#if selectedSaveId && saveHasRocket === false}
    <div class="bg-[var(--warning-glow)] border border-[var(--warning)]/30 rounded-xl p-4 mb-5">
      <div class="flex items-center gap-3 mb-2">
        <svg class="w-5 h-5 text-[var(--warning)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        <span class="text-sm text-[var(--warning)] font-medium">存档 "{selectedSaveId}" 缺少 Rocket 配置</span>
      </div>
      <p class="text-xs text-[var(--text-secondary)] mb-3">RCON 和插件功能需要 Rocket 配置文件。运行一次服务端来自动生成。</p>
      <div class="flex items-center gap-3">
        <button
          class="px-5 py-2 bg-gradient-to-r from-[var(--warning)] to-amber-600 hover:from-amber-500 hover:to-[var(--warning)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={initRocketForSave} disabled={rocketInitRunning || rocketInitDone}
        >
          {#if rocketInitRunning}
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
            初始化中...
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            初始化 Rocket 配置
          {/if}
        </button>
        {#if rocketInitDone}
          <span class="text-xs text-[var(--success)] flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            初始化成功
          </span>
        {/if}
      </div>
      {#if rocketInitRunning && rocketInitLogs.length > 0}
        <div class="mt-3 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 max-h-32 overflow-y-auto">
          {#each rocketInitLogs as log}
            <p class="text-xs text-[var(--text-secondary)] leading-5 font-mono">{log}</p>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Tabs -->
  <div class="flex gap-2 mb-5 flex-wrap">
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'save' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('save')}
    >
      存档配置
    </button>
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'workshop' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('workshop')}
    >
      创意工坊模组
    </button>
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'plugins' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('plugins')}
    >
      插件管理
    </button>
  </div>

  {#if activeTab === 'save'}
    <!-- Commands.dat Editor -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
      <h2 class="text-base font-semibold text-[var(--text-primary)] mb-5 flex items-center gap-2">
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
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 md:gap-5">
          <!-- Name -->
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">服务器名称</span>
            <input type="text" bind:value={cmdName} placeholder="My Unturned Server"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Map -->
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">地图</span>
            <input type="text" bind:value={cmdMap} placeholder="PEI"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Port -->
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">端口</span>
            <input type="number" bind:value={cmdPort} min="1024" max="65535"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- MaxPlayers -->
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">最大玩家数</span>
            <input type="number" bind:value={cmdMaxPlayers} min="1" max="200"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Password -->
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">服务器密码</span>
            <input type="text" bind:value={cmdPassword} placeholder="留空表示无密码"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Owner -->
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">管理员 SteamID64</span>
            <input type="text" bind:value={cmdOwner} placeholder="76561198000000000"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <!-- Perspective -->
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">视角</span>
            <select bind:value={cmdPerspective}
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors cursor-pointer">
              <option value="First">第一人称</option>
              <option value="Third">第三人称</option>
              <option value="Both">两者皆可</option>
              <option value="Vehicle">载具</option>
            </select>
          </div>

          <!-- GSLT -->
          <div class="md:col-span-2">
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">GSLT (Game Server Login Token)</span>
            <input type="text" bind:value={cmdGslt} placeholder="可选，用于在服务器浏览器中显示"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>
        </div>

        <!-- Toggle switches -->
        <div class="flex flex-wrap gap-4 sm:gap-8 mt-5 pt-5 border-t border-[var(--border)]">
          <span class="flex items-center gap-3 cursor-pointer">
            <button
              class="w-10 h-6 rounded-full transition-colors cursor-pointer {cmdPve ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
              onclick={() => cmdPve = !cmdPve}
              aria-label="切换 PvE 模式"
            >
              <div class="w-4 h-4 rounded-full bg-white transform transition-transform {cmdPve ? 'translate-x-5' : 'translate-x-1'}"></div>
            </button>
            <span class="text-sm text-[var(--text-secondary)]">PvE 模式</span>
          </span>

          <span class="flex items-center gap-3 cursor-pointer">
            <button
              class="w-10 h-6 rounded-full transition-colors cursor-pointer {cmdCheats ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
              onclick={() => cmdCheats = !cmdCheats}
              aria-label="切换作弊模式"
            >
              <div class="w-4 h-4 rounded-full bg-white transform transition-transform {cmdCheats ? 'translate-x-5' : 'translate-x-1'}"></div>
            </button>
            <span class="text-sm text-[var(--text-secondary)]">启用作弊</span>
          </span>
        </div>

      {/if}
    </div>

    <!-- RCON Config -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6 mt-5">
      <h2 class="text-base font-semibold text-[var(--text-primary)] mb-5 flex items-center gap-2">
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
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 md:gap-5">
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">RCON 端口</span>
            <input type="number" bind:value={rconPort} min="1024" max="65535"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">RCON 密码</span>
            <div class="relative">
              <input type={showRconPassword ? "text" : "password"} bind:value={rconPassword} placeholder="输入 RCON 密码"
                class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 pr-20 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              <div class="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
                <button type="button"
                  class="p-1.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
                  onclick={() => showRconPassword = !showRconPassword}
                >
                  {#if showRconPassword}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.542 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" /></svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
                  {/if}
                </button>
                <button type="button"
                  class="p-1.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
                  onclick={() => rconPassword = generatePassword()}
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

    <div class="mt-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="text-xs text-[var(--text-muted)]">保存会同时同步 Commands.dat 与当前存档的 RCON 配置。</p>
        {#if message}
          <p class="mt-1 flex items-center gap-1 text-xs font-medium text-[var(--success)]">
            <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            {message}
          </p>
        {/if}
      </div>
      <button
        class="px-6 py-2.5 bg-gradient-to-r from-[var(--accent)] to-blue-600 hover:from-blue-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center justify-center gap-2 shadow-lg disabled:opacity-40 disabled:cursor-not-allowed"
        onclick={saveCommandsDat}
        disabled={saving || loading || !selectedSaveId}
      >
        {#if saving}
          <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          保存中...
        {:else if message}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          已保存
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          保存配置
        {/if}
      </button>
    </div>

  {:else if activeTab === 'workshop'}
    <!-- Workshop Tab -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
      <div class="flex flex-wrap items-center justify-between gap-3 mb-5">
        <h2 class="text-base font-semibold text-[var(--text-primary)] flex items-center gap-2">
          <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          创意工坊模组配置
        </h2>
        <button
          onclick={openWorkshopUrl}
          class="px-4 py-2 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[var(--accent)] transition-all cursor-pointer flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
          打开创意工坊
        </button>
      </div>

      {#if workshopLoading}
        <div class="flex items-center justify-center py-10">
          <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if workshopConfig}
        <!-- Add Mod Section -->
        <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4 mb-5">
          <h3 class="text-sm font-medium text-[var(--text-primary)] mb-3">添加模组</h3>
          <div class="flex flex-col gap-3 sm:flex-row sm:items-end">
            <div class="flex-1">
              <span class="block text-xs text-[var(--text-muted)] mb-1.5">创意工坊 ID</span>
              <input type="text" bind:value={newModId} placeholder="输入 ID，多个用逗号或空格分隔"
                class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
            </div>
            <div class="flex-1">
              <span class="block text-xs text-[var(--text-muted)] mb-1.5">备注（可选）</span>
              <input type="text" bind:value={newModNote} placeholder="添加中文备注..."
                class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
            </div>
            <button
              class="px-5 py-2 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center gap-2 flex-shrink-0"
              onclick={addWorkshopMod}
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              添加
            </button>
          </div>
        </div>

        <!-- Mod List -->
        <div class="mb-5">
          <h3 class="text-sm font-medium text-[var(--text-primary)] mb-3">已添加模组 ({workshopConfig.file_ids.length})</h3>
          {#if workshopConfig.file_ids.length === 0}
            <div class="text-center py-8 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg">
              <svg class="w-10 h-10 text-[var(--text-muted)] mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
              </svg>
              <p class="text-[var(--text-muted)] text-sm">暂无模组</p>
              <p class="text-[var(--text-muted)] text-xs mt-1">在上方输入创意工坊 ID 添加模组</p>
            </div>
          {:else}
            <div class="space-y-2 max-h-[400px] overflow-y-auto pr-1">
              {#each workshopConfig.file_ids as modId, index}
                <div class="flex items-center gap-3 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 hover:border-[var(--accent)] transition-all group">
                  <div class="w-8 h-8 rounded-lg bg-[var(--accent-subtle)] flex items-center justify-center flex-shrink-0">
                    <span class="text-xs font-medium text-[var(--accent-light)]">{index + 1}</span>
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <button
                        onclick={() => openModPage(modId)}
                        class="text-sm font-medium text-[var(--accent-light)] hover:underline cursor-pointer"
                      >
                        {modId}
                      </button>
                    </div>
                    <input
                      type="text"
                      value={workshopModNotes[String(modId)] || ""}
                      onblur={(e) => {
                        onWorkshopModNoteChange(String(modId), (e.target as HTMLInputElement).value);
                        saveWorkshopModNotes();
                      }}
                      placeholder="点击添加备注..."
                      class="w-full bg-transparent border-none text-xs text-[var(--text-muted)] placeholder:text-[var(--text-muted)] focus:outline-none focus:text-[var(--text-primary)] mt-1 p-0"
                    />
                  </div>
                  <button
                    class="p-1.5 text-[var(--text-muted)] hover:text-[var(--danger)] opacity-0 group-hover:opacity-100 transition-all cursor-pointer"
                    onclick={() => removeWorkshopMod(modId)}
                    title="移除模组"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Advanced Settings -->
        <details class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg">
          <summary class="px-4 py-3 cursor-pointer text-sm font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors">
            高级配置
          </summary>
          <div class="px-4 pb-4 pt-2 border-t border-[var(--border)]">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1.5">查询缓存时间（秒）</span>
                <input type="number" bind:value={workshopConfig.query_cache_max_age_seconds} min="0"
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              </div>
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1.5">最大查询重试次数</span>
                <input type="number" bind:value={workshopConfig.max_query_retries} min="0"
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              </div>
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1.5">更新检测关服倒计时（秒）</span>
                <input type="number" bind:value={workshopConfig.shutdown_update_detected_timer} min="0"
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              </div>
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1.5">忽略子项的模组 ID</span>
                <input type="text" bind:value={ignoreChildrenInput} placeholder="多个用逗号分隔"
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              </div>
            </div>

            <div class="flex flex-wrap gap-4 sm:gap-8 mt-4 pt-4 border-t border-[var(--border)]">
              <span class="flex items-center gap-3 cursor-pointer">
                <button
                  class="w-10 h-6 rounded-full transition-colors cursor-pointer {workshopConfig.use_cached_downloads ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
                  onclick={() => workshopConfig = { ...workshopConfig, use_cached_downloads: !workshopConfig.use_cached_downloads }}
                  aria-label="切换使用缓存下载"
                >
                  <div class="w-4 h-4 rounded-full bg-white transform transition-transform {workshopConfig.use_cached_downloads ? 'translate-x-5' : 'translate-x-1'}"></div>
                </button>
                <span class="text-sm text-[var(--text-secondary)]">使用缓存下载</span>
              </span>

              <span class="flex items-center gap-3 cursor-pointer">
                <button
                  class="w-10 h-6 rounded-full transition-colors cursor-pointer {workshopConfig.should_monitor_updates ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
                  onclick={() => workshopConfig = { ...workshopConfig, should_monitor_updates: !workshopConfig.should_monitor_updates }}
                  aria-label="切换监控更新"
                >
                  <div class="w-4 h-4 rounded-full bg-white transform transition-transform {workshopConfig.should_monitor_updates ? 'translate-x-5' : 'translate-x-1'}"></div>
                </button>
                <span class="text-sm text-[var(--text-secondary)]">监控模组更新</span>
              </span>
            </div>

            <div class="mt-4">
              <span class="block text-xs text-[var(--text-muted)] mb-1.5">更新检测关服提示消息</span>
              <input type="text" bind:value={workshopConfig.shutdown_update_detected_message} placeholder="Workshop file update detected, shutdown in: {0}"
                class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
            </div>
            <div class="mt-3">
              <span class="block text-xs text-[var(--text-muted)] mb-1.5">关服踢出消息</span>
              <input type="text" bind:value={workshopConfig.shutdown_kick_message} placeholder="Shutdown for Workshop file update."
                class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
            </div>
          </div>
        </details>

        <!-- Save Button -->
        <div class="mt-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <p class="text-xs text-[var(--text-muted)]">配置将保存到 WorkshopDownloadConfig.json</p>
          <button
            class="px-6 py-2.5 bg-gradient-to-r from-[var(--accent)] to-blue-600 hover:from-blue-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center justify-center gap-2 shadow-lg disabled:opacity-40 disabled:cursor-not-allowed"
            onclick={saveWorkshopConfig}
            disabled={workshopSaving || workshopLoading || !selectedSaveId}
          >
            {#if workshopSaving}
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              保存中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              保存创意工坊配置
            {/if}
          </button>
        </div>
      {/if}
    </div>

  {:else}
    <!-- Plugins Tab -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
      <div class="flex flex-wrap items-center justify-between gap-3 mb-5">
        <h2 class="text-base font-semibold text-[var(--text-primary)] flex items-center gap-2">
          <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          已安装插件
        </h2>
        <button
          class="px-4 py-2 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[var(--accent)] transition-all cursor-pointer flex items-center gap-2"
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
                  <p class="text-sm font-medium text-[var(--text-primary)] truncate">{plugin.name}</p>
                  <p class="text-xs text-[var(--text-muted)] truncate">{plugin.file_name}</p>
                </div>
              </div>
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1">备注</span>
                <input
                  type="text"
                  value={pluginNotes[plugin.name] || ""}
                  onblur={(e) => onPluginNoteBlur(plugin.name, (e.target as HTMLInputElement).value)}
                  placeholder="添加中文备注..."
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                />
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>



