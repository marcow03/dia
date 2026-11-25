import { isNode, type Diagram, type Element, type End, type Line, type Point, type Shape } from './types'

export const PX_PER_CM = 40
export const PT_PER_CM = 28.3465

export function find(d: Diagram, id: string) {
  return d.elements.find((e) => e.id === id)
}

export function center(s: Shape): Point {
  return { x: s.x + s.w / 2, y: s.y + s.h / 2 }
}

function endShape(d: Diagram, e: End): Shape | undefined {
  if (!isNode(e)) return
  const el = find(d, e.node)
  return el?.type === 'shape' ? el : undefined
}

export function endCenter(d: Diagram, e: End): Point {
  const s = endShape(d, e)
  return s ? center(s) : isNode(e) ? { x: 0, y: 0 } : e
}

/** Point where the ray from the shape center towards `p` leaves the shape. */
export function border(s: Shape, p: Point): Point {
  const c = center(s)
  const [dx, dy, hw, hh] = [p.x - c.x, p.y - c.y, s.w / 2, s.h / 2]
  if (dx === 0 && dy === 0) return c
  let t: number
  if (s.shape === 'ellipse') t = 1 / Math.hypot(dx / hw, dy / hh)
  else if (s.shape === 'diamond') t = 1 / (Math.abs(dx) / hw + Math.abs(dy) / hh)
  else t = Math.min(hw / Math.abs(dx), hh / Math.abs(dy))
  return { x: c.x + dx * t, y: c.y + dy * t }
}

/** Resolved points of a line: start, via..., end, with node ends clipped to the shape border. */
export function linePoints(d: Diagram, l: Line): Point[] {
  const from = endCenter(d, l.from)
  const to = endCenter(d, l.to)
  const first = l.via[0] ?? to
  const last = l.via[l.via.length - 1] ?? from
  const fs = endShape(d, l.from)
  const ts = endShape(d, l.to)
  return [fs ? border(fs, first) : from, ...l.via, ts ? border(ts, last) : to]
}

export const isBezier = (l: Line) => l.curved && l.via.length >= 1 && l.via.length <= 2

export function linePath(d: Diagram, l: Line): string {
  const p = linePoints(d, l)
  const f = (q: Point) => `${q.x} ${q.y}`
  if (isBezier(l)) {
    const end = p[p.length - 1]
    return l.via.length === 1
      ? `M${f(p[0])} Q${f(l.via[0])} ${f(end)}`
      : `M${f(p[0])} C${f(l.via[0])} ${f(l.via[1])} ${f(end)}`
  }
  return 'M' + p.map(f).join(' L')
}

/** Direction for arrowheads at the start or end of a line. */
export function arrowAt(d: Diagram, l: Line, atEnd: boolean): { tip: Point; from: Point } {
  const p = linePoints(d, l)
  return atEnd ? { tip: p[p.length - 1], from: p[p.length - 2] } : { tip: p[0], from: p[1] }
}

export function arrowHead(tip: Point, from: Point, size: number): string {
  const a = Math.atan2(tip.y - from.y, tip.x - from.x)
  const pt = (r: number, da: number) => `${tip.x - r * Math.cos(a + da)} ${tip.y - r * Math.sin(a + da)}`
  return `M${tip.x} ${tip.y} L${pt(size, 0.4)} L${pt(size * 0.65, 0)} L${pt(size, -0.4)} Z`
}

export function lineMid(d: Diagram, l: Line): Point {
  const p = linePoints(d, l)
  if (isBezier(l)) {
    const c = [p[0], ...l.via, p[p.length - 1]]
    const t = 0.5
    if (c.length === 3)
      return { x: 0.25 * c[0].x + 0.5 * c[1].x + 0.25 * c[2].x, y: 0.25 * c[0].y + 0.5 * c[1].y + 0.25 * c[2].y }
    const k = [(1 - t) ** 3, 3 * (1 - t) ** 2 * t, 3 * (1 - t) * t ** 2, t ** 3]
    return { x: k.reduce((s, v, i) => s + v * c[i].x, 0), y: k.reduce((s, v, i) => s + v * c[i].y, 0) }
  }
  const lens = p.slice(1).map((q, i) => Math.hypot(q.x - p[i].x, q.y - p[i].y))
  let rest = lens.reduce((a, b) => a + b, 0) / 2
  for (let i = 0; i < lens.length; i++) {
    if (rest <= lens[i] || i === lens.length - 1) {
      const t = lens[i] ? rest / lens[i] : 0
      return { x: p[i].x + (p[i + 1].x - p[i].x) * t, y: p[i].y + (p[i + 1].y - p[i].y) * t }
    }
    rest -= lens[i]
  }
  return p[0]
}

export function bounds(d: Diagram, e: Element) {
  if (e.type === 'shape') return { x: e.x, y: e.y, w: e.w, h: e.h }
  const p = [...linePoints(d, e), ...e.via]
  const xs = p.map((q) => q.x)
  const ys = p.map((q) => q.y)
  const x = Math.min(...xs)
  const y = Math.min(...ys)
  return { x, y, w: Math.max(...xs) - x, h: Math.max(...ys) - y }
}

export function shapeAt(d: Diagram, p: Point, exclude?: string): Shape | undefined {
  for (let i = d.elements.length - 1; i >= 0; i--) {
    const e = d.elements[i]
    if (e.type === 'shape' && e.id !== exclude && p.x >= e.x && p.x <= e.x + e.w && p.y >= e.y && p.y <= e.y + e.h)
      return e
  }
}

export const snap = (v: number, step = 0.25) => Math.round(v / step) * step
