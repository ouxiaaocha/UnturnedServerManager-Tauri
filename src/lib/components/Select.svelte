<script lang="ts">
  import type { Snippet } from "svelte";

  /**
   * Select 下拉框组件
   * 统一的下拉选择器样式和行为
   */

  type SelectValue = string | number | boolean | null | undefined;

  let {
    value = $bindable(),
    options = [],
    placeholder = "请选择",
    disabled = false,
    size = 'md',
    fullWidth = false,
    class: className = '',
    onchange,
    children,
    ...props
  }: {
    value?: SelectValue;
    options?: Array<{ value: SelectValue; label: string; disabled?: boolean }>;
    placeholder?: string;
    disabled?: boolean;
    size?: 'sm' | 'md' | 'lg';
    fullWidth?: boolean;
    class?: string;
    onchange?: (e: Event) => void;
    children?: Snippet;
    [key: string]: unknown;
  } = $props();

  const baseClasses = "bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)] transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed appearance-none";

  const sizeClasses = {
    sm: "pl-3 pr-8 py-1.5 text-xs",
    md: "pl-4 pr-9 py-2 text-sm",
    lg: "pl-4 pr-10 py-2.5 text-base"
  };

  const iconSizes = {
    sm: "w-3 h-3 right-2.5",
    md: "w-4 h-4 right-3",
    lg: "w-5 h-5 right-3.5"
  };

  const widthClass = $derived(fullWidth ? "w-full" : "min-w-[200px]");
</script>

<div class="relative inline-block {widthClass}">
  <select
    bind:value
    {disabled}
    class="{baseClasses} {sizeClasses[size]} {widthClass} {className}"
    {onchange}
    {...props}
  >
    {#if children}
      {@render children()}
    {:else if options.length > 0}
      {#if placeholder && !value}
        <option value="" disabled selected>{placeholder}</option>
      {/if}
      {#each options as option}
        <option value={option.value} disabled={option.disabled}>
          {option.label}
        </option>
      {/each}
    {/if}
  </select>

  <!-- 自定义下拉箭头图标 -->
  <div class="absolute top-1/2 -translate-y-1/2 {iconSizes[size]} pointer-events-none text-[var(--text-muted)]">
    <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </div>
</div>

<style>
  /* 隐藏默认的下拉箭头 */
  select {
    background-image: none;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  /* 悬停效果 */
  select:not(:disabled):hover {
    border-color: var(--border-hover);
    background-color: var(--bg-card);
    box-shadow: 0 2px 4px 0 rgba(0, 0, 0, 0.08);
  }

  /* 聚焦效果 */
  select:focus {
    box-shadow: 0 0 0 3px var(--accent-subtle), 0 2px 4px 0 rgba(0, 0, 0, 0.08);
  }

  /* 下拉菜单容器样式 */
  select option {
    background-color: var(--bg-card);
    color: var(--text-primary);
    padding: 0.75rem 1rem;
    margin: 0.125rem 0.25rem;
    border-radius: 0.375rem;
    cursor: pointer;
    transition: all 0.15s ease;
    font-size: inherit;
    line-height: 1.5;
  }

  /* 选中的 option 样式 */
  select option:checked,
  select option:checked:hover {
    background: linear-gradient(135deg, var(--accent-subtle) 0%, var(--accent-subtle) 100%);
    color: var(--accent-light);
    font-weight: 500;
    position: relative;
  }

  /* 悬停 option 样式 */
  select option:hover {
    background-color: var(--bg-hover, var(--bg-secondary));
    color: var(--text-primary);
  }

  /* 禁用的 option 样式 */
  select option:disabled {
    color: var(--text-muted);
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* 禁用状态 */
  select:disabled {
    background-color: var(--bg-secondary);
    cursor: not-allowed;
  }

  /* 箭头图标动画 */
  div svg {
    transition: transform 0.2s ease;
  }

  select:focus + div svg {
    transform: translateY(-50%) rotate(180deg);
  }

  /* Firefox 特定样式 */
  @-moz-document url-prefix() {
    select option {
      padding: 0.5rem 0.75rem;
    }
  }

  /* Webkit 浏览器下拉菜单美化 */
  @supports (-webkit-appearance: none) {
    select {
      -webkit-appearance: none;
    }

    /* 自定义滚动条 */
    select::-webkit-scrollbar {
      width: 8px;
    }

    select::-webkit-scrollbar-track {
      background: var(--bg-primary);
      border-radius: 4px;
    }

    select::-webkit-scrollbar-thumb {
      background: var(--border);
      border-radius: 4px;
    }

    select::-webkit-scrollbar-thumb:hover {
      background: var(--border-hover);
    }
  }
</style>
