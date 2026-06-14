<script lang="ts">
  let {
    hover = false,
    padding = 'md',
    class: className = '',
    children,
    header,
    footer
  }: {
    hover?: boolean;
    padding?: 'none' | 'sm' | 'md' | 'lg';
    class?: string;
    children?: any;
    header?: any;
    footer?: any;
  } = $props();

  const baseClasses = "bg-[var(--bg-card)] border border-[var(--border)] rounded-xl shadow-sm transition-all";

  const hoverClasses = hover ? "hover:bg-[var(--bg-card-hover)] hover:border-[var(--border-hover)] hover:shadow-md cursor-pointer" : "";

  const paddingClasses = {
    none: "",
    sm: "p-3",
    md: "p-5",
    lg: "p-6"
  };
</script>

<div class="{baseClasses} {hoverClasses} {paddingClasses[padding]} {className}">
  {#if header}
    <div class="card-header mb-4 pb-4 border-b border-[var(--border)]">
      {@render header()}
    </div>
  {/if}

  <div class="card-body">
    {@render children()}
  </div>

  {#if footer}
    <div class="card-footer mt-4 pt-4 border-t border-[var(--border)]">
      {@render footer()}
    </div>
  {/if}
</div>
