# 006: The editor draws shapes itself and uses typst.ts for labels

## Context
`typst.ts` 0.7 (Typst in WASM) bundles Typst 0.14, but CeTZ 0.5 needs Typst ≥ 0.15. Recompiling the whole canvas on every drag would also be too slow.

## Decision
- Shapes, lines and arrowheads are drawn natively as SVG, in cm world units under one pan/zoom transform.
- Only labels go through typst.ts: each label compiles to SVG once, is cached by its source text, and is inlined as a nested `<svg>`. Renders are queued, because the compiler holds a single main file.
- typst.ts's interactive extras (`<script>`, `<style>`, text-selection `foreignObject`) are stripped from the output.

## Consequences
- Editing stays instant, and math looks the same as in the exported document.
- Fonts load from jsDelivr on first use, so the editor needs a network connection. Self-host the fonts if offline use matters.
- dia-core is ~3 MB of WASM (most of it tylax), and typst.ts adds its compiler and renderer modules.
