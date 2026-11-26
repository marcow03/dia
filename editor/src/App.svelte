<script lang="ts">
  import { reorder } from './lib/arrange'
  import Canvas from './lib/Canvas.svelte'
  import CodePanel from './lib/CodePanel.svelte'
  import { ready } from './lib/core'
  import { download, openFiles, pickFile, pickImage } from './lib/files'
  import Icon from './lib/Icon.svelte'
  import Menu from './lib/Menu.svelte'
  import {
    app,
    checkpoint,
    copy,
    duplicate,
    expandGroups,
    groupSelection,
    ungroupSelection,
    loadDocuments,
    newDocument,
    zoomBy,
    zoomTo,
    zoomToFit,
    paste,
    redo,
    remove,
    selectAll,
    undo,
    type Tool,
  } from './lib/state.svelte'
  import Settings from './lib/Settings.svelte'
  import StylePanel from './lib/StylePanel.svelte'
  import Toolbar from './lib/Toolbar.svelte'
  import ZoomControls from './lib/ZoomControls.svelte'
  import { bounds, find, PX_PER_CM } from './lib/geometry'

  let menuOpen = $state(false)
  const loaded = Promise.all([ready, loadDocuments()])

  // Files opened with the installed app (via the manifest's file handlers).
  type LaunchParams = { files: { getFile(): Promise<File> }[] }
  const launchQueue = (window as unknown as { launchQueue?: { setConsumer(f: (p: LaunchParams) => void): void } }).launchQueue
  launchQueue?.setConsumer(async ({ files }) => {
    await loaded
    if (files.length) openFiles(await Promise.all(files.map((f) => f.getFile())))
  })

  const toolKeys: Record<string, Tool> = {
    v: 'select',
    h: 'hand',
    r: 'rect',
    o: 'ellipse',
    d: 'diamond',
    l: 'line',
    a: 'arrow',
    t: 'text',
  }

  function keydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      ;(document.activeElement as HTMLElement | null)?.blur()
      app.selection = []
      app.tool = 'select'
      app.showCode = false
      app.showSettings = false
      menuOpen = false
      return
    }
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return
    const mod = e.metaKey || e.ctrlKey
    const k = e.key.toLowerCase()
    // Arrows or vim's hjkl: move the selection (⇧: by a pixel), or with ⌃⇧ jump to the adjacent object.
    // Plain ⌃+arrows belong to macOS (switching desktops). hjkl only act as arrows with a selection
    // (or ⌃⇧), so h and l stay tool shortcuts otherwise.
    const dir = directions[k]
    const adjacent = e.ctrlKey && e.shiftKey
    if (dir && !e.metaKey && !e.altKey && (e.ctrlKey ? adjacent : k.startsWith('arrow') || app.selection.length)) {
      e.preventDefault()
      if (adjacent) selectAdjacent(dir)
      else if (app.selection.length) nudge(dir, e.shiftKey ? 1 / (app.view.zoom * PX_PER_CM) : 0.25)
      return
    }
    if (mod && k === 'z') e.shiftKey ? redo() : undo()
    else if (mod && k === 'y') redo()
    else if (mod && k === 'c') copy()
    else if (mod && k === 'x') {
      copy()
      checkpoint()
      remove(app.selection)
    } else if (mod && k === 'v') paste()
    else if (mod && k === 'd') duplicate()
    else if (mod && k === 'a') selectAll()
    else if (mod && k === 's') download('cetz', 'typ')
    else if (mod && k === 'o') pickFile()
    else if (mod && k === 'n') newDocument()
    else if (mod && k === ',') app.showSettings = true
    else if (mod && k === 'g') e.shiftKey ? ungroupSelection() : groupSelection()
    else if (mod && e.code === 'BracketRight') reorder(e.shiftKey ? 'front' : 'forward')
    else if (mod && e.code === 'BracketLeft') reorder(e.shiftKey ? 'back' : 'backward')
    else if (mod && (k === '=' || k === '+')) zoomBy(1.25)
    else if (mod && k === '-') zoomBy(1 / 1.25)
    else if (mod && k === '0') zoomTo(1)
    else if (mod) return
    else if (k === 'backspace' || k === 'delete') {
      if (!app.selection.length) return
      checkpoint()
      remove(app.selection)
    } else if (k === 'enter' && app.selection.length === 1) app.editing = app.selection[0]
    else if (k === 'i') pickImage()
    else if (e.shiftKey && e.code === 'Digit1') zoomToFit()
    else if (e.shiftKey && e.code === 'KeyC') app.showCode = !app.showCode
    else if (toolKeys[k]) app.tool = toolKeys[k]
    else return
    e.preventDefault()
  }

  const directions: Record<string, [number, number]> = {
    arrowleft: [-1, 0],
    arrowright: [1, 0],
    arrowup: [0, -1],
    arrowdown: [0, 1],
    h: [-1, 0],
    l: [1, 0],
    k: [0, -1],
    j: [0, 1],
  }

  function nudge([ux, uy]: [number, number], step: number) {
    checkpoint()
    const [dx, dy] = [ux * step, uy * step]
    for (const id of app.selection) {
      const e = find(app.diagram, id)
      if (e?.type === 'shape') {
        e.x += dx
        e.y += dy
      } else if (e?.type === 'line') {
        e.via = e.via.map((p) => ({ x: p.x + dx, y: p.y + dy }))
        if (!('node' in e.from)) e.from = { x: e.from.x + dx, y: e.from.y + dy }
        if (!('node' in e.to)) e.to = { x: e.to.x + dx, y: e.to.y + dy }
      }
    }
  }

  /**
   * Selects the closest object whose center lies within 60° of `dir` from the selection's center
   * (or from the middle of the window when nothing is selected).
   */
  function selectAdjacent([ux, uy]: [number, number]) {
    const d = app.diagram
    const mid = (e: (typeof d.elements)[number]) => {
      const b = bounds(d, e)
      return { x: b.x + b.w / 2, y: b.y + b.h / 2 }
    }
    const current = app.selection.map((id) => find(d, id)).filter((e) => e !== undefined)
    const scale = app.view.zoom * PX_PER_CM
    const from = current.length
      ? mid(current[0])
      : { x: (innerWidth / 2 - app.view.x) / scale, y: (innerHeight / 2 - app.view.y) / scale }
    const skip = new Set(app.selection)
    let best: { id: string; score: number } | null = null
    for (const e of d.elements) {
      if (skip.has(e.id)) continue
      const c = mid(e)
      const [vx, vy] = [c.x - from.x, c.y - from.y]
      const dist = Math.hypot(vx, vy)
      const along = vx * ux + vy * uy
      if (current.length && (along <= 0 || along < dist * Math.cos(Math.PI / 3))) continue
      const score = current.length ? dist * (2 - along / dist) : dist
      if (!best || score < best.score) best = { id: e.id, score }
    }
    if (best) app.selection = expandGroups([best.id])
  }

  function focusName(e: FocusEvent) {
    if (app.name === 'untitled') app.name = ''
    ;(e.target as HTMLInputElement).select()
  }
