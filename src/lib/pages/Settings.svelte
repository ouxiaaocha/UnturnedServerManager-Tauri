<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { open as openShell } from "@tauri-apps/plugin-shell";
  import { toastStore } from "../stores/toast.svelte";
  import { escapeHtml } from "$lib/utils";

  declare const __APP_VERSION__: string;

  // --- 路径配置状态 ---
  let steamCmdPath = $state("");
  let serverRoot = $state("");
  let saving = $state(false);
  let existingConfig = $state<any>(null);

  // --- 运行环境检测状态 ---
  type EnvironmentItem = {
    key: string;
    label: string;
    ok: boolean;
    required: boolean;
    message: string;
    path?: string;
  };
  type EnvironmentReport = {
    ok: boolean;
    saveId?: string;
    items: EnvironmentItem[];
  };
  let environmentReport = $state<EnvironmentReport | null>(null);
  let environmentChecking = $state(false);
  let environmentFullChecking = $state(false);
  let environmentRepairing = $state("");
  let requiredEnvironmentItems = $derived(environmentReport?.items?.filter((item) => item.required) || []);
  let failedEnvironmentItems = $derived(requiredEnvironmentItems.filter((item) => !item.ok));
  let environmentReadyCount = $derived(requiredEnvironmentItems.filter((item) => item.ok).length);
  let environmentTotalCount = $derived(requiredEnvironmentItems.length);
  let steamConnectivityItem = $derived(environmentReport?.items?.find((item) => item.key === "steamcmd_connectivity"));

  // --- 窗口行为配置状态 ---
  let closeToTray = $state(false);
  let closeActionRemembered = $state(false);

  // --- 更新检测状态 ---
  type CheckStatus = "idle" | "checking" | "has_update" | "up_to_date" | "error";
  let checkStatus = $state<CheckStatus>("idle");
  let updateInfo = $state<any>(null);
  let errorMsg = $state("");

  async function loadConfig() {
    try {
      const config: any = await invoke("get_config");
      if (config.servers && config.servers.length > 0) {
        const s = config.servers[0];
        existingConfig = s;
        steamCmdPath = s.steamCmdPath || "";
        serverRoot = s.serverRoot || "";
      }

      const appSettings: any = await invoke("get_app_settings");
      closeToTray = appSettings.closeToTray || false;
      closeActionRemembered = appSettings.closeActionRemembered || false;

      await checkEnvironment(false);
    } catch (e) { console.error("加载配置失败:", e); }
  }

  async function save() {
    saving = true;
    try {
      const server = {
        ...(existingConfig || {}),
        steamCmdPath,
        serverRoot,
      };
      // save_config 期望 ServersConfig 结构体，包含 servers 数组
      await invoke("save_config", { servers: { servers: [server] } });
      toastStore.success("保存成功");
      await checkEnvironment(false);
    } catch (e: any) {
      toastStore.error(`保存失败: ${e}`);
    }
    saving = false;
  }

  async function browseSteamCmdPath() {
    const selected = await openDialog({
      title: "选择 steamcmd.exe",
      multiple: false,
      directory: false,
      filters: [{ name: "SteamCMD", extensions: ["exe"] }],
    });
    if (typeof selected === "string") {
      steamCmdPath = selected;
    }
  }

  async function browseServerRoot() {
    const selected = await openDialog({
      title: "选择服务端目录",
      multiple: false,
      directory: true,
    });
    if (typeof selected === "string") {
      serverRoot = selected;
    }
  }

  async function checkEnvironment(includeSteamTest = false) {
    if (includeSteamTest) {
      environmentFullChecking = true;
    } else {
      environmentChecking = true;
    }
    try {
      environmentReport = await invoke("check_runtime_environment", { includeSteamTest }) as EnvironmentReport;
      if (includeSteamTest) {
        toastStore.success(environmentReport.ok ? "检测完成，运行条件正常" : "检测完成，请查看异常项");
      }
    } catch (e: any) {
      toastStore.error(`运行环境检测失败: ${e}`);
    } finally {
      environmentChecking = false;
      environmentFullChecking = false;
    }
  }

  async function repairEnvironment(target: "rocket" | "bridge" | "all") {
    environmentRepairing = target;
    try {
      const message = await invoke("install_runtime_requirement", { target }) as string;
      toastStore.success(message);
      await checkEnvironment(false);
    } catch (e: any) {
      toastStore.error(`安装失败: ${e}`);
    } finally {
      environmentRepairing = "";
    }
  }

  async function checkUpdate() {
    checkStatus = "checking";
    errorMsg = "";
    updateInfo = null;
    try {
      const info: any = await invoke("check_for_updates");
      updateInfo = info;
      checkStatus = info.has_update ? "has_update" : "up_to_date";
    } catch (e: any) {
      checkStatus = "error";
      errorMsg = typeof e === "string" ? e : "网络连接失败，请检查网络后重试";
    }
  }

  /** 将 ISO 时间格式化为 YYYY-MM-DD */
  function formatDate(iso: string): string {
    if (!iso) return "";
    try {
      return new Date(iso).toLocaleDateString("zh-CN", {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
      });
    } catch {
      return iso.slice(0, 10);
    }
  }

  /** 打开外部链接 */
  async function openUrl(url: string) {
    try {
      await openShell(url);
    } catch {
      // fallback: 尝试用 window.open
      window.open(url, "_blank");
    }
  }

  async function saveClosePreference() {
    try {
      await invoke("save_close_preference", {
        closeToTray,
        remember: closeActionRemembered
      });
      toastStore.success("关闭行为已保存");
    } catch (e: any) {
      toastStore.error(`保存失败: ${e}`);
    }
  }

  async function resetClosePreference() {
    try {
      await invoke("save_close_preference", {
        closeToTray: false,
        remember: false
      });
      closeToTray = false;
      closeActionRemembered = false;
      toastStore.success("已重置关闭行为");
    } catch (e: any) {
      toastStore.error(`重置失败: ${e}`);
    }
  }

  /**
   * 简单 markdown 转 HTML，结果通过 {@html} 注入。
   * 安全约束：必须先 escapeHtml（转义 & < > " '）再做下面的格式替换。
   * 顺序颠倒、或在 escapeHtml 之前新增任何替换，都会引入 XSS。
   * 链接 href 仅允许 https?://。updateInfo.body 来自 GitHub release（半可信来源）。
   */
  function renderMarkdown(text: string): string {
    if (!text) return "";
    return escapeHtml(text)
      // 标题
      .replace(/^### (.+)$/gm, '<h4 class="text-sm font-semibold text-[var(--text-primary)] mt-3 mb-1">$1</h4>')
      .replace(/^## (.+)$/gm, '<h3 class="text-base font-semibold text-[var(--text-primary)] mt-4 mb-2">$1</h3>')
      // 列表项
      .replace(/^- (.+)$/gm, '<div class="flex gap-2 ml-1"><span class="text-[var(--accent)]">•</span><span>$1</span></div>')
      // 粗体
      .replace(/\*\*(.+?)\*\*/g, '<strong class="font-semibold text-[var(--text-primary)]">$1</strong>')
      // 链接
      .replace(/\[(.+?)\]\((https?:\/\/[^)\s]+)\)/g, '<a href="$2" class="text-[var(--accent-light)] hover:underline" target="_blank" rel="noreferrer">$1</a>')
      // 换行
      .replace(/\n/g, "<br/>");
  }

  onMount(() => {
    loadConfig();
    // 页面加载时自动检测更新
    checkUpdate();
  });
</script>

<div class="flex flex-col gap-5">
  <div class="overflow-hidden rounded-2xl border border-[var(--border-accent)] bg-[linear-gradient(135deg,rgba(15,159,143,0.12),rgba(255,255,255,0.88)_46%,rgba(47,111,237,0.08))] p-5 shadow-[var(--shadow-md)]">
    <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
      <div class="min-w-0">
        <h1 class="text-2xl font-bold text-[var(--text-primary)]">设置</h1>
      </div>
      <div class="grid min-w-[260px] grid-cols-2 gap-3">
        <div class="rounded-xl border border-white/70 bg-white/75 p-3 shadow-[var(--shadow-sm)]">
          <p class="text-[11px] text-[var(--text-muted)]">当前存档</p>
          <p class="mt-1 truncate text-sm font-bold text-[var(--text-primary)]">{environmentReport?.saveId || existingConfig?.id || "未配置"}</p>
        </div>
        <div class="rounded-xl border border-white/70 bg-white/75 p-3 shadow-[var(--shadow-sm)]">
          <p class="text-[11px] text-[var(--text-muted)]">运行条件</p>
          <p class="mt-1 text-sm font-bold {environmentReport?.ok ? 'text-[var(--success)]' : failedEnvironmentItems.length ? 'text-[var(--warning)]' : 'text-[var(--text-secondary)]'}">
            {environmentTotalCount ? `${environmentReadyCount}/${environmentTotalCount}` : "未检测"}
          </p>
        </div>
      </div>
    </div>
  </div>

  <div class="grid grid-cols-1 gap-5 xl:grid-cols-[minmax(0,1.45fr)_minmax(360px,0.85fr)]">
    <div class="space-y-5">
      <section class="overflow-hidden rounded-2xl border border-[var(--border)] bg-[var(--bg-card)] shadow-[var(--shadow-sm)]">
        <div class="flex flex-col gap-3 border-b border-[var(--border)] bg-[var(--bg-secondary)]/70 px-5 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 class="mt-1 text-base font-bold text-[var(--text-primary)]">核心路径</h2>
          </div>
          <button
            class="inline-flex items-center justify-center gap-2 rounded-lg bg-gradient-to-r from-[var(--accent)] to-cyan-600 px-5 py-2.5 text-sm font-medium text-white transition-all duration-[var(--transition-normal)] hover:from-cyan-500 hover:to-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-50"
            onclick={save}
            disabled={saving}
          >
            {#if saving}
              <div class="h-4 w-4 rounded-full border-2 border-white border-t-transparent animate-spin"></div>
              保存中...
            {:else}
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              保存设置
            {/if}
          </button>
        </div>
        <div class="grid gap-4 p-5 lg:grid-cols-2">
          <div class="block rounded-xl border border-[var(--border)] bg-[var(--bg-primary)]/65 p-4 transition-all duration-[var(--transition-normal)] focus-within:border-[var(--accent)] focus-within:bg-white">
            <div class="mb-2 flex items-center justify-between gap-3">
              <span class="text-xs font-semibold text-[var(--text-secondary)]">SteamCMD 路径</span>
              <button
                type="button"
                class="shrink-0 rounded-md border border-[var(--border)] bg-white px-2.5 py-1 text-[11px] font-medium text-[var(--text-secondary)] transition-all hover:border-[var(--accent)] hover:text-[var(--accent-light)]"
                onclick={browseSteamCmdPath}
              >浏览</button>
            </div>
            <input
              type="text"
              bind:value={steamCmdPath}
              placeholder="C:\SteamCMD\steamcmd.exe"
              class="w-full border-0 bg-transparent p-0 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none"
            />
            <span class="mt-2 block text-[11px] text-[var(--danger)]">路径避免中文，防止启动异常</span>
          </div>
          <div class="block rounded-xl border border-[var(--border)] bg-[var(--bg-primary)]/65 p-4 transition-all duration-[var(--transition-normal)] focus-within:border-[var(--accent)] focus-within:bg-white">
            <div class="mb-2 flex items-center justify-between gap-3">
              <span class="text-xs font-semibold text-[var(--text-secondary)]">服务端目录</span>
              <button
                type="button"
                class="shrink-0 rounded-md border border-[var(--border)] bg-white px-2.5 py-1 text-[11px] font-medium text-[var(--text-secondary)] transition-all hover:border-[var(--accent)] hover:text-[var(--accent-light)]"
                onclick={browseServerRoot}
              >浏览</button>
            </div>
            <input
              type="text"
              bind:value={serverRoot}
              placeholder="C:\SteamCMD\steamapps\common\U3DS"
              class="w-full border-0 bg-transparent p-0 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none"
            />
            <span class="mt-2 block text-[11px] text-[var(--danger)]">需包含 Unturned.exe，路径避免中文</span>
          </div>
        </div>
      </section>

      <section class="overflow-hidden rounded-2xl border border-[var(--border)] bg-[var(--bg-card)] shadow-[var(--shadow-sm)]">
        <div class="border-b border-[var(--border)] bg-[var(--bg-secondary)]/70 px-5 py-4">
          <h2 class="text-base font-bold text-[var(--text-primary)]">窗口行为</h2>
          <p class="mt-1 text-xs text-[var(--text-muted)]">配置窗口关闭和系统托盘行为</p>
        </div>
        <div class="p-5 space-y-4">
          <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-primary)]/70 p-4">
            <div class="flex items-start justify-between gap-4">
              <div class="flex-1">
                <p class="font-semibold text-[var(--text-primary)]">点击关闭按钮时</p>
                <p class="mt-1 text-xs text-[var(--text-muted)]">
                  {closeToTray ? "最小化到系统托盘" : "退出程序"}
                </p>
              </div>
              <label class="relative inline-flex cursor-pointer items-center">
                <input
                  type="checkbox"
                  bind:checked={closeToTray}
                  onchange={saveClosePreference}
                  class="peer sr-only"
                />
                <div class="peer h-6 w-11 rounded-full bg-[var(--bg-elevated)] after:absolute after:left-[2px] after:top-[2px] after:h-5 after:w-5 after:rounded-full after:border after:border-[var(--border)] after:bg-white after:transition-all after:content-[''] peer-checked:bg-[var(--accent)] peer-checked:after:translate-x-full peer-focus:ring-2 peer-focus:ring-[var(--accent)]"></div>
              </label>
            </div>
          </div>

          {#if closeActionRemembered}
            <div class="rounded-xl border border-[var(--border-accent)] bg-[var(--accent-subtle)] p-4">
              <div class="flex items-start gap-3">
                <svg class="h-5 w-5 shrink-0 text-[var(--accent)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <div class="flex-1">
                  <p class="text-sm font-medium text-[var(--accent-light)]">已记住关闭行为</p>
                  <p class="mt-1 text-xs text-[var(--text-muted)]">关闭窗口时不会再显示确认对话框</p>
                </div>
              </div>
              <button
                onclick={resetClosePreference}
                class="mt-3 rounded-lg border border-[var(--border)] bg-white px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)] transition-all hover:border-[var(--accent)] hover:text-[var(--accent-light)]"
              >
                重置为首次设置
              </button>
            </div>
          {/if}

          <div class="rounded-xl border border-dashed border-[var(--border-hover)] bg-[var(--bg-primary)]/60 p-4">
            <p class="text-xs text-[var(--text-muted)]">
              提示：应用最小化到系统托盘后，可通过点击托盘图标或右键菜单重新打开窗口
            </p>
          </div>
        </div>
      </section>

      <section class="overflow-hidden rounded-2xl border border-[var(--border)] bg-[var(--bg-card)] shadow-[var(--shadow-sm)]">
        <div class="flex flex-col gap-3 border-b border-[var(--border)] bg-[var(--bg-secondary)]/70 px-5 py-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <div class="flex flex-wrap items-center gap-2">
              <h2 class="text-base font-bold text-[var(--text-primary)]">运行环境检测</h2>
              {#if environmentReport}
                <span class="rounded-full border px-2.5 py-0.5 text-[11px] font-semibold {environmentReport.ok ? 'border-[var(--success)]/30 bg-[var(--success)]/10 text-[var(--success)]' : 'border-[var(--warning)]/30 bg-[var(--warning)]/10 text-[var(--warning)]'}">
                  {environmentReport.ok ? "核心条件正常" : `${failedEnvironmentItems.length} 项需处理`}
                </span>
              {/if}
            </div>
          </div>
          <div class="flex flex-wrap gap-2">
            <button
              class="inline-flex items-center justify-center gap-1.5 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-3 py-2 text-xs font-medium text-[var(--text-secondary)] transition-all duration-[var(--transition-normal)] hover:border-[var(--accent)] hover:text-[var(--accent-light)] disabled:cursor-not-allowed disabled:opacity-50"
              onclick={() => checkEnvironment(false)}
              disabled={environmentChecking || environmentFullChecking}
            >
              {#if environmentChecking}
                <div class="h-3.5 w-3.5 rounded-full border-2 border-[var(--accent)] border-t-transparent animate-spin"></div>
                检测中...
              {:else}
                快速检测
              {/if}
            </button>
            <button
              class="inline-flex items-center justify-center gap-1.5 rounded-lg border border-[var(--border-accent)] bg-[var(--accent-subtle)] px-3 py-2 text-xs font-semibold text-[var(--accent-light)] transition-all duration-[var(--transition-normal)] hover:border-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-50"
              onclick={() => checkEnvironment(true)}
              disabled={environmentChecking || environmentFullChecking}
            >
              {#if environmentFullChecking}
                <div class="h-3.5 w-3.5 rounded-full border-2 border-[var(--accent)] border-t-transparent animate-spin"></div>
                测试中...
              {:else}
                完整检测
              {/if}
            </button>
          </div>
        </div>

        <div class="p-5">
          <div class="mb-4 grid gap-3 md:grid-cols-3">
            <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-primary)]/70 p-4">
              <p class="text-[11px] text-[var(--text-muted)]">核心条件</p>
              <p class="mt-2 text-2xl font-bold text-[var(--text-primary)]">{environmentTotalCount ? `${environmentReadyCount}/${environmentTotalCount}` : "--"}</p>
            </div>
            <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-primary)]/70 p-4">
              <p class="text-[11px] text-[var(--text-muted)]">SteamCMD</p>
              <p class="mt-2 text-sm font-bold {steamConnectivityItem?.ok ? 'text-[var(--success)]' : 'text-[var(--text-secondary)]'}">
                {steamConnectivityItem ? (steamConnectivityItem.ok ? "可用" : "待测试") : "未测试"}
              </p>
            </div>
            <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-primary)]/70 p-4">
              <p class="text-[11px] text-[var(--text-muted)]">本地命令</p>
              <p class="mt-2 text-sm font-bold {environmentReport?.items?.find((item) => item.key === 'bridge_dll')?.ok ? 'text-[var(--success)]' : 'text-[var(--warning)]'}">
                {environmentReport?.items?.find((item) => item.key === 'bridge_dll')?.ok ? "Bridge 就绪" : "需安装 Bridge"}
              </p>
            </div>
          </div>

          {#if environmentReport?.items?.length}
            <div class="grid gap-2">
              {#each environmentReport.items as item (item.key)}
                <div class="group rounded-xl border border-[var(--border)] bg-white/60 p-3 transition-all duration-[var(--transition-normal)] hover:border-[var(--border-hover)] hover:bg-white">
                  <div class="flex items-start gap-3">
                    <div class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg {item.ok ? 'bg-[var(--success-glow)] text-[var(--success)]' : item.required ? 'bg-[var(--danger-glow)] text-[var(--danger)]' : 'bg-[var(--warning-glow)] text-[var(--warning)]'}">
                      {#if item.ok}
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                      {:else}
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v4m0 4h.01M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" />
                        </svg>
                      {/if}
                    </div>
                    <div class="min-w-0 flex-1">
                      <div class="flex flex-wrap items-center gap-2">
                        <span class="text-sm font-semibold text-[var(--text-primary)]">{item.label}</span>
                        <span class="rounded-full px-2 py-0.5 text-[11px] font-medium {item.ok ? 'bg-[var(--success)]/10 text-[var(--success)]' : item.required ? 'bg-[var(--danger)]/10 text-[var(--danger)]' : 'bg-[var(--warning)]/10 text-[var(--warning)]'}">
                          {item.ok ? "正常" : item.required ? "缺失" : "需检查"}
                        </span>
                      </div>
                      <p class="mt-1 text-xs text-[var(--text-secondary)]">{item.message}</p>
                      {#if item.path}
                        <p class="mt-1 truncate font-mono text-[11px] text-[var(--text-muted)]">{item.path}</p>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="rounded-xl border border-dashed border-[var(--border-hover)] bg-[var(--bg-primary)]/60 p-5 text-sm text-[var(--text-muted)]">
              保存路径后点击「快速检测」检查运行条件
            </div>
          {/if}
        </div>
      </section>
    </div>

    <aside class="space-y-5">
      <section class="rounded-2xl border border-[var(--border)] bg-[var(--bg-card)] p-5 shadow-[var(--shadow-sm)]">
        <h2 class="text-base font-bold text-[var(--text-primary)]">修复与重装</h2>
        <div class="mt-4 grid gap-2">
          <button
            class="inline-flex items-center justify-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 text-xs font-medium text-[var(--text-secondary)] transition-all duration-[var(--transition-normal)] hover:border-[var(--accent)] hover:text-[var(--accent-light)] disabled:cursor-not-allowed disabled:opacity-50"
            onclick={() => repairEnvironment("rocket")}
            disabled={!!environmentRepairing || !serverRoot.trim()}
          >
            {#if environmentRepairing === "rocket"}
              <div class="h-3.5 w-3.5 rounded-full border-2 border-[var(--accent)] border-t-transparent animate-spin"></div>
              安装中...
            {:else}
              重新安装 Rocket
            {/if}
          </button>
          <button
            class="inline-flex items-center justify-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 text-xs font-medium text-[var(--text-secondary)] transition-all duration-[var(--transition-normal)] hover:border-[var(--accent)] hover:text-[var(--accent-light)] disabled:cursor-not-allowed disabled:opacity-50"
            onclick={() => repairEnvironment("bridge")}
            disabled={!!environmentRepairing || !serverRoot.trim()}
          >
            {#if environmentRepairing === "bridge"}
              <div class="h-3.5 w-3.5 rounded-full border-2 border-[var(--accent)] border-t-transparent animate-spin"></div>
              安装中...
            {:else}
              重新安装 Bridge
            {/if}
          </button>
          <button
            class="inline-flex items-center justify-center gap-2 rounded-lg bg-gradient-to-r from-[var(--accent)] to-cyan-600 px-4 py-2.5 text-xs font-semibold text-white transition-all duration-[var(--transition-normal)] hover:from-cyan-500 hover:to-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-50"
            onclick={() => repairEnvironment("all")}
            disabled={!!environmentRepairing || !serverRoot.trim()}
          >
            {#if environmentRepairing === "all"}
              <div class="h-3.5 w-3.5 rounded-full border-2 border-white border-t-transparent animate-spin"></div>
              修复中...
            {:else}
              一键修复缺失项
            {/if}
          </button>
        </div>
      </section>

      <section class="rounded-2xl border border-[var(--border)] bg-[var(--bg-card)] p-5 shadow-[var(--shadow-sm)]">
        <div class="flex items-start justify-between gap-3">
          <div>
            <h2 class="text-base font-bold text-[var(--text-primary)]">软件更新</h2>
          </div>
          <span class="inline-flex items-center gap-1.5 rounded-lg border border-[var(--border-accent)] bg-[var(--accent-subtle)] px-3 py-1 text-xs font-semibold text-[var(--accent-light)]">
            <span class="h-1.5 w-1.5 rounded-full bg-[var(--success)]"></span>
            v{__APP_VERSION__}
          </span>
        </div>
        <div class="mt-4 grid grid-cols-2 gap-2">
          <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-primary)]/70 p-3">
            <p class="text-[11px] text-[var(--text-muted)]">本地版本</p>
            <p class="mt-1 font-mono text-sm font-bold text-[var(--text-primary)]">v{updateInfo?.current_version || __APP_VERSION__}</p>
          </div>
          <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-primary)]/70 p-3">
            <p class="text-[11px] text-[var(--text-muted)]">云端版本</p>
            <p class="mt-1 font-mono text-sm font-bold {updateInfo?.latest_version ? 'text-[var(--text-primary)]' : 'text-[var(--text-muted)]'}">
              {updateInfo?.latest_version ? `v${updateInfo.latest_version}` : "未检测"}
            </p>
          </div>
        </div>
        <button
          class="mt-4 inline-flex w-full items-center justify-center gap-1.5 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 text-xs font-medium text-[var(--text-secondary)] transition-all duration-[var(--transition-normal)] hover:border-[var(--accent)] hover:text-[var(--accent-light)] disabled:cursor-not-allowed disabled:opacity-50"
          onclick={checkUpdate}
          disabled={checkStatus === "checking"}
        >
          {#if checkStatus === "checking"}
            <div class="h-3.5 w-3.5 rounded-full border-2 border-[var(--accent)] border-t-transparent animate-spin"></div>
            检测中...
          {:else}
            <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            检查更新
          {/if}
        </button>

        {#if checkStatus === "has_update" && updateInfo}
          <div class="mt-4 rounded-xl border border-[var(--success)]/30 bg-[var(--success)]/5 p-4">
            <p class="text-sm font-semibold text-[var(--success)]">发现新版本 v{updateInfo.latest_version}</p>
            {#if updateInfo.published_at}
              <p class="mt-1 text-xs text-[var(--text-muted)]">发布时间：{formatDate(updateInfo.published_at)}</p>
            {/if}
            {#if updateInfo.body}
              <div class="mt-3 max-h-48 overflow-y-auto rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] p-3 text-xs leading-relaxed text-[var(--text-muted)]">
                {@html renderMarkdown(updateInfo.body)}
              </div>
            {/if}
            {#if updateInfo.html_url}
              <button
                onclick={() => openUrl(updateInfo.html_url)}
                class="mt-3 inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-[var(--accent)] to-cyan-600 px-4 py-2 text-xs font-medium text-white transition-all duration-[var(--transition-normal)] hover:from-cyan-500 hover:to-[var(--accent)]"
              >
                前往下载
              </button>
            {/if}
          </div>
        {:else if checkStatus === "up_to_date"}
          <div class="mt-4 rounded-xl border border-[var(--success)]/30 bg-[var(--success)]/5 p-4 text-sm text-[var(--success)]">
            当前已是最新版本
          </div>
        {:else if checkStatus === "error"}
          <div class="mt-4 rounded-xl border border-[var(--danger)]/30 bg-[var(--danger)]/5 p-4">
            <p class="text-sm font-semibold text-[var(--danger)]">检测更新失败</p>
            <p class="mt-1 text-xs text-[var(--text-muted)]">{errorMsg}</p>
          </div>
        {/if}
      </section>
    </aside>
  </div>
</div>
