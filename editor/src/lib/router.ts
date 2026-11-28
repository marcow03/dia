import { anchorPoint, bounds, center, find } from './geometry'
import { isNode, type Anchor, type Diagram, type End, type Line, type Point, type Shape } from './types'

/** Clearance around shapes, and how much a bend costs compared to 1 cm of length. */
const MARGIN = 0.4
const BEND = 1.5

type Dir = 0 | 1 | 2 | 3
const STEPS: [number, number][] = [
  [0, -1],
  [1, 0],
  [0, 1],
  [-1, 0],
]
const SIDES: Record<Dir, Anchor> = { 0: 'north', 1: 'east', 2: 'south', 3: 'west' }

type Rect = { x0: number; y0: number; x1: number; y1: number }
type EndInfo = { p: Point; dir: Dir | null; anchor?: Anchor }
export type Route = { points: Point[]; fromAnchor?: Anchor; toAnchor?: Anchor }

const dirTowards = (dx: number, dy: number): Dir =>
  Math.abs(dx) >= Math.abs(dy) ? (dx >= 0 ? 1 : 3) : dy >= 0 ? 2 : 0

/** Where a line leaves a shape: its anchor, or the side facing the other end; free points have no direction. */
function endInfo(d: Diagram, e: End, other: Point): EndInfo {
  const s = isNode(e) ? find(d, e.node) : undefined
  if (!isNode(e)) return { p: e, dir: null }
  if (s?.type !== 'shape') return { p: { x: 0, y: 0 }, dir: null }
  const c = center(s)
  if (e.anchor) {
    const p = anchorPoint(s, e.anchor)
    return { p, dir: dirTowards(p.x - c.x, p.y - c.y), anchor: e.anchor }
  }
  const dir = dirTowards(other.x - c.x, other.y - c.y)
  return { p: anchorPoint(s as Shape, SIDES[dir]), dir, anchor: SIDES[dir] }
}

const endTarget = (d: Diagram, e: End): Point => {
  if (!isNode(e)) return e
  const s = find(d, e.node)
  return s?.type === 'shape' ? center(s) : { x: 0, y: 0 }
}

const cache = new Map<string, Route>()

/**
 * Orthogonal route around all shapes: a shortest path with bend penalty on the grid formed by
 * the shapes' padded edges and the two line ends.
 */
export function autoRoute(d: Diagram, l: Line): Route {
  const obstacles: Rect[] = d.elements
    .filter((e) => e.type === 'shape')
    .map((e) => {
      const b = bounds(d, e)
      return { x0: b.x - MARGIN, y0: b.y - MARGIN, x1: b.x + b.w + MARGIN, y1: b.y + b.h + MARGIN }
    })
  const s = endInfo(d, l.from, endTarget(d, l.to))
  const t = endInfo(d, l.to, endTarget(d, l.from))
  const key = JSON.stringify([s, t, obstacles])
  const hit = cache.get(key)
  if (hit) return hit

  const stub = (e: EndInfo) =>
    e.dir === null ? e.p : { x: e.p.x + STEPS[e.dir][0] * (MARGIN + 0.05), y: e.p.y + STEPS[e.dir][1] * (MARGIN + 0.05) }
  const [sp, tp] = [stub(s), stub(t)]
  const corners = search(obstacles, sp, tp, s.dir, t.dir === null ? null : (((t.dir + 2) % 4) as Dir))
  const route = { points: simplify([s.p, sp, ...corners, tp, t.p]), fromAnchor: s.anchor, toAnchor: t.anchor }
  if (cache.size > 500) cache.clear()
  cache.set(key, route)
  return route
}

function axis(values: number[]): number[] {
  const v = [...new Set(values.map((x) => Math.round(x * 1000) / 1000))].sort((a, b) => a - b)
  return [...v, ...v.slice(1).map((x, i) => (x + v[i]) / 2)].sort((a, b) => a - b)
}

const inside = (r: Rect, x: number, y: number) => x > r.x0 + 1e-6 && x < r.x1 - 1e-6 && y > r.y0 + 1e-6 && y < r.y1 - 1e-6

