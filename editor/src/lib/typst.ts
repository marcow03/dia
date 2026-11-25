import { $typst } from '@myriaddreamin/typst.ts'
import compilerUrl from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url'
import rendererUrl from '@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm?url'
import type { Style } from './types'

$typst.setCompilerInitOptions({ getModule: () => compilerUrl })
$typst.setRendererInitOptions({ getModule: () => rendererUrl })

/** Rendered label: inner SVG markup with width and height in pt. */
export type Rendered = { svg: string; w: number; h: number } | { error: string }

const cache = new Map<string, Promise<Rendered>>()
let queue: Promise<unknown> = Promise.resolve()

type TextStyle = Pick<Style, 'font' | 'size' | 'bold' | 'italic' | 'color'>

export function renderLabel(source: string, style: TextStyle): Promise<Rendered> {
  const text = [`size: ${style.size}pt`, `fill: rgb("${style.color}")`]
  if (style.font) text.push(`font: "${style.font}"`)
  if (style.bold) text.push('weight: "bold"')
  if (style.italic) text.push('style: "italic"')
  const mainContent = `#set page(width: auto, height: auto, margin: 0pt, fill: none)
#set text(${text.join(', ')})
#set align(center)
${source}`
  let hit = cache.get(mainContent)
  if (!hit) {
    // The compiler holds one main file, so renders must not interleave.
    hit = queue.then(() => compile(mainContent))
    queue = hit
    cache.set(mainContent, hit)
  }
  return hit
}

async function compile(mainContent: string): Promise<Rendered> {
  try {
    // Drop typst.ts' interactive extras; the script is not valid XML inside a data URL.
    const svg = (await $typst.svg({ mainContent }))
      .replace(/<script[\s\S]*?<\/script>/g, '')
      .replace(/<style[\s\S]*?<\/style>/g, '')
      .replace(/<foreignObject[\s\S]*?<\/foreignObject>/g, '')
    const box = svg.match(/viewBox="([\d.\s-]+)"/)?.[1].split(/\s+/).map(Number)
    if (!box) return { error: 'no output' }
    const inner = svg.slice(svg.indexOf('>') + 1, svg.lastIndexOf('</svg>'))
    return { svg: inner, w: box[2], h: box[3] }
  } catch (e) {
    return { error: String(e) }
  }
}
