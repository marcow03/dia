# dia

> [!WARNING]
> I wanted something like this for a long time, but couldn't find it. Then I had some tokens left and a couple of hours to spare, so I built it.
>
> IT WAS ENTIRELY WRITTEN BY OPUS 5.5. So use it at your own risk. I give no warranty or garantee that it will work for you.

A small, Typst-first and CetZ-native diagram editor for the browser.

**Editor**

- Draw rectangles, ellipses, diamonds, lines, arrows, text and images; group, rotate, align and layer them.
- Labels are Typst markup with math (`$x^2$`), rendered live with typst.ts, edited in a small code editor with highlighting and optional vim mode.
- Saves CeTZ (`.typ`) and imports/exports TikZ (`.tex`) and draw.io (`.drawio`); exports SVG, PNG and JPEG.

Try it here: [dia](https://marcow03.github.io/dia/)

## Formats

dia-core reads every format into one intermediate model and writes it back out, so every pair converts in both directions.

| | read | write |
| --- | :---: | :---: |
| CeTZ / Typst (`.typ`) | ✓ | ✓ |
| TikZ / LaTeX (`.tex`) | ✓ | ✓ |
| draw.io (`.drawio`, also compressed) | ✓ | ✓ |
| SVG / PNG / JPEG (editor only) | | ✓ |

## Shortcuts

`⌘` is `Ctrl` on Windows and Linux.

**Tools** (the letter is shown under each toolbar button)

| Key | Tool |
| --- | --- |
| `V` | Select |
| `H` | Hand (pan) |
| `R` / `O` / `D` | Rectangle / ellipse / diamond |
| `L` / `A` | Line / arrow |
| `T` | Text |
| `I` | Insert image |

**Editing**

| Key | Action |
| --- | --- |
| Double-click | Edit label (on empty canvas: new text) |
| `Enter` | Edit the selected element's label |
| `⌘Z` / `⇧⌘Z` (`⌘Y`) | Undo / redo |
| `⌘C` / `⌘X` / `⌘V` | Copy / cut / paste |
| `⌘D` | Duplicate |
| `⌘A` | Select all |
| `Delete` / `Backspace` | Delete selection |
| `⇧`-click | Add to / remove from selection |
| `Esc` | Deselect, back to select tool, close menus and dialogs |

**Arrange**

| Key | Action |
| --- | --- |
| Arrows or `h j k l` | Move selection by one grid step (0.25 cm) |
| `⇧` + arrows / `hjkl` | Move selection by one screen pixel |
| `⌃⇧` + arrows / `hjkl` | Select the adjacent object in that direction |
| `⌘G` / `⇧⌘G` | Group / ungroup |
| `⌘]` / `⌘[` | Bring forward / send backward |
| `⇧⌘]` / `⇧⌘[` | Bring to front / send to back |
| `⇧` while rotating | Snap to 15° |

**View**

| Key | Action |
| --- | --- |
| Scroll | Pan |
| Pinch or `⌘` + scroll | Zoom |
| `Space` + drag, middle-drag | Pan |
| `⌘+` / `⌘−` / `⌘0` | Zoom in / out / 100% |
| `⇧1` | Zoom to fit |

**File and panels**

| Key | Action |
| --- | --- |
| `⌘N` | New diagram |
| `⌘O` | Open file (also drag and drop) |
| `⌘S` | Export CeTZ (`.typ`) |
| `⇧C` | Show CeTZ code |
| `⌘,` | Settings |

**Label editor**

| Key | Action |
| --- | --- |
| `⌘Enter` | Done |
| `Esc` | Cancel |
| `Enter` | New line |
| `Tab` | Insert tab |
| `:w` / `:wq` / `:q` | Done / done / cancel (vim mode, enable in Settings) |

## Development

| Path | What |
| --- | --- |
| `crates/dia-core` | Model, readers and writers for CeTZ, TikZ, draw.io |
| `crates/dia-cli` | `dia [-f FROM] [-t TO] [-o OUT] INPUT` |
| `crates/dia-wasm` | WASM bindings used by the editor |
| `editor` | Svelte 5 + TypeScript editor |
| `docs/design-decisions` | Why things are the way they are |

```sh
cargo test
cargo run -p dia-cli -- tests/fixtures/sample.drawio -o out.typ

cd editor
npm install
npm run dev        # builds dia-wasm with wasm-pack, then starts Vite
```

Requires Rust with the `wasm32-unknown-unknown` target and Node.js. Typst fonts are loaded from jsDelivr (and cached by the service worker).

## License

[CC BY-NC 4.0](LICENSE): free to use, share and adapt for non-commercial purposes with attribution.
