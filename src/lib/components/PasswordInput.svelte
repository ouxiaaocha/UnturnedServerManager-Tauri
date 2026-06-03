<script lang="ts">
  import { generatePassword } from "$lib/utils";

  let {
    value = $bindable(""),
    placeholder = "输入密码",
    label = "密码",
    showGenerate = true,
  }: {
    value?: string;
    placeholder?: string;
    label?: string;
    showGenerate?: boolean;
  } = $props();

  let showPassword = $state(false);

  function handleGenerate() {
    value = generatePassword();
  }
</script>

<div>
  {#if label}
    <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">{label}</span>
  {/if}
  <div class="relative">
    <input
      type={showPassword ? "text" : "password"}
      bind:value
      {placeholder}
      class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 pr-20 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
    />
    <div class="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
      <button
        type="button"
        class="p-1.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
        onclick={() => showPassword = !showPassword}
        aria-label={showPassword ? "隐藏密码" : "显示密码"}
      >
        {#if showPassword}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.542 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
          </svg>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
          </svg>
        {/if}
      </button>
      {#if showGenerate}
        <button
          type="button"
          class="p-1.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
          onclick={handleGenerate}
          title="生成随机密码"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
          </svg>
        </button>
      {/if}
    </div>
  </div>
</div>
