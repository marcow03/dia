<script lang="ts">
  import { icons } from './icons'

  let { name }: { name: string } = $props()
</script>

<svg viewBox="0 0 20 20" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
  {@html icons[name]}
</svg>
