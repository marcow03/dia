import init, * as wasm from './wasm/dia_wasm'
import type { Diagram } from './types'

export type FormatInfo = { name: string; extensions: string[] }

export const ready = init()

export function read(input: string, from: string): { diagram: Diagram; warnings: string[] } {
  return JSON.parse(wasm.read(input, from))
}

export function write(diagram: Diagram, to: string): string {
  return wasm.write(JSON.stringify(diagram), to)
}

export function formatForFile(name: string): string | undefined {
  const ext = name.split('.').pop()?.toLowerCase() ?? ''
  const formats: FormatInfo[] = JSON.parse(wasm.formats())
  return formats.find((f) => f.extensions.includes(ext))?.name
}
