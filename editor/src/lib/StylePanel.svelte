<script lang="ts">
  import { align, distribute, matchSize, reorder, type Align, type Order } from './arrange'
  import ColorPicker from './ColorPicker.svelte'
  import { arrowHead, find, PT_PER_CM } from './geometry'
  import Icon from './Icon.svelte'
  import NumberInput from './NumberInput.svelte'
  import Select from './Select.svelte'
  import { app, checkpoint, groupSelection, ungroupSelection, type DrawTool } from './state.svelte'
  import { arrows, fonts, type Arrow, type Element, type Style } from './types'

  type Kind = 'rect' | 'shape' | 'text' | 'image' | 'line'
  type Target = { style: Style; kind: Kind; el?: Element }

  const toolKind: Record<DrawTool, Kind> = {
    rect: 'rect',
    ellipse: 'shape',
    diamond: 'shape',
    text: 'text',
    line: 'line',
    arrow: 'line',
  }

  const kindOf = (e: Element): Kind =>
    e.type === 'line' ? 'line' : e.shape === 'ellipse' || e.shape === 'diamond' ? 'shape' : e.shape

  /** The selection, or the active tool's defaults when nothing is selected. */
  const targets = $derived.by((): Target[] => {
    if (app.selection.length) {
      return app.selection
        .map((id) => find(app.diagram, id))
        .filter((e) => e !== undefined)
        .map((e) => ({ style: e.style, kind: kindOf(e), el: e }))
    }
    if (app.tool === 'select' || app.tool === 'hand' || app.tool === 'image') return []
    return [{ style: app.defaults[app.tool], kind: toolKind[app.tool] }]
  })
  const editingDefaults = $derived(!app.selection.length)
  const s = $derived(targets[0]?.style)
  const has = (...kinds: Kind[]) => targets.some((t) => kinds.includes(t.kind))
  const lines = $derived(targets.flatMap((t) => (t.el?.type === 'line' ? [t.el] : [])))
  const shapeCount = $derived(targets.filter((t) => t.kind !== 'line').length)
  const moved = $derived(targets.some((t) => t.el?.label_at))
  const shapes = $derived(targets.flatMap((t) => (t.el?.type === 'shape' ? [t.el] : [])))
  const grouped = $derived(targets.some((t) => t.el?.group))

  function setRotation(deg: number) {
    change(() => {
      const r = ((((deg + 180) % 360) + 360) % 360) - 180
      for (const e of shapes) e.rotate = r || undefined
    })
  }

  const strokeKeys: (keyof Style)[] = ['stroke', 'fill', 'width', 'dash']
  const textKeys: (keyof Style)[] = ['font', 'size', 'bold', 'italic', 'color', 'align']

  function change(f: () => void) {
    if (!editingDefaults) checkpoint()
    f()
  }

  /**
   * Sets a style key on all targets. Stroke settings become the default for every shape tool;
   * text settings become the default either for labels or for the text tool, matching the targets.
   */
  function set<K extends keyof Style>(key: K, value: Style[K]) {
    change(() => targets.forEach((t) => (t.style[key] = value)))
    const labels = has('rect', 'shape', 'line')
    const texts = has('text')
    for (const [tool, style] of Object.entries(app.defaults)) {
      const isText = tool === 'text'
      if (strokeKeys.includes(key) && !isText) style[key] = value
      if (textKeys.includes(key) && (isText ? texts : labels)) style[key] = value
    }
  }

  function direction(start: boolean, end: boolean) {
    change(() => {
      for (const t of targets) {
        if (t.kind !== 'line') continue
        const head = t.style.end !== 'none' ? t.style.end : t.style.start !== 'none' ? t.style.start : 'stealth'
        t.style.start = start ? head : 'none'
        t.style.end = end ? head : 'none'
      }
    })
  }

  const setCurved = (curved: boolean) => change(() => lines.forEach((l) => (l.curved = curved)))
  const centerLabels = () => change(() => targets.forEach((t) => t.el && (t.el.label_at = undefined)))

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
    {#if editingDefaults}
      <header>New {app.tool}</header>
    {:else}
      <section>
        <h3>Layer</h3>
        <div class="icons four">
          {#each orders as [how, icon, title]}
            <button {title} onclick={() => reorder(how)}><Icon name={icon} /></button>
          {/each}
        </div>
        {#if targets.length >= 2 || grouped}
          <div class="seg">
            <button disabled={targets.length < 2} title="Group (⌘G)" onclick={groupSelection}>Group</button>
            <button disabled={!grouped} title="Ungroup (⇧⌘G)" onclick={ungroupSelection}>Ungroup</button>
          </div>
        {/if}
      </section>
      {#if shapeCount >= 2}
        <section>
          <h3>Arrange</h3>
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
    {/if}

    {#if !editingDefaults && shapes.length}
      <section>
        <h3>Rotation</h3>
        <div class="row">
          <div class="seg grow">
            <button title="Rotate left" onclick={() => setRotation((shapes[0].rotate ?? 0) - 90)}>−90°</button>
            <button title="Reset rotation" onclick={() => setRotation(0)}>0°</button>
            <button title="Rotate right" onclick={() => setRotation((shapes[0].rotate ?? 0) + 90)}>+90°</button>
          </div>
          <NumberInput label="Rotation" unit="°" step={15} min={-360} value={shapes[0].rotate ?? 0} onchange={setRotation} />
        </div>
      </section>
    {/if}

    {#if has('rect', 'shape', 'line')}
      <section>
        <h3>Stroke</h3>
        <div class="row">
          <ColorPicker value={s.stroke} allowNone onchange={(v) => set('stroke', v)} />
          <NumberInput label="Stroke width" value={s.width} step={0.25} onchange={(v) => set('width', v)} />
        </div>
        <div class="seg">
          {#each ['solid', 'dashed', 'dotted'] as const as dash}
            <button class:on={s.dash === dash} onclick={() => set('dash', dash)}>{dash}</button>
          {/each}
        </div>
        {#if has('rect')}
          <div class="row">
            <span class="label">Corner radius</span>
            <NumberInput label="Corner radius" value={s.radius * PT_PER_CM} onchange={(v) => set('radius', v / PT_PER_CM)} />
          </div>
        {/if}
      </section>
    {/if}

    {#if has('rect', 'shape')}
      <section>
        <h3>Fill</h3>
        <div class="row">
          <ColorPicker value={s.fill} allowNone onchange={(v) => set('fill', v)} />
        </div>
      </section>
    {/if}

    {#if has('line')}
      <section>
        <h3>Arrows</h3>
        <div class="seg">
          <button title="No arrows" class:on={s.start === 'none' && s.end === 'none'} onclick={() => direction(false, false)}>—</button>
          <button title="Forward" class:on={s.start === 'none' && s.end !== 'none'} onclick={() => direction(false, true)}>→</button>
          <button title="Backward" class:on={s.start !== 'none' && s.end === 'none'} onclick={() => direction(true, false)}>←</button>
          <button title="Both" class:on={s.start !== 'none' && s.end !== 'none'} onclick={() => direction(true, true)}>↔</button>
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
        <h3>
          {has('text') && !has('rect', 'shape', 'line') ? 'Text' : 'Label'}
          {#if moved}<button class="link" onclick={centerLabels}>Center</button>{/if}
        </h3>
        <div class="row">
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
        <div class="seg">
          {#each [['left', 'textLeft'], ['center', 'textCenter'], ['right', 'textRight']] as const as [a, icon]}
            <button class="icon" class:on={(s.align ?? 'left') === a} title="Align {a}" onclick={() => set('align', a)}>
              <Icon name={icon} />
            </button>
          {/each}
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
    max-height: calc(100vh - 24px);
    overflow-y: auto;
    padding: 2px 12px;
    background: var(--panel);
    border-radius: 14px;
    box-shadow: var(--shadow);
  }
  header {
    padding: 10px 0 2px;
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    text-transform: capitalize;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 0 12px;
  }
  section + section {
    border-top: 1px solid var(--line);
  }
  h3 {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
  }
  .link {
    min-width: 0;
    height: 18px;
    padding: 0 6px;
    font-size: 11px;
    color: var(--accent);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .grow {
    flex: 1;
  }
  .label {
    font-size: 12px;
    color: var(--text);
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
  .seg button:disabled {
    opacity: 0.35;
    cursor: default;
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
  .icons.four {
    grid-template-columns: repeat(4, 1fr);
  }
  .icons button {
    min-width: 0;
    width: 34px;
    height: 30px;
  }
  .icons button:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .seg .icon :global(svg) {
    width: 16px;
    height: 16px;
  }
  .b {
    font-weight: 700;
  }
  .i {
    font-style: italic;
    font-family: Georgia, serif;
  }
</style>
