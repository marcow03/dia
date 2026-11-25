<script lang="ts">
  import { download, exports, pickFile } from './files'
  import Icon from './Icon.svelte'
  import { app, deleteDocument, newDocument, openDocument } from './state.svelte'

  let { onclose }: { onclose: () => void } = $props()
  let showExport = $state(false)

  const run = (f: () => void) => () => {
    f()
    onclose()
  }

  function ago(t: number) {
    const s = (Date.now() - t) / 1000
    if (s < 60) return 'just now'
    if (s < 3600) return `${Math.floor(s / 60)} min ago`
    if (s < 86400) return `${Math.floor(s / 3600)} h ago`
    return new Date(t).toLocaleDateString()
  }
</script>

<div class="menu" role="menu" tabindex="-1" onpointerdown={(e) => e.stopPropagation()}>
  <button onclick={run(() => newDocument())}>New diagram<kbd>⌘N</kbd></button>
  <button onclick={run(pickFile)}>Open file…<kbd>⌘O</kbd></button>
  <button onclick={run(() => download('cetz', 'typ'))}>Save as Typst<kbd>⌘S</kbd></button>
  <button class="expand" class:open={showExport} onclick={() => (showExport = !showExport)}>
    Export<Icon name="chevron" />
  </button>
  {#if showExport}
    <div class="sub">
      {#each exports as x}
        <button onclick={run(() => download(x.format, x.ext))}>{x.label}<span>.{x.ext}</span></button>
      {/each}
    </div>
  {/if}
  {#if app.recents.length}
    <div class="heading">Recent</div>
    <div class="recents">
      {#each app.recents as doc (doc.id)}
        <div class="recent" class:current={doc.id === app.docId}>
          <button class="open" onclick={run(() => openDocument(doc.id))}>
            <span class="name">{doc.name || 'untitled'}</span>
            <span class="time">{ago(doc.updated)}</span>
          </button>
          <button class="del" title="Delete" onclick={() => deleteDocument(doc.id)}><Icon name="close" /></button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    width: 260px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border-radius: 12px;
    box-shadow: var(--shadow);
    z-index: 30;
  }
  button {
    justify-content: space-between;
    width: 100%;
    height: 32px;
    padding: 0 10px;
    font-size: 13px;
  }
  kbd,
  span {
    font: 11px var(--font);
    color: var(--muted);
  }
  .expand :global(svg) {
    width: 16px;
    color: var(--muted);
    transition: transform 0.15s;
  }
  .expand.open :global(svg) {
    transform: rotate(90deg);
  }
  .sub {
    padding-left: 10px;
  }
  .heading {
    margin: 8px 10px 4px;
    padding-top: 8px;
    border-top: 1px solid var(--line);
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
  }
  .recents {
    max-height: 280px;
    overflow-y: auto;
  }
  .recent {
    display: flex;
    align-items: center;
    border-radius: 9px;
  }
  .recent.current {
    background: var(--subtle);
  }
  .recent .open {
    flex: 1;
    min-width: 0;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    color: var(--text);
  }
  .del {
    width: 28px;
    min-width: 28px;
    padding: 0;
    color: var(--muted);
    opacity: 0;
  }
  .recent:hover .del {
    opacity: 1;
  }
  .del :global(svg) {
    width: 14px;
  }
</style>
