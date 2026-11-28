export type Point = { x: number; y: number }
export type Anchor = 'north' | 'north-east' | 'east' | 'south-east' | 'south' | 'south-west' | 'west' | 'north-west'
export const anchors: Anchor[] = ['north', 'north-east', 'east', 'south-east', 'south', 'south-west', 'west', 'north-west']
/** A free point, or bound to a shape: at an anchor, or where the line meets the border. */
export type End = Point | { node: string; anchor?: Anchor }
export type ShapeKind = 'rect' | 'ellipse' | 'diamond' | 'text' | 'image'
export type Dash = 'solid' | 'dashed' | 'dotted'
export type Align = 'left' | 'center' | 'right'
export type Arrow = 'none' | 'stealth' | 'triangle' | 'open' | 'circle' | 'diamond' | 'bar'

export const arrows: Arrow[] = ['none', 'stealth', 'triangle', 'open', 'circle', 'diamond', 'bar']
export const fonts = ['Libertinus Serif', 'New Computer Modern', 'DejaVu Sans Mono']

export type Style = {
  stroke: string | null
  fill: string | null
  width: number
  dash: Dash
  radius: number
  start: Arrow
  end: Arrow
  font: string | null
  size: number
  bold: boolean
  italic: boolean
  color: string
  align: Align
}

/** Label placement: offset from a shape's center (cm), or position along a line (0..1). */
export type LabelAt = { dx: number; dy: number } | { t: number }
type Base = { id: string; style: Style; label: string; label_at?: LabelAt; group?: string }
/** `rotate` is clockwise around the center, in degrees. */
export type Shape = Base & {
  type: 'shape'
  shape: ShapeKind
  x: number
  y: number
  w: number
  h: number
  src?: string
  rotate?: number
}
/** `auto` routes the line around shapes in the editor; exports get its corners as plain waypoints. */
export type Line = Base & { type: 'line'; from: End; to: End; via: Point[]; curved: boolean; auto?: boolean }
export type Element = Shape | Line
export type Diagram = { elements: Element[] }

export const defaultStyle = (): Style => ({
  stroke: '#000000',
  fill: null,
  width: 1,
  dash: 'solid',
  radius: 0,
  start: 'none',
  end: 'none',
  font: null,
  size: 14,
  bold: false,
  italic: false,
  color: '#000000',
  align: 'left',
})

export const isNode = (e: End): e is { node: string } => 'node' in e
