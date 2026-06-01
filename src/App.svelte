<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import DashboardPage from "./lib/pages/Dashboard.svelte";
  import ServerPage from "./lib/pages/Server.svelte";
  import RconPage from "./lib/pages/Rcon.svelte";
  import SettingsPage from "./lib/pages/Settings.svelte";
  import LogsPage from "./lib/pages/Logs.svelte";
  import UpdatePage from "./lib/pages/Update.svelte";
  import SchedulePage from "./lib/pages/Schedule.svelte";
  import WizardPage from "./lib/pages/Wizard.svelte";
  import AboutPage from "./lib/pages/About.svelte";
  import SavePage from "./lib/pages/Save.svelte";

  let currentPage = $state("dashboard");
  let showWizard = $state(false);
  let loaded = $state(false);
  let isMaximized = $state(false);

  const navItems = [
    { id: "dashboard", label: "仪表盘", icon: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" },
    { id: "server", label: "服务器", icon: "M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" },
    { id: "rcon", label: "RCON", icon: "M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" },
    { id: "schedule", label: "定时任务", icon: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" },
    { id: "update", label: "更新", icon: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" },
    { id: "logs", label: "日志", icon: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" },
    { id: "save", label: "存档", icon: "M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" },
    { id: "settings", label: "设置", icon: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z" },
  ];

  async function checkFirstRun() {
    try {
      const isFirst = await invoke("is_first_run") as boolean;
      showWizard = isFirst;
    } catch {
      showWizard = true;
    }
    loaded = true;
  }

  function onWizardComplete() {
    showWizard = false;
  }

  function isTauriRuntime() {
    return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  }

  function getAppWindow() {
    return isTauriRuntime() ? getCurrentWindow() : null;
  }

  async function refreshMaximizedState() {
    const appWindow = getAppWindow();
    if (!appWindow) return;
    try {
      isMaximized = await appWindow.isMaximized();
    } catch {
      isMaximized = false;
    }
  }

  // 窗口控制命令仅在 Tauri 运行时可用，浏览器预览中静默忽略

  async function minimizeWindow() {
    try {
      await getAppWindow()?.minimize();
    } catch {}
  }

  async function toggleWindowMaximize() {
    try {
      await getAppWindow()?.toggleMaximize();
      await refreshMaximizedState();
    } catch {}
  }

  async function closeWindow() {
    try {
      await getAppWindow()?.close();
    } catch {}
  }

  async function startWindowDrag(event: MouseEvent) {
    if (event.button !== 0 || event.detail > 1) return;
    try {
      await getAppWindow()?.startDragging();
    } catch {}
  }

  function onTitlebarDoubleClick(event: MouseEvent) {
    if (event.button !== 0) return;
    void toggleWindowMaximize();
  }

  onMount(() => {
    void checkFirstRun();
    void refreshMaximizedState();

    const updateMaximizedState = () => void refreshMaximizedState();
    window.addEventListener("resize", updateMaximizedState);

    return () => {
      window.removeEventListener("resize", updateMaximizedState);
    };
  });
</script>

<div class="flex h-screen flex-col bg-[var(--bg-primary)]">
  <header class="relative z-30 flex h-11 shrink-0 select-none items-center border-b border-[var(--border)] bg-[var(--bg-secondary)]/92 shadow-[var(--shadow-sm)] backdrop-blur">
    <div
      class="flex min-w-0 flex-1 items-center gap-3 self-stretch px-3 sm:px-4"
      role="presentation"
      onpointerdown={startWindowDrag}
      ondblclick={onTitlebarDoubleClick}
    >
      <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-[var(--accent)] to-[var(--action)] shadow-[var(--shadow-glow)]">
        <svg class="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
        </svg>
      </div>
      <div class="min-w-0">
        <p class="truncate text-xs font-semibold text-[var(--text-primary)] sm:text-sm">Unturned 服务器管理工具</p>
        <p class="hidden text-[10px] text-[var(--text-muted)] sm:block">Windows Portable</p>
      </div>
    </div>

    <div class="flex h-full shrink-0 items-stretch">
      <button
        type="button"
        class="flex h-full w-11 items-center justify-center text-[var(--text-secondary)] transition-colors duration-[var(--transition-fast)] hover:bg-[var(--bg-card-hover)] hover:text-[var(--text-primary)]"
        aria-label="最小化窗口"
        title="最小化"
        onclick={minimizeWindow}
      >
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-width="1.8" d="M6 18h12" />
        </svg>
      </button>
      <button
        type="button"
        class="flex h-full w-11 items-center justify-center text-[var(--text-secondary)] transition-colors duration-[var(--transition-fast)] hover:bg-[var(--bg-card-hover)] hover:text-[var(--text-primary)]"
        aria-label={isMaximized ? "还原窗口" : "最大化窗口"}
        title={isMaximized ? "还原" : "最大化"}
        onclick={toggleWindowMaximize}
      >
        {#if isMaximized}
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linejoin="round" stroke-width="1.7" d="M9 9h9v9H9z" />
            <path stroke-linejoin="round" stroke-width="1.7" d="M6 15V6h9" />
          </svg>
        {:else}
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linejoin="round" stroke-width="1.7" d="M7 7h10v10H7z" />
          </svg>
        {/if}
      </button>
      <button
        type="button"
        class="flex h-full w-12 items-center justify-center text-[var(--text-secondary)] transition-colors duration-[var(--transition-fast)] hover:bg-[var(--danger-glow)] hover:text-[var(--danger)]"
        aria-label="关闭窗口"
        title="关闭"
        onclick={closeWindow}
      >
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.9" d="M6 6l12 12M18 6L6 18" />
        </svg>
      </button>
    </div>
  </header>

{#if !loaded}
  <div class="flex min-h-0 flex-1 items-center justify-center bg-[var(--bg-primary)]">
    <div class="flex flex-col items-center gap-4">
      <div class="w-12 h-12 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
      <p class="text-[var(--text-secondary)] text-sm">加载中...</p>
    </div>
  </div>
{:else if showWizard}
  <div class="min-h-0 flex-1 overflow-hidden">
    <WizardPage onComplete={onWizardComplete} />
  </div>
{:else}
  <div class="flex min-h-0 flex-1 flex-col md:flex-row">
    <!-- 侧边栏导航 -->
    <nav class="relative z-10 flex w-full shrink-0 flex-col border-b border-[var(--border)] bg-[var(--bg-secondary)]/95 shadow-[var(--shadow-sm)] backdrop-blur md:h-full md:w-[232px] md:border-b-0 md:border-r">
      <ul class="flex gap-2 overflow-x-auto px-3 py-3 md:mt-3 md:block md:flex-1 md:space-y-1 md:overflow-visible md:py-0">
        {#each navItems as item}
          <li class="shrink-0 md:shrink">
            <button
              class="flex w-full items-center gap-2 rounded-lg border px-3 py-2.5 text-left transition-all duration-[var(--transition-normal)] md:gap-3
                {currentPage === item.id
                  ? 'border-[var(--border-accent)] bg-[var(--accent-subtle)] text-[var(--accent-light)] shadow-[var(--shadow-sm)]'
                  : 'border-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-card-hover)] hover:text-[var(--text-primary)]'}"
              onclick={() => (currentPage = item.id)}
            >
              <svg class="h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={item.icon} />
              </svg>
              <span class="whitespace-nowrap text-sm font-medium">{item.label}</span>
              {#if currentPage === item.id}
                <div class="ml-auto hidden h-1.5 w-1.5 rounded-full bg-[var(--accent)] md:block"></div>
              {/if}
            </button>
          </li>
        {/each}
      </ul>

      <div class="border-t border-[var(--border)] px-3 pb-3 pt-3">
        <button
          class="flex w-full items-center gap-3 rounded-lg border px-3 py-2.5 text-left transition-all duration-[var(--transition-normal)]
            {currentPage === 'about'
              ? 'border-[var(--border-accent)] bg-[var(--accent-subtle)] text-[var(--accent-light)]'
              : 'border-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-card-hover)] hover:text-[var(--text-primary)]'}"
          onclick={() => (currentPage = 'about')}
        >
          <svg class="h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-sm font-medium">关于</span>
          {#if currentPage === 'about'}
            <div class="ml-auto h-1.5 w-1.5 rounded-full bg-[var(--accent)]"></div>
          {/if}
        </button>
        <p class="hidden text-[10px] text-[var(--text-muted)] px-3 mt-2 md:block">v2.0.0</p>
      </div>
    </nav>

    <main class="flex min-h-0 flex-1 flex-col overflow-hidden p-4 sm:p-5 lg:p-6">
      <div class="mx-auto flex min-h-0 w-full max-w-[1280px] flex-1 flex-col">
        {#if currentPage === "dashboard"}
          <DashboardPage />
        {:else if currentPage === "server"}
          <ServerPage />
        {:else if currentPage === "rcon"}
          <RconPage />
        {:else if currentPage === "schedule"}
          <SchedulePage />
        {:else if currentPage === "update"}
          <UpdatePage />
        {:else if currentPage === "logs"}
          <LogsPage />
        {:else if currentPage === "save"}
          <SavePage />
        {:else if currentPage === "settings"}
          <SettingsPage />
        {:else if currentPage === "about"}
          <AboutPage />
        {/if}
      </div>
    </main>
  </div>
{/if}
</div>
