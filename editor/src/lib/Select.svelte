<script lang="ts" generics="T extends string">
  import type { Snippet } from 'svelte'
  import Icon from './Icon.svelte'

  let {
    value,
    options,
    onchange,
    label,
    item,
  }: {
    value: T
    options: { value: T; label: string }[]
    onchange: (v: T) => void
    label: string
    item?: Snippet<[T, string]>
  } = $props()

  let open = $state(false)
  const current = $derived(options.find((o) => o.value === value) ?? options[0])
</script>

<svelte:window onpointerdown={() => (open = false)} />

<div class="select" role="presentation" onpointerdown={(e) => e.stopPropagation()}>
  <button class="trigger" class:open aria-label={label} aria-haspopup="listbox" onclick={() => (open = !open)}>
    <span class="value">
      {#if item}{@render item(current.value, current.label)}{:else}{current.label}{/if}
    </span>
    <span class="chev"><Icon name="chevron" /></span>
  </button>
  {#if open}
    <div class="list" role="listbox">
      {#each options as o}
        <button
          role="option"
          aria-selected={o.value === value}
          class:on={o.value === value}
          onclick={() => {
            onchange(o.value)
            open = false
          }}
        >
          {#if item}{@render item(o.value, o.label)}{:else}{o.label}{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .select {
    position: relative;
    flex: 1;
    min-width: 0;
  }
  .trigger {
    width: 100%;
    height: 28px;
    padding: 0 6px 0 8px;
    justify-content: space-between;
    gap: 4px;
    border-radius: 7px;
    background: var(--subtle);
    font-size: 12px;
  }
  .chev {
    display: flex;
    flex: none;
    color: var(--muted);
    transform: rotate(90deg);
  }
  .chev :global(svg) {
    width: 14px;
  }
  .value {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .list {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    z-index: 25;
    padding: 4px;
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border-radius: 10px;
    box-shadow: var(--shadow);
  }
  .list button {
    justify-content: flex-start;
    gap: 6px;
    height: 28px;
    padding: 0 8px;
    font-size: 12px;
    text-transform: capitalize;
    border-radius: 6px;
  }
  .list button.on {
    background: var(--accent);
    color: white;
  }
</style>
