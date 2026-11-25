# 004: Readers parse a defined subset and warn on the rest

## Context
CeTZ and TikZ are programming languages: they have loops, macros and computed coordinates. Evaluating them in full is out of scope.

## Decision
- Each reader supports what dia's writers emit, plus common hand-written constructs:
  - **CeTZ:** `rect`, `circle`, `polygon` (4 sides), `line`, `bezier`, `content`. Strokes and fills in all common forms, and named or angle anchors.
  - **TikZ:** `\node`, `\draw`, `\path`, `\fill`, `\coordinate`, `\definecolor`. Options, colour mixes (`red!20`), arrow specs, relative, polar and `-|` coordinates, and `right=of a` positioning.
  - **drawio:** plain and compressed (deflate + base64 + URL-encoding) pages, containers, `UserObject`s and edge-label cells.
- Anything else is skipped and reported in a `Warnings` list. It never fails silently and never panics. Parse errors are only fatal when the input is unreadable.
- CeTZ is parsed with the official `typst-syntax` parser. TikZ uses a small hand-written tokenizer, because there is no reusable TikZ parser in Rust.

## Consequences
- Imports are predictable, and users can see what was dropped.
- `cargo test` checks the round-trip and every cross-format chain on a shared sample diagram, and the writers' output compiles with `typst`, `pdflatex` and draw.io.
