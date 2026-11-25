<script lang="ts">
  let {
    value,
    onchange,
    step = 1,
    min = 0,
    unit = 'pt',
    label,
  }: { value: number; onchange: (v: number) => void; step?: number; min?: number; unit?: string; label: string } = $props()

  const round = (v: number) => Math.round(v * 100) / 100
  const commit = (v: number) => Number.isFinite(v) && onchange(round(Math.max(min, v)))
</script>

<div class="number">
  <button aria-label="Decrease {label}" onclick={() => commit(value - step)}>−</button>
  <input
    aria-label={label}
    inputmode="decimal"
    value={round(value)}
    onchange={(e) => commit(parseFloat((e.target as HTMLInputElement).value))}
    onkeydown={(e) => {
      e.stopPropagation()
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
    width: 22px;
    height: 28px;
    border-radius: 7px;
    color: var(--muted);
    font-size: 14px;
  }
  button:hover {
    background: var(--line);
    color: var(--text);
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
