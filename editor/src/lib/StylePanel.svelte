<script lang="ts">
  import { find } from './geometry'
  import Icon from './Icon.svelte'
  import { app, checkpoint } from './state.svelte'
  import type { Dash, Style } from './types'

  const strokes = ['#000000', '#8e8e93', '#ff3b30', '#ff9500', '#ffcc00', '#34c759', '#007aff', '#af52de']
  const fills = ['#ffffff', '#f2f2f7', '#ffe5e3', '#fff1dc', '#fff8d6', '#e3f7e8', '#e0efff', '#f3e5fa']

  const items = $derived(app.selection.map((id) => find(app.diagram, id)).filter((e) => e !== undefined))
  const first = $derived(items[0])
  const hasShapes = $derived(items.some((e) => e.type === 'shape' && e.shape !== 'text'))
  const hasLines = $derived(items.some((e) => e.type === 'line'))
  const hasRects = $derived(items.some((e) => e.type === 'shape' && e.shape === 'rect'))

  function set<K extends keyof Style>(key: K, value: Style[K]) {
    checkpoint()
    for (const e of items) {
      if (key === 'fill' && e.type === 'line') continue
      if (e.type === 'shape' && e.shape === 'text' && key !== 'fill') continue
      e.style[key] = value
    }
  }

  function toggleCurved() {
    checkpoint()
    for (const e of items) if (e.type === 'line') e.curved = !e.curved
  }
</script>

{#if first && (hasShapes || hasLines)}
  <div class="panel">
    <div class="row">
      <span class="label">Stroke</span>
      <div class="swatches">
        {#each strokes as c}
          <button class="swatch" class:on={first.style.stroke === c} style:--c={c} aria-label={c} onclick={() => set('stroke', c)}></button>
        {/each}
        <button class="swatch none" class:on={first.style.stroke === null} aria-label="no stroke" onclick={() => set('stroke', null)}>
          <Icon name="none" />
        </button>
      </div>
    </div>
    {#if hasShapes}
      <div class="row">
        <span class="label">Fill</span>
        <div class="swatches">
          {#each fills as c}
            <button class="swatch" class:on={first.style.fill === c} style:--c={c} aria-label={c} onclick={() => set('fill', c)}></button>
          {/each}
          <button class="swatch none" class:on={first.style.fill === null} aria-label="no fill" onclick={() => set('fill', null)}>
            <Icon name="none" />
          </button>
        </div>
      </div>
    {/if}
    <div class="row">
      <span class="label">Width</span>
      <div class="seg">
        {#each [[0.5, 'Thin'], [1, 'Regular'], [2, 'Bold']] as const as [w, name]}
          <button class:on={first.style.width === w} onclick={() => set('width', w)}>{name}</button>
        {/each}
      </div>
    </div>
    <div class="row">
      <span class="label">Dash</span>
      <div class="seg">
        {#each ['solid', 'dashed', 'dotted'] as const as dash}
          <button class:on={first.style.dash === dash} onclick={() => set('dash', dash as Dash)}>{dash}</button>
        {/each}
      </div>
    </div>
    {#if hasRects}
      <div class="row">
        <span class="label">Corners</span>
        <div class="seg">
          <button class:on={first.style.radius === 0} onclick={() => set('radius', 0)}>Sharp</button>
          <button class:on={first.style.radius > 0} onclick={() => set('radius', 0.2)}>Rounded</button>
        </div>
      </div>
    {/if}
    {#if hasLines}
      <div class="row">
        <span class="label">Arrows</span>
        <div class="seg">
          <button class:on={first.style.start_arrow} onclick={() => set('start_arrow', !first.style.start_arrow)}>Start</button>
          <button class:on={first.style.end_arrow} onclick={() => set('end_arrow', !first.style.end_arrow)}>End</button>
          <button class:on={first.type === 'line' && first.curved} onclick={toggleCurved}>Curved</button>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .panel {
    position: absolute;
    top: 64px;
    right: 16px;
    width: 232px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: var(--panel);
    border-radius: 14px;
    box-shadow: var(--shadow);
  }
  .row {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .label {
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
  }
  .swatches {
    display: grid;
    grid-template-columns: repeat(9, 1fr);
    gap: 4px;
  }
  .swatch {
    min-width: 0;
    width: 20px;
    height: 20px;
    padding: 0;
    border-radius: 50%;
    background: var(--c);
    box-shadow: inset 0 0 0 1px rgb(0 0 0 / 0.12);
  }
  .swatch.on {
    box-shadow: 0 0 0 2px var(--panel), 0 0 0 3.5px var(--accent);
  }
  .swatch.none {
    background: var(--panel);
    color: var(--muted);
  }
  .swatch.none :global(svg) {
    width: 16px;
    height: 16px;
  }
  .seg {
    display: flex;
    padding: 2px;
    gap: 2px;
    background: var(--subtle);
    border-radius: 8px;
  }
  .seg button {
    flex: 1;
    height: 26px;
    font-size: 12px;
    text-transform: capitalize;
    border-radius: 6px;
  }
  .seg button.on {
    background: var(--panel);
    box-shadow: 0 1px 2px rgb(0 0 0 / 0.12);
  }
</style>
