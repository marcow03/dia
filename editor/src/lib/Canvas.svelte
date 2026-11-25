<script lang="ts">
  import ElementView from './ElementView.svelte'
  import { bounds, find, linePoints, PX_PER_CM, shapeAt, snap } from './geometry'
  import { openFiles, readImage } from './files'
  import LabelEditor from './LabelEditor.svelte'
  import { addLine, addShape, app, checkpoint, remove } from './state.svelte'
  import { isNode, type Element, type End, type Point, type Shape } from './types'

  type Drag =
    | { kind: 'pan'; sx: number; sy: number; vx: number; vy: number }
    | { kind: 'move'; start: Point; orig: Element[] }
    | { kind: 'resize'; id: string; corner: string; orig: Shape }
    | { kind: 'create'; id: string; start: Point }
    | { kind: 'end'; id: string; which: 'from' | 'to' }
    | { kind: 'via'; id: string; index: number }
    | { kind: 'marquee'; start: Point; now: Point; additive: string[] }

  let svg: SVGSVGElement
  let drag = $state<Drag | null>(null)
  let hover = $state<string | null>(null)
  let spaceDown = $state(false)
  // Pointer capture retargets dblclick to the svg, so remember what the press hit.
  let pressedId: string | undefined

  const scale = $derived(app.view.zoom * PX_PER_CM)
  const d = $derived(app.diagram)
  const selected = $derived(app.selection.map((id) => find(d, id)).filter((e) => e !== undefined))
  const handle = $derived(8 / scale)

  function world(ev: { clientX: number; clientY: number }): Point {
    const r = svg.getBoundingClientRect()
    return { x: (ev.clientX - r.left - app.view.x) / scale, y: (ev.clientY - r.top - app.view.y) / scale }
  }

  const snapP = (p: Point) => ({ x: snap(p.x), y: snap(p.y) })

  function endAt(p: Point, exclude?: string): End {
    const s = shapeAt(d, p, exclude)
    return s && s.shape !== 'text' ? { node: s.id } : snapP(p)
  }

  function down(ev: PointerEvent) {
    if (app.editing) return
    svg.setPointerCapture(ev.pointerId)
    const p = world(ev)
    if (ev.button === 1 || spaceDown) {
      drag = { kind: 'pan', sx: ev.clientX, sy: ev.clientY, vx: app.view.x, vy: app.view.y }
      return
    }
    if (ev.button !== 0) return
    const target = ev.target as globalThis.Element
    const handleEl = target.closest?.('[data-handle]') as HTMLElement | null
    const id = (target.closest?.('[data-id]') as HTMLElement | null)?.dataset.id
    pressedId = id

    if (app.tool === 'rect' || app.tool === 'ellipse' || app.tool === 'diamond') {
      checkpoint()
      const s = snapP(p)
      const nid = addShape(app.tool, s.x, s.y, 0, 0)
      app.selection = [nid]
      drag = { kind: 'create', id: nid, start: s }
      return
    }
    if (app.tool === 'text') {
      checkpoint()
      const s = snapP(p)
      const nid = addShape('text', s.x - 1, s.y - 0.4, 2, 0.8)
      app.selection = [nid]
      app.editing = nid
      app.tool = 'select'
      return
    }
    if (app.tool === 'line' || app.tool === 'arrow') {
      checkpoint()
      const from = endAt(p)
      const nid = addLine(isNode(from) ? from.node : from, snapP(p), app.tool)
      app.selection = [nid]
      drag = { kind: 'end', id: nid, which: 'to' }
      return
    }

    if (handleEl) {
      const [kind, hid, arg] = handleEl.dataset.handle!.split(':')
      const el = find(d, hid)!
      checkpoint()
      if (kind === 'resize' && el.type === 'shape') drag = { kind: 'resize', id: hid, corner: arg, orig: { ...el } }
      else if (kind === 'end') drag = { kind: 'end', id: hid, which: arg as 'from' | 'to' }
      else if (kind === 'via') drag = { kind: 'via', id: hid, index: +arg }
      else if (kind === 'add' && el.type === 'line') {
        el.via.splice(+arg, 0, snapP(p))
        drag = { kind: 'via', id: hid, index: +arg }
      }
      return
    }
    if (id) {
      if (ev.shiftKey) {
        app.selection = app.selection.includes(id) ? app.selection.filter((s) => s !== id) : [...app.selection, id]
      } else if (!app.selection.includes(id)) {
        app.selection = [id]
      }
      checkpoint()
      drag = { kind: 'move', start: p, orig: $state.snapshot(selected) as Element[] }
      return
    }
    drag = { kind: 'marquee', start: p, now: p, additive: ev.shiftKey ? app.selection : [] }
    if (!ev.shiftKey) app.selection = []
  }

  function move(ev: PointerEvent) {
    const p = world(ev)
    if (!drag) {
      hover = app.tool === 'line' || app.tool === 'arrow' ? (shapeAt(d, p)?.id ?? null) : null
      return
    }
    switch (drag.kind) {
      case 'pan':
        app.view.x = drag.vx + ev.clientX - drag.sx
        app.view.y = drag.vy + ev.clientY - drag.sy
        break
      case 'move': {
        const dx = snap(p.x - drag.start.x)
        const dy = snap(p.y - drag.start.y)
        const shift = (q: Point) => ({ x: q.x + dx, y: q.y + dy })
        for (const o of drag.orig) {
          const el = find(d, o.id)
          if (!el) continue
          if (el.type === 'shape' && o.type === 'shape') {
            el.x = o.x + dx
            el.y = o.y + dy
          } else if (el.type === 'line' && o.type === 'line') {
            el.via = o.via.map(shift)
            if (!isNode(o.from)) el.from = shift(o.from)
            if (!isNode(o.to)) el.to = shift(o.to)
          }
        }
        break
      }
      case 'resize': {
        const el = find(d, drag.id) as Shape
        const o = drag.orig
        const q = snapP(p)
        const fx = drag.corner.includes('w') ? o.x + o.w : o.x
        const fy = drag.corner.includes('n') ? o.y + o.h : o.y
        el.x = Math.min(fx, q.x)
        el.y = Math.min(fy, q.y)
        el.w = Math.max(Math.abs(q.x - fx), 0.25)
        el.h = Math.max(Math.abs(q.y - fy), 0.25)
        if (el.shape === 'image') {
          el.h = (el.w * o.h) / o.w
          if (drag.corner.includes('n')) el.y = fy - el.h
        }
        break
      }
      case 'create': {
        const el = find(d, drag.id) as Shape
        const q = snapP(p)
        el.x = Math.min(drag.start.x, q.x)
        el.y = Math.min(drag.start.y, q.y)
        el.w = Math.abs(q.x - drag.start.x)
        el.h = Math.abs(q.y - drag.start.y)
        break
      }
      case 'end': {
        const el = find(d, drag.id)
        if (el?.type !== 'line') break
        const other = el[drag.which === 'to' ? 'from' : 'to']
        const end = endAt(p, isNode(other) ? other.node : undefined)
        el[drag.which] = end
        hover = isNode(end) ? end.node : null
        break
      }
      case 'via': {
        const el = find(d, drag.id)
        if (el?.type === 'line') el.via[drag.index] = snapP(p)
        break
      }
      case 'marquee': {
        drag.now = p
        const r = rectOf(drag.start, p)
        const inside = d.elements
          .filter((e) => {
            const b = bounds(d, e)
            return b.x >= r.x && b.y >= r.y && b.x + b.w <= r.x + r.w && b.y + b.h <= r.y + r.h
          })
          .map((e) => e.id)
        app.selection = [...new Set([...drag.additive, ...inside])]
      }
    }
  }

  function up() {
    if (drag?.kind === 'create') {
      const el = find(d, drag.id) as Shape
      if (el.w < 0.5 || el.h < 0.5) {
        el.w = 3
        el.h = el.shape === 'ellipse' || el.shape === 'diamond' ? 2 : 1.5
        el.x = drag.start.x - el.w / 2
        el.y = drag.start.y - el.h / 2
      }
      app.tool = 'select'
    }
    if (drag?.kind === 'end' && (app.tool === 'line' || app.tool === 'arrow')) {
      const el = find(d, drag.id)
      if (el?.type === 'line') {
        const [a, b] = linePoints(d, el)
        if (Math.hypot(a.x - b.x, a.y - b.y) < 0.2 && !(isNode(el.from) && isNode(el.to))) remove([el.id])
      }
      app.tool = 'select'
    }
    drag = null
    hover = null
  }

  function dblclick(ev: MouseEvent) {
    const id = pressedId
    if (id && app.tool === 'select') {
      app.selection = [id]
      app.editing = id
      return
    }
    checkpoint()
    const p = snapP(world(ev))
    const nid = addShape('text', p.x - 1, p.y - 0.4, 2, 0.8)
    app.selection = [nid]
    app.editing = nid
  }

  function wheel(ev: WheelEvent) {
    ev.preventDefault()
    if (ev.ctrlKey || ev.metaKey) {
      const r = svg.getBoundingClientRect()
      const [mx, my] = [ev.clientX - r.left, ev.clientY - r.top]
      const zoom = Math.min(8, Math.max(0.1, app.view.zoom * Math.exp(-ev.deltaY * 0.01)))
      const k = zoom / app.view.zoom
      app.view = { x: mx - (mx - app.view.x) * k, y: my - (my - app.view.y) * k, zoom }
    } else {
      app.view.x -= ev.deltaX
      app.view.y -= ev.deltaY
    }
  }

  function rectOf(a: Point, b: Point) {
    return { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), w: Math.abs(a.x - b.x), h: Math.abs(a.y - b.y) }
  }

  async function drop(ev: DragEvent) {
    ev.preventDefault()
    const files = [...(ev.dataTransfer?.files ?? [])]
    const images = files.filter((f) => f.type.startsWith('image/'))
    if (!images.length) return files.length && openFiles(files)
    checkpoint()
    let p = world(ev)
    for (const file of images) {
      const { src, w, h } = await readImage(file)
      app.selection = [addShape('image', snap(p.x - w / 2), snap(p.y - h / 2), w, h, src)]
      p = { x: p.x + 0.5, y: p.y + 0.5 }
    }
  }

  function keys(ev: KeyboardEvent, isDown: boolean) {
    if (ev.code === 'Space' && !(ev.target instanceof HTMLTextAreaElement || ev.target instanceof HTMLInputElement))
      spaceDown = isDown
  }

  const cursor = $derived(
    drag?.kind === 'pan' || spaceDown ? 'grabbing' : app.tool === 'select' ? 'default' : app.tool === 'text' ? 'text' : 'crosshair',
  )
  const grid = $derived(scale * 0.5)
