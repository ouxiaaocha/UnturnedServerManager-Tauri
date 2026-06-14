<script lang="ts">
  /**
   * 自定义下拉框组件 - 完全自定义的下拉菜单
   * 提供更丰富的视觉效果和更好的用户体验
   */

  import { onMount } from 'svelte';

  let {
    value = $bindable(),
    options = [],
    placeholder = "请选择",
    disabled = false,
    size = 'md',
    fullWidth = false,
    class: className = '',
    onchange,
  }: {
    value?: any;
    options: Array<{ value: any; label: string; disabled?: boolean }>;
    placeholder?: string;
    disabled?: boolean;
    size?: 'sm' | 'md' | 'lg';
    fullWidth?: boolean;
    class?: string;
    onchange?: (value: any) => void;
  } = $props();

  let isOpen = $state(false);
  let dropdownRef: HTMLDivElement | null = $state(null);
  let buttonRef: HTMLButtonElement | null = $state(null);

  const sizeClasses = {
    sm: "text-xs py-1.5 px-3",
    md: "text-sm py-2 px-4",
    lg: "text-base py-2.5 px-4"
  };

  const iconSizes = {
    sm: "w-3 h-3",
    md: "w-4 h-4",
    lg: "w-5 h-5"
  };

  const selectedOption = $derived(
    options.find(opt => opt.value === value)
  );

  function handleSelect(optionValue: any) {
    if (disabled) return;
    value = optionValue;
    isOpen = false;
    onchange?.(optionValue);
  }

  function toggleDropdown() {
    if (!disabled) {
      isOpen = !isOpen;
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (
      dropdownRef &&
      buttonRef &&
      !dropdownRef.contains(event.target as Node) &&
      !buttonRef.contains(event.target as Node)
    ) {
      isOpen = false;
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  });
</script>

<div class="relative inline-block {fullWidth ? 'w-full' : 'min-w-[200px]'} {className}">
  <!-- 选择按钮 -->
  <button
    bind:this={buttonRef}
    type="button"
    onclick={toggleDropdown}
    disabled={disabled}
    class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)] transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-between gap-2 {sizeClasses[size]} {isOpen ? 'border-[var(--accent)] ring-2 ring-[var(--accent)]/20' : ''}"
    style="box-shadow: {isOpen ? '0 0 0 3px var(--accent-subtle), 0 2px 4px 0 rgba(0, 0, 0, 0.08)' : '0 1px 2px 0 rgba(0, 0, 0, 0.05)'}"
  >
    <span class="flex-1 text-left truncate {!selectedOption ? 'text-[var(--text-muted)]' : ''}">
      {selectedOption ? selectedOption.label : placeholder}
    </span>
    <svg
      class="{iconSizes[size]} text-[var(--text-muted)] transition-transform {isOpen ? 'rotate-180' : ''}"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  <!-- 下拉菜单 -->
  {#if isOpen}
    <div
      bind:this={dropdownRef}
      class="absolute z-50 w-full mt-1 bg-[var(--bg-card)] border border-[var(--border)] rounded-lg shadow-lg overflow-hidden animate-slideDown"
      style="box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.08);"
    >
      <div class="max-h-60 overflow-y-auto py-1 custom-scrollbar">
        {#each options as option (option.value)}
          <button
            type="button"
            onclick={() => handleSelect(option.value)}
            disabled={option.disabled}
            class="w-full text-left px-3 py-2 text-sm transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 {value === option.value ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] font-medium' : 'text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
          >
            <!-- 选中标记 -->
            {#if value === option.value}
              <svg class="w-4 h-4 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            {:else}
              <div class="w-4 h-4 flex-shrink-0"></div>
            {/if}
            <span class="flex-1 truncate">{option.label}</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  /* 下拉动画 */
  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .animate-slideDown {
    animation: slideDown 0.15s ease-out;
  }

  /* 自定义滚动条 */
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }

  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }

  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
  }

  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: var(--border-hover);
  }

  /* Firefox 滚动条 */
  .custom-scrollbar {
    scrollbar-width: thin;
    scrollbar-color: var(--border) transparent;
  }
</style>
