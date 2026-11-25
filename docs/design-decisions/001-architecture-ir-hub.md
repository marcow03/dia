# 001: Pandoc-style architecture with one IR

## Context
dia-core converts between drawio, CeTZ and TikZ, and more formats will follow. Writing a converter for every pair grows as N².

## Decision
Every format has one reader (text → IR) and one writer (IR → text). Conversion is always `read(from)` then `write(to)`, the way pandoc works.

- Formats are registered in `formats::FORMATS` as plain `Format { name, extensions, read, write }` structs with function pointers. There is no trait object and no plugin system.
- To add a format, write one module and add one line to the registry.
- `dia-cli` (`dia -f tikz -t cetz in.tex`) and `dia-wasm` are thin wrappers around `dia_core::{read, write, convert}`.

## Consequences
- N formats need 2N functions.
- A conversion can only keep what the IR can express (see 002), so the IR should grow on purpose, driven by what the editor needs.
