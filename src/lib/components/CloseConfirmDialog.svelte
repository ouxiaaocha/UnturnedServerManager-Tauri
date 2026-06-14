<script lang="ts">
  import Button from "./Button.svelte";

  let {
    show = $bindable(false),
    onConfirm,
    onCancel
  }: {
    show: boolean;
    onConfirm: (closeToTray: boolean, remember: boolean) => void;
    onCancel: () => void;
  } = $props();

  let rememberChoice = $state(false);
  let loading = $state(false);

  function handleMinimize() {
    loading = true;
    onConfirm(true, rememberChoice);
  }

  function handleQuit() {
    loading = true;
    onConfirm(false, rememberChoice);
  }

  function handleClose() {
    if (!loading) {
      onCancel();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      handleClose();
    }
  }
</script>

{#if show}
  <div
    class="fixed inset-0 z-[10000] flex items-center justify-center bg-black/50 backdrop-blur-sm"
    onclick={handleBackdropClick}
    role="presentation"
  >
    <div
      class="relative w-full max-w-md rounded-2xl border border-[var(--border)] bg-[var(--bg-card)] p-6 shadow-[var(--shadow-lg)]"
      role="dialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
    >
      <div class="flex items-start gap-4">
        <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-[var(--warning-glow)]">
          <svg class="h-6 w-6 text-[var(--warning)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <div class="min-w-0 flex-1">
          <h3 id="dialog-title" class="text-lg font-bold text-[var(--text-primary)]">
            关闭窗口
          </h3>
          <p class="mt-2 text-sm text-[var(--text-secondary)]">
            选择关闭方式：最小化到系统托盘继续运行，或退出程序
          </p>
        </div>
      </div>

      <div class="mt-6 space-y-3">
        <button
          onclick={handleMinimize}
          disabled={loading}
          class="flex w-full items-center gap-3 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] p-4 text-left transition-all hover:border-[var(--accent)] hover:bg-[var(--accent-subtle)] disabled:cursor-not-allowed disabled:opacity-50"
        >
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-[var(--accent-subtle)] text-[var(--accent)]">
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
            </svg>
          </div>
          <div class="min-w-0 flex-1">
            <p class="font-semibold text-[var(--text-primary)]">最小化到托盘</p>
            <p class="text-xs text-[var(--text-muted)]">程序将在后台继续运行</p>
          </div>
        </button>

        <button
          onclick={handleQuit}
          disabled={loading}
          class="flex w-full items-center gap-3 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] p-4 text-left transition-all hover:border-[var(--danger)] hover:bg-[var(--danger-glow)] disabled:cursor-not-allowed disabled:opacity-50"
        >
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-[var(--danger-glow)] text-[var(--danger)]">
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </div>
          <div class="min-w-0 flex-1">
            <p class="font-semibold text-[var(--text-primary)]">退出程序</p>
            <p class="text-xs text-[var(--text-muted)]">完全关闭应用程序</p>
          </div>
        </button>
      </div>

      <label class="mt-4 flex items-center gap-2 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={rememberChoice}
          disabled={loading}
          class="h-4 w-4 rounded border-[var(--border)] text-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)] focus:ring-offset-2"
        />
        <span class="text-sm text-[var(--text-secondary)]">记住我的选择，下次不再询问</span>
      </label>

      <div class="mt-6 flex justify-end">
        <Button
          variant="ghost"
          size="sm"
          onclick={handleClose}
          disabled={loading}
        >
          取消
        </Button>
      </div>
    </div>
  </div>
{/if}
