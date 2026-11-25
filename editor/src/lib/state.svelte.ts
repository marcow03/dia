import { bounds, endCenter, find, PT_PER_CM, PX_PER_CM, rotatePoint } from './geometry'
import { renderLabel } from './typst'
import { deleteDoc, listDocs, loadDoc, saveDoc, type DocMeta } from './docs'
import { defaultStyle, isNode, type Diagram, type Element, type Point, type Shape, type ShapeKind, type Style } from './types'

export type Tool = 'select' | 'hand' | ShapeKind | 'line' | 'arrow'
export type DrawTool = Exclude<Tool, 'select' | 'hand' | 'image'>

const CURRENT_KEY = 'dia:current'
const SETTINGS_KEY = 'dia:settings'

export type Settings = { vim: boolean }

function loadSettings(): Settings {
  try {
    return { vim: false, ...JSON.parse(localStorage.getItem(SETTINGS_KEY) ?? '{}') }
  } catch {
    return { vim: false }
  }
}
const DEFAULTS_KEY = 'dia:tool-defaults'

export function factoryDefaults(): Record<DrawTool, Style> {
  const d = () => defaultStyle()
  return {
    rect: d(),
    ellipse: d(),
    diamond: d(),
    line: d(),
    arrow: { ...d(), end: 'stealth' as const },
    text: { ...d(), stroke: null },
  }
}

function toolDefaults(): Record<DrawTool, Style> {
  const all = factoryDefaults()
  try {
    return { ...all, ...JSON.parse(localStorage.getItem(DEFAULTS_KEY) ?? '{}') }
  } catch {
    return all
  }
}

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
  /** Style given to new elements, per tool; edited in the style panel while a tool is active. */
  defaults: toolDefaults(),
  settings: loadSettings(),
  showSettings: false,
  /** Label being typed in the label editor, rendered live in place. */
  draft: null as string | null,
})

/** Loads the last opened document from the browser. */
export async function loadDocuments() {
  app.recents = await listDocs()
  const id = localStorage.getItem(CURRENT_KEY) ?? app.recents[0]?.id
  if (id) await openDocument(id)
}

