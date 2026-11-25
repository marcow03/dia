<script lang="ts">
  let {
    value,
    onchange,
    step = 1,
    min = 0,
    unit = 'pt',
    prefix,
    label,
  }: {
    value: number
    onchange: (v: number) => void
    step?: number
    min?: number
    unit?: string
    prefix?: string
    label: string
  } = $props()

  const round = (v: number) => Math.round(v * 100) / 100
  const commit = (v: number) => Number.isFinite(v) && onchange(round(Math.max(min, v)))
</script>

<div class="number" title={label}>
  {#if prefix}<span class="prefix">{prefix}</span>{/if}
  <button aria-label="Decrease {label}" onclick={() => commit(value - step)}>−</button>
  <input
    aria-label={label}
    inputmode="decimal"
    value={round(value)}
    onchange={(e) => commit(parseFloat((e.target as HTMLInputElement).value))}
    onkeydown={(e) => {
      if (e.key !== 'Escape') e.stopPropagation()
      if (e.key === 'Enter') (e.target as HTMLInputElement).blur()
      if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
        e.preventDefault()
        commit(value + (e.key === 'ArrowUp' ? step : -step))
      }
    }}
  />
  <span>{unit}</span>
  <button aria-label="Increase {label}" onclick={() => commit(value + step)}>+</button>
</div>

<style>
  .number {
    display: flex;
    align-items: center;
    height: 28px;
    margin-left: auto;
    border-radius: 7px;
    background: var(--subtle);
    font-size: 11px;
    color: var(--muted);
  }
  button {
    min-width: 0;
    width: 18px;
    height: 18px;
    margin: 0 3px;
    border-radius: 5px;
    color: var(--muted);
    font-size: 13px;
  }
  button:hover {
    background: var(--line);
    color: var(--text);
  }
  .prefix {
    padding-left: 8px;
    font-weight: 600;
  }
  input {
    width: 30px;
    border: none;
    background: transparent;
    font: 12px var(--font);
    color: var(--text);
    text-align: right;
    outline: none;
  }
</style>
