<script lang="ts">
  import { arrowAt, arrowHead, center, labelPoint, linePath, PT_PER_CM } from './geometry'
  import Label from './Label.svelte'
  import { app, fitTextBox } from './state.svelte'
  import type { Element } from './types'

  let { el, scale }: { el: Element; scale: number } = $props()

  const d = $derived(app.diagram)
  const sw = $derived(el.style.width / PT_PER_CM)
  const dash = $derived(
    el.style.dash === 'dashed' ? `${sw * 4} ${sw * 3}` : el.style.dash === 'dotted' ? `0 ${sw * 2.5}` : undefined,
  )
  const cap = $derived(el.style.dash === 'dotted' ? 'round' : 'butt')
  const stroke = $derived(el.style.stroke ?? 'none')
  const fill = $derived(el.style.fill ?? 'none')
  const editing = $derived(app.editing === el.id)
  const handle = $derived(app.selection.length === 1 && app.selection[0] === el.id ? el.id : undefined)
  const lp = $derived(labelPoint(d, el))

  // Text boxes follow their content: label, font, size, weight.
  $effect(() => {
    if (el.type === 'shape' && el.shape === 'text' && el.label && !editing) fitTextBox(el)
  })
</script>

<g data-id={el.id}>
  {#if el.type === 'shape'}
    {@const c = center(el)}
    {@const at = el.label_at && 'dx' in el.label_at ? el.label_at : { dx: 0, dy: 0 }}
    <g transform={el.rotate ? `rotate(${el.rotate} ${c.x} ${c.y})` : undefined}>
    {#if el.shape === 'rect'}
      <rect x={el.x} y={el.y} width={el.w} height={el.h} rx={el.style.radius} {fill} {stroke} stroke-width={sw} stroke-dasharray={dash} stroke-linecap={cap} />
    {:else if el.shape === 'ellipse'}
      <ellipse cx={c.x} cy={c.y} rx={el.w / 2} ry={el.h / 2} {fill} {stroke} stroke-width={sw} stroke-dasharray={dash} stroke-linecap={cap} />
    {:else if el.shape === 'diamond'}
      <polygon
        points="{c.x},{el.y} {el.x + el.w},{c.y} {c.x},{el.y + el.h} {el.x},{c.y}"
        {fill}
        {stroke}
        stroke-width={sw}
        stroke-dasharray={dash}
        stroke-linecap={cap}
      />
    {:else if el.shape === 'image'}
      {#if el.src?.startsWith('data:')}
        <image href={el.src} x={el.x} y={el.y} width={el.w} height={el.h} preserveAspectRatio="none" />
      {:else}
        <rect class="missing" x={el.x} y={el.y} width={el.w} height={el.h} stroke-width={1 / scale} />
        <text class="missing-label" x={c.x} y={c.y} font-size={12 / scale} text-anchor="middle" dominant-baseline="middle">{el.src}</text>
      {/if}
      <rect x={el.x} y={el.y} width={el.w} height={el.h} fill="transparent" />
    {:else}
      <rect x={el.x} y={el.y} width={el.w} height={el.h} fill="transparent" />
    {/if}
    {#if el.label && !editing}
      <Label
        source={el.label}
        style={el.style}
        cx={c.x + at.dx}
        cy={c.y + at.dy}
        handle={el.shape === 'text' ? undefined : handle}
      />
    {/if}
    </g>
  {:else}
    {@const path = linePath(d, el)}
    <path d={path} fill="none" stroke="transparent" stroke-width={12 / scale} class="hit" />
    <path d={path} fill="none" {stroke} stroke-width={sw} stroke-dasharray={dash} stroke-linecap="round" stroke-linejoin="round" />
    {#each [arrowAt(d, el, false), arrowAt(d, el, true)] as a, i}
      {@const head = arrowHead(i ? el.style.end : el.style.start, a.tip, a.from, 0.22 + sw * 3)}
      {#if head.d}
        <path d={head.d} fill={head.filled ? stroke : 'none'} {stroke} stroke-width={sw} stroke-linejoin="round" stroke-linecap="round" />
      {/if}
    {/each}
    {#if el.label && !editing}
      <Label source={el.label} style={el.style} cx={lp.x} cy={lp.y} background {handle} />
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
  image,
  text {
    pointer-events: none;
  }
  .missing {
    fill: var(--subtle);
    stroke: var(--muted);
    stroke-dasharray: 4 4;
  }
  .missing-label {
    fill: var(--muted);
    font-family: var(--mono);
  }
</style>
