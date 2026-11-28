# 008: Text style, arrowheads, transparency and images

## Decision
- **Text style** lives in `Style`: `font`, `size` (pt, default 14), `bold`, `italic`, `color`. The 14pt default is stated once per document (CeTZ `#set text(size: 14pt)`, a TikZ picture-wide `font=\fontsize{14}{17}`, drawio's explicit `fontSize`), so writers only emit sizes that differ from it. How each format writes it:
  - CeTZ: `text(font:, size:, weight:, style:, fill:)[label]`
  - TikZ: `font=\bfseries\itshape\fontsize{..}{..}\selectfont`, `text=`. The font family is not exported, because LaTeX font names don't map to Typst's.
  - drawio: `fontSize`, `fontStyle` bitmask, `fontColor`, `fontFamily`.
- **Text alignment** (`align`: left, centre, right; default left) aligns the lines of a label. CeTZ wraps the body in `align(…)`, TikZ uses the node's `align=`, drawio uses `align=` for text elements only (on shapes it would also move the text inside the shape).
- **Arrowheads** are one enum per end: `none | stealth | triangle | open | circle | diamond | bar`. Direction is just which ends have a head.
  - CeTZ marks: `stealth`, `>`, `straight`, `o`, `<>`, `|`.
  - TikZ arrows.meta: `Stealth`, `Triangle`, `Straight Barb`, `Circle`, `Diamond`, `Bar`.
  - drawio: `classic`, `block`, `open`, `oval`, `diamond`, `dash`.
- **Transparency** is stored in the colour itself, as `#rrggbbaa`. CeTZ uses it as is. TikZ and drawio split it into separate opacity keys.
- **Images** are `ShapeKind::Image` with a `src` that is a path or a data URL.
  - CeTZ embeds data URL images as base64: `#let image-i1 = base64.decode("…")` with `@preview/based`, so the `.typ` file is self-contained. The reader turns such bindings back into data URLs (type from the leading bytes). The editor setting *Embed images in CeTZ* (on by default) can switch this off; images are then referenced by path like in TikZ.
  - TikZ references a file. Data URLs are exported as `<id>.<ext>`, and the editor downloads those files next to the diagram.
  - drawio embeds data URLs in the style string, written without `;base64` because `;` separates style entries.
  - When opening a `.typ`/`.tex` file, image files picked alongside it are matched by name.

## Consequences
- The editor's three fonts are the ones typst.ts ships (Libertinus Serif, New Computer Modern, DejaVu Sans Mono). Any other font name still round-trips.
