<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-shell";
  import { toastStore } from "../stores/toast.svelte";

  declare const __APP_VERSION__: string;

  // --- 路径配置状态 ---
  let steamCmdPath = $state("");
  let serverRoot = $state("");
  let saving = $state(false);
  let existingConfig: any = null;

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
    } catch (e: any) {
      toastStore.error(`保存失败: ${e}`);
    }
    saving = false;
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
      await open(url);
    } catch {
      // fallback: 尝试用 window.open
      window.open(url, "_blank");
    }
  }

  /** 简单 markdown 转 HTML */
  function renderMarkdown(text: string): string {
    if (!text) return "";
    return text
      // 标题
      .replace(/^### (.+)$/gm, '<h4 class="text-sm font-semibold text-[var(--text-primary)] mt-3 mb-1">$1</h4>')
      .replace(/^## (.+)$/gm, '<h3 class="text-base font-semibold text-[var(--text-primary)] mt-4 mb-2">$1</h3>')
      // 列表项
      .replace(/^- (.+)$/gm, '<div class="flex gap-2 ml-1"><span class="text-[var(--accent)]">•</span><span>$1</span></div>')
      // 粗体
      .replace(/\*\*(.+?)\*\*/g, '<strong class="font-semibold text-[var(--text-primary)]">$1</strong>')
      // 链接
      .replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2" class="text-[var(--accent-light)] hover:underline" onclick="event.preventDefault(); window.__openUrl && window.__openUrl(\'$2\')">$1</a>')
      // 换行
      .replace(/\n/g, "<br/>");
  }

  $effect(() => { loadConfig(); });

  // 页面加载时自动检测更新
  $effect(() => { checkUpdate(); });
</script>

<div class="flex flex-col gap-6">
  <!-- 页面标题 -->
  <div>
    <h1 class="text-2xl font-bold text-[var(--text-primary)]">设置</h1>
    <p class="text-sm text-[var(--text-muted)] mt-1">管理路径配置与软件更新</p>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- ========== 左栏：路径配置 ========== -->
    <div class="space-y-4">
      <h2 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider flex items-center gap-2">
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
          <p class="text-xs text-[var(--danger)] mt-1.5">⚠ 目录不能包含中文字符，否则可能导致服务器无法启动</p>
        </div>
        <div>
          <span class="block text-xs text-[var(--text-muted)] mb-2">服务端目录</span>
          <input type="text" bind:value={serverRoot} placeholder="C:\SteamCMD\steamapps\common\U3DS"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
          <p class="text-xs text-[var(--danger)] mt-1.5">⚠ 目录不能包含中文字符，否则可能导致服务器无法启动</p>
        </div>
      </div>

      <!-- 保存按钮 -->
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
      </div>
    </div>

    <!-- ========== 右栏：关于软件 & 更新检测 ========== -->
    <div class="space-y-4">
      <h2 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider flex items-center gap-2">
        <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
        关于软件
      </h2>
      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 space-y-5">
        <!-- 当前版本 & 检查按钮 -->
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="text-sm text-[var(--text-secondary)]">当前版本</span>
            <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-lg border border-[var(--border-accent)] bg-[var(--accent-subtle)] text-xs font-semibold text-[var(--accent-light)]">
              <span class="h-1.5 w-1.5 rounded-full bg-[var(--success)]"></span>
              v{__APP_VERSION__}
            </span>
          </div>
          <button
            class="px-4 py-2 text-xs font-medium rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] text-[var(--text-secondary)] hover:border-[var(--accent)] hover:text-[var(--accent-light)] transition-all duration-[var(--transition-normal)] disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center gap-1.5"
            onclick={checkUpdate}
            disabled={checkStatus === "checking"}
          >
            {#if checkStatus === "checking"}
              <div class="w-3.5 h-3.5 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
              检测中...
            {:else}
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              检查更新
            {/if}
          </button>
        </div>

        <!-- 检测结果 -->
        {#if checkStatus === "has_update" && updateInfo}
          <div class="rounded-lg border border-emerald-500/30 bg-emerald-500/5 p-4 space-y-3">
            <div class="flex items-center gap-2">
              <span class="text-lg">🆕</span>
              <span class="text-sm font-semibold text-emerald-400">发现新版本 v{updateInfo.latest_version}</span>
            </div>
            {#if updateInfo.published_at}
              <p class="text-xs text-[var(--text-muted)]">发布时间：{formatDate(updateInfo.published_at)}</p>
            {/if}
            {#if updateInfo.body}
              <div class="mt-2">
                <p class="text-xs font-medium text-[var(--text-secondary)] mb-2">更新日志</p>
                <div class="text-xs text-[var(--text-muted)] leading-relaxed bg-[var(--bg-primary)] rounded-lg p-3 max-h-48 overflow-y-auto border border-[var(--border)]">
                  {@html renderMarkdown(updateInfo.body)}
                </div>
              </div>
            {/if}
            {#if updateInfo.html_url}
              <button
                onclick={() => openUrl(updateInfo.html_url)}
                class="inline-flex items-center gap-2 px-4 py-2 mt-1 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-white text-xs font-medium rounded-lg transition-all duration-[var(--transition-normal)] cursor-pointer"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                </svg>
                前往下载
              </button>
            {/if}
          </div>
        {:else if checkStatus === "up_to_date"}
          <div class="rounded-lg border border-[var(--success)]/30 bg-[var(--success)]/5 p-4 flex items-center gap-3">
            <svg class="w-5 h-5 text-[var(--success)] shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span class="text-sm text-[var(--success)]">当前已是最新版本</span>
          </div>
        {:else if checkStatus === "error"}
          <div class="rounded-lg border border-[var(--danger)]/30 bg-[var(--danger)]/5 p-4 flex items-center gap-3">
            <svg class="w-5 h-5 text-[var(--danger)] shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <div class="min-w-0">
              <span class="text-sm text-[var(--danger)]">检测更新失败</span>
              <p class="text-xs text-[var(--text-muted)] mt-0.5">{errorMsg}</p>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
