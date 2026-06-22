<script lang="ts">
  import SelectCustom from './SelectCustom.svelte';

  let {
    saves = [],
    value = $bindable(""),
    onChange,
    placeholder = "未找到存档",
  }: {
    saves: { id: string; name?: string }[];
    value?: string;
    onChange?: (value: string) => void;
    placeholder?: string;
  } = $props();

  // 转换为 SelectCustom 需要的格式
  const options = $derived(
    saves.map(save => ({
      value: save.id,
      label: save.name ? `${save.id} - ${save.name}` : save.id
    }))
  );
</script>

{#if saves.length === 0}
  <span class="text-sm text-[var(--text-muted)]">{placeholder}</span>
{:else}
  <SelectCustom
    bind:value
    options={options}
    onchange={onChange}
    placeholder="请选择存档"
    size="md"
  />
{/if}
