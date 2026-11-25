import { $typst } from '@myriaddreamin/typst.ts'
import compilerUrl from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url'
import rendererUrl from '@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm?url'

$typst.setCompilerInitOptions({ getModule: () => compilerUrl })
$typst.setRendererInitOptions({ getModule: () => rendererUrl })

/** Rendered label: inner SVG markup with width and height in pt. */
export type Rendered = { svg: string; w: number; h: number } | { error: string }

const cache = new Map<string, Promise<Rendered>>()
let queue: Promise<unknown> = Promise.resolve()

export function renderLabel(source: string): Promise<Rendered> {
  let hit = cache.get(source)
  if (!hit) {
    // The compiler holds one main file, so renders must not interleave.
    hit = queue.then(() => compile(source))
    queue = hit
    cache.set(source, hit)
  }
  return hit
}

async function compile(source: string): Promise<Rendered> {
  const mainContent = `#set page(width: auto, height: auto, margin: 0pt, fill: none)
#set text(size: 11pt)
#set align(center)
${source}`
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
