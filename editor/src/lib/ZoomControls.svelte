<script lang="ts">
  import { app, zoomBy, zoomTo, zoomToFit } from './state.svelte'
</script>

<div class="zoom">
  <button title="Zoom out (⌘−)" onclick={() => zoomBy(1 / 1.25)}>−</button>
  <button class="pct" title="Reset zoom (⌘0)" onclick={() => zoomTo(1)}>{Math.round(app.view.zoom * 100)}%</button>
  <button title="Zoom in (⌘+)" onclick={() => zoomBy(1.25)}>+</button>
  <button class="fit" title="Zoom to fit (⇧1)" onclick={zoomToFit}>Fit</button>
</div>

<style>
  .zoom {
    position: absolute;
    left: 16px;
    bottom: 20px;
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px;
    background: var(--panel);
    border-radius: 12px;
    box-shadow: var(--shadow);
  }
  button {
    min-width: 30px;
    height: 30px;
    font-size: 15px;
    color: var(--muted);
  }
  .pct,
  .fit {
    padding: 0 8px;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }
  .pct {
    min-width: 52px;
  }
</style>