</script>

<svelte:window onkeydown={keydown} onpointerdown={() => (menuOpen = false)} />

<main>
  {#await loaded}
    <div class="loading">Loading…</div>
  {:then}
    <Canvas />
    <header class="top">
      <div class="title">
        <div class="menu-anchor" role="presentation" onpointerdown={(e) => e.stopPropagation()}>
          <button class="icon" class:on={menuOpen} title="Menu" onclick={() => (menuOpen = !menuOpen)}>
            <Icon name="menu" />
          </button>
          {#if menuOpen}<Menu onclose={() => (menuOpen = false)} />{/if}
        </div>
        <span class="logo">dia</span>
        <input
          bind:value={app.name}
          onfocus={focusName}
          onblur={() => (app.name = app.name.trim() || 'untitled')}
          onkeydown={(e) => e.key === 'Enter' && (e.target as HTMLInputElement).blur()}
          placeholder="untitled"
          spellcheck="false"
          aria-label="Document name"
        />
      </div>
    </header>
    <StylePanel />
    <Toolbar />
    <ZoomControls />
    {#if app.showCode}<CodePanel />{/if}
    {#if app.diagram.elements.length === 0}
      <p class="empty">Pick a tool below, or double-click anywhere to write Typst.<br />Drop a .typ, .tex or .drawio file or an image here.</p>
    {/if}
    {#if app.showSettings}<Settings />{/if}
    {#if app.toast}<div class="toast">{app.toast}</div>{/if}
  {/await}
</main>

<style>
  main {
    position: fixed;
    inset: 0;
    overflow: hidden;
  }
  .loading,
  .empty {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: var(--muted);
    text-align: center;
    line-height: 1.6;
    pointer-events: none;
  }
  .top {
    position: absolute;
    top: 12px;
    left: 16px;
    right: 16px;
    display: flex;
    justify-content: space-between;
    pointer-events: none;
  }
  .top > * {
    pointer-events: auto;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 5px;
    background: var(--panel);
    border-radius: 12px;
    box-shadow: var(--shadow);
  }
  .menu-anchor {
    position: relative;
  }
  .logo {
    padding: 0 12px 0 6px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .title input {
    width: 180px;
    height: 30px;
    padding: 0 8px;
    border: none;
    border-radius: 8px;
    background: transparent;
    font: inherit;
    color: var(--muted);
    outline: none;
  }
  .title input:hover {
    background: var(--subtle);
  }
  .title input:focus {
    background: var(--subtle);
    color: var(--text);
  }
  .toast {
    position: absolute;
    bottom: 80px;
    left: 50%;
    transform: translateX(-50%);
    max-width: 560px;
    padding: 10px 16px;
    background: var(--text);
    color: var(--panel);
    font-size: 13px;
    border-radius: 10px;
    box-shadow: var(--shadow);
  }
</style>
