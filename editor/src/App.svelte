<script lang="ts">
  import Canvas from './lib/Canvas.svelte'
  import CodePanel from './lib/CodePanel.svelte'
  import { ready } from './lib/core'
  import { download, pickFile, pickImage } from './lib/files'
  import Icon from './lib/Icon.svelte'
  import Menu from './lib/Menu.svelte'
  import {
    app,
    checkpoint,
    copy,
    duplicate,
    loadDocuments,
    newDocument,
    paste,
    redo,
    remove,
    selectAll,
    undo,
    type Tool,
  } from './lib/state.svelte'
  import StylePanel from './lib/StylePanel.svelte'
  import Toolbar from './lib/Toolbar.svelte'
  import { find } from './lib/geometry'

  let menuOpen = $state(false)
  const loaded = Promise.all([ready, loadDocuments()])

  const toolKeys: Record<string, Tool> = {
    v: 'select',
    r: 'rect',
    o: 'ellipse',
    d: 'diamond',
    l: 'line',
    a: 'arrow',
    t: 'text',
  }

  function keydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return
    const mod = e.metaKey || e.ctrlKey
    const k = e.key.toLowerCase()
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
    else if (mod) return
    else if (k === 'backspace' || k === 'delete') {
      if (!app.selection.length) return
      checkpoint()
      remove(app.selection)
    } else if (k === 'escape') {
      app.selection = []
      app.tool = 'select'
    } else if (k === 'enter' && app.selection.length === 1) app.editing = app.selection[0]
    else if (k.startsWith('arrow') && app.selection.length) nudge(k, e.shiftKey ? 1 : 0.25)
    else if (k === 'i') pickImage()
    else if (toolKeys[k]) app.tool = toolKeys[k]
    else return
    e.preventDefault()
  }

  function nudge(key: string, step: number) {
    checkpoint()
    const [dx, dy] = { arrowleft: [-step, 0], arrowright: [step, 0], arrowup: [0, -step], arrowdown: [0, step] }[key]!
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
      <div class="actions">
        <button class="icon" class:on={app.showCode} title="Show CeTZ code" onclick={() => (app.showCode = !app.showCode)}>
          <Icon name="code" />
        </button>
      </div>
    </header>
    <StylePanel />
    <Toolbar />
    {#if app.showCode}<CodePanel />{/if}
    {#if app.diagram.elements.length === 0}
      <p class="empty">Pick a tool below, or double-click anywhere to write Typst.<br />Drop a .typ, .tex or .drawio file or an image here.</p>
    {/if}
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
  .title,
  .actions {
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
