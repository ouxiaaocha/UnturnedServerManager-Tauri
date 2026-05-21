<div class="flex flex-col gap-5 h-full overflow-y-auto">
  <div>
    <h1 class="text-2xl font-bold text-white">关于</h1>
    <p class="text-sm text-[var(--text-muted)] mt-1">应用信息与作者</p>
  </div>

  <div class="flex-1 flex flex-col items-center justify-center gap-8">
    <!-- App Info Card -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-8 w-full max-w-md text-center">
      <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-[var(--accent)] to-[var(--action)] flex items-center justify-center mx-auto mb-5">
        <svg class="w-9 h-9 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
        </svg>
      </div>
      <h2 class="text-xl font-bold text-white mb-1">Unturned 服务器管理工具</h2>
      <p class="text-sm text-[var(--text-muted)] mb-4">一站式 Unturned 服务器管理客户端</p>
      <div class="inline-block bg-[var(--bg-elevated)] text-[var(--accent-light)] text-xs font-mono px-3 py-1 rounded-full">
        v1.0.0
      </div>
    </div>

    <!-- Author Card -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-8 w-full max-w-md">
      <h3 class="text-sm font-medium text-[var(--text-muted)] uppercase tracking-wider mb-5 text-center">开发者</h3>
      <div class="flex flex-col items-center gap-4">
        <div class="w-14 h-14 rounded-full bg-gradient-to-br from-[var(--accent)] to-cyan-600 flex items-center justify-center">
          <span class="text-xl font-bold text-white">欧</span>
        </div>
        <div class="text-center">
          <p class="text-lg font-semibold text-white">欧小茶</p>
          <p class="text-sm text-[var(--text-muted)] mt-1">独立开发者</p>
        </div>
      </div>

      <div class="mt-6 space-y-3">
        <a
          href="mailto:56896686@qq.com"
          class="flex items-center gap-3 px-4 py-3 rounded-lg bg-[var(--bg-primary)] hover:bg-[var(--bg-elevated)] border border-[var(--border)] transition-colors group"
        >
          <svg class="w-4 h-4 text-[var(--text-muted)] group-hover:text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
          <div class="flex-1 min-w-0">
            <p class="text-xs text-[var(--text-muted)]">邮箱</p>
            <p class="text-sm text-white truncate">56896686@qq.com</p>
          </div>
        </a>

        <a
          href="https://github.com/ouxiaaocha?tab=repositories"
          target="_blank"
          rel="noopener noreferrer"
          class="flex items-center gap-3 px-4 py-3 rounded-lg bg-[var(--bg-primary)] hover:bg-[var(--bg-elevated)] border border-[var(--border)] transition-colors group"
        >
          <svg class="w-4 h-4 text-[var(--text-muted)] group-hover:text-[var(--accent-light)]" fill="currentColor" viewBox="0 0 24 24">
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
          </svg>
          <div class="flex-1 min-w-0">
            <p class="text-xs text-[var(--text-muted)]">GitHub</p>
            <p class="text-sm text-white truncate">github.com/ouxiaaocha</p>
          </div>
          <svg class="w-3.5 h-3.5 text-[var(--text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
        </a>
      </div>
    </div>
  </div>
</div>
