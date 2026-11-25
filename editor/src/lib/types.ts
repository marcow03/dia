export type Point = { x: number; y: number }
export type End = Point | { node: string }
export type ShapeKind = 'rect' | 'ellipse' | 'diamond' | 'text'
export type Dash = 'solid' | 'dashed' | 'dotted'

export type Style = {
  stroke: string | null
  fill: string | null
  width: number
  dash: Dash
  radius: number
  start_arrow: boolean
  end_arrow: boolean
}

type Base = { id: string; style: Style; label: string }
export type Shape = Base & { type: 'shape'; shape: ShapeKind; x: number; y: number; w: number; h: number }
export type Line = Base & { type: 'line'; from: End; to: End; via: Point[]; curved: boolean }
export type Element = Shape | Line
export type Diagram = { elements: Element[] }

export const defaultStyle = (): Style => ({
  stroke: '#000000',
  fill: null,
  width: 1,
  dash: 'solid',
  radius: 0,
  start_arrow: false,
  end_arrow: false,
})

export const isNode = (e: End): e is { node: string } => 'node' in e
