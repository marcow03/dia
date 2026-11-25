<script lang="ts">
  let {
    value,
    allowNone = false,
    onchange,
  }: { value: string | null; allowNone?: boolean; onchange: (v: string | null) => void } = $props()

  const palette = [
    '#000000', '#3a3a3c', '#8e8e93', '#c7c7cc', '#e5e5ea', '#ffffff',
    '#ff3b30', '#ff9500', '#ffcc00', '#34c759', '#00c7be', '#007aff',
    '#5856d6', '#af52de', '#ff2d55', '#a2845e', '#30b0c7', '#1d1d1f',
    '#ffe5e3', '#fff1dc', '#fff8d6', '#e3f7e8', '#e0efff', '#f3e5fa',
  ]

  let open = $state(false)
  const rgb = $derived(value?.slice(0, 7) ?? '#000000')
  const alpha = $derived(value && value.length === 9 ? parseInt(value.slice(7), 16) / 255 : 1)

  function emit(color: string, a: number) {
    onchange(a >= 1 ? color : color + Math.round(a * 255).toString(16).padStart(2, '0'))
  }

  function hexInput(e: Event) {
    const v = (e.target as HTMLInputElement).value.trim()
    if (/^#?[0-9a-f]{6}([0-9a-f]{2})?$/i.test(v)) onchange((v.startsWith('#') ? v : '#' + v).toLowerCase())
  }
</script>

<svelte:window onpointerdown={() => (open = false)} onkeydown={(e) => e.key === 'Escape' && (open = false)} />

<div class="picker" role="presentation" onpointerdown={(e) => e.stopPropagation()}>
  <button class="current" class:none={!value} aria-label="Pick color" onclick={() => (open = !open)}>
    <span style:background={value ?? 'transparent'}></span>
  </button>
  {#if open}
    <div class="pop">
      <div class="grid">
        {#each palette as c}
          <button class="sw" class:on={rgb === c && value} style:--c={c} aria-label={c} onclick={() => emit(c, alpha)}></button>
        {/each}
      </div>
      <div class="row">
        <input type="color" value={rgb} oninput={(e) => emit((e.target as HTMLInputElement).value, alpha)} />
        <input class="hex" value={value ?? ''} placeholder="none" onchange={hexInput} spellcheck="false" />
        {#if allowNone}
          <button class="clear" class:on={!value} onclick={() => onchange(null)}>None</button>
        {/if}
      </div>
      <label class="row">
        <span>Opacity</span>
        <input type="range" min="0" max="1" step="0.05" value={alpha} disabled={!value} oninput={(e) => emit(rgb, +(e.target as HTMLInputElement).value)} />
        <span class="pct">{Math.round(alpha * 100)}%</span>
      </label>
    </div>
  {/if}
</div>

<style>
  .picker {
    position: relative;
  }
  .current {
    min-width: 0;
    width: 28px;
    height: 28px;
    padding: 0;
    border-radius: 8px;
    overflow: hidden;
    background: repeating-conic-gradient(#e5e5ea 0 25%, #fff 0 50%) 0 0 / 8px 8px;
    box-shadow: inset 0 0 0 1px rgb(0 0 0 / 0.12);
  }
  .current span {
    width: 100%;
    height: 100%;
  }
  .current.none span {
    background: linear-gradient(to top right, transparent 46%, #ff3b30 46% 54%, transparent 54%) !important;
  }
  .pop {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 20;
    width: 220px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: var(--panel);
    border-radius: 12px;
    box-shadow: var(--shadow);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 6px;
  }
  .sw {
    min-width: 0;
    height: 24px;
    padding: 0;
    border-radius: 6px;
    background: var(--c);
    box-shadow: inset 0 0 0 1px rgb(0 0 0 / 0.1);
  }
  .sw:hover {
    background: var(--c);
    transform: scale(1.08);
  }
  .sw.on {
    box-shadow: 0 0 0 2px var(--panel), 0 0 0 3.5px var(--accent);
  }
  .row {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--muted);
  }
  input[type='color'] {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    border-radius: 6px;
    background: none;
    cursor: pointer;
  }
  .hex {
    flex: 1;
    min-width: 0;
    height: 28px;
    padding: 0 8px;
    border: none;
    border-radius: 6px;
    background: var(--subtle);
    font: 12px var(--mono);
    color: var(--text);
    outline: none;
  }
  .clear {
    height: 28px;
    padding: 0 8px;
    font-size: 12px;
  }
  input[type='range'] {
    flex: 1;
    min-width: 0;
    width: 100%;
    accent-color: var(--accent);
  }
  .pct {
    width: 34px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
</style>
