import init, * as wasm from './wasm/dia_wasm'
import { resolveRoutes } from './router'
import { app } from './state.svelte'
import type { Diagram } from './types'

export type FormatInfo = { name: string; extensions: string[] }

export const ready = init()

export function read(input: string, from: string): { diagram: Diagram; warnings: string[] } {
  return JSON.parse(wasm.read(input, from))
}

/** Mirrors `dia_core::image_path`: embedded images are exported as `<id>.<ext>`. */
export function imagePath(id: string, src: string) {
  const m = src.match(/^data:image\/([a-z]+)/)
  return m ? `${id}.${m[1] === 'jpeg' ? 'jpg' : m[1]}` : src
}

export function write(diagram: Diagram, to: string): string {
  let d = resolveRoutes(diagram)
  // CeTZ embeds data URL images as base64; without the setting they become file references.
  if (to === 'cetz' && !app.settings.embedImages) {
    d = {
      elements: d.elements.map((e) => (e.type === 'shape' && e.src?.startsWith('data:') ? { ...e, src: imagePath(e.id, e.src) } : e)),
    }
  }
  return wasm.write(JSON.stringify(d), to)
}

export function formatForFile(name: string): string | undefined {
  const ext = name.split('.').pop()?.toLowerCase() ?? ''
  const formats: FormatInfo[] = JSON.parse(wasm.formats())
  return formats.find((f) => f.extensions.includes(ext))?.name
}
