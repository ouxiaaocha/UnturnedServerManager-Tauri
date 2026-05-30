<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { generatePassword } from "$lib/utils";

  let step = $state(0);
  let steamCmdPath = $state("");
  let serverRoot = $state("");
  let serverId = $state("PEI");
  let rconPort = $state(27115);
  let rconPassword = $state("changeme");
  let error = $state("");
  let saving = $state(false);
  let detecting = $state(false);
  let showPassword = $state(false);

  // Per-save RCON config for step 5
  let configSaveId = $state("");
  let rconConfigMap = $state<Record<string, { port: number; password: string }>>({});

  // Auto-download state
  let downloadingS = $state(false);
  let downloadMsg = $state("");
  let downloadLogs = $state<string[]>([]);

  // Rocket module state
  let rocketInstalled = $state<boolean | null>(null);
  let rocketChecking = $state(false);
  let rocketInstalling = $state(false);

  // Save init state
  let saveName = $state("Server");
  let saveInitRunning = $state(false);
  let saveInitDone = $state(false);

  // Existing saves detection
  let existingSaves = $state<any[]>([]);
  let selectedSaveId = $state("");
  let saveHasRocket = $state<boolean | null>(null);
  let saveChecking = $state(false);

  // Retry tracking
  let lastFailedAction = $state("");

  function appendLog(msg: string) {
    downloadLogs.push(msg);
    if (downloadLogs.length > 200) {
      downloadLogs = downloadLogs.slice(-100);
    }
  }

  const { onComplete }: { onComplete: () => void } = $props();

  const chineseWarning = "⚠ 目录不能包含中文字符，否则可能导致服务器无法启动";

  async function autoDetect() {
    detecting = true;
    error = "";
    try {
      const result: any = await invoke("auto_detect_paths");
      if (result.steam_cmd_path) steamCmdPath = result.steam_cmd_path;
      if (result.server_root) serverRoot = result.server_root;
      if (result.server_id) serverId = result.server_id;
      if (!result.steam_cmd_path) error = "未能自动检测到 SteamCMD，请手动选择或自动下载";
    } catch (e: any) {
      error = `检测失败: ${e}`;
    }
    detecting = false;
  }

  async function browseSteamCmd() {
    const selected = await open({ title: "选择 SteamCMD 所在文件夹", directory: true });
    if (selected) {
      const dir = selected as string;
      steamCmdPath = dir.endsWith("steamcmd.exe") ? dir : `${dir}\\steamcmd.exe`;
    }
  }

  async function browseServerRoot() {
    const selected = await open({
      title: "选择 Unturned 服务端目录（包含 Unturned.exe）",
      directory: true,
    });
    if (selected) serverRoot = selected as string;
  }

  async function autoDownloadSteamcmd() {
    downloadingS = true;
    downloadLogs = [];
    downloadMsg = "正在下载...";
    error = "";
    lastFailedAction = "";
    const unlisten = await listen<string>("installer-progress", (event) => {
      const msg = event.payload;
      if (msg.startsWith("DONE:")) {
        steamCmdPath = msg.slice(5);
        downloadMsg = "下载完成！";
        unlisten();
        downloadingS = false;
      } else if (msg.startsWith("ERROR:")) {
        error = msg.slice(6);
        downloadMsg = "下载失败";
        lastFailedAction = "steamcmd";
        unlisten();
        downloadingS = false;
      } else {
        appendLog(msg);
        downloadMsg = msg;
      }
    });
    try {
      await invoke("download_steamcmd");
    } catch (e: any) {
      error = `启动失败: ${e}`;
      downloadMsg = "启动失败";
      lastFailedAction = "steamcmd";
      unlisten();
      downloadingS = false;
    }
  }

  async function autoDownloadServer() {
    downloadingS = true;
    downloadLogs = [];
    downloadMsg = "正在下载...";
    error = "";
    lastFailedAction = "";
    const unlisten = await listen<string>("installer-progress", (event) => {
      const msg = event.payload;
      if (msg.startsWith("DONE:")) {
        serverRoot = msg.slice(5);
        downloadMsg = "下载完成！";
        unlisten();
        downloadingS = false;
      } else if (msg.startsWith("ERROR:")) {
        error = msg.slice(6);
        downloadMsg = "下载失败";
        lastFailedAction = "server";
        unlisten();
        downloadingS = false;
      } else {
        appendLog(msg);
        downloadMsg = msg;
      }
    });
    try {
      await invoke("download_server", { steamcmdPath: steamCmdPath });
    } catch (e: any) {
      error = `启动失败: ${e}`;
      downloadMsg = "启动失败";
      lastFailedAction = "server";
      unlisten();
      downloadingS = false;
    }
  }

  async function checkExistingSaves() {
    saveChecking = true;
    error = "";
    try {
      existingSaves = await invoke("list_server_saves", { serverRoot }) as any[];
      if (existingSaves.length > 0) {
        selectedSaveId = existingSaves[0].id;
        await checkSelectedSaveRocket();
      } else {
        saveHasRocket = null;
      }
    } catch (e: any) {
      error = `检测存档失败: ${e}`;
    }
    saveChecking = false;
  }

  async function checkSelectedSaveRocket() {
    if (!selectedSaveId || !serverRoot) {
      saveHasRocket = null;
      return;
    }
    try {
      saveHasRocket = await invoke("check_save_rocket_status", { serverRoot, saveId: selectedSaveId }) as boolean;
    } catch {
      saveHasRocket = false;
    }
  }

  async function checkRocketModule() {
    rocketChecking = true;
    error = "";
    try {
      rocketInstalled = await invoke("detect_rocket_module", { serverRoot }) as boolean;
    } catch (e: any) {
      error = `检测失败: ${e}`;
      rocketInstalled = null;
    }
    rocketChecking = false;
  }

  async function installRocketModule() {
    rocketInstalling = true;
    downloadLogs = [];
    downloadMsg = "正在安装...";
    error = "";
    lastFailedAction = "";
    const unlisten = await listen<string>("installer-progress", (event) => {
      const msg = event.payload;
      if (msg.startsWith("DONE:")) {
        rocketInstalled = true;
        rocketInstalling = false;
        unlisten();
      } else if (msg.startsWith("ERROR:")) {
        error = msg.slice(6);
        lastFailedAction = "rocket";
        rocketInstalling = false;
        unlisten();
      } else {
        appendLog(msg);
        downloadMsg = msg;
      }
    });
    try {
      await invoke("install_rocket_module", { serverRoot });
    } catch (e: any) {
      error = `启动失败: ${e}`;
      lastFailedAction = "rocket";
      rocketInstalling = false;
      unlisten();
    }
  }

  async function startSaveInit() {
    saveInitRunning = true;
    saveInitDone = false;
    downloadLogs = [];
    downloadMsg = "正在初始化...";
    error = "";
    lastFailedAction = "";
    const unlisten = await listen<string>("installer-progress", (event) => {
      const msg = event.payload;
      if (msg.startsWith("DONE:")) {
        saveInitDone = true;
        serverId = msg.slice(5);
        saveInitRunning = false;
        unlisten();
        // Re-check saves and Rocket status after successful init
        checkExistingSaves();
      } else if (msg.startsWith("ERROR:")) {
        error = msg.slice(6);
        lastFailedAction = "save";
        saveInitRunning = false;
        unlisten();
      } else {
        appendLog(msg);
        downloadMsg = msg;
      }
    });
    try {
      // Use selectedSaveId if initializing Rocket for existing save, otherwise use saveName
      const initName = existingSaves.length > 0 && selectedSaveId ? selectedSaveId : saveName;
      await invoke("init_server_save", { serverRoot, saveName: initName });
    } catch (e: any) {
      error = `启动失败: ${e}`;
      lastFailedAction = "save";
      saveInitRunning = false;
      unlisten();
    }
  }

  function retryLastAction() {
    switch (lastFailedAction) {
      case "steamcmd": autoDownloadSteamcmd(); break;
      case "server": autoDownloadServer(); break;
      case "rocket": installRocketModule(); break;
      case "save": startSaveInit(); break;
    }
  }

  function validateStep(): boolean {
    error = "";
    switch (step) {
      case 1:
        if (!steamCmdPath.trim()) { error = "请指定 SteamCMD 路径"; return false; }
        return true;
      case 2:
        if (!serverRoot.trim()) { error = "请指定服务端目录"; return false; }
        return true;
      case 3:
        if (!serverId.trim()) { error = "请输入存档名称"; return false; }
        if (/[\/\\.]/.test(serverId)) { error = "存档名称不能包含路径分隔符"; return false; }
        if (/[一-鿿]/.test(serverId)) { error = "存档名称不能包含中文字符"; return false; }
        if (serverId.length > 64) { error = "存档名称过长"; return false; }
        return true;
      case 5:
        for (const [id, rcon] of Object.entries(rconConfigMap)) {
          if (!rcon.password.trim()) { error = `存档 "${id}" 的 RCON 密码不能为空`; return false; }
          if (rcon.port < 1024 || rcon.port > 65535) { error = `存档 "${id}" 的端口范围: 1024-65535`; return false; }
        }
        return true;
      default: return true;
    }
  }

  function initStep5Rcon() {
    if (existingSaves.length > 0) {
      configSaveId = selectedSaveId || existingSaves[0].id;
      for (const save of existingSaves) {
        if (!rconConfigMap[save.id]) {
          rconConfigMap[save.id] = { port: 27115, password: generatePassword() };
        }
      }
    } else {
      configSaveId = serverId || "Server";
      if (!rconConfigMap[configSaveId]) {
        rconConfigMap[configSaveId] = { port: 27115, password: generatePassword() };
      }
    }
  }

  function next() {
    if (validateStep()) {
      step++;
      if (step === 3) checkRocketModule();
      if (step === 4) checkExistingSaves();
      if (step === 5) initStep5Rcon();
    }
  }
  function prev() { error = ""; step--; }

  async function finish() {
    if (!validateStep()) return;
    saving = true;
    try {
      // Save global config with the first save's RCON as default
      const firstId = existingSaves.length > 0 ? existingSaves[0].id : serverId;
      const firstRcon = rconConfigMap[firstId] || { port: 27115, password: "changeme" };
      await invoke("save_wizard_config", {
        steamCmdPath, serverRoot,
        serverId: firstId,
        rconPort: firstRcon.port,
        rconPassword: firstRcon.password,
      });

      // Save RCON config for each additional save
      for (const save of existingSaves) {
        const rcon = rconConfigMap[save.id];
        if (rcon && save.id !== firstId) {
          try {
            await invoke("save_rocket_rcon_config", {
              saveId: save.id,
              port: rcon.port,
              password: rcon.password,
            });
          } catch { /* non-fatal */ }
        }
      }

      onComplete();
    } catch (e: any) {
      error = `保存失败: ${e}`;
    }
    saving = false;
  }

  const stepLabels = ["欢迎", "SteamCMD", "服务端", "Rocket", "存档", "RCON"];
