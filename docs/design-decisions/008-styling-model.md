# 008: Text style, arrowheads, transparency and images

## Decision
- **Text style** lives in `Style`: `font`, `size` (pt), `bold`, `italic`, `color`. How each format writes it:
  - CeTZ: `text(font:, size:, weight:, style:, fill:)[label]`
  - TikZ: `font=\bfseries\itshape\fontsize{..}{..}\selectfont`, `text=`. The font family is not exported, because LaTeX font names don't map to Typst's.
  - drawio: `fontSize`, `fontStyle` bitmask, `fontColor`, `fontFamily`.
- **Arrowheads** are one enum per end: `none | stealth | triangle | open | circle | diamond | bar`. Direction is just which ends have a head.
  - CeTZ marks: `stealth`, `>`, `straight`, `o`, `<>`, `|`.
  - TikZ arrows.meta: `Stealth`, `Triangle`, `Straight Barb`, `Circle`, `Diamond`, `Bar`.
  - drawio: `classic`, `block`, `open`, `oval`, `diamond`, `dash`.
- **Transparency** is stored in the colour itself, as `#rrggbbaa`. CeTZ uses it as is. TikZ and drawio split it into separate opacity keys.
- **Images** are `ShapeKind::Image` with a `src` that is a path or a data URL.
  - CeTZ and TikZ reference a file. Data URLs are exported as `<id>.<ext>`, and the editor downloads those files next to the diagram.
  - drawio embeds data URLs in the style string, written without `;base64` because `;` separates style entries.
  - When opening a `.typ`/`.tex` file, image files picked alongside it are matched by name.

## Consequences
- The editor's three fonts are the ones typst.ts ships (Libertinus Serif, New Computer Modern, DejaVu Sans Mono). Any other font name still round-trips.
