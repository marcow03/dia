<script lang="ts">
  import ElementView from './ElementView.svelte'
  import { anchorPoint, bounds, center, find, labelPoint, pickShape, shapesAt, lineProject, linePoints, PX_PER_CM, rotatePoint, shapeAt, snap } from './geometry'
  import { openFiles, readImage } from './files'
  import LabelEditor from './LabelEditor.svelte'
  import { freezeRoute } from './router'
  import { addLine, addShape, app, checkpoint, expandGroups, remove, rotateElements, zoomBy } from './state.svelte'
  import { anchors, isNode, type Anchor, type Element, type End, type Point, type Shape } from './types'

  type Drag =
    | { kind: 'pan'; sx: number; sy: number; vx: number; vy: number }
    | { kind: 'move'; start: Point; orig: Element[] }
    | { kind: 'resize'; id: string; corner: string; orig: Shape }
    | { kind: 'create'; id: string; start: Point }
    | { kind: 'end'; id: string; which: 'from' | 'to' }
    | { kind: 'via'; id: string; index: number }
    | { kind: 'label'; id: string }
    | { kind: 'add'; id: string; index: number; start: Point }
    | { kind: 'marquee'; start: Point; now: Point; additive: string[] }
    | { kind: 'rotate'; pivot: Point; start: number; orig: Element[] }

  let svg: SVGSVGElement
  let drag = $state<Drag | null>(null)
  let hover = $state<string | null>(null)
  let hoverAnchor = $state<Anchor | null>(null)
  let guides = $state<{ x1: number; y1: number; x2: number; y2: number }[]>([])
  /** Aspect ratio the size snapped to, shown as a diagonal across the shape. */
  let diagonal = $state<number | null>(null)
  let sizeMarks = $state<{ x: number; y: number; w: number; h: number; dim: 'w' | 'h' }[]>([])
  let spaceDown = $state(false)
  let shiftDown = $state(false)
  // Pointer capture retargets dblclick to the svg, so remember what the press hit.
  let pressedId: string | undefined
  let deselectOnClick: string[] | null = null

  const scale = $derived(app.view.zoom * PX_PER_CM)
  const d = $derived(app.diagram)
  const selected = $derived(app.selection.map((id) => find(d, id)).filter((e) => e !== undefined))
  const handle = $derived(8 / scale)
  const single = $derived(selected.length === 1 && selected[0].type === 'shape' ? selected[0] : null)
  /** Bounding box of the whole selection. */
  const selBox = $derived.by(() => {
    if (!selected.length) return null
    const bs = selected.map((e) => bounds(d, e))
    const x = Math.min(...bs.map((b) => b.x))
    const y = Math.min(...bs.map((b) => b.y))
    return { x, y, w: Math.max(...bs.map((b) => b.x + b.w)) - x, h: Math.max(...bs.map((b) => b.y + b.h)) - y }
  })
  const groupBoxes = $derived.by(() => {
    const groups = [...new Set(selected.map((e) => e.group).filter((g) => g))]
    return groups.map((g) => {
      const bs = d.elements.filter((e) => e.group === g).map((e) => bounds(d, e))
      const x = Math.min(...bs.map((b) => b.x)) - 0.15
      const y = Math.min(...bs.map((b) => b.y)) - 0.15
      return { x, y, w: Math.max(...bs.map((b) => b.x + b.w)) + 0.15 - x, h: Math.max(...bs.map((b) => b.y + b.h)) + 0.15 - y }
    })
  })
  const angleOf = (p: Point, c: Point) => (Math.atan2(p.y - c.y, p.x - c.x) * 180) / Math.PI

  function world(ev: { clientX: number; clientY: number }): Point {
    const r = svg.getBoundingClientRect()
    return { x: (ev.clientX - r.left - app.view.x) / scale, y: (ev.clientY - r.top - app.view.y) / scale }
  }

  const snapP = (p: Point) => ({ x: snap(p.x), y: snap(p.y) })

  /** Anchor within reach of `p` (10 screen px), on any shape but `exclude`. */
  function anchorAt(p: Point, exclude?: string): { node: string; anchor: Anchor } | null {
    let best: { node: string; anchor: Anchor; dist: number } | null = null
    for (const e of d.elements) {
      if (e.type !== 'shape' || e.shape === 'text' || e.id === exclude) continue
      for (const a of anchors) {
        const q = anchorPoint(e, a)
        const dist = Math.hypot(q.x - p.x, q.y - p.y)
        if (dist < 10 / scale && (!best || dist < best.dist)) best = { node: e.id, anchor: a, dist }
      }
    }
    return best && { node: best.node, anchor: best.anchor }
  }

  /** A line end at `p`: snapped to a nearby anchor, bound to the shape under it, or a free point. */
  function endAt(p: Point, exclude?: string, free = false): End {
    // ⇧ places the end exactly at the pointer: no anchors, no binding, no grid (also inside shapes).
    if (free) return p
    const a = anchorAt(p, exclude)
    if (a) return a
    const s = shapeAt(d, p, exclude)
    return s && s.shape !== 'text' ? { node: s.id } : snapP(p)
  }

  function down(ev: PointerEvent) {
    if (app.editing) return
    svg.setPointerCapture(ev.pointerId)
    const p = world(ev)
    const target = ev.target as globalThis.Element
    const handleEl = target.closest?.('[data-handle]') as HTMLElement | null
    let id = (target.closest?.('[data-id]') as HTMLElement | null)?.dataset.id
    // Lines are hit by their stroke; for shapes decide geometrically so hollow shapes don't block.
    if (id && find(d, id)?.type === 'shape') id = pickShape(d, p, 6 / scale)?.id ?? id
    // ⌥-click selects the next object below the pointer, cycling through the stack.
    if (ev.altKey && app.tool === 'select') {
      const stack = d.elements.filter((e) => e.type === 'shape' && shapesAt(d, p).includes(e)).reverse()
      const at = stack.findIndex((e) => app.selection.includes(e.id))
      if (stack.length) id = stack[(at + 1) % stack.length].id
    }
    // ⇧ on empty canvas pans like the hand tool.
    const shiftPan = ev.shiftKey && app.tool === 'select' && !id && !handleEl
    if (ev.button === 1 || spaceDown || app.tool === 'hand' || shiftPan) {
      drag = { kind: 'pan', sx: ev.clientX, sy: ev.clientY, vx: app.view.x, vy: app.view.y }
      return
    }
    if (ev.button !== 0) return
    pressedId = id ?? handleEl?.dataset.handle?.split(':')[1]

    if (app.tool === 'rect' || app.tool === 'ellipse' || app.tool === 'diamond') {
      checkpoint()
      const s = snapP(p)
      const nid = addShape(app.tool, s.x, s.y, 0, 0)
      app.selection = [nid]
      drag = { kind: 'create', id: nid, start: s }
      return
    }
    if (app.tool === 'text') {
      // Keep the browser from moving focus away from the label editor that opens now.
      ev.preventDefault()
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
      const from = endAt(p, undefined, ev.shiftKey)
      const nid = addLine(from, ev.shiftKey ? p : snapP(p), app.tool)
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
      else if (kind === 'via') {
        // Editing a smart route by hand turns it into a plain line through its corners.
        if (el.type === 'line' && el.smart) freezeRoute(d, el)
        drag = { kind: 'via', id: hid, index: +arg }
      }
      else if (kind === 'label') drag = { kind: 'label', id: hid }
      else if (kind === 'add') drag = { kind: 'add', id: hid, index: +arg, start: p }
      else if (kind === 'rotate') {
        const pivot = single ? center(single) : { x: selBox!.x + selBox!.w / 2, y: selBox!.y + selBox!.h / 2 }
        drag = { kind: 'rotate', pivot, start: angleOf(p, pivot), orig: $state.snapshot(selected) as Element[] }
      }
      return
    }
    if (id) {
      const ids = expandGroups([id])
      if (ev.shiftKey && app.selection.includes(id)) {
        // Deselect on release unless this turns into a (⇧ free) drag.
        deselectOnClick = ids
      } else if (ev.shiftKey) {
        app.selection = [...new Set([...app.selection, ...ids])]
      } else if (!app.selection.includes(id)) {
        app.selection = ids
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
      hover = (app.tool === 'line' || app.tool === 'arrow') && !ev.shiftKey ? (anchorAt(p)?.node ?? shapeAt(d, p)?.id ?? null) : null
      hoverAnchor = hover ? (anchorAt(p)?.anchor ?? null) : null
      return
    }
    switch (drag.kind) {
      case 'pan':
        app.view.x = drag.vx + ev.clientX - drag.sx
        app.view.y = drag.vy + ev.clientY - drag.sy
        break
      case 'move': {
        if (Math.hypot(p.x - drag.start.x, p.y - drag.start.y) > 3 / scale) deselectOnClick = null
        // ⇧ moves freely, pixel by pixel, without grid or guide snapping.
        const [dx, dy] = ev.shiftKey
          ? ((guides = []), [p.x - drag.start.x, p.y - drag.start.y])
          : alignDelta(drag.orig, p.x - drag.start.x, p.y - drag.start.y)
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
        if (o.rotate) {
          // Resize in the shape's own frame, then move the new center back into place.
          const c0 = center(o)
          sizeTo(el, fx, fy, rotatePoint(p, c0, -o.rotate), ev.shiftKey ? o.w / o.h : undefined, o.w / o.h)
          const c = rotatePoint(center(el), c0, o.rotate)
          el.x = c.x - el.w / 2
          el.y = c.y - el.h / 2
          sizeMarks = []
          break
        }
        if (el.shape !== 'image') {
          sizeTo(el, fx, fy, p, ev.shiftKey ? o.w / o.h : undefined, o.w / o.h)
          break
        }
        el.x = Math.min(fx, q.x)
        el.w = Math.max(Math.abs(q.x - fx), 0.25)
        el.h = (el.w * o.h) / o.w
        el.y = drag.corner.includes('n') ? fy - el.h : fy
        break
      }
      case 'create': {
        sizeTo(find(d, drag.id) as Shape, drag.start.x, drag.start.y, p, ev.shiftKey ? 1 : undefined)
        break
      }
      case 'end': {
        const el = find(d, drag.id)
        if (el?.type !== 'line') break
        const other = el[drag.which === 'to' ? 'from' : 'to']
        const end = endAt(p, isNode(other) ? other.node : undefined, ev.shiftKey)
        el[drag.which] = end
        hover = isNode(end) ? end.node : null
        hoverAnchor = isNode(end) ? (end.anchor ?? null) : null
        break
      }
      case 'add': {
        // Only a real drag inserts a waypoint, so clicks (and double-clicks) on the dot do not.
        const el = find(d, drag.id)
        if (el?.type !== 'line' || Math.hypot(p.x - drag.start.x, p.y - drag.start.y) < 4 / scale) break
        if (el.smart) freezeRoute(d, el)
        el.via.splice(drag.index, 0, snapP(p))
        drag = { kind: 'via', id: drag.id, index: drag.index }
        break
      }
      case 'label': {
        const el = find(d, drag.id)
        if (!el) break
        guides = []
        if (el.type === 'line') {
          const t = lineProject(d, el, p)
          el.label_at = Math.abs(t - 0.5) < 0.03 ? undefined : { t: Math.round(t * 1000) / 1000 }
          break
        }
        const c = center(el)
        const tol = 6 / scale
        const lp = rotatePoint(p, c, -(el.rotate ?? 0))
        let dx = Math.max(-el.w / 2, Math.min(el.w / 2, lp.x - c.x))
        let dy = Math.max(-el.h / 2, Math.min(el.h / 2, lp.y - c.y))
        if (Math.abs(dx) < tol) {
          dx = 0
          guides.push({ x1: c.x, y1: el.y, x2: c.x, y2: el.y + el.h })
        }
        if (Math.abs(dy) < tol) {
          dy = 0
          guides.push({ x1: el.x, y1: c.y, x2: el.x + el.w, y2: c.y })
        }
        const r = (v: number) => Math.round(v * 1000) / 1000
        el.label_at = dx || dy ? { dx: r(dx), dy: r(dy) } : undefined
        break
      }
      case 'via': {
        const el = find(d, drag.id)
        if (el?.type !== 'line') break
        let q = ev.shiftKey ? p : snapP(p)
        if (!ev.shiftKey) {
          // Line up with the neighbouring points so horizontal and vertical segments stay straight.
          const pts = linePoints(d, el)
          for (const n of [pts[drag.index], pts[drag.index + 2]]) {
            if (Math.abs(p.x - n.x) < 8 / scale) q = { ...q, x: n.x }
            if (Math.abs(p.y - n.y) < 8 / scale) q = { ...q, y: n.y }
          }
        }
        el.via[drag.index] = q
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
        app.selection = expandGroups([...new Set([...drag.additive, ...inside])])
        break
      }
      case 'rotate': {
        let a = angleOf(p, drag.pivot) - drag.start
        const snapped = Math.round(a / 15) * 15
        if (ev.shiftKey || Math.abs(a - snapped) < 3) a = snapped
        rotateElements(drag.orig, a, drag.pivot)
        break
      }
    }
  }

  function up() {
    if (deselectOnClick) {
      const ids = deselectOnClick
      app.selection = app.selection.filter((s) => !ids.includes(s))
      deselectOnClick = null
    }
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
    hoverAnchor = null
    guides = []
    sizeMarks = []
    diagonal = null
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

  /**
   * Sizes a shape from the fixed corner (fx, fy) towards `p`, snapping width and height
   * to those of other shapes (marking all matches) or to the grid.
   */
  function sizeTo(el: Shape, fx: number, fy: number, p: Point, lock?: number, ratio?: number) {
    const others = d.elements.filter((e): e is Shape => e.type === 'shape' && e.id !== el.id)
    const tol = 6 / scale
    const fit = (raw: number, dim: 'w' | 'h') => {
      const m = others.find((o) => Math.abs(o[dim] - raw) < tol)
      return m ? m[dim] : Math.max(snap(raw), 0.25)
    }
    const [rw, rh] = [Math.abs(p.x - fx), Math.abs(p.y - fy)]
    diagonal = null
    if (lock) {
      // ⇧ keeps the aspect ratio: the dimension the pointer pushes further wins.
      el.w = Math.max(rw, rh * lock, 0.25)
      el.h = el.w / lock
    } else {
      el.w = fit(rw, 'w')
      el.h = fit(rh, 'h')
      // Aspect helper: snap to a square or back to the original ratio when close.
      for (const r of [1, ratio]) {
        if (r && Math.abs(rw / r - rh) < tol) {
          el.h = el.w / r
          diagonal = r
          break
        }
      }
    }
    el.x = p.x < fx ? fx - el.w : fx
    el.y = p.y < fy ? fy - el.h : fy
    const marks = (dim: 'w' | 'h') => {
      const same = others.filter((o) => Math.abs(o[dim] - el[dim]) < 1e-9)
      return same.length ? [el, ...same].map(({ x, y, w, h }) => ({ x, y, w, h, dim })) : []
    }
    sizeMarks = [...marks('w'), ...marks('h')]
  }

  /**
   * Snaps a move so the moved shapes' edges or centers line up with other shapes,
   * falling back to the grid. Sets the guide lines to draw.
   */
  function alignDelta(orig: Element[], rawX: number, rawY: number): [number, number] {
    const moving = orig.filter((o) => o.type === 'shape') as Shape[]
    guides = []
    if (!moving.length) return [snap(rawX), snap(rawY)]
    const ids = new Set(orig.map((o) => o.id))
    const others = d.elements.filter((e): e is Shape => e.type === 'shape' && !ids.has(e.id))
    const box = {
      x: Math.min(...moving.map((m) => m.x)),
      y: Math.min(...moving.map((m) => m.y)),
      r: Math.max(...moving.map((m) => m.x + m.w)),
      b: Math.max(...moving.map((m) => m.y + m.h)),
    }
    const tol = 6 / scale
    const axis = (lo: number, hi: number, raw: number, edges: (s: Shape) => number[]) => {
      let best: { delta: number; at: number } | null = null
      for (const v of [lo, (lo + hi) / 2, hi]) {
        for (const o of others) {
          for (const t of edges(o)) {
            const delta = t - v
            if (Math.abs(delta - raw) < tol && (!best || Math.abs(delta - raw) < Math.abs(best.delta - raw)))
              best = { delta, at: t }
          }
        }
      }
      return best
    }
    const bx = axis(box.x, box.r, rawX, (o) => [o.x, o.x + o.w / 2, o.x + o.w])
    const by = axis(box.y, box.b, rawY, (o) => [o.y, o.y + o.h / 2, o.y + o.h])
    const dx = bx ? bx.delta : snap(rawX)
    const dy = by ? by.delta : snap(rawY)
    const span = (a: number[], b: number[]) => [Math.min(...a, ...b) - 0.3, Math.max(...a, ...b) + 0.3]
    const ys = others.flatMap((o) => [o.y, o.y + o.h])
    const xs = others.flatMap((o) => [o.x, o.x + o.w])
    if (bx) {
      const [y1, y2] = span([box.y + dy, box.b + dy], ys)
      guides.push({ x1: bx.at, y1, x2: bx.at, y2 })
    }
    if (by) {
      const [x1, x2] = span([box.x + dx, box.r + dx], xs)
      guides.push({ x1, y1: by.at, x2, y2: by.at })
    }
    return [dx, dy]
  }

  function wheel(ev: WheelEvent) {
    ev.preventDefault()
    if (ev.ctrlKey || ev.metaKey) {
      const r = svg.getBoundingClientRect()
      zoomBy(Math.exp(-ev.deltaY * 0.01), { x: ev.clientX - r.left, y: ev.clientY - r.top })
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
    if (ev.target instanceof HTMLTextAreaElement || ev.target instanceof HTMLInputElement) return
    if (ev.code === 'Space') spaceDown = isDown
    if (ev.key === 'Shift') shiftDown = isDown
  }

  // Svelte registers wheel listeners as passive, which would let the browser zoom the page.
  $effect(() => {
    svg.addEventListener('wheel', wheel, { passive: false })
    return () => svg.removeEventListener('wheel', wheel)
  })

  const cursor = $derived(
    drag?.kind === 'pan'
      ? 'grabbing'
      : spaceDown || app.tool === 'hand' || (shiftDown && app.tool === 'select' && !drag)
        ? 'grab'
        : app.tool === 'select' ? 'default' : app.tool === 'text' ? 'text' : 'crosshair',
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
    <g class="elements">
      {#each d.elements as el (el.id)}
        <ElementView {el} {scale} />
      {/each}
    </g>

    {#if hover}
      {@const h = find(d, hover)}
      {#if h?.type === 'shape'}
        {@const c = center(h)}
        <rect
          class="hover"
          transform={h.rotate ? `rotate(${h.rotate} ${c.x} ${c.y})` : undefined}
          x={h.x}
          y={h.y}
          width={h.w}
          height={h.h}
          rx={0.1}
          stroke-width={3 / scale}
        />
        {#if h.shape !== 'text'}
          {#each anchors as a}
            {@const q = anchorPoint(h, a)}
            <circle class="anchor" class:on={hoverAnchor === a} cx={q.x} cy={q.y} r={(hoverAnchor === a ? 5 : 3.5) / scale} stroke-width={1.5 / scale} />
          {/each}
        {/if}
      {/if}
    {/if}

    {#each groupBoxes as b}
      <rect class="group" x={b.x} y={b.y} width={b.w} height={b.h} rx={0.1} stroke-width={1 / scale} stroke-dasharray="{4 / scale} {3 / scale}" />
    {/each}

    {#if selBox && !app.editing && !drag && selected.some((e) => e.type === 'shape')}
      {@const r = single ?? selBox}
      {@const c = { x: r.x + r.w / 2, y: r.y + r.h / 2 }}
      {@const top = r.y - 22 / scale}
      <g transform={single?.rotate ? `rotate(${single.rotate} ${c.x} ${c.y})` : undefined}>
        <line class="stem" x1={c.x} y1={r.y} x2={c.x} y2={top} stroke-width={1 / scale} />
        <circle class="handle rotate" data-handle="rotate:{selected[0].id}" cx={c.x} cy={top} r={handle / 1.6} stroke-width={1.5 / scale} />
      </g>
    {/if}

    {#each selected as el (el.id)}
      {#if el.type === 'shape'}
        {@const c = center(el)}
        <g transform={el.rotate ? `rotate(${el.rotate} ${c.x} ${c.y})` : undefined}>
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
        </g>
      {:else}
        {@const pts = linePoints(d, el)}
        {#if selected.length === 1 && !app.editing}
          {#if el.smart || !el.curved || el.via.length === 0}
            {@const lp = el.label ? labelPoint(d, el) : null}
            {#each pts.slice(1) as q, i}
              {@const mid = { x: (q.x + pts[i].x) / 2, y: (q.y + pts[i].y) / 2 }}
              {@const f = lp && Math.hypot(mid.x - lp.x, mid.y - lp.y) < 0.6 ? 0.2 : 0.5}
              <circle
                class="add"
                data-handle="add:{el.id}:{i}"
                cx={pts[i].x + (q.x - pts[i].x) * f}
                cy={pts[i].y + (q.y - pts[i].y) * f}
                r={handle / 2.5}
                stroke-width={1.5 / scale}
              />
            {/each}
          {/if}
          {#each el.smart ? pts.slice(1, -1) : el.via as q, i}
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

    {#if diagonal && (drag?.kind === 'resize' || drag?.kind === 'create')}
      {@const e = find(d, drag.id)}
      {#if e?.type === 'shape'}
        {@const c = center(e)}
        <line
          class="guide"
          transform={e.rotate ? `rotate(${e.rotate} ${c.x} ${c.y})` : undefined}
          x1={e.x}
          y1={e.y}
          x2={e.x + e.w}
          y2={e.y + e.h}
          stroke-width={1 / scale}
          stroke-dasharray="{4 / scale} {3 / scale}"
        />
      {/if}
    {/if}

    {#each sizeMarks as m}
      {@const t = 6 / scale}
      {#if m.dim === 'w'}
        {@const y = m.y - 12 / scale}
        <path class="guide" d="M{m.x} {y}H{m.x + m.w}M{m.x} {y - t / 2}v{t}M{m.x + m.w} {y - t / 2}v{t}" stroke-width={1 / scale} />
      {:else}
        {@const x = m.x - 12 / scale}
        <path class="guide" d="M{x} {m.y}V{m.y + m.h}M{x - t / 2} {m.y}h{t}M{x - t / 2} {m.y + m.h}h{t}" stroke-width={1 / scale} />
      {/if}
    {/each}

    {#each guides as g}
      <line class="guide" x1={g.x1} y1={g.y1} x2={g.x2} y2={g.y2} stroke-width={1 / scale} />
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
  .anchor {
    fill: white;
    stroke: var(--accent);
    pointer-events: none;
  }
  .anchor.on {
    fill: var(--accent);
  }
  .group {
    fill: none;
    stroke: var(--accent);
    pointer-events: none;
  }
  .stem {
    stroke: var(--accent);
    pointer-events: none;
  }
  .rotate {
    cursor: grab;
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
  .guide {
    fill: none;
    stroke: #ff2d55;
    pointer-events: none;
  }
  .marquee {
    fill: var(--accent);
    fill-opacity: 0.06;
    stroke: var(--accent);
  }
</style>
