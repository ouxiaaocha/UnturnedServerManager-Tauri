<script lang="ts">
  declare const __APP_VERSION__: string;

  const capabilities = [
    { label: "仪表盘监控", value: "状态 / PID / CPU / 内存 / 网络" },
    { label: "服务器控制", value: "启动 / 停止 / 重启 / 强制停止" },
    { label: "RCON 控制台", value: "远程命令与自动连接" },
    { label: "存档配置", value: "Commands.dat / GSLT / 地图 / 端口" },
    { label: "创意工坊模组", value: "模组管理 / 备注 / 更新监控" },
    { label: "插件管理", value: "Rocket 插件 / 配置目录" },
    { label: "日志中心", value: "软件 / 操作 / 游戏日志" },
    { label: "定时任务", value: "每日 / 每周 / 间隔重启" },
  ];

  const buildInfo = [
    { label: "版本", value: `v${__APP_VERSION__}` },
    { label: "框架", value: "Tauri v2" },
    { label: "前端", value: "Svelte 5" },
    { label: "平台", value: "Windows Portable" },
  ];
</script>

<div class="h-full overflow-y-auto">
  <div class="mb-6">
    <h1 class="text-2xl font-bold text-[var(--text-primary)]">关于</h1>
    <p class="mt-1 text-sm text-[var(--text-muted)]">应用信息、维护者与构建状态</p>
  </div>

  <div class="grid gap-5 xl:grid-cols-[minmax(0,1.25fr)_minmax(320px,0.75fr)]">
    <section class="overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--bg-card)]">
      <div class="border-b border-[var(--border)] bg-[var(--accent-subtle)] px-6 py-7">
        <div class="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
          <div class="flex items-center gap-4">
            <div class="flex h-16 w-16 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-[var(--accent)] to-[var(--action)] shadow-[var(--shadow-glow)]">
              <svg class="h-9 w-9 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
              </svg>
            </div>
            <div class="min-w-0">
              <p class="text-xs font-semibold uppercase tracking-wider text-[var(--accent-light)]">Unturned Server Manager</p>
              <h2 class="mt-1 text-2xl font-bold leading-tight text-[var(--text-primary)]">Unturned 服务器管理工具</h2>
              <p class="mt-2 max-w-2xl text-sm text-[var(--text-secondary)]">面向单机部署和轻量运维的桌面控制台，把服务端启动、RCON、存档、插件、更新和定时任务收在一个工作台里。</p>
            </div>
          </div>
          <div class="flex shrink-0 items-center gap-2 rounded-lg border border-[var(--border-accent)] bg-white/70 px-3 py-2 text-xs font-medium text-[var(--accent-light)]">
            <span class="h-2 w-2 rounded-full bg-[var(--success)]"></span>
            当前版本 v{__APP_VERSION__}
          </div>
        </div>
      </div>

      <div class="grid border-t border-[var(--border)] md:grid-cols-2">
        {#each capabilities as item}
          <div class="border-b border-[var(--border)] px-5 py-4 md:even:border-l md:[&:nth-last-child(-n+2)]:border-b-0">
            <p class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">{item.label}</p>
            <p class="mt-2 text-sm font-medium text-[var(--text-primary)]">{item.value}</p>
          </div>
        {/each}
      </div>
    </section>

    <aside class="space-y-5">
      <section class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] p-5">
        <h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-muted)]">构建信息</h3>
        <div class="space-y-3">
          {#each buildInfo as item}
            <div class="flex items-center justify-between gap-4 border-b border-[var(--border)] pb-3 last:border-0 last:pb-0">
              <span class="text-sm text-[var(--text-secondary)]">{item.label}</span>
              <span class="text-sm font-medium text-[var(--text-primary)]">{item.value}</span>
            </div>
          {/each}
        </div>
      </section>

      <section class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] p-5">
        <h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-muted)]">开发者</h3>
        <div class="mb-5 flex items-center gap-3">
          <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-[var(--accent)] to-cyan-600 text-lg font-bold text-white">
            欧
          </div>
          <div>
            <p class="text-base font-semibold text-[var(--text-primary)]">欧小茶</p>
            <p class="text-xs text-[var(--text-muted)]">独立开发者</p>
          </div>
        </div>

        <div class="space-y-3">
          <a
            href="mailto:56896686@qq.com"
            class="flex items-center gap-3 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-3 transition-colors hover:border-[var(--border-accent)] hover:bg-[var(--bg-card-hover)]"
          >
            <svg class="h-4 w-4 shrink-0 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
            <div class="min-w-0">
              <p class="text-xs text-[var(--text-muted)]">邮箱</p>
              <p class="truncate text-sm font-medium text-[var(--text-primary)]">56896686@qq.com</p>
            </div>
          </a>

          <a
            href="https://github.com/ouxiaaocha?tab=repositories"
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center gap-3 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-3 transition-colors hover:border-[var(--border-accent)] hover:bg-[var(--bg-card-hover)]"
          >
            <svg class="h-4 w-4 shrink-0 text-[var(--accent-light)]" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
            </svg>
            <div class="min-w-0 flex-1">
              <p class="text-xs text-[var(--text-muted)]">GitHub</p>
              <p class="truncate text-sm font-medium text-[var(--text-primary)]">github.com/ouxiaaocha</p>
            </div>
            <svg class="h-3.5 w-3.5 shrink-0 text-[var(--text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
          </a>
        </div>
      </section>
    </aside>
  </div>
</div>