/** Dijkstra over (grid point, heading); returns the corner points between `a` and `b`. */
function search(obs: Rect[], a: Point, b: Point, startDir: Dir | null, endDir: Dir | null): Point[] {
  const xs = axis([a.x, b.x, ...obs.flatMap((r) => [r.x0, r.x1])])
  const ys = axis([a.y, b.y, ...obs.flatMap((r) => [r.y0, r.y1])])
  const idx = (v: number[], x: number) => v.findIndex((y) => Math.abs(y - x) < 1e-3)
  const [ai, aj, bi, bj] = [idx(xs, a.x), idx(ys, a.y), idx(xs, b.x), idx(ys, b.y)]
  const free = (i: number, j: number) => !obs.some((r) => inside(r, xs[i], ys[j]))
  const W = xs.length
  const key = (i: number, j: number, dir: number) => (j * W + i) * 5 + dir
  const dist = new Map<number, number>()
  const prev = new Map<number, number>()
  const heap: [number, number, number, number][] = [[0, ai, aj, startDir ?? 4]]
  dist.set(key(ai, aj, startDir ?? 4), 0)
  let goal = -1
  while (heap.length) {
    heap.sort((p, q) => q[0] - p[0])
    const [cost, i, j, dir] = heap.pop()!
    const k = key(i, j, dir)
    if (cost > (dist.get(k) ?? Infinity)) continue
    if (i === bi && j === bj) {
      goal = k
      break
    }
    for (const nd of [0, 1, 2, 3] as Dir[]) {
      if (dir !== 4 && nd === (dir + 2) % 4) continue
      const ni = i + STEPS[nd][0]
      const nj = j + STEPS[nd][1]
      if (ni < 0 || nj < 0 || ni >= W || nj >= ys.length || !free(ni, nj)) continue
      // Grid lines include all obstacle edges, so a segment is clear when its midpoint is.
      if (obs.some((r) => inside(r, (xs[i] + xs[ni]) / 2, (ys[j] + ys[nj]) / 2))) continue
      let c = cost + Math.abs(xs[ni] - xs[i]) + Math.abs(ys[nj] - ys[j]) + (dir !== 4 && nd !== dir ? BEND : 0)
      if (ni === bi && nj === bj && endDir !== null && nd !== endDir) c += BEND
      const nk = key(ni, nj, nd)
      if (c < (dist.get(nk) ?? Infinity)) {
        dist.set(nk, c)
        prev.set(nk, k)
        heap.push([c, ni, nj, nd])
      }
    }
  }
  if (goal < 0) return [{ x: b.x, y: a.y }]
  const path: Point[] = []
  for (let k: number | undefined = goal; k !== undefined; k = prev.get(k)) {
    const cell = Math.floor(k / 5)
    path.push({ x: xs[cell % W], y: ys[Math.floor(cell / W)] })
  }
  return path.reverse().slice(1, -1)
}

/** Drops repeated and collinear points. */
function simplify(points: Point[]): Point[] {
  const out: Point[] = []
  for (const p of points) {
    const q = out[out.length - 1]
    if (q && Math.abs(q.x - p.x) < 1e-6 && Math.abs(q.y - p.y) < 1e-6) continue
    const r = out[out.length - 2]
    if (r && q && Math.abs((q.x - r.x) * (p.y - r.y) - (q.y - r.y) * (p.x - r.x)) < 1e-9) out.pop()
    out.push(p)
  }
  return out
}

/** The diagram as the file formats see it: auto-routed lines become plain lines through their corners. */
export function resolveRoutes(d: Diagram): Diagram {
  return {
    elements: d.elements.map((e) => {
      if (e.type !== 'line' || !e.auto) return e
      const r = autoRoute(d, e)
      const end = (x: End, anchor?: Anchor): End => (isNode(x) ? { node: x.node, anchor } : x)
      const { auto: _, ...line } = e
      return { ...line, from: end(e.from, r.fromAnchor), to: end(e.to, r.toAnchor), via: r.points.slice(1, -1), curved: false }
    }),
  }
}
