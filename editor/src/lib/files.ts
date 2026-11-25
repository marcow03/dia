import { formatForFile, read, write } from './core'
import { PX_PER_CM, snap } from './geometry'
import { addShape, app, checkpoint, newDocument, toast } from './state.svelte'

export const exports = [
  { format: 'cetz', ext: 'typ', label: 'CeTZ (Typst)' },
  { format: 'tikz', ext: 'tex', label: 'TikZ (LaTeX)' },
  { format: 'drawio', ext: 'drawio', label: 'draw.io' },
]

/** Mirrors `dia_core::image_path`: embedded images are exported as `<id>.<ext>`. */
function imagePath(id: string, src: string) {
  const m = src.match(/^data:image\/([a-z]+)/)
  return m ? `${id}.${m[1] === 'jpeg' ? 'jpg' : m[1]}` : src
}

function save(name: string, data: Blob | string) {
  const a = document.createElement('a')
  a.href = typeof data === 'string' ? data : URL.createObjectURL(data)
  a.download = name
  a.click()
  setTimeout(() => URL.revokeObjectURL(a.href), 1000)
}

export function download(format: string, ext: string) {
  try {
    save(`${app.name || 'untitled'}.${ext}`, new Blob([write(app.diagram, format)], { type: 'text/plain' }))
    if (format === 'drawio') return
    for (const e of app.diagram.elements) {
      if (e.type === 'shape' && e.src?.startsWith('data:')) save(imagePath(e.id, e.src), e.src)
    }
  } catch (e) {
    toast(String(e))
  }
}

const readAs = (file: File, kind: 'text' | 'url') =>
  new Promise<string>((resolve, reject) => {
    const r = new FileReader()
    r.onload = () => resolve(r.result as string)
    r.onerror = () => reject(r.error)
    kind === 'text' ? r.readAsText(file) : r.readAsDataURL(file)
  })

/** Reads an image as data URL with a size in cm, between 1 and 8 cm wide. */
export async function readImage(file: File) {
  const src = await readAs(file, 'url')
  const img = new Image()
  img.src = src
  await img.decode()
  const w = Math.min(Math.max(img.naturalWidth / (96 / 2.54), 1), 8)
  return { src, w, h: (w * img.naturalHeight) / img.naturalWidth }
}

/** Opens a diagram file; image files picked alongside it fill in referenced images. */
export async function openFiles(files: File[]) {
  const file = files.find((f) => !f.type.startsWith('image/'))
  if (!file) return
  const format = formatForFile(file.name)
  if (!format) return toast(`Unsupported file: ${file.name}`)
  try {
    const { diagram, warnings } = read(await readAs(file, 'text'), format)
    const images = new Map<string, string>()
    for (const f of files) if (f.type.startsWith('image/')) images.set(f.name, await readAs(f, 'url'))
    for (const e of diagram.elements) {
      if (e.type === 'shape' && e.src && images.has(e.src)) e.src = images.get(e.src)
    }
    newDocument(diagram, file.name.replace(/\.[^.]+$/, ''))
    if (warnings.length) toast(`Imported with ${warnings.length} warning(s): ${warnings[0]}`)
  } catch (e) {
    toast(String(e))
  }
}

function pick(accept: string, multiple: boolean, then: (files: File[]) => void) {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = accept
  input.multiple = multiple
  input.onchange = () => input.files && then([...input.files])
  input.click()
}

export const pickFile = () => pick('.typ,.tex,.tikz,.drawio,.xml,image/*', true, openFiles)

export function pickImage() {
  pick('image/*', false, async ([file]) => {
    const { src, w, h } = await readImage(file)
    const r = document.querySelector('svg.canvas')!.getBoundingClientRect()
    const scale = app.view.zoom * PX_PER_CM
    const cx = (r.width / 2 - app.view.x) / scale
    const cy = (r.height / 2 - app.view.y) / scale
    checkpoint()
    app.selection = [addShape('image', snap(cx - w / 2), snap(cy - h / 2), w, h, src)]
    app.tool = 'select'
  })
}
