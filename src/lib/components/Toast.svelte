<script lang="ts">
  import { toastStore } from "../stores/toast.svelte";

  const toasts = $derived(toastStore.toasts);
</script>

{#if toasts.length > 0}
  <div class="fixed top-14 right-4 z-[9999] flex flex-col gap-2 pointer-events-none">
    {#each toasts as toast (toast.id)}
      <div
        class="pointer-events-auto flex items-center gap-3 rounded-lg border px-4 py-3 text-sm shadow-[var(--shadow-lg)] backdrop-blur-sm animate-slide-in
          {toast.type === 'success'
            ? 'border-[var(--success)]/30 bg-[var(--success)]/10 text-[var(--success)]'
            : toast.type === 'error'
              ? 'border-[var(--danger)]/30 bg-[var(--danger)]/10 text-[var(--danger)]'
              : 'border-[var(--border-accent)] bg-[var(--bg-card)] text-[var(--text-primary)]'}"
      >
        {#if toast.type === 'success'}
          <svg class="h-5 w-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        {:else if toast.type === 'error'}
          <svg class="h-5 w-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        {:else}
          <svg class="h-5 w-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        {/if}
        <span class="font-medium">{toast.message}</span>
        <button
          type="button"
          class="ml-2 shrink-0 opacity-60 hover:opacity-100 transition-opacity"
          aria-label="关闭提示"
          onclick={() => toastStore.dismiss(toast.id)}
        >
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateX(100%) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateX(0) scale(1);
    }
  }

  .animate-slide-in {
    animation: slide-in 0.3s cubic-bezier(0.22, 1, 0.36, 1) forwards;
  }
</style>
