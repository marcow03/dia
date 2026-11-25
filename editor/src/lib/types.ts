export type Point = { x: number; y: number }
export type End = Point | { node: string }
export type ShapeKind = 'rect' | 'ellipse' | 'diamond' | 'text' | 'image'
export type Dash = 'solid' | 'dashed' | 'dotted'
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
}

type Base = { id: string; style: Style; label: string }
export type Shape = Base & { type: 'shape'; shape: ShapeKind; x: number; y: number; w: number; h: number; src?: string }
export type Line = Base & { type: 'line'; from: End; to: End; via: Point[]; curved: boolean }
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
  size: 11,
  bold: false,
  italic: false,
  color: '#000000',
})

export const isNode = (e: End): e is { node: string } => 'node' in e
