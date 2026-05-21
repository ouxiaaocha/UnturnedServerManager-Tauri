<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
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

  $effect(() => { checkFirstRun(); });
</script>

{#if !loaded}
  <div class="flex items-center justify-center h-screen bg-[var(--bg-primary)]">
    <div class="flex flex-col items-center gap-4">
      <div class="w-12 h-12 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
      <p class="text-[var(--text-secondary)] text-sm">加载中...</p>
    </div>
  </div>
{:else if showWizard}
  <WizardPage onComplete={onWizardComplete} />
{:else}
  <div class="flex h-screen bg-[var(--bg-primary)]">
    <!-- Sidebar Navigation -->
    <nav class="w-[220px] bg-[var(--bg-secondary)] flex flex-col border-r border-[var(--border)] relative z-10">
      <!-- Logo Area -->
      <div class="px-5 py-6 border-b border-[var(--border)]">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-[var(--accent)] to-[var(--action)] flex items-center justify-center">
            <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
            </svg>
          </div>
          <div>
            <h1 class="text-sm font-semibold text-white">Unturned</h1>
            <p class="text-[10px] text-[var(--text-muted)]">服务器管理工具</p>
          </div>
        </div>
      </div>

      <!-- Navigation Items -->
      <ul class="mt-3 flex-1 px-3 space-y-1">
        {#each navItems as item}
          <li>
            <button
              class="w-full text-left px-3 py-2.5 rounded-lg transition-all duration-[var(--transition-normal)] flex items-center gap-3 cursor-pointer
                {currentPage === item.id
                  ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]'
                  : 'text-[var(--text-secondary)] hover:text-white hover:bg-[var(--bg-card)] border border-transparent'}"
              onclick={() => (currentPage = item.id)}
            >
              <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={item.icon} />
              </svg>
              <span class="text-sm font-medium">{item.label}</span>
              {#if currentPage === item.id}
                <div class="ml-auto w-1.5 h-1.5 rounded-full bg-[var(--accent)]"></div>
              {/if}
            </button>
          </li>
        {/each}
      </ul>

      <!-- Version Info & About -->
      <div class="px-3 pb-3 border-t border-[var(--border)] pt-3">
        <button
          class="w-full text-left px-3 py-2.5 rounded-lg transition-all duration-[var(--transition-normal)] flex items-center gap-3 cursor-pointer
            {currentPage === 'about'
              ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]'
              : 'text-[var(--text-secondary)] hover:text-white hover:bg-[var(--bg-card)] border border-transparent'}"
          onclick={() => (currentPage = 'about')}
        >
          <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-sm font-medium">关于</span>
          {#if currentPage === 'about'}
            <div class="ml-auto w-1.5 h-1.5 rounded-full bg-[var(--accent)]"></div>
          {/if}
        </button>
        <p class="text-[10px] text-[var(--text-muted)] px-3 mt-2">v1.0.0</p>
      </div>
    </nav>

    <!-- Main Content Area -->
    <main class="flex-1 overflow-hidden p-6 flex flex-col min-h-0">
      <div class="max-w-[1200px] mx-auto w-full flex-1 flex flex-col min-h-0">
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
