<script lang="ts">
  import ColorPicker from './ColorPicker.svelte'
  import Icon from './Icon.svelte'
  import NumberInput from './NumberInput.svelte'
  import Select from './Select.svelte'
  import { app, factoryDefaults } from './state.svelte'
  import { fonts, type Style } from './types'

  const shown = $derived(app.defaults.rect)
  const fontOptions = [{ value: '', label: 'Typst default' }, ...fonts.map((f) => ({ value: f, label: f }))]

  /** Sets a default on every tool; stroke settings skip the text tool. */
  function setDefault<K extends keyof Style>(key: K, value: Style[K], strokeOnly = false) {
    for (const [tool, style] of Object.entries(app.defaults)) {
      if (!(strokeOnly && tool === 'text')) style[key] = value
    }
  }

  const close = () => (app.showSettings = false)
</script>

<div class="backdrop" role="presentation" onpointerdown={close}>
  <div class="dialog" role="dialog" aria-label="Settings" tabindex="-1" onpointerdown={(e) => e.stopPropagation()}>
    <header>
      <h2>Settings</h2>
      <button class="close" title="Close" onclick={close}><Icon name="close" /></button>
    </header>

    <section>
      <h3>Editor</h3>
      <label class="row">
        <span>
          Vim mode
          <small>Modal editing in the label editor; <code>:w</code> saves, <code>:q</code> cancels.</small>
        </span>
        <input type="checkbox" class="switch" bind:checked={app.settings.vim} />
      </label>
    </section>

    <section>
      <h3>Export</h3>
      <label class="row">
        <span>
          Embed images in CeTZ
          <small>Images are stored in the <code>.typ</code> file as base64 (via the <code>based</code> package) instead of separate files.</small>
        </span>
        <input type="checkbox" class="switch" bind:checked={app.settings.embedImages} />
      </label>
    </section>

    <section>
      <h3>New elements</h3>
      <div class="row">
        <span>Font</span>
        <div class="control">
          <Select label="Default font" value={shown.font ?? ''} options={fontOptions} onchange={(v) => setDefault('font', v || null)} />
        </div>
      </div>
      <div class="row">
        <span>Font size</span>
        <NumberInput label="Default font size" value={shown.size} min={1} onchange={(v) => setDefault('size', v)} />
      </div>
      <div class="row">
        <span>Text color</span>
        <ColorPicker value={shown.color} onchange={(v) => setDefault('color', v ?? '#000000')} />
      </div>
      <div class="row">
        <span>Stroke color</span>
        <ColorPicker value={shown.stroke} allowNone onchange={(v) => setDefault('stroke', v, true)} />
      </div>
      <div class="row">
        <span>Stroke width</span>
        <NumberInput label="Default stroke width" value={shown.width} step={0.25} onchange={(v) => setDefault('width', v, true)} />
      </div>
      <div class="row">
        <span>Fill color</span>
        <ColorPicker value={shown.fill} allowNone onchange={(v) => setDefault('fill', v, true)} />
      </div>
      <div class="row">
        <span></span>
        <button class="reset" onclick={() => (app.defaults = factoryDefaults())}>Reset to defaults</button>
      </div>
    </section>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.12);
  }
  .dialog {
    width: min(440px, calc(100vw - 32px));
    padding: 4px 20px 12px;
    background: var(--panel);
    border-radius: 16px;
    box-shadow: 0 0 0 0.5px rgb(0 0 0 / 0.1), 0 20px 60px rgb(0 0 0 / 0.18);
    outline: none;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 0 4px;
  }
  h2 {
    margin: 0;
    font-size: 15px;
  }
  .close :global(svg) {
    width: 16px;
    height: 16px;
    color: var(--muted);
  }
  section {
    padding: 10px 0;
  }
  section + section {
    border-top: 1px solid var(--line);
  }
  h3 {
    margin: 0 0 6px;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    min-height: 40px;
    font-size: 13px;
  }
  .row > span {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  small {
    font-size: 11px;
    color: var(--muted);
  }
  code {
    font: 11px var(--mono);
  }
  .control {
    display: flex;
    width: 200px;
  }
  .reset {
    padding: 0 12px;
    height: 30px;
    font-size: 12px;
    color: var(--accent);
  }
  .switch {
    appearance: none;
    position: relative;
    flex: none;
    width: 38px;
    height: 22px;
    margin: 0;
    border-radius: 11px;
    background: var(--line);
    cursor: pointer;
    transition: background 0.15s;
  }
  .switch::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 1px 3px rgb(0 0 0 / 0.25);
    transition: transform 0.15s;
  }
  .switch:checked {
    background: #34c759;
  }
  .switch:checked::after {
    transform: translateX(16px);
  }
</style>
