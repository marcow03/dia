<script lang="ts">
  import { untrack } from 'svelte'
  import { center, find, labelPoint, PT_PER_CM } from './geometry'
  import { app, checkpoint, remove } from './state.svelte'
  import { renderLabel } from './typst'

  let { id, scale }: { id: string; scale: number } = $props()

  const initial = untrack(() => find(app.diagram, id)?.label ?? '')
  let value = $state(initial)
  let area: HTMLTextAreaElement

  const el = $derived(find(app.diagram, id))
  const pos = $derived.by(() => {
    if (!el) return { x: 0, y: 0 }
    const c = labelPoint(app.diagram, el)
    return { x: app.view.x + c.x * scale, y: app.view.y + c.y * scale }
  })
  const fontPx = $derived(((el?.style.size ?? 11) / PT_PER_CM) * scale)

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
      const r = await renderLabel(label, e.style)
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

<textarea
  bind:this={area}
  bind:value
  onkeydown={keydown}
  onblur={commit}
  rows="1"
  spellcheck="false"
  placeholder="Typst…"
  style:left="{pos.x}px"
  style:top="{pos.y}px"
  style:font-size="{fontPx}px"
  style:color={el?.style.color}
  style:font-weight={el?.style.bold ? 700 : 400}
  style:font-style={el?.style.italic ? 'italic' : 'normal'}
></textarea>

<style>
  textarea {
    position: absolute;
    transform: translate(-50%, -50%);
    z-index: 10;
    min-width: 2ch;
    field-sizing: content;
    resize: none;
    padding: 2px 4px;
    border: none;
    border-radius: 3px;
    background: transparent;
    outline: 1px dashed var(--accent);
    outline-offset: 2px;
    font-family: 'Libertinus Serif', 'Iowan Old Style', Georgia, serif;
    line-height: 1.25;
    text-align: center;
    caret-color: var(--accent);
  }
  textarea::placeholder {
    color: var(--muted);
  }
</style>
