<script lang="ts">
  let {
    variant = 'primary',
    size = 'md',
    disabled = false,
    loading = false,
    type = 'button',
    class: className = '',
    onclick,
    children,
    ...props
  }: {
    variant?: 'primary' | 'secondary' | 'danger' | 'success' | 'warning' | 'ghost';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    loading?: boolean;
    type?: 'button' | 'submit' | 'reset';
    class?: string;
    onclick?: (e: MouseEvent) => void;
    children?: any;
    [key: string]: any;
  } = $props();

  const baseClasses = "inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none";

  const variantClasses = {
    primary: "bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-white shadow-sm hover:shadow-md",
    secondary: "bg-[var(--bg-card)] border border-[var(--border)] hover:bg-[var(--bg-card-hover)] hover:border-[var(--border-hover)] text-[var(--text-primary)]",
    danger: "bg-[var(--danger)] hover:bg-[#c23939] text-white shadow-sm",
    success: "bg-[var(--success)] hover:bg-[#128754] text-white shadow-sm",
    warning: "bg-[var(--warning)] hover:bg-[#b36d0f] text-white shadow-sm",
    ghost: "hover:bg-[var(--bg-elevated)] text-[var(--text-primary)]"
  };

  const sizeClasses = {
    sm: "px-3 py-1.5 text-xs",
    md: "px-4 py-2 text-sm",
    lg: "px-6 py-3 text-base"
  };
</script>

<button
  {type}
  disabled={disabled || loading}
  class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}"
  {onclick}
  {...props}
>
  {#if loading}
    <svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" aria-hidden="true">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
    <span class="sr-only">加载中</span>
  {/if}
  {@render children?.()}
</button>
