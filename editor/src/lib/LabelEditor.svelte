<script lang="ts" module>
  import { Vim } from '@replit/codemirror-vim'

  // `:w` / `:wq` submit and `:q` cancels the active label editor in vim mode.
  let active: { submit: () => void; cancel: () => void } | null = null
  Vim.defineEx('write', 'w', () => active?.submit())
  Vim.defineEx('wq', 'wq', () => active?.submit())
  Vim.defineEx('quit', 'q', () => active?.cancel())
</script>

<script lang="ts">
  import { untrack } from 'svelte'
  import CodeEditor from './CodeEditor.svelte'
  import { bounds, find, labelPoint } from './geometry'
  import { app, checkpoint, remove } from './state.svelte'

  let { id, scale }: { id: string; scale: number } = $props()

  const initial = untrack(() => find(app.diagram, id)?.label ?? '')
  let value = initial
  app.draft = initial

  const el = $derived(find(app.diagram, id))
  /** The editor sits just below the element, centered on its label, so the label stays visible. */
  const pos = $derived.by(() => {
    if (!el) return { x: 0, y: 0 }
    const lp = labelPoint(app.diagram, el)
    const b = bounds(app.diagram, el)
    const below = el.type === 'shape' ? b.y + b.h : lp.y + 0.4
    return { x: app.view.x + lp.x * scale, y: app.view.y + below * scale + 10 }
  })

  /** In Typst a single newline is a space; typed line breaks in plain markup become `\` breaks. */
  function typstBreaks(text: string) {
    const lines = text.split('\n')
    const code = (l: string) => /^\s*#|[[({,]\s*$/.test(l)
    return lines
      .map((l, i) => {
        const t = l.trimEnd()
        const next = lines[i + 1]
        const skip =
          next === undefined || !t || !next.trim() || t.endsWith('\\') || code(t) || code(next) || /^\s*[\])}]/.test(next)
        return skip ? t : `${t} \\`
      })
      .join('\n')
  }

  function commit() {
    const target = id
    if (app.editing !== target) return
    const label = typstBreaks(value.trim())
    app.editing = null
    app.draft = null
    active = null
    const e = find(app.diagram, target)
    if (!e) return
    if (e.type === 'shape' && e.shape === 'text' && !label) {
      remove([target])
      return
    }
    if (label === e.label) return
    checkpoint()
    e.label = label
  }

  function cancel() {
    value = initial
    commit()
  }

  active = { submit: commit, cancel }
</script>

<div class="editor" style:left="{pos.x}px" style:top="{pos.y}px">
  <CodeEditor
    value={initial}
    autofocus
    placeholder="Typst, e.g. $x^2$"
    onchange={(v) => {
      value = v
      app.draft = typstBreaks(v.trim())
    }}
    onsubmit={commit}
    oncancel={cancel}
    onblur={commit}
  />
  <div class="hint">
    {#if app.settings.vim}<kbd>:w</kbd> done · <kbd>:q</kbd> cancel{:else}<kbd>⌘↵</kbd> done · <kbd>esc</kbd> cancel{/if}
  </div>
</div>

<style>
  .editor {
    position: absolute;
    transform: translateX(-50%);
    z-index: 10;
    display: flex;
    flex-direction: column;
    min-width: 280px;
    max-width: 560px;
    max-height: 320px;
    background: var(--panel);
    border-radius: 10px;
    box-shadow: var(--shadow), 0 0 0 1.5px var(--accent);
    overflow: hidden;
  }
  .editor :global(.cm-editor) {
    max-height: 290px;
  }
  .hint {
    padding: 4px 12px 5px;
    border-top: 1px solid var(--line);
    font-size: 11px;
    color: var(--muted);
  }
  kbd {
    font: 11px var(--mono);
    color: var(--text);
  }
</style>
