<script lang="ts">
  import { pickImage } from './files'
  import Icon from './Icon.svelte'
  import { app, redo, undo, type Tool } from './state.svelte'

  const tools: [Tool, string, string][] = [
    ['select', 'Select', 'V'],
    ['rect', 'Rectangle', 'R'],
    ['ellipse', 'Ellipse', 'O'],
    ['diamond', 'Diamond', 'D'],
    ['line', 'Line', 'L'],
    ['arrow', 'Arrow', 'A'],
    ['text', 'Text', 'T'],
  ]
</script>

<div class="bar">
  {#each tools as [tool, title, key]}
    <button class="tool" class:active={app.tool === tool} title={title} onclick={() => (app.tool = tool)}>
      <Icon name={tool} />
      <span>{key}</span>
    </button>
  {/each}
  <button class="tool" title="Image" onclick={pickImage}><Icon name="image" /><span>I</span></button>
  <span class="sep"></span>
  <button title="Undo (⌘Z)" onclick={undo}><Icon name="undo" /></button>
  <button title="Redo (⇧⌘Z)" onclick={redo}><Icon name="redo" /></button>
</div>

<style>
  .bar {
    position: absolute;
    left: 50%;
    bottom: 20px;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 5px;
    background: var(--panel);
    border-radius: 14px;
    box-shadow: var(--shadow);
  }
  .sep {
    width: 1px;
    height: 20px;
    margin: 0 4px;
    background: var(--line);
  }
  .tool {
    flex-direction: column;
    gap: 1px;
    width: 40px;
    height: 44px;
  }
  .tool span {
    font-size: 9px;
    font-weight: 600;
    line-height: 1;
    color: var(--muted);
  }
  button.active {
    background: var(--accent);
    color: white;
  }
  button.active span {
    color: rgb(255 255 255 / 0.8);
  }
</style>
