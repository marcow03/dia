# 003: Typst is the canonical label markup

## Decision
Labels are stored as Typst markup, with `$…$` for math. Other markup is converted at the format boundary (`text.rs`):

- TikZ: LaTeX ⇄ Typst through the [`tylax`](https://crates.io/crates/tylax) crate. A small post-pass turns tylax's `x^(2)` back into `x^2`.
- drawio: HTML with MathJax delimiters (`\(…\)`, `$$…$$`) is stripped to text, its math goes through the LaTeX path, and `<br>` becomes Typst's `\`. Exported files set `math="1"`.

## Consequences
- The editor and CeTZ output never lose markup.
- Round-trips through LaTeX are as good as tylax. Exotic macros may not survive.
