<script lang="ts">
  import { untrack } from 'svelte'
  import { find, labelPoint, PT_PER_CM } from './geometry'
  import { app, checkpoint, remove } from './state.svelte'

  let { id, scale }: { id: string; scale: number } = $props()

  const initial = untrack(() => find(app.diagram, id)?.label ?? '')
  let value = $state(initial)
  let area: HTMLTextAreaElement

  const el = $derived(find(app.diagram, id))
  const isText = $derived(el?.type === 'shape' && el.shape === 'text')
  const pos = $derived.by(() => {
    if (!el) return { x: 0, y: 0 }
    const c = el.type === 'shape' && el.shape === 'text' ? { x: el.x, y: el.y } : labelPoint(app.diagram, el)
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
    const label = typstBreaks(value.trim())
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
  }

  /** In Typst a single newline is a space; turn typed line breaks into `\` line breaks. */
  function typstBreaks(text: string) {
    const lines = text.split('\n')
    return lines
      .map((l, i) => {
        const t = l.trimEnd()
        const next = lines[i + 1]
        return next === undefined || !t || !next.trim() || t.endsWith('\\') ? t : `${t} \\`
      })
      .join('\n')
  }

  /** Text boxes grow with the input while typing; they are refit to the rendered Typst afterwards. */
  function input() {
    const e = el
    if (e?.type !== 'shape' || e.shape !== 'text') return
    e.w = Math.max(e.w, area.offsetWidth / scale)
    e.h = Math.max(e.h, area.offsetHeight / scale)
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
  oninput={input}
  onblur={commit}
  class:text={isText}
  rows="1"
  spellcheck="false"
  placeholder="Typst…"
  style:left="{pos.x}px"
  style:top="{pos.y}px"
  style:font-size="{fontPx}px"
  style:color={el?.style.color}
  style:font-weight={el?.style.bold ? 700 : 400}
  style:font-style={el?.style.italic ? 'italic' : 'normal'}
  style:text-align={el?.style.align ?? 'left'}
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
    text-align: left;
    caret-color: var(--accent);
  }
  textarea.text {
    transform: none;
  }
  textarea::placeholder {
    color: var(--muted);
  }
</style>
