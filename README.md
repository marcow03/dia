# dia

A small diagram editor built around Typst, and **dia-core**, a pandoc-like converter between diagram formats.

| crate / dir | what |
| --- | --- |
| `crates/dia-core` | IR + readers/writers for CeTZ (`.typ`), TikZ (`.tex`), drawio (`.drawio`) |
| `crates/dia-cli` | `dia [-f FROM] [-t TO] [-o OUT] INPUT` |
| `crates/dia-wasm` | WASM bindings used by the editor |
| `editor` | Svelte 5 + TypeScript browser editor |
| `docs/design-decisions` | why things are the way they are |

## Use

```sh
cargo test
cargo run -p dia-cli -- tests/fixtures/sample.drawio -o out.tex

cd editor
npm install
npm run dev        # builds dia-wasm via wasm-pack, then starts Vite
```

Requires Rust with the `wasm32-unknown-unknown` target and Node.js. The editor loads Typst fonts from jsDelivr.
