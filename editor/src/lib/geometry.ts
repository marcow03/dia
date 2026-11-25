import { isNode, type Arrow, type Diagram, type Element, type End, type Line, type Point, type Shape } from './types'

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

/** SVG path of an arrowhead; open heads are stroked, the rest filled. */
export function arrowHead(kind: Arrow, tip: Point, from: Point, size: number): { d: string; filled: boolean } {
  const a = Math.atan2(tip.y - from.y, tip.x - from.x)
  const pt = (r: number, da: number) => `${tip.x - r * Math.cos(a + da)} ${tip.y - r * Math.sin(a + da)}`
  const T = `${tip.x} ${tip.y}`
  switch (kind) {
    case 'stealth':
      return { d: `M${T} L${pt(size, 0.4)} L${pt(size * 0.65, 0)} L${pt(size, -0.4)} Z`, filled: true }
    case 'triangle':
      return { d: `M${T} L${pt(size, 0.35)} L${pt(size, -0.35)} Z`, filled: true }
    case 'open':
      return { d: `M${pt(size, 0.45)} L${T} L${pt(size, -0.45)}`, filled: false }
    case 'circle': {
      const r = size / 3
      return { d: `M${pt(0, 0)} A${r} ${r} 0 1 0 ${pt(2 * r, 0)} A${r} ${r} 0 1 0 ${pt(0, 0)} Z`, filled: true }
    }
    case 'diamond': {
      const h = size / 2
      const side = (s: number) => `${tip.x - h * Math.cos(a) + s * h * 0.5 * Math.sin(a)} ${tip.y - h * Math.sin(a) - s * h * 0.5 * Math.cos(a)}`
      return { d: `M${T} L${side(1)} L${pt(size, 0)} L${side(-1)} Z`, filled: true }
    }
    case 'bar': {
      const r = size / 2.5
      const perp = (s: number) => `${tip.x + s * r * Math.sin(a)} ${tip.y - s * r * Math.cos(a)}`
      return { d: `M${perp(1)} L${perp(-1)}`, filled: false }
    }
    default:
      return { d: '', filled: false }
  }
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
