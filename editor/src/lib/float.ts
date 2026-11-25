/**
 * Svelte action that pins a popover below its anchor (or above, when there is no room) with
 * fixed positioning, so scrolling or clipping containers cannot cut it off.
 */
export function float(node: HTMLElement, { anchor, matchWidth = false }: { anchor: HTMLElement; matchWidth?: boolean }) {
  const place = () => {
    const a = anchor.getBoundingClientRect()
    if (matchWidth) node.style.width = `${a.width}px`
    const r = node.getBoundingClientRect()
    const left = Math.min(Math.max(8, a.left), innerWidth - r.width - 8)
    const below = a.bottom + 6
    const top = below + r.height > innerHeight - 8 ? Math.max(8, a.top - r.height - 6) : below
    Object.assign(node.style, { position: 'fixed', left: `${left}px`, top: `${top}px` })
  }
  place()
  addEventListener('resize', place)
  addEventListener('scroll', place, true)
  return {
    destroy() {
      removeEventListener('resize', place)
      removeEventListener('scroll', place, true)
    },
  }
}
