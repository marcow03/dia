<script lang="ts">
  import ColorPicker from './ColorPicker.svelte'
  import { find, PT_PER_CM } from './geometry'
  import { app, checkpoint } from './state.svelte'
  import { arrows, fonts, type Arrow, type Style } from './types'

  const items = $derived(app.selection.map((id) => find(app.diagram, id)).filter((e) => e !== undefined))
  const first = $derived(items[0])
  const s = $derived(first?.style)
  const isPlain = (e: (typeof items)[number]) => e.type === 'shape' && (e.shape === 'text' || e.shape === 'image')
  const hasStroke = $derived(items.some((e) => !isPlain(e)))
  const hasShapes = $derived(items.some((e) => e.type === 'shape' && !isPlain(e)))
  const hasRects = $derived(items.some((e) => e.type === 'shape' && e.shape === 'rect'))
  const hasLines = $derived(items.some((e) => e.type === 'line'))
  const hasText = $derived(items.some((e) => !(e.type === 'shape' && e.shape === 'image')))

  function set<K extends keyof Style>(key: K, value: Style[K]) {
    checkpoint()
    for (const e of items) e.style[key] = value
  }

  function direction(start: boolean, end: boolean) {
    checkpoint()
    for (const e of items) {
      if (e.type !== 'line') continue
      const head = e.style.end !== 'none' ? e.style.end : e.style.start !== 'none' ? e.style.start : 'stealth'
      e.style.start = start ? head : 'none'
      e.style.end = end ? head : 'none'
    }
  }

  function toggleCurved() {
    checkpoint()
    for (const e of items) if (e.type === 'line') e.curved = !e.curved
  }

  const num = (e: Event) => +(e.target as HTMLInputElement).value
  const round = (v: number) => Math.round(v * 10) / 10
</script>

{#if first && s && (hasStroke || hasText)}
  <div class="panel">
    {#if hasStroke}
      <section>
        <div class="row">
          <span class="label">Stroke</span>
          <ColorPicker value={s.stroke} allowNone onchange={(v) => set('stroke', v)} />
          <label class="num">
            <input type="number" min="0" step="0.25" value={s.width} onchange={(e) => set('width', Math.max(0, num(e)))} />
            <span>pt</span>
          </label>
        </div>
        <div class="seg">
          {#each ['solid', 'dashed', 'dotted'] as const as dash}
            <button class:on={s.dash === dash} onclick={() => set('dash', dash)}>{dash}</button>
          {/each}
        </div>
      </section>
    {/if}

    {#if hasShapes}
      <section>
        <div class="row">
          <span class="label">Fill</span>
          <ColorPicker value={s.fill} allowNone onchange={(v) => set('fill', v)} />
          {#if hasRects}
            <span class="label right">Radius</span>
            <label class="num">
              <input
                type="number"
                min="0"
                step="1"
                value={round(s.radius * PT_PER_CM)}
                onchange={(e) => set('radius', Math.max(0, num(e)) / PT_PER_CM)}
              />
              <span>pt</span>
            </label>
          {/if}
        </div>
      </section>
    {/if}

    {#if hasLines}
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
          <select value={s.start} onchange={(e) => set('start', (e.target as HTMLSelectElement).value as Arrow)} aria-label="Start head">
            {#each arrows as a}<option value={a}>{a === 'none' ? 'No start' : a}</option>{/each}
          </select>
          <select value={s.end} onchange={(e) => set('end', (e.target as HTMLSelectElement).value as Arrow)} aria-label="End head">
            {#each arrows as a}<option value={a}>{a === 'none' ? 'No end' : a}</option>{/each}
          </select>
        </div>
        <div class="seg">
          <button class:on={first.type === 'line' && !first.curved} onclick={() => first.type === 'line' && first.curved && toggleCurved()}>Straight</button>
          <button class:on={first.type === 'line' && first.curved} onclick={() => first.type === 'line' && !first.curved && toggleCurved()}>Curved</button>
        </div>
      </section>
    {/if}

    {#if hasText}
      <section>
        <div class="row">
          <span class="label">Text</span>
          <ColorPicker value={s.color} onchange={(v) => set('color', v ?? '#000000')} />
          <label class="num">
            <input type="number" min="1" step="1" value={s.size} onchange={(e) => set('size', Math.max(1, num(e)))} />
            <span>pt</span>
          </label>
        </div>
        <div class="row">
          <select class="grow" value={s.font ?? ''} onchange={(e) => set('font', (e.target as HTMLSelectElement).value || null)} aria-label="Font">
            <option value="">Default font</option>
            {#each fonts as f}<option value={f}>{f}</option>{/each}
          </select>
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
    top: 64px;
    right: 16px;
    width: 248px;
    padding: 4px 12px;
    background: var(--panel);
    border-radius: 14px;
    box-shadow: var(--shadow);
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
    width: 48px;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
  }
  .label.right {
    width: auto;
    margin-left: auto;
  }
  .grow {
    flex: 1;
  }
  .num {
    display: flex;
    align-items: center;
    gap: 2px;
    height: 28px;
    padding: 0 8px 0 2px;
    margin-left: auto;
    border-radius: 7px;
    background: var(--subtle);
    font-size: 11px;
    color: var(--muted);
  }
  .num input {
    width: 44px;
    border: none;
    background: transparent;
    font: 12px var(--font);
    color: var(--text);
    text-align: right;
    outline: none;
  }
  select {
    flex: 1;
    min-width: 0;
    height: 28px;
    padding: 0 6px;
    border: none;
    border-radius: 7px;
    background: var(--subtle);
    font: 12px var(--font);
    color: var(--text);
    text-transform: capitalize;
    outline: none;
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
  .b {
    font-weight: 700;
  }
  .i {
    font-style: italic;
    font-family: Georgia, serif;
  }
</style>
