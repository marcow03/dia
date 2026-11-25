<script lang="ts">
  import { read, write } from './core'
  import CodeEditor from './CodeEditor.svelte'
  import Icon from './Icon.svelte'
  import { app, setDiagram } from './state.svelte'

  let code = $derived(write(app.diagram, 'cetz'))
  let error = $state('')
  let timer: ReturnType<typeof setTimeout>

  function input(v: string) {
    code = v
    clearTimeout(timer)
    timer = setTimeout(() => {
      try {
        const { diagram, warnings } = read(code, 'cetz')
        setDiagram(diagram)
        error = warnings.join('\n')
      } catch (e) {
        error = String(e)
      }
    }, 500)
  }

  const close = () => (app.showCode = false)

  async function copy() {
    await navigator.clipboard.writeText(code)
    close()
  }
</script>

<div class="backdrop" role="presentation" onpointerdown={close}>
  <div class="dialog" role="dialog" aria-label="CeTZ code" tabindex="-1" onpointerdown={(e) => e.stopPropagation()}>
    <header>
      <h2>CeTZ</h2>
      <div class="actions">
        <button class="copy" onclick={copy}>Copy</button>
        <button class="close" title="Close" onclick={close}><Icon name="close" /></button>
      </div>
    </header>
    <div class="body"><CodeEditor value={code} onchange={input} /></div>
    {#if error}<pre class="error">{error}</pre>{/if}
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
    display: flex;
    flex-direction: column;
    width: min(760px, calc(100vw - 32px));
    height: min(640px, calc(100vh - 64px));
    background: var(--panel);
    border-radius: 16px;
    box-shadow: 0 0 0 0.5px rgb(0 0 0 / 0.1), 0 20px 60px rgb(0 0 0 / 0.18);
    overflow: hidden;
    outline: none;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px 10px 20px;
    border-bottom: 1px solid var(--line);
  }
  h2 {
    margin: 0;
    font-size: 15px;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .copy {
    height: 30px;
    padding: 0 14px;
    font-size: 13px;
    font-weight: 500;
    color: white;
    background: var(--accent);
  }
  .copy:hover {
    background: #0071e3;
  }
  .close :global(svg) {
    width: 16px;
    height: 16px;
    color: var(--muted);
  }
  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .error {
    margin: 0;
    padding: 8px 20px;
    font: 11px/1.4 var(--mono);
    color: #ff3b30;
    border-top: 1px solid var(--line);
    white-space: pre-wrap;
  }
</style>