</script>

<div class="flex h-full items-center justify-center bg-[var(--bg-primary)] p-4">
  <div class="w-full max-w-[680px] bg-[var(--bg-card)] border border-[var(--border)] rounded-2xl p-5 shadow-[var(--shadow-lg)] max-h-[92vh] overflow-y-auto sm:p-8">
    <!-- Step Indicator -->
    <div class="mb-8 grid grid-cols-3 justify-items-center gap-4 sm:mb-10 sm:flex sm:items-start sm:justify-center sm:gap-3">
      {#each [0, 1, 2, 3, 4, 5] as i}
        <div class="flex items-center gap-3">
          <div class="flex flex-col items-center gap-1.5">
            <div class="w-8 h-8 rounded-full transition-all duration-[var(--transition-slow)] flex items-center justify-center text-xs font-semibold
              {i === step ? 'bg-[var(--accent)] text-[var(--text-primary)] shadow-lg shadow-[var(--accent-glow)]' : i < step ? 'bg-[var(--success)] text-[var(--text-primary)]' : 'bg-[var(--bg-elevated)] text-[var(--text-muted)] border border-[var(--border)]'}">
              {#if i < step}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                </svg>
              {:else}{i + 1}{/if}
            </div>
            <span class="text-[10px] {i === step ? 'text-[var(--accent-light)]' : 'text-[var(--text-muted)]'}">{stepLabels[i]}</span>
          </div>
          {#if i < 5}
            <div class="hidden sm:block w-12 h-0.5 {i < step ? 'bg-[var(--success)]' : 'bg-[var(--border)]'} transition-colors duration-[var(--transition-slow)] mb-5"></div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Step 0: Welcome -->
    {#if step === 0}
      <div class="text-center py-4">
        <div class="w-16 h-16 mx-auto mb-5 rounded-2xl bg-gradient-to-br from-[var(--accent)] to-[var(--action)] flex items-center justify-center shadow-lg shadow-[var(--accent-glow)]">
          <svg class="w-8 h-8 text-[var(--text-primary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
          </svg>
        </div>
        <h2 class="text-xl font-bold text-[var(--text-primary)] mb-2">欢迎使用</h2>
        <p class="text-sm text-[var(--text-secondary)] mb-1">Unturned 服务器管理工具</p>
        <p class="text-xs text-[var(--text-muted)] mt-6 leading-6">
          首次运行需要配置 SteamCMD 和服务端路径。<br/>
          如果没有 SteamCMD 和服务端，可使用自动下载功能一键安装。
        </p>
      </div>

    <!-- Step 1: SteamCMD -->
    {:else if step === 1}
      <div>
        <h2 class="text-lg font-bold text-[var(--text-primary)] mb-2">SteamCMD 路径</h2>
        <p class="text-xs text-[var(--text-muted)] mb-5">请指定 steamcmd.exe 路径。如果没有安装，可使用自动下载功能。</p>
        <input type="text" bind:value={steamCmdPath}
          placeholder="C:\SteamCMD\steamcmd.exe" readonly
          class="w-full bg-[var(--bg-primary)] border rounded-lg px-4 py-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] mb-2 focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)] {steamCmdPath ? 'border-[var(--success)]' : 'border-[var(--border)]'}" />
        {#if steamCmdPath}
          <p class="text-xs text-[var(--success)] mb-2 flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            已选择 SteamCMD 路径
          </p>
        {:else}
          <p class="text-xs text-[var(--danger)] mb-2">{chineseWarning}</p>
        {/if}
        <div class="flex flex-col gap-3 mb-4 sm:flex-row sm:flex-wrap">
          <button class="px-5 py-2.5 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center gap-2"
            onclick={browseSteamCmd}>
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>浏览文件夹
          </button>
          <button class="px-5 py-2.5 bg-[var(--bg-elevated)] border border-[var(--border)] text-[var(--text-secondary)] text-sm rounded-lg transition-all hover:text-[var(--text-primary)] hover:border-[var(--accent)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
            onclick={autoDetect} disabled={detecting}>
            {#if detecting}
              <div class="w-4 h-4 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>检测中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>自动检测
            {/if}
          </button>
          <button class="px-5 py-2.5 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
            onclick={autoDownloadSteamcmd} disabled={downloadingS}>
            {#if downloadingS}
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>下载中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>自动下载
            {/if}
          </button>
        </div>
        {#if downloadingS && downloadLogs.length > 0}
          <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 max-h-40 overflow-y-auto">
            <p class="text-xs text-[var(--text-muted)] mb-2">{downloadMsg}</p>
            {#each downloadLogs as log}
              <p class="text-xs text-[var(--text-secondary)] leading-5 font-mono">{log}</p>
            {/each}
          </div>
        {/if}
      </div>

    <!-- Step 2: Server Root -->
    {:else if step === 2}
      <div>
        <h2 class="text-lg font-bold text-[var(--text-primary)] mb-2">服务端目录</h2>
        <p class="text-xs text-[var(--text-muted)] mb-5">请指定 Unturned 服务端根目录（包含 Unturned.exe）。如果没有安装，可使用自动下载功能。</p>
        <input type="text" bind:value={serverRoot}
          placeholder="C:\SteamCMD\steamapps\common\U3DS" readonly
          class="w-full bg-[var(--bg-primary)] border rounded-lg px-4 py-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] mb-2 focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)] {serverRoot ? 'border-[var(--success)]' : 'border-[var(--border)]'}" />
        {#if serverRoot}
          <p class="text-xs text-[var(--success)] mb-2 flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            已选择服务端目录
          </p>
        {:else}
          <p class="text-xs text-[var(--danger)] mb-2">{chineseWarning}</p>
        {/if}
        <div class="flex flex-col gap-3 mb-4 sm:flex-row sm:flex-wrap">
          <button class="px-5 py-2.5 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center gap-2"
            onclick={browseServerRoot}>
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>浏览文件夹
          </button>
          <button class="px-5 py-2.5 bg-[var(--bg-elevated)] border border-[var(--border)] text-[var(--text-secondary)] text-sm rounded-lg transition-all hover:text-[var(--text-primary)] hover:border-[var(--accent)] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
            onclick={autoDetect} disabled={detecting}>
            {#if detecting}
              <div class="w-4 h-4 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>检测中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>自动检测
            {/if}
          </button>
          <button class="px-5 py-2.5 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
            onclick={autoDownloadServer} disabled={downloadingS || !steamCmdPath.trim()}>
            {#if downloadingS}
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>下载中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>自动下载
            {/if}
          </button>
        </div>
        {#if !steamCmdPath.trim()}
          <p class="text-xs text-[var(--warning)]">请先在 Step 1 配置 SteamCMD 路径</p>
        {/if}
        {#if downloadingS && downloadLogs.length > 0}
          <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 max-h-40 overflow-y-auto">
            <p class="text-xs text-[var(--text-muted)] mb-2">{downloadMsg}</p>
            {#each downloadLogs as log}
              <p class="text-xs text-[var(--text-secondary)] leading-5 font-mono">{log}</p>
            {/each}
          </div>
        {/if}
      </div>

    <!-- Step 5: Server Config -->
    {:else if step === 5}
      <div>
        <h2 class="text-lg font-bold text-[var(--text-primary)] mb-2">RCON 配置</h2>
        <p class="text-xs text-[var(--text-muted)] mb-5">为每个存档配置独立的 RCON 远程控制端口和密码。RCON 用于远程管理服务器。</p>

        <!-- Save Selector (when multiple saves exist) -->
        {#if existingSaves.length > 1}
          <div class="mb-4">
            <span class="block text-xs text-[var(--text-muted)] mb-2">选择存档</span>
            <select bind:value={configSaveId}
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-3 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors cursor-pointer">
              {#each existingSaves as save}
                <option value={save.id}>{save.id}{save.name ? ` - ${save.name}` : ''}</option>
              {/each}
            </select>
          </div>
        {/if}

        <!-- RCON Config for selected save -->
        <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4 space-y-4">
          <p class="text-xs font-medium text-[var(--accent-light)] flex items-center gap-2">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
            </svg>
            存档: {configSaveId}
          </p>
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <div>
              <span class="block text-xs text-[var(--text-muted)] mb-2">RCON 端口</span>
              <input type="number" value={rconConfigMap[configSaveId]?.port ?? 27115}
                oninput={(e) => {
                  const val = parseInt((e.target as HTMLInputElement).value) || 27115;
                  if (rconConfigMap[configSaveId]) rconConfigMap[configSaveId].port = val;
                }}
                min="1024" max="65535"
                class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-3 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
            </div>
            <div>
              <span class="block text-xs text-[var(--text-muted)] mb-2">RCON 密码</span>
              <div class="relative">
                <input type={showPassword ? "text" : "password"}
                  value={rconConfigMap[configSaveId]?.password ?? ""}
                  oninput={(e) => {
                    const val = (e.target as HTMLInputElement).value;
                    if (rconConfigMap[configSaveId]) rconConfigMap[configSaveId].password = val;
                  }}
                  placeholder="密码"
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-3 pr-20 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
                <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-0.5">
                  <button type="button"
                    class="p-1.5 rounded hover:bg-[var(--bg-card-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
                    onclick={() => showPassword = !showPassword}
                    title={showPassword ? "隐藏密码" : "显示密码"}>
                    {#if showPassword}
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" /></svg>
                    {:else}
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
                    {/if}
                  </button>
                  <button type="button"
                    class="p-1.5 rounded hover:bg-[var(--bg-card-hover)] text-[var(--text-muted)] hover:text-[var(--accent-light)] transition-colors cursor-pointer"
                    onclick={() => {
                      if (rconConfigMap[configSaveId]) rconConfigMap[configSaveId].password = generatePassword();
                    }} title="自动生成密码">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        {#if existingSaves.length > 1}
          <p class="text-xs text-[var(--text-muted)] mt-3 flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            切换下拉选项可为不同存档配置不同的 RCON 端口和密码
          </p>
        {/if}
      </div>

    <!-- Step 3: Rocket Module -->
    {:else if step === 3}
      <div>
        <h2 class="text-lg font-bold text-[var(--text-primary)] mb-2">Rocket 插件框架</h2>
        <p class="text-xs text-[var(--text-muted)] mb-5">Rocket.Unturned 是服务器的插件框架，支持安装各种插件扩展功能。</p>

        <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4 mb-4">
          <div class="flex items-center gap-3">
            {#if rocketChecking}
              <div class="w-5 h-5 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
              <span class="text-sm text-[var(--text-secondary)]">正在检测...</span>
            {:else if rocketInstalled === true}
              <svg class="w-5 h-5 text-[var(--success)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span class="text-sm text-[var(--success)] font-medium">Rocket.Unturned 已安装</span>
            {:else if rocketInstalled === false}
              <svg class="w-5 h-5 text-[var(--warning)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
              <span class="text-sm text-[var(--warning)]">未检测到 Rocket.Unturned</span>
            {:else}
              <span class="text-sm text-[var(--text-muted)]">尚未检测</span>
            {/if}
          </div>
        </div>

        {#if rocketInstalled === false}
          <div class="mb-4">
            <p class="text-xs text-[var(--text-secondary)] mb-3">Rocket.Unturned 随服务端自带，只需从 Extras 目录复制到 Modules 目录即可。</p>
            <button class="px-5 py-2.5 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
              onclick={installRocketModule} disabled={rocketInstalling}>
              {#if rocketInstalling}
                <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>安装中...
              {:else}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                </svg>自动安装 Rocket
              {/if}
            </button>
          </div>
        {/if}

        {#if rocketInstalling && downloadLogs.length > 0}
          <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 max-h-32 overflow-y-auto">
            {#each downloadLogs as log}
              <p class="text-xs text-[var(--text-secondary)] leading-5 font-mono">{log}</p>
            {/each}
          </div>
        {/if}
      </div>

    <!-- Step 4: Save Initialization -->
    {:else if step === 4}
      <div>
        <h2 class="text-lg font-bold text-[var(--text-primary)] mb-2">存档管理</h2>
        <p class="text-xs text-[var(--text-muted)] mb-5">检测并管理服务器存档。存档用于保存游戏世界数据，RCON 需要存档中的 Rocket 配置才能正常工作。</p>

        {#if saveChecking}
          <div class="flex items-center gap-3 py-6 justify-center">
            <div class="w-5 h-5 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
            <span class="text-sm text-[var(--text-secondary)]">正在检测存档...</span>
          </div>

        {:else if existingSaves.length === 0 && !saveInitDone}
          <!-- No saves found -->
          <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4 mb-4">
            <div class="flex items-center gap-3 mb-3">
              <svg class="w-5 h-5 text-[var(--warning)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
              <span class="text-sm text-[var(--warning)] font-medium">未检测到任何存档</span>
            </div>
            <p class="text-xs text-[var(--text-secondary)] mb-3">首次使用需要初始化一个存档，服务端会自动创建世界数据和配置文件。</p>
          </div>

          <div class="mb-4">
            <span class="block text-xs text-[var(--text-muted)] mb-2">存档名称</span>
            <input type="text" bind:value={saveName} placeholder="Server"
              disabled={saveInitRunning || saveInitDone}
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors disabled:opacity-50" />
            <p class="text-xs text-[var(--danger)] mt-1">⚠ 不能包含中文字符和特殊符号</p>
          </div>

          <div class="flex flex-col gap-3 mb-4 sm:flex-row">
            <button class="px-5 py-2.5 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
              onclick={startSaveInit} disabled={saveInitRunning || saveInitDone || !saveName.trim()}>
              {#if saveInitRunning}
                <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>初始化中...
              {:else}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>开始初始化
              {/if}
            </button>
          </div>

        {:else}
          <!-- Saves found -->
          <div class="bg-[var(--success-glow)] border border-[var(--success)]/30 rounded-lg p-4 mb-4">
            <div class="flex items-center gap-3">
              <svg class="w-5 h-5 text-[var(--success)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span class="text-sm text-[var(--success)] font-medium">检测到 {existingSaves.length} 个存档</span>
            </div>
          </div>

          {#if existingSaves.length > 1}
            <div class="mb-4">
              <span class="block text-xs text-[var(--text-muted)] mb-2">选择存档</span>
              <select bind:value={selectedSaveId} onchange={checkSelectedSaveRocket}
                class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-3 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors cursor-pointer">
                {#each existingSaves as save}
                  <option value={save.id}>{save.id}{save.name ? ` - ${save.name}` : ''}</option>
                {/each}
              </select>
            </div>
          {/if}

          <!-- Rocket status for selected save -->
          {#if saveHasRocket === true}
            <div class="bg-[var(--success-glow)] border border-[var(--success)]/30 rounded-lg p-4 mb-4">
              <div class="flex items-center gap-3">
                <svg class="w-5 h-5 text-[var(--success)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span class="text-sm text-[var(--success)] font-medium">存档 "{selectedSaveId}" 的 Rocket 配置已就绪</span>
              </div>
            </div>
          {:else if saveHasRocket === false}
            <div class="bg-[var(--warning-glow)] border border-[var(--warning)]/30 rounded-lg p-4 mb-4">
              <div class="flex items-center gap-3 mb-3">
                <svg class="w-5 h-5 text-[var(--warning)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                </svg>
                <span class="text-sm text-[var(--warning)] font-medium">存档 "{selectedSaveId}" 缺少 Rocket 配置</span>
              </div>
              <p class="text-xs text-[var(--text-secondary)] mb-3">RCON 功能需要 Rocket 配置文件。需要运行一次服务端来自动生成。</p>
              <button class="px-5 py-2.5 bg-gradient-to-r from-[var(--warning)] to-amber-600 hover:from-amber-500 hover:to-[var(--warning)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
                onclick={startSaveInit} disabled={saveInitRunning || saveInitDone}>
                {#if saveInitRunning}
                  <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>初始化中...
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>初始化 Rocket 配置
                {/if}
              </button>
            </div>
          {/if}
        {/if}

        {#if saveInitDone}
          <div class="bg-[var(--success-glow)] border border-[var(--success)]/30 rounded-lg p-3 mb-4">
            <div class="flex items-center justify-between gap-2">
              <p class="text-sm text-[var(--success)]">存档初始化成功！点击"完成"保存配置。</p>
              <button class="px-3 py-1 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] rounded hover:bg-[var(--bg-card-hover)] transition-colors cursor-pointer flex-shrink-0"
                onclick={() => { saveInitDone = false; }}>
                重新初始化
              </button>
            </div>
          </div>
        {/if}

        {#if saveInitRunning && downloadLogs.length > 0}
          <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 max-h-40 overflow-y-auto">
            <p class="text-xs text-[var(--text-muted)] mb-2">{downloadMsg}</p>
            {#each downloadLogs as log}
              <p class="text-xs text-[var(--text-secondary)] leading-5 font-mono">{log}</p>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Error Message -->
    {#if error}
      <div class="mt-4 px-3 py-2 rounded-lg bg-[var(--danger-glow)] flex items-center gap-2">
        <svg class="w-4 h-4 text-[var(--danger)] flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="text-xs text-[var(--danger)] flex-1">{error}</p>
        {#if lastFailedAction}
          <button class="px-3 py-1 text-xs bg-[var(--danger)] hover:bg-red-600 text-[var(--text-primary)] rounded transition-colors cursor-pointer flex-shrink-0"
            onclick={retryLastAction}>
            重试
          </button>
        {/if}
      </div>
    {/if}

    <!-- Navigation -->
    <div class="flex flex-wrap justify-between gap-3 mt-8">
      <button
        class="px-5 py-2.5 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors disabled:invisible cursor-pointer flex items-center gap-1"
        onclick={prev} disabled={step === 0}>
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 17l-5-5m0 0l5-5m-5 5h12" />
        </svg>上一步
      </button>
      {#if step < 5}
        <button class="px-6 py-2.5 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center gap-2"
          onclick={next}>
          {step === 0 ? '开始配置' : '下一步'}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
          </svg>
        </button>
      {:else}
        <button class="px-6 py-2.5 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2 shadow-lg shadow-[var(--success-glow)]"
          onclick={finish} disabled={saving}>
          {#if saving}
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>保存中...
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>完成
          {/if}
        </button>
      {/if}
    </div>
  </div>
</div>



