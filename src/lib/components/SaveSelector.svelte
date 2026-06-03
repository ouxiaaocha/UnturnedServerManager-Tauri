<script lang="ts">
  let {
    saves = [],
    value = $bindable(""),
    onChange,
    placeholder = "未找到存档",
  }: {
    saves: { id: string; name?: string }[];
    value?: string;
    onChange?: () => void;
    placeholder?: string;
  } = $props();
</script>

{#if saves.length === 0}
  <span class="text-sm text-[var(--text-muted)]">{placeholder}</span>
{:else}
  <select
    bind:value
    onchange={onChange}
    class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors cursor-pointer min-w-[200px]"
  >
    {#each saves as save (save.id)}
      <option value={save.id}>{save.id}{save.name ? ` - ${save.name}` : ''}</option>
    {/each}
  </select>
{/if}
