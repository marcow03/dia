import { endCenter, find } from './geometry'
import { defaultStyle, isNode, type Diagram, type Element, type Point, type ShapeKind } from './types'

export type Tool = 'select' | ShapeKind | 'line'

const STORAGE_KEY = 'dia:document'

function load(): { diagram: Diagram; name: string } {
  try {
    const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '')
    if (Array.isArray(saved.diagram?.elements)) return saved
  } catch {}
  return { diagram: { elements: [] }, name: 'untitled' }
}

const saved = load()

export const app = $state({
  diagram: saved.diagram,
  name: saved.name,
  selection: [] as string[],
  tool: 'select' as Tool,
  editing: null as string | null,
  view: { x: 80, y: 80, zoom: 1 },
  showCode: false,
  toast: '',
})

$effect.root(() => {
  $effect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ diagram: app.diagram, name: app.name }))
  })
})

const undoStack: string[] = []
const redoStack: string[] = []

/** Call before every user-visible change so it can be undone. */
export function checkpoint() {
  undoStack.push(JSON.stringify(app.diagram))
  if (undoStack.length > 200) undoStack.shift()
  redoStack.length = 0
}

function restore(from: string[], to: string[]) {
  const s = from.pop()
  if (!s) return
  to.push(JSON.stringify(app.diagram))
  app.diagram = JSON.parse(s)
  app.selection = app.selection.filter((id) => find(app.diagram, id))
  app.editing = null
}

export const undo = () => restore(undoStack, redoStack)
export const redo = () => restore(redoStack, undoStack)

export function setDiagram(d: Diagram, name?: string) {
  checkpoint()
  app.diagram = d
  app.selection = []
  app.editing = null
  if (name) app.name = name
}

export function newId(prefix: string) {
  for (let i = 1; ; i++) if (!find(app.diagram, prefix + i)) return prefix + i
}

export function addShape(shape: ShapeKind, x: number, y: number, w: number, h: number) {
  const id = newId(shape === 'text' ? 't' : 'n')
  const style = defaultStyle()
  if (shape === 'text') style.stroke = null
  app.diagram.elements.push({ id, type: 'shape', shape, x, y, w, h, style, label: '' })
  return id
}

export function addLine(from: Element['id'] | Point, to: Point) {
  const id = newId('e')
  const style = { ...defaultStyle(), end_arrow: true }
  const fromEnd = typeof from === 'string' ? { node: from } : from
  app.diagram.elements.push({ id, type: 'line', from: fromEnd, to, via: [], curved: false, style, label: '' })
  return id
}

/** Removes elements; lines attached to removed shapes keep their position as free ends. */
export function remove(ids: string[]) {
  const d = app.diagram
  for (const e of d.elements) {
    if (e.type !== 'line') continue
    for (const k of ['from', 'to'] as const) {
      const end = e[k]
      if (isNode(end) && ids.includes(end.node)) e[k] = endCenter(d, end)
    }
  }
  d.elements = d.elements.filter((e) => !ids.includes(e.id))
  app.selection = app.selection.filter((id) => !ids.includes(id))
}

let clipboard: Element[] = []

export function copy() {
  clipboard = app.selection.map((id) => $state.snapshot(find(app.diagram, id)!) as Element)
}

/** Pastes copies shifted by `offset`, re-linking lines between copied shapes. */
export function paste(items = clipboard, offset = 0.5) {
  if (!items.length) return
  checkpoint()
  const ids = new Map<string, string>()
  const copies = structuredClone(items)
  for (const e of copies) {
    const id = newId(e.type === 'line' ? 'e' : e.type === 'shape' && e.shape === 'text' ? 't' : 'n')
    ids.set(e.id, id)
    e.id = id
    app.diagram.elements.push(e)
  }
  const move = (p: Point) => ({ x: p.x + offset, y: p.y + offset })
  for (const e of copies) {
    const el = find(app.diagram, e.id)!
    if (el.type === 'shape') {
      el.x += offset
      el.y += offset
    } else {
      el.via = el.via.map(move)
      for (const k of ['from', 'to'] as const) {
        const end = el[k]
        if (!isNode(end)) el[k] = move(end)
        else if (ids.has(end.node)) el[k] = { node: ids.get(end.node)! }
      }
    }
  }
  app.selection = copies.map((e) => e.id)
}

export function duplicate() {
  copy()
  paste()
}

export function selectAll() {
  app.selection = app.diagram.elements.map((e) => e.id)
}

let toastTimer: ReturnType<typeof setTimeout>
export function toast(msg: string) {
  app.toast = msg
  clearTimeout(toastTimer)
  toastTimer = setTimeout(() => (app.toast = ''), 4000)
}