let saveTimer: ReturnType<typeof setTimeout>
$effect.root(() => {
  $effect(() => localStorage.setItem(DEFAULTS_KEY, JSON.stringify(app.defaults)))
  $effect(() => localStorage.setItem(SETTINGS_KEY, JSON.stringify(app.settings)))
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
  const style = shape === 'image' ? { ...defaultStyle(), stroke: null } : structuredClone($state.snapshot(app.defaults[shape as DrawTool]))
  app.diagram.elements.push({ id, type: 'shape', shape, x, y, w, h, style, label: '', ...(src && { src }) })
  return id
}

/** Sizes a text box to its rendered label, keeping the top-left corner in place. */
export async function fitTextBox(e: Shape, source = e.label) {
  const r = await renderLabel(source, e.style)
  if (!('svg' in r)) return
  const w = Math.max(r.w / PT_PER_CM + 0.3, 0.5)
  const h = Math.max(r.h / PT_PER_CM + 0.2, 0.5)
  if (Math.abs(e.w - w) > 0.01 || Math.abs(e.h - h) > 0.01) {
    e.w = w
    e.h = h
  }
}

export function addLine(from: Element['id'] | Point, to: Point, tool: 'line' | 'arrow') {
  const id = newId('e')
  const style = structuredClone($state.snapshot(app.defaults[tool]))
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
  const groups = new Map<string, string>()
  const used = new Set(app.diagram.elements.map((e) => e.group))
  for (const e of copies) {
    if (!e.group) continue
    if (!groups.has(e.group)) {
      let n = 1
      while (used.has(`g${n}`)) n++
      used.add(`g${n}`)
      groups.set(e.group, `g${n}`)
    }
    e.group = groups.get(e.group)
  }
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

/** Adds all members of any group touched by `ids`. */
export function expandGroups(ids: string[]): string[] {
  const groups = new Set(ids.map((id) => find(app.diagram, id)?.group).filter((g) => g))
  if (!groups.size) return ids
  const members = app.diagram.elements.filter((e) => e.group && groups.has(e.group)).map((e) => e.id)
  return [...new Set([...ids, ...members])]
}

/** Groups the selection, keeping members adjacent in the stacking order at the topmost member. */
export function groupSelection() {
  if (app.selection.length < 2) return
  checkpoint()
  const sel = new Set(app.selection)
  const used = new Set(app.diagram.elements.map((e) => e.group))
  let n = 1
  while (used.has(`g${n}`)) n++
  const members = app.diagram.elements.filter((e) => sel.has(e.id))
  for (const e of members) e.group = `g${n}`
  const lastIndex = Math.max(...members.map((e) => app.diagram.elements.indexOf(e)))
  const before = app.diagram.elements.slice(0, lastIndex + 1).filter((e) => !sel.has(e.id))
  const after = app.diagram.elements.slice(lastIndex + 1)
  app.diagram.elements = [...before, ...members, ...after]
}

export function ungroupSelection() {
  checkpoint()
  for (const id of app.selection) {
    const e = find(app.diagram, id)
    if (e) delete e.group
  }
}

/** Rotates elements (from their snapshots) clockwise by `deg` around `pivot`. */
export function rotateElements(orig: Element[], deg: number, pivot: Point) {
  for (const o of orig) {
    const e = find(app.diagram, o.id)
    if (!e) continue
    if (e.type === 'shape' && o.type === 'shape') {
      const c = rotatePoint({ x: o.x + o.w / 2, y: o.y + o.h / 2 }, pivot, deg)
      e.x = c.x - o.w / 2
      e.y = c.y - o.h / 2
      const r = ((((o.rotate ?? 0) + deg) % 360) + 360) % 360
      e.rotate = Math.round((r > 180 ? r - 360 : r) * 100) / 100 || undefined
    } else if (e.type === 'line' && o.type === 'line') {
      e.via = o.via.map((p) => rotatePoint(p, pivot, deg))
      if (!isNode(o.from)) e.from = rotatePoint(o.from, pivot, deg)
      if (!isNode(o.to)) e.to = rotatePoint(o.to, pivot, deg)
    }
  }
}

export function duplicate() {
  copy()
  paste()
}

export function selectAll() {
  app.selection = app.diagram.elements.map((e) => e.id)
}

/** Zooms by `factor` keeping the screen point `at` (default: window center) fixed. */
export function zoomBy(factor: number, at = { x: innerWidth / 2, y: innerHeight / 2 }) {
  const zoom = Math.min(8, Math.max(0.1, app.view.zoom * factor))
  const k = zoom / app.view.zoom
  app.view = { x: at.x - (at.x - app.view.x) * k, y: at.y - (at.y - app.view.y) * k, zoom }
}

export function zoomTo(zoom: number) {
  zoomBy(zoom / app.view.zoom)
}

/** Fits all elements into the window. */
export function zoomToFit() {
  const bs = app.diagram.elements.map((e) => bounds(app.diagram, e))
  if (!bs.length) return (app.view = { x: 80, y: 80, zoom: 1 })
  const x = Math.min(...bs.map((b) => b.x))
  const y = Math.min(...bs.map((b) => b.y))
  const w = Math.max(...bs.map((b) => b.x + b.w)) - x
  const h = Math.max(...bs.map((b) => b.y + b.h)) - y
  const pad = 120
  const zoom = Math.min(4, Math.max(0.1, Math.min((innerWidth - 2 * pad) / w, (innerHeight - 2 * pad) / h) / PX_PER_CM))
  const s = zoom * PX_PER_CM
  app.view = { x: innerWidth / 2 - (x + w / 2) * s, y: innerHeight / 2 - (y + h / 2) * s, zoom }
}

let toastTimer: ReturnType<typeof setTimeout>
export function toast(msg: string) {
  app.toast = msg
  clearTimeout(toastTimer)
  toastTimer = setTimeout(() => (app.toast = ''), 4000)
}
