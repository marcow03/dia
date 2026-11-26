import { bounds } from './geometry'
import { app } from './state.svelte'

const MARGIN = 0.5
/** Raster exports use 3x the 96 dpi screen resolution. */
const RASTER_PX_PER_CM = (96 / 2.54) * 3

/** The rendered diagram as a standalone SVG, sized in cm, without selection or editing overlays. */
export function diagramSvg(): { svg: string; w: number; h: number } {
  const d = app.diagram
  const bs = d.elements.map((e) => bounds(d, e))
  if (!bs.length) throw new Error('Nothing to export')
  const x = Math.min(...bs.map((b) => b.x)) - MARGIN
  const y = Math.min(...bs.map((b) => b.y)) - MARGIN
  const w = Math.max(...bs.map((b) => b.x + b.w)) + MARGIN - x
  const h = Math.max(...bs.map((b) => b.y + b.h)) + MARGIN - y
  const content = document.querySelector('svg.canvas g.elements')!.cloneNode(true) as SVGGElement
  content.querySelectorAll('.grab, .hit').forEach((n) => n.remove())
  const inner = new XMLSerializer().serializeToString(content).replaceAll('var(--canvas)', '#ffffff')
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${w}cm" height="${h}cm" viewBox="${x} ${y} ${w} ${h}">${inner}</svg>`
  return { svg, w, h }
}

export async function diagramRaster(type: 'image/png' | 'image/jpeg'): Promise<Blob> {
  const { svg, w, h } = diagramSvg()
  const img = new Image()
  img.src = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }))
  await img.decode()
  const canvas = document.createElement('canvas')
  canvas.width = Math.round(w * RASTER_PX_PER_CM)
  canvas.height = Math.round(h * RASTER_PX_PER_CM)
  const ctx = canvas.getContext('2d')!
  if (type === 'image/jpeg') {
    ctx.fillStyle = '#ffffff'
    ctx.fillRect(0, 0, canvas.width, canvas.height)
  }
  ctx.drawImage(img, 0, 0, canvas.width, canvas.height)
  URL.revokeObjectURL(img.src)
  return new Promise((resolve, reject) =>
    canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('Export failed'))), type, 0.92),
  )
}
