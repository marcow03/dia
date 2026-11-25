<script lang="ts">
  import { read, write } from './core'
  import CodeEditor from './CodeEditor.svelte'
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
</script>

<aside class="code">
  <header>
    <span>CeTZ</span>
    <button onclick={() => navigator.clipboard.writeText(code)}>Copy</button>
  </header>
  <div class="body"><CodeEditor value={code} onchange={input} /></div>
  {#if error}<pre class="error">{error}</pre>{/if}
</aside>

<style>
  .code {
    position: absolute;
    top: 64px;
    left: 16px;
    bottom: 16px;
    width: min(440px, 40vw);
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border-radius: 14px;
    box-shadow: var(--shadow);
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 8px 8px 14px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    border-bottom: 1px solid var(--line);
  }
  header button {
    font-size: 12px;
    padding: 0 10px;
    height: 26px;
  }
  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .error {
    margin: 0;
    padding: 8px 14px;
    font: 11px/1.4 var(--mono);
    color: #ff3b30;
    border-top: 1px solid var(--line);
    white-space: pre-wrap;
  }
</style>
