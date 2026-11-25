<script lang="ts">
  import { align, distribute, matchSize, reorder, type Align, type Order } from './arrange'
  import ColorPicker from './ColorPicker.svelte'
  import Icon from './Icon.svelte'
  import { arrowHead, find, PT_PER_CM } from './geometry'
  import NumberInput from './NumberInput.svelte'
  import Select from './Select.svelte'
  import { app, checkpoint, type DrawTool } from './state.svelte'
  import { arrows, fonts, type Arrow, type Line, type Style } from './types'

  type Kind = 'rect' | 'shape' | 'text' | 'image' | 'line'
  type Target = { style: Style; kind: Kind; line?: Line }

  const kindOf: Record<DrawTool, Kind> = {
    rect: 'rect',
    ellipse: 'shape',
    diamond: 'shape',
    text: 'text',
    line: 'line',
    arrow: 'line',
  }

  /** The selection, or the active tool's defaults when nothing is selected. */
  const targets = $derived.by((): Target[] => {
    if (app.selection.length) {
      return app.selection
        .map((id) => find(app.diagram, id))
        .filter((e) => e !== undefined)
        .map((e) =>
          e.type === 'line'
            ? { style: e.style, kind: 'line', line: e }
            : { style: e.style, kind: e.shape === 'rect' ? 'rect' : e.shape === 'ellipse' || e.shape === 'diamond' ? 'shape' : e.shape },
        )
    }
    if (app.tool === 'select' || app.tool === 'hand' || app.tool === 'image') return []
    return [{ style: app.defaults[app.tool], kind: kindOf[app.tool] }]
  })
  const editingDefaults = $derived(!app.selection.length)
  const s = $derived(targets[0]?.style)
  const has = (...kinds: Kind[]) => targets.some((t) => kinds.includes(t.kind))
  const lines = $derived(targets.flatMap((t) => (t.line ? [t.line] : [])))

  function change(f: () => void) {
    if (!editingDefaults) checkpoint()
    f()
  }

  /** Keys shared by all tools: setting them once updates every tool's defaults. */
  const shared: (keyof Style)[] = ['stroke', 'fill', 'width', 'dash', 'font', 'size', 'bold', 'italic', 'color']
  const textOnly: (keyof Style)[] = ['font', 'size', 'bold', 'italic', 'color']

  function set<K extends keyof Style>(key: K, value: Style[K]) {
    change(() => targets.forEach((t) => (t.style[key] = value)))
    if (!shared.includes(key)) return
    for (const [tool, style] of Object.entries(app.defaults)) {
      if (tool !== 'text' || textOnly.includes(key)) style[key] = value
    }
  }

  function direction(start: boolean, end: boolean) {
    change(() => {
      for (const t of targets) {
        if (t.kind !== 'line') continue
        const st = t.style
        const head = st.end !== 'none' ? st.end : st.start !== 'none' ? st.start : 'stealth'
        st.start = start ? head : 'none'
        st.end = end ? head : 'none'
      }
    })
  }

  const setCurved = (curved: boolean) => change(() => lines.forEach((l) => (l.curved = curved)))

  const shapeCount = $derived(targets.filter((t) => t.kind !== 'line').length)
  const alignments: [Align, string, string][] = [
    ['left', 'alignLeft', 'Align left'],
    ['center', 'alignCenter', 'Align centers horizontally'],
    ['right', 'alignRight', 'Align right'],
    ['top', 'alignTop', 'Align top'],
    ['middle', 'alignMiddle', 'Align centers vertically'],
    ['bottom', 'alignBottom', 'Align bottom'],
  ]

  const orders: [Order, string, string][] = [
    ['back', 'toBack', 'Send to back (⇧⌘[)'],
    ['backward', 'backward', 'Send backward (⌘[)'],
    ['forward', 'forward', 'Bring forward (⌘])'],
    ['front', 'toFront', 'Bring to front (⇧⌘])'],
  ]

  const headOptions = (none: string) => arrows.map((a) => ({ value: a, label: a === 'none' ? none : a }))
  const fontOptions = [{ value: '', label: 'Default font' }, ...fonts.map((f) => ({ value: f, label: f }))]
</script>

