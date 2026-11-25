import { endCenter, find } from './geometry'
import { deleteDoc, listDocs, loadDoc, saveDoc, type DocMeta } from './docs'
import { defaultStyle, isNode, type Diagram, type Element, type Point, type ShapeKind } from './types'

export type Tool = 'select' | ShapeKind | 'line' | 'arrow'

const CURRENT_KEY = 'dia:current'

export const app = $state({
  diagram: { elements: [] } as Diagram,
  name: 'untitled',
  docId: crypto.randomUUID() as string,
  recents: [] as DocMeta[],
  selection: [] as string[],
  tool: 'select' as Tool,
  editing: null as string | null,
  view: { x: 80, y: 80, zoom: 1 },
  showCode: false,
  toast: '',
})

/** Loads the last opened document from the browser. */
export async function loadDocuments() {
  app.recents = await listDocs()
  const id = localStorage.getItem(CURRENT_KEY) ?? app.recents[0]?.id
  if (id) await openDocument(id)
}

let saveTimer: ReturnType<typeof setTimeout>
$effect.root(() => {
  $effect(() => {
    const doc = { id: app.docId, name: app.name, diagram: $state.snapshot(app.diagram) as Diagram, updated: Date.now() }
    if (!doc.diagram.elements.length && doc.name === 'untitled') return
    clearTimeout(saveTimer)
    saveTimer = setTimeout(async () => {
      await saveDoc(doc)
      localStorage.setItem(CURRENT_KEY, doc.id)
      app.recents = await listDocs()
    }, 300)
  })
})

function load(doc: { id: string; name: string; diagram: Diagram }) {
  app.docId = doc.id
  app.name = doc.name
  app.diagram = doc.diagram
  app.selection = []
  app.editing = null
  undoStack.length = 0
  redoStack.length = 0
}

export function newDocument(diagram: Diagram = { elements: [] }, name = 'untitled') {
  load({ id: crypto.randomUUID(), name, diagram })
}

export async function openDocument(id: string) {
  const doc = await loadDoc(id)
  if (doc) load(doc)
  localStorage.setItem(CURRENT_KEY, id)
}

export async function deleteDocument(id: string) {
  await deleteDoc(id)
  app.recents = await listDocs()
  if (id === app.docId) newDocument()
}

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

export function setDiagram(d: Diagram) {
  checkpoint()
  app.diagram = d
  app.selection = []
  app.editing = null
}

export function newId(prefix: string) {
  for (let i = 1; ; i++) if (!find(app.diagram, prefix + i)) return prefix + i
}

export function addShape(shape: ShapeKind, x: number, y: number, w: number, h: number, src?: string) {
  const id = newId(shape === 'text' ? 't' : shape === 'image' ? 'i' : 'n')
  const style = defaultStyle()
  if (shape === 'text' || shape === 'image') style.stroke = null
  app.diagram.elements.push({ id, type: 'shape', shape, x, y, w, h, style, label: '', ...(src && { src }) })
  return id
}

export function addLine(from: Element['id'] | Point, to: Point, arrow: boolean) {
  const id = newId('e')
  const style = { ...defaultStyle(), end: arrow ? ('stealth' as const) : ('none' as const) }
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
