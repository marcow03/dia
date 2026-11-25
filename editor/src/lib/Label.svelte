<script lang="ts">
  import { PT_PER_CM } from './geometry'
  import { renderLabel, type Rendered } from './typst'
  import type { Style } from './types'

  let {
    source,
    style,
    cx,
    cy,
    background = false,
    handle,
  }: {
    source: string
    style: Style
    cx: number
    cy: number
    background?: boolean
    handle?: string
  } = $props()

  let result = $state<Rendered | null>(null)

  $effect(() => {
    let current = true
    renderLabel(source, style).then((r) => current && (result = r))
    return () => (current = false)
  })

  const ok = $derived(result && 'svg' in result ? result : null)
  const w = $derived(ok ? ok.w / PT_PER_CM : 0)
  const h = $derived(ok ? ok.h / PT_PER_CM : 0)
</script>

{#if result && 'error' in result}
  <text x={cx} y={cy} class="error" text-anchor="middle" dominant-baseline="middle">{source}</text>
{:else if ok}
  {#if background}
    <rect x={cx - w / 2 - 0.1} y={cy - h / 2 - 0.06} width={w + 0.2} height={h + 0.12} rx="0.08" fill="var(--canvas)" />
  {/if}
  {#if handle}
    <rect class="grab" data-handle="label:{handle}" x={cx - w / 2 - 0.1} y={cy - h / 2 - 0.06} width={w + 0.2} height={h + 0.12} rx="0.08" />
  {/if}
  <svg x={cx - w / 2} y={cy - h / 2} width={w} height={h} viewBox="0 0 {ok.w} {ok.h}" overflow="visible">
    {@html ok.svg}
  </svg>
{/if}

<style>
  svg,
  text,
  rect {
    pointer-events: none;
  }
  rect.grab {
    fill: transparent;
    stroke: var(--accent);
    stroke-width: 0.025;
    stroke-dasharray: 0.06 0.06;
    pointer-events: all;
    cursor: move;
  }
  .error {
    font: 0.3px ui-monospace, monospace;
    fill: #ff3b30;
  }
</style>