{#snippet head(a: Arrow, name: string, flip = false)}
  {@const h = arrowHead(a, { x: flip ? 3 : 25, y: 6 }, { x: flip ? 25 : 3, y: 6 }, 9)}
  <svg width="28" height="12" viewBox="0 0 28 12" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round">
    <path d="M3 6H25" />
    {#if h.d}<path d={h.d} fill={h.filled ? 'currentColor' : 'none'} />{/if}
  </svg>
  <span>{name}</span>
{/snippet}

{#if s}
  <div class="panel">
    {#if editingDefaults}<div class="hint">New {app.tool}</div>{/if}

    {#if !editingDefaults}
      <section>
        <div class="row">
          <span class="label">Layer</span>
          <div class="icons grow layer">
            {#each orders as [how, icon, title]}
              <button {title} onclick={() => reorder(how)}><Icon name={icon} /></button>
            {/each}
          </div>
        </div>
      </section>
    {/if}

    {#if !editingDefaults && shapeCount >= 2}
      <section>
        <div class="icons">
          {#each alignments as [how, icon, title]}
            <button {title} onclick={() => align(how)}><Icon name={icon} /></button>
          {/each}
        </div>
        <div class="icons">
          <button title="Equal horizontal gaps" disabled={shapeCount < 3} onclick={() => distribute('x')}><Icon name="distributeH" /></button>
          <button title="Equal vertical gaps" disabled={shapeCount < 3} onclick={() => distribute('y')}><Icon name="distributeV" /></button>
          <button title="Same width" onclick={() => matchSize('w')}><Icon name="sameWidth" /></button>
          <button title="Same height" onclick={() => matchSize('h')}><Icon name="sameHeight" /></button>
        </div>
      </section>
    {/if}

    {#if has('rect', 'shape', 'line')}
      <section>
        <div class="row">
          <span class="label">Stroke</span>
          <ColorPicker value={s.stroke} allowNone onchange={(v) => set('stroke', v)} />
          <NumberInput label="Stroke width" value={s.width} step={0.25} onchange={(v) => set('width', v)} />
        </div>
        <div class="seg">
          {#each ['solid', 'dashed', 'dotted'] as const as dash}
            <button class:on={s.dash === dash} onclick={() => set('dash', dash)}>{dash}</button>
          {/each}
        </div>
      </section>
    {/if}

    {#if has('rect', 'shape')}
      <section>
        <div class="row">
          <span class="label">Fill</span>
          <ColorPicker value={s.fill} allowNone onchange={(v) => set('fill', v)} />
          {#if has('rect')}
            <NumberInput
              label="Corner radius"
              prefix="R"
              value={s.radius * PT_PER_CM}
              onchange={(v) => set('radius', v / PT_PER_CM)}
            />
          {/if}
        </div>
      </section>
    {/if}

    {#if has('line')}
      <section>
        <div class="row">
          <span class="label">Arrows</span>
          <div class="seg grow">
            <button title="No arrows" class:on={s.start === 'none' && s.end === 'none'} onclick={() => direction(false, false)}>—</button>
            <button title="Forward" class:on={s.start === 'none' && s.end !== 'none'} onclick={() => direction(false, true)}>→</button>
            <button title="Backward" class:on={s.start !== 'none' && s.end === 'none'} onclick={() => direction(true, false)}>←</button>
            <button title="Both" class:on={s.start !== 'none' && s.end !== 'none'} onclick={() => direction(true, true)}>↔</button>
          </div>
        </div>
        <div class="row">
          <Select label="Start head" value={s.start} options={headOptions('Start')} onchange={(v) => set('start', v)}>
            {#snippet item(a, name)}{@render head(a, name, true)}{/snippet}
          </Select>
          <Select label="End head" value={s.end} options={headOptions('End')} onchange={(v) => set('end', v)}>
            {#snippet item(a, name)}{@render head(a, name)}{/snippet}
          </Select>
        </div>
        {#if lines.length}
          <div class="seg">
            <button class:on={!lines[0].curved} onclick={() => setCurved(false)}>Straight</button>
            <button class:on={lines[0].curved} onclick={() => setCurved(true)}>Curved</button>
          </div>
        {/if}
      </section>
    {/if}

    {#if has('rect', 'shape', 'text', 'line')}
      <section>
        <div class="row">
          <span class="label">Text</span>
          <ColorPicker value={s.color} onchange={(v) => set('color', v ?? '#000000')} />
          <NumberInput label="Font size" value={s.size} min={1} onchange={(v) => set('size', v)} />
        </div>
        <div class="row">
          <Select label="Font" value={s.font ?? ''} options={fontOptions} onchange={(v) => set('font', v || null)} />
          <div class="seg">
            <button class="b" class:on={s.bold} title="Bold" onclick={() => set('bold', !s.bold)}>B</button>
            <button class="i" class:on={s.italic} title="Italic" onclick={() => set('italic', !s.italic)}>I</button>
          </div>
        </div>
      </section>
    {/if}
  </div>
{/if}

<style>
  .panel {
    position: absolute;
    top: 12px;
    right: 16px;
    width: 256px;
    padding: 4px 12px;
    background: var(--panel);
    border-radius: 14px;
    box-shadow: var(--shadow);
  }
  .hint {
    padding: 8px 0 0;
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    text-transform: capitalize;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 0;
  }
  section + section {
    border-top: 1px solid var(--line);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .label {
    width: 44px;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
  }
  .grow {
    flex: 1;
  }
  .row :global(.value span) {
    text-transform: capitalize;
  }
  .seg {
    display: flex;
    padding: 2px;
    gap: 2px;
    background: var(--subtle);
    border-radius: 8px;
  }
  .seg button {
    flex: 1;
    min-width: 28px;
    height: 24px;
    font-size: 12px;
    text-transform: capitalize;
    border-radius: 6px;
  }
  .seg button.on {
    background: var(--panel);
    box-shadow: 0 1px 2px rgb(0 0 0 / 0.12);
  }
  .icons {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    justify-items: center;
  }
  .icons button {
    min-width: 0;
    width: 34px;
    height: 30px;
    color: var(--text);
  }
  .icons.layer {
    grid-template-columns: repeat(4, 1fr);
  }
  .icons button:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .b {
    font-weight: 700;
  }
  .i {
    font-style: italic;
    font-family: Georgia, serif;
  }
</style>
