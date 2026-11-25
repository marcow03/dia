<script lang="ts">
  import { untrack } from 'svelte'
  import { center, find, lineMid, PT_PER_CM } from './geometry'
  import { app, checkpoint, remove } from './state.svelte'
  import { renderLabel } from './typst'

  let { id, scale }: { id: string; scale: number } = $props()

  const initial = untrack(() => find(app.diagram, id)?.label ?? '')
  let value = $state(initial)
  let area: HTMLTextAreaElement

  const pos = $derived.by(() => {
    const e = find(app.diagram, id)
    if (!e) return { x: 0, y: 0 }
    const c = e.type === 'shape' ? center(e) : lineMid(app.diagram, e)
    return { x: app.view.x + c.x * scale, y: app.view.y + c.y * scale }
  })

  $effect(() => {
    area.focus()
    area.select()
  })

  async function commit() {
    const target = id
    if (app.editing !== target) return
    const label = value.trim()
    app.editing = null
    const e = find(app.diagram, target)
    if (!e) return
    if (e.type === 'shape' && e.shape === 'text' && !label) {
      remove([target])
      return
    }
    if (label === e.label) return
    checkpoint()
    e.label = label
    if (e.type === 'shape' && e.shape === 'text') {
      const r = await renderLabel(label)
      if ('svg' in r) {
        const c = center(e)
        e.w = Math.max(r.w / PT_PER_CM + 0.3, 0.5)
        e.h = Math.max(r.h / PT_PER_CM + 0.2, 0.5)
        e.x = c.x - e.w / 2
        e.y = c.y - e.h / 2
      }
    }
  }

  function keydown(ev: KeyboardEvent) {
    ev.stopPropagation()
    if (ev.key === 'Escape') {
      value = initial
      commit()
    } else if (ev.key === 'Enter' && !ev.shiftKey) {
      ev.preventDefault()
      commit()
    }
  }
</script>

<div class="editor" style:left="{pos.x}px" style:top="{pos.y}px">
  <textarea
    bind:this={area}
    bind:value
    onkeydown={keydown}
    onblur={commit}
    rows={Math.max(1, value.split('\n').length)}
    spellcheck="false"
    placeholder="Typst, e.g. $x^2$"
  ></textarea>
  <div class="hint">Typst markup · <kbd>$…$</kbd> math · <kbd>↵</kbd> done</div>
</div>

<style>
  .editor {
    position: absolute;
    transform: translate(-50%, -50%);
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  textarea {
    min-width: 180px;
    max-width: 420px;
    field-sizing: content;
    resize: none;
    padding: 8px 12px;
    border: none;
    border-radius: 10px;
    background: var(--panel);
    box-shadow: var(--shadow), 0 0 0 2px var(--accent);
    font: 14px/1.4 var(--mono);
    color: var(--text);
    text-align: center;
    outline: none;
  }
  .hint {
    font-size: 11px;
    color: var(--muted);
    background: var(--panel);
    padding: 2px 8px;
    border-radius: 6px;
    box-shadow: var(--shadow);
  }
  kbd {
    font: inherit;
    color: var(--text);
  }
</style>
