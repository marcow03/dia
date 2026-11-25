import { find } from './geometry'
import { app, checkpoint } from './state.svelte'
import type { Shape } from './types'

export type Align = 'left' | 'center' | 'right' | 'top' | 'middle' | 'bottom'

const selectedShapes = () =>
  app.selection.map((id) => find(app.diagram, id)).filter((e): e is Shape => e?.type === 'shape')

export function align(how: Align) {
  const ss = selectedShapes()
  if (ss.length < 2) return
  checkpoint()
  const left = Math.min(...ss.map((s) => s.x))
  const right = Math.max(...ss.map((s) => s.x + s.w))
  const top = Math.min(...ss.map((s) => s.y))
  const bottom = Math.max(...ss.map((s) => s.y + s.h))
  for (const s of ss) {
    if (how === 'left') s.x = left
    if (how === 'center') s.x = (left + right) / 2 - s.w / 2
    if (how === 'right') s.x = right - s.w
    if (how === 'top') s.y = top
    if (how === 'middle') s.y = (top + bottom) / 2 - s.h / 2
    if (how === 'bottom') s.y = bottom - s.h
  }
}

/** Spaces shapes so the gaps between neighbours are equal. */
export function distribute(axis: 'x' | 'y') {
  const ss = selectedShapes()
  if (ss.length < 3) return
  checkpoint()
  const size = axis === 'x' ? 'w' : 'h'
  ss.sort((a, b) => a[axis] - b[axis])
  const first = ss[0][axis]
  const last = ss[ss.length - 1][axis] + ss[ss.length - 1][size]
  const gap = (last - first - ss.reduce((sum, s) => sum + s[size], 0)) / (ss.length - 1)
  let pos = first
  for (const s of ss) {
    s[axis] = pos
    pos += s[size] + gap
  }
}

/** Gives all shapes the size of the first selected one, keeping their centers. */
export function matchSize(dim: 'w' | 'h') {
  const ss = selectedShapes()
  if (ss.length < 2) return
  checkpoint()
  const target = ss[0][dim]
  for (const s of ss.slice(1)) {
    const pos = dim === 'w' ? 'x' : 'y'
    s[pos] += (s[dim] - target) / 2
    s[dim] = target
  }
}