</script>

<svelte:window onkeydown={(e) => keys(e, true)} onkeyup={(e) => keys(e, false)} />

<svg
  bind:this={svg}
  class="canvas"
  style:cursor
  role="application"
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  ondblclick={dblclick}
  onwheel={wheel}
  ondragover={(e) => e.preventDefault()}
  ondrop={drop}
>
  <defs>
    <pattern id="grid" width={grid} height={grid} patternUnits="userSpaceOnUse" x={app.view.x} y={app.view.y}>
      <circle cx="1" cy="1" r="1" fill="var(--grid)" />
    </pattern>
  </defs>
  <rect width="100%" height="100%" fill="url(#grid)" />
  <g transform="translate({app.view.x} {app.view.y}) scale({scale})">
    {#each d.elements as el (el.id)}
      <ElementView {el} {scale} />
    {/each}

    {#if hover}
      {@const h = find(d, hover)}
      {#if h?.type === 'shape'}
        <rect class="hover" x={h.x} y={h.y} width={h.w} height={h.h} rx={0.1} stroke-width={3 / scale} />
      {/if}
    {/if}

    {#each selected as el (el.id)}
      {#if el.type === 'shape'}
        <rect class="sel" x={el.x} y={el.y} width={el.w} height={el.h} stroke-width={1.5 / scale} />
        {#if selected.length === 1 && !app.editing}
          {#each [['nw', el.x, el.y], ['ne', el.x + el.w, el.y], ['sw', el.x, el.y + el.h], ['se', el.x + el.w, el.y + el.h]] as [c, x, y]}
            <rect
              class="handle"
              data-handle="resize:{el.id}:{c}"
              style:cursor={c === 'nw' || c === 'se' ? 'nwse-resize' : 'nesw-resize'}
              x={+x - handle / 2}
              y={+y - handle / 2}
              width={handle}
              height={handle}
              rx={handle / 4}
              stroke-width={1.5 / scale}
            />
          {/each}
        {/if}
      {:else}
        {@const pts = linePoints(d, el)}
        {#if selected.length === 1 && !app.editing}
          {#if !el.curved || el.via.length === 0}
            {#each pts.slice(1) as q, i}
              <circle
                class="add"
                data-handle="add:{el.id}:{i}"
                cx={(q.x + pts[i].x) / 2}
                cy={(q.y + pts[i].y) / 2}
                r={handle / 2.5}
                stroke-width={1.5 / scale}
              />
            {/each}
          {/if}
          {#each el.via as q, i}
            <circle class="handle" data-handle="via:{el.id}:{i}" cx={q.x} cy={q.y} r={handle / 2} stroke-width={1.5 / scale} />
          {/each}
          {#each [['from', pts[0]], ['to', pts[pts.length - 1]]] as const as [which, q]}
            <circle class="handle" data-handle="end:{el.id}:{which}" cx={q.x} cy={q.y} r={handle / 1.6} stroke-width={1.5 / scale} />
          {/each}
        {:else}
          {#each [pts[0], pts[pts.length - 1]] as q}
            <circle class="dot" cx={q.x} cy={q.y} r={handle / 3} />
          {/each}
        {/if}
      {/if}
    {/each}

    {#if drag?.kind === 'marquee'}
      {@const r = rectOf(drag.start, drag.now)}
      <rect class="marquee" x={r.x} y={r.y} width={r.w} height={r.h} stroke-width={1 / scale} />
    {/if}
  </g>
</svg>

{#if app.editing}
  {#key app.editing}
    <LabelEditor id={app.editing} {scale} />
  {/key}
{/if}

<style>
  .canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    background: var(--canvas);
    touch-action: none;
    user-select: none;
  }
  .sel {
    fill: none;
    stroke: var(--accent);
    pointer-events: none;
  }
  .hover {
    fill: none;
    stroke: var(--accent);
    stroke-opacity: 0.35;
    pointer-events: none;
  }
  .handle {
    fill: white;
    stroke: var(--accent);
  }
  .add {
    fill: var(--accent);
    fill-opacity: 0.25;
    stroke: var(--accent);
    cursor: copy;
  }
  .dot {
    fill: var(--accent);
    pointer-events: none;
  }
  .marquee {
    fill: var(--accent);
    fill-opacity: 0.06;
    stroke: var(--accent);
  }
</style>
