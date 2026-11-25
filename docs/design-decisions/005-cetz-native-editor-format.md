# 005: CeTZ is the editor's native file format

## Decision
The editor saves `.typ` files containing a standalone CeTZ document (`#import "@preview/cetz:0.5.3"` plus `cetz.canvas`). Ids become `name:` arguments and labels become `content("id", [...])`, so the file can be edited by hand and reopened.

## Consequences
- A saved diagram compiles with `typst compile` as is, and can be pasted into any Typst document.
- dia → CeTZ → dia is lossless for everything the IR holds. This is tested.
- The code panel shows the live CeTZ output and is editable: changes are parsed back into the diagram.
