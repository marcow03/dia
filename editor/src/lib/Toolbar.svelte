<script lang="ts">
  import { pickImage } from './files'
  import Icon from './Icon.svelte'
  import { app, redo, undo, type Tool } from './state.svelte'

  const tools: [Tool, string, string][] = [
    ['select', 'Select', 'V'],
    ['hand', 'Hand', 'H'],
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
    <div class="tool">
      <button class:active={app.tool === tool} {title} onclick={() => (app.tool = tool)}><Icon name={tool} /></button>
      <span>{key}</span>
    </div>
  {/each}
  <div class="tool">
    <button title="Image" onclick={pickImage}><Icon name="image" /></button>
    <span>I</span>
  </div>
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
    align-items: flex-start;
    gap: 2px;
    padding: 6px 6px 9px;
    background: var(--panel);
    border-radius: 14px;
    box-shadow: var(--shadow);
  }
  .sep {
    width: 1px;
    height: 20px;
    margin: 7px 4px 0;
    background: var(--line);
  }
  .tool {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
  }
  .tool span {
    font-size: 9px;
    font-weight: 600;
    line-height: 1;
    color: #a1a1a6;
  }
  button.active {
    background: var(--accent);
    color: white;
  }
</style>
