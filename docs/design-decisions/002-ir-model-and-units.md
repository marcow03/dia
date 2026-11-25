# 002: IR model and units

## Decision
The IR (`ir.rs`) is small and semantic. It describes shapes and connections, not drawing commands:

- `Shape { shape: rect | ellipse | diamond | text, x, y, w, h }`
- `Line { from, to, via, curved }`. An `End` is either a point or `{ node: id }`, a binding to a shape. With `curved` and 1–2 `via` points, the via points are bezier control points. Otherwise they are polyline waypoints.
- `rotate` on shapes: clockwise degrees around the centre. CeTZ: a `scope` with `rotate(-θ, origin: centre)` (anchor angles are then measured in the rotated frame), TikZ: `rotate=-θ`, drawio: `rotation=θ`.
- `group` on elements: a flat group id; members are kept adjacent in z-order. CeTZ: `scope({ // group g1 … })` (a CeTZ `group` would hide member names from arrows), TikZ: `\begin{scope}[local bounding box=g1]`, drawio: a `group` cell with members in relative coordinates.
- `Style { stroke, fill, width (pt), dash, radius (cm), start_arrow, end_arrow }`. Colours are hex strings, and `None` means no stroke or no fill.
- `label`: Typst markup (see 003). `label_at` is either an offset from the shape centre or a position `t` along a line. CeTZ writes it as `(rel:, to:)` or `(name:, anchor: t%)`, TikZ as `label={[shift=…]center:…}` or `node[pos=t]`, drawio as asymmetric spacing or the edge geometry's `x`.

Canonical units are **centimetres, y pointing down**, matching screen coordinates. Readers and writers convert at the boundary:
- CeTZ and TikZ: cm, y up, so y is negated.
- drawio: px, y down, at 40 px/cm, so drawio's default 120×60 shape becomes 3×1.5 cm.

The same structure is serialized with serde as the editor's JSON model (`editor/src/lib/types.ts`), so there is no second model to keep in sync.

## Consequences
- Line ends bound to shapes survive every format: CeTZ angle anchors `(name: "a", anchor: 30deg)`, TikZ `(a)`, drawio `source`/`target`.
- Diamonds are written as CeTZ `polygon(c, 4, radius: (rx, ry))`, because closed `line` paths have no border anchors.
- Things the IR lacks (arbitrary paths, groups, rotation) get dropped with a warning. Add them to the IR when they are needed.
