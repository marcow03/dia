<script lang="ts">
  import { arrowAt, arrowHead, center, lineMid, linePath, PT_PER_CM } from './geometry'
  import Label from './Label.svelte'
  import { app } from './state.svelte'
  import type { Element } from './types'

  let { el, scale }: { el: Element; scale: number } = $props()

  const d = $derived(app.diagram)
  const sw = $derived(el.style.width / PT_PER_CM)
  const dash = $derived(
    el.style.dash === 'dashed' ? `${sw * 4} ${sw * 3}` : el.style.dash === 'dotted' ? `0 ${sw * 2.5}` : undefined,
  )
  const stroke = $derived(el.style.stroke ?? 'none')
  const fill = $derived(el.style.fill ?? 'none')
</script>

<g data-id={el.id}>
  {#if el.type === 'shape'}
    {@const c = center(el)}
    {#if el.shape === 'rect'}
      <rect x={el.x} y={el.y} width={el.w} height={el.h} rx={el.style.radius} {fill} {stroke} stroke-width={sw} stroke-dasharray={dash} />
    {:else if el.shape === 'ellipse'}
      <ellipse cx={c.x} cy={c.y} rx={el.w / 2} ry={el.h / 2} {fill} {stroke} stroke-width={sw} stroke-dasharray={dash} />
    {:else if el.shape === 'diamond'}
      <polygon
        points="{c.x},{el.y} {el.x + el.w},{c.y} {c.x},{el.y + el.h} {el.x},{c.y}"
        {fill}
        {stroke}
        stroke-width={sw}
        stroke-dasharray={dash}
      />
    {:else}
      <rect x={el.x} y={el.y} width={el.w} height={el.h} fill="transparent" />
    {/if}
    {#if el.label && app.editing !== el.id}
      <Label source={el.label} cx={c.x} cy={c.y} />
    {/if}
  {:else}
    {@const path = linePath(d, el)}
    <path d={path} fill="none" stroke="transparent" stroke-width={12 / scale} class="hit" />
    <path d={path} fill="none" {stroke} stroke-width={sw} stroke-dasharray={dash} stroke-linecap="round" stroke-linejoin="round" />
    {#each [el.style.start_arrow && arrowAt(d, el, false), el.style.end_arrow && arrowAt(d, el, true)] as a}
      {#if a}
        <path d={arrowHead(a.tip, a.from, 0.22 + sw * 3)} fill={stroke} />
      {/if}
    {/each}
    {#if el.label && app.editing !== el.id}
      {@const m = lineMid(d, el)}
      <Label source={el.label} cx={m.x} cy={m.y} background />
    {/if}
  {/if}
</g>

<style>
  .hit {
    cursor: pointer;
  }
  rect,
  ellipse,
  polygon {
    pointer-events: all;
  }
</style>
