<script lang="ts">
  import { read, write } from './core'
  import { app, setDiagram } from './state.svelte'

  let code = $state('')
  let error = $state('')
  let focused = $state(false)
  let timer: ReturnType<typeof setTimeout>

  $effect(() => {
    const out = write(app.diagram, 'cetz')
    if (!focused) code = out
  })

  function input() {
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
  <textarea
    bind:value={code}
    oninput={input}
    onfocus={() => (focused = true)}
    onblur={() => (focused = false)}
    onkeydown={(e) => e.key !== 'Escape' && e.stopPropagation()}
    spellcheck="false"
  ></textarea>
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
  textarea {
    flex: 1;
    padding: 12px 14px;
    border: none;
    resize: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font: 12px/1.6 var(--mono);
    white-space: pre;
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
