use super::Format;
use crate::ir::*;
use crate::{Error, Result, Warnings};
use std::fmt::Write;
use typst_syntax::ast::{self, AstNode, Expr};

pub const FORMAT: Format = Format {
    name: "cetz",
    extensions: &["typ"],
    read,
    write,
};

pub const CETZ_VERSION: &str = "0.5.3";

pub fn write(d: &Diagram) -> String {
    let mut out = format!(
        "#import \"@preview/cetz:{CETZ_VERSION}\"\n#set page(width: auto, height: auto, margin: 0.5cm)\n\n#cetz.canvas({{\n  import cetz.draw: *\n"
    );
    for e in &d.elements {
        let mut args = Vec::new();
        let call = match &e.kind {
            Kind::Shape { shape, x, y, w, h } => {
                let (x2, y2) = (x + w, y + h);
                match shape {
                    ShapeKind::Rect => {
                        args.push(pt(*x, *y));
                        args.push(pt(x2, y2));
                        "rect"
                    }
                    ShapeKind::Ellipse => {
                        args.push(pt(x + w / 2.0, y + h / 2.0));
                        args.push(if w == h {
                            format!("radius: {}", num(w / 2.0))
                        } else {
                            format!("radius: ({}, {})", num(w / 2.0), num(h / 2.0))
                        });
                        "circle"
                    }
                    ShapeKind::Diamond => {
                        args.push(pt(x + w / 2.0, y + h / 2.0));
                        args.push("4".into());
                        args.push(format!("radius: ({}, {})", num(w / 2.0), num(h / 2.0)));
                        "polygon"
                    }
                    ShapeKind::Text => {
                        args.push(pt(*x, *y));
                        args.push(pt(x2, y2));
                        args.push(format!("[{}]", e.label));
                        "content"
                    }
                }
            }
            Kind::Line { from, to, via, curved } => {
                let first = via.first().copied().unwrap_or_else(|| d.center(to));
                let last = via.last().copied().unwrap_or_else(|| d.center(from));
                let bezier = *curved && (1..=2).contains(&via.len());
                args.push(end(d, from, first));
                if !bezier {
                    args.extend(via.iter().map(|p| pt(p.x, p.y)));
                }
                args.push(end(d, to, last));
                if bezier {
                    args.extend(via.iter().map(|p| pt(p.x, p.y)));
                    "bezier"
                } else {
                    "line"
                }
            }
        };
        let is_text = matches!(
            e.kind,
            Kind::Shape {
                shape: ShapeKind::Text,
                ..
            }
        );
        let is_line = matches!(e.kind, Kind::Line { .. });
        if !is_line || !e.label.is_empty() || is_text {
            args.push(format!("name: \"{}\"", e.id));
        }
        if !is_text {
            style_args(&e.style, !is_line, &mut args);
        }
        writeln!(out, "  {call}({})", args.join(", ")).unwrap();
        if !e.label.is_empty() && !is_text {
            if is_line {
                writeln!(
                    out,
                    "  content(\"{}.mid\", [{}], frame: \"rect\", fill: white, stroke: none, padding: 0.1)",
                    e.id, e.label
                )
            } else {
                writeln!(out, "  content(\"{}\", [{}])", e.id, e.label)
            }
            .unwrap();
        }
    }
    out.push_str("})\n");
    out
}

fn pt(x: f64, y: f64) -> String {
    format!("({}, {})", num(x), num(-y))
}

fn end(d: &Diagram, e: &End, toward: Point) -> String {
    match e {
        End::Point(p) => pt(p.x, p.y),
        End::Node { node } => {
            let c = d.center(e);
            let deg = (-(toward.y - c.y)).atan2(toward.x - c.x).to_degrees();
            format!("(name: \"{node}\", anchor: {}deg)", num(deg))
        }
    }
}

fn style_args(s: &Style, closed: bool, args: &mut Vec<String>) {
    let d = Style::default();
    if s.stroke.is_none() {
        args.push("stroke: none".into());
    } else if s.stroke != d.stroke || s.width != d.width || s.dash != d.dash {
        let mut parts = vec![
            format!("paint: {}", color(s.stroke.as_deref().unwrap())),
            format!("thickness: {}pt", num(s.width)),
        ];
        if s.dash != Dash::Solid {
            parts.push(format!(
                "dash: \"{}\"",
                if s.dash == Dash::Dashed { "dashed" } else { "dotted" }
            ));
        }
        args.push(format!("stroke: ({})", parts.join(", ")));
    }
    if let (Some(f), true) = (&s.fill, closed) {
        args.push(format!("fill: {}", color(f)));
    }
    if s.radius > 0.0 {
        args.push(format!("radius: {}", num(s.radius)));
    }
    let mut marks = Vec::new();
    if s.start_arrow {
        marks.push("start: \"stealth\"");
    }
    if s.end_arrow {
        marks.push("end: \"stealth\"");
    }
    if !marks.is_empty() {
        args.push(format!(
            "mark: ({}, fill: {})",
            marks.join(", "),
            color(s.stroke.as_deref().unwrap_or("#000000"))
        ));
    }
}

fn color(hex: &str) -> String {
    format!("rgb(\"{hex}\")")
}

// ---------- reader ----------

#[derive(Debug, Clone)]
enum Value {
    Num(f64),
    Angle(f64),
    Str(String),
    Bool(bool),
    None,
    Color(String),
    Array(Vec<Value>),
    Dict(Vec<(String, Value)>),
    Content(String),
    Other,
}

impl Value {
    fn num(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            _ => None,
        }
    }

    fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Dict(items) => items.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

struct Call {
    pos: Vec<Value>,
    named: Vec<(String, Value)>,
}

impl Call {
    fn get(&self, key: &str) -> Option<&Value> {
        self.named.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn name(&self) -> Option<String> {
        match self.get("name") {
            Some(Value::Str(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

pub fn read(src: &str, warnings: &mut Warnings) -> Result<Diagram> {
    let root = typst_syntax::parse(src);
    if let Some(err) = root.errors().first() {
        return Err(Error::Parse(err.message.to_string()));
    }
    let mut d = Diagram::default();
    let mut labels = Vec::new();
    walk(&root, &mut d, &mut labels, warnings);
    for (target, text) in labels {
        let id = target.split('.').next().unwrap_or_default();
        match d.elements.iter_mut().find(|e| e.id == id) {
            Some(e) => e.label = text,
            None => warnings.push(format!("content anchored to unknown element \"{target}\"")),
        }
    }
    Ok(d)
}

const DRAW_FNS: &[&str] = &["rect", "circle", "polygon", "line", "bezier", "content"];
const UNSUPPORTED_FNS: &[&str] = &[
    "arc",
    "grid",
    "hobby",
    "catmull",
    "merge-path",
    "mark",
    "set-style",
    "arc-through",
    "n-star",
];

fn walk(node: &typst_syntax::SyntaxNode, d: &mut Diagram, labels: &mut Vec<(String, String)>, w: &mut Warnings) {
    if let Some(call) = node.cast::<ast::FuncCall>()
        && let Expr::Ident(id) = call.callee()
    {
        let name = id.get().as_str();
        if DRAW_FNS.contains(&name) {
            element(name, call_args(call.args()), d, labels, w);
            return;
        }
        if UNSUPPORTED_FNS.contains(&name) {
            w.push(format!("unsupported cetz function `{name}` skipped"));
            return;
        }
    }
    for child in node.children() {
        walk(child, d, labels, w);
    }
}

fn call_args(args: ast::Args) -> Call {
    let mut call = Call {
        pos: vec![],
        named: vec![],
    };
    for arg in args.items() {
        match arg {
            ast::Arg::Pos(e) => call.pos.push(eval(e)),
            ast::Arg::Named(n) => call.named.push((n.name().get().to_string(), eval(n.expr()))),
            ast::Arg::Spread(_) => {}
        }
    }
    call
}

fn eval(e: Expr) -> Value {
    match e {
        Expr::Int(i) => Value::Num(i.get() as f64),
        Expr::Float(f) => Value::Num(f.get()),
        Expr::Numeric(n) => {
            use typst_syntax::ast::Unit;
            let (v, unit) = n.get();
            match unit {
                Unit::Cm => Value::Num(v),
                Unit::Mm => Value::Num(v / 10.0),
                Unit::Pt => Value::Num(v / 28.3465),
                Unit::In => Value::Num(v * 2.54),
                Unit::Deg => Value::Angle(v),
                Unit::Rad => Value::Angle(v.to_degrees()),
                _ => Value::Other,
            }
        }
        Expr::Str(s) => Value::Str(s.get().to_string()),
        Expr::Bool(b) => Value::Bool(b.get()),
        Expr::None(_) => Value::None,
        Expr::Parenthesized(p) => eval(p.expr()),
        Expr::Array(a) => Value::Array(
            a.items()
                .filter_map(|i| match i {
                    ast::ArrayItem::Pos(e) => Some(eval(e)),
                    _ => None,
                })
                .collect(),
        ),
        Expr::Dict(dict) => Value::Dict(
            dict.items()
                .filter_map(|i| match i {
                    ast::DictItem::Named(n) => Some((n.name().get().to_string(), eval(n.expr()))),
                    _ => None,
                })
                .collect(),
        ),
        Expr::ContentBlock(c) => Value::Content(c.body().to_untyped().clone().into_text().trim().to_string()),
        Expr::Ident(i) => named_color(i.get())
            .map(|c| Value::Color(c.into()))
            .unwrap_or(Value::Other),
        Expr::Unary(u) if u.op() == ast::UnOp::Neg => match eval(u.expr()) {
            Value::Num(n) => Value::Num(-n),
            Value::Angle(n) => Value::Angle(-n),
            _ => Value::Other,
        },
        Expr::Binary(b) if b.op() == ast::BinOp::Add => {
            // `1pt + red` stroke shorthand
            let mut items = Vec::new();
            for v in [eval(b.lhs()), eval(b.rhs())] {
                match v {
                    Value::Num(n) => items.push(("thickness".into(), Value::Num(n))),
                    Value::Color(c) => items.push(("paint".into(), Value::Color(c))),
                    _ => {}
                }
            }
            Value::Dict(items)
        }
        Expr::FuncCall(call) => match call.callee() {
            Expr::Ident(id) if id.get() == "rgb" => {
                let args = call_args(call.args());
                match args.pos.as_slice() {
                    [Value::Str(s)] => Value::Color(normalize_hex(s)),
                    [r, g, b, ..] => {
                        let c = |v: &Value| v.num().unwrap_or(0.0).clamp(0.0, 255.0) as u8;
                        Value::Color(format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b)))
                    }
                    _ => Value::Other,
                }
            }
            Expr::FieldAccess(f) => eval(f.target()),
            _ => Value::Other,
        },
        Expr::FieldAccess(f) => eval(f.target()),
        _ => Value::Other,
    }
}

fn normalize_hex(s: &str) -> String {
    let h = s.trim_start_matches('#');
    if h.len() == 3 {
        let mut out = String::from("#");
        for c in h.chars() {
            out.push(c);
            out.push(c);
        }
        out.to_lowercase()
    } else {
        format!("#{}", h.to_lowercase())
    }
}

pub fn named_color(name: &str) -> Option<&'static str> {
    Some(match name {
        "black" => "#000000",
        "white" => "#ffffff",
        "gray" => "#aaaaaa",
        "silver" => "#dddddd",
        "red" => "#ff4136",
        "blue" => "#0074d9",
        "green" => "#2ecc40",
        "yellow" => "#ffdc00",
        "orange" => "#ff851b",
        "purple" => "#b10dc9",
        "navy" => "#001f3f",
        "aqua" => "#7fdbff",
        "teal" => "#39cccc",
        "maroon" => "#85144b",
        "olive" => "#3d9970",
        "lime" => "#01ff70",
        "fuchsia" => "#f012be",
        "eastern" => "#239dad",
        _ => return None,
    })
}

enum Coord {
    P(Point),
    Node(String),
}

fn coord(v: &Value) -> Option<Coord> {
    match v {
        Value::Array(a) if a.len() >= 2 => Some(Coord::P(Point::new(a[0].num()?, -a[1].num()?))),
        Value::Str(s) => Some(Coord::Node(s.split('.').next()?.to_string())),
        Value::Dict(_) => match v.get("name") {
            Some(Value::Str(s)) => Some(Coord::Node(s.clone())),
            _ => None,
        },
        _ => None,
    }
}

fn to_end(c: Coord) -> End {
    match c {
        Coord::P(p) => End::Point(p),
        Coord::Node(node) => End::Node { node },
    }
}

fn element(f: &str, call: Call, d: &mut Diagram, labels: &mut Vec<(String, String)>, w: &mut Warnings) {
    let coords: Vec<Coord> = call.pos.iter().filter_map(coord).collect();
    let mut style = read_style(&call);
    let id = call.name();
    let kind = match f {
        "rect" => match coords.as_slice() {
            [Coord::P(a), Coord::P(b)] => shape(ShapeKind::Rect, *a, *b),
            _ => return w.push("rect with unsupported coordinates skipped".into()),
        },
        "circle" | "polygon" => {
            let Some(Coord::P(c)) = coords.first() else {
                return w.push(format!("{f} with unsupported coordinates skipped"));
            };
            if f == "polygon" && (call.pos.get(1).and_then(Value::num) != Some(4.0) || call.get("angle").is_some()) {
                return w.push("only 4-sided, unrotated polygons (diamonds) are supported".into());
            }
            let (rx, ry) = match call.get("radius") {
                Some(Value::Num(r)) => (*r, *r),
                Some(Value::Array(a)) if a.len() == 2 => (a[0].num().unwrap_or(1.0), a[1].num().unwrap_or(1.0)),
                _ => (1.0, 1.0),
            };
            let kind = if f == "polygon" {
                ShapeKind::Diamond
            } else {
                ShapeKind::Ellipse
            };
            shape(kind, Point::new(c.x - rx, c.y - ry), Point::new(c.x + rx, c.y + ry))
        }
        "content" => {
            let text = call.pos.iter().find_map(|v| match v {
                Value::Content(s) => Some(s.clone()),
                Value::Str(s) if call.pos.len() == 1 => Some(s.clone()),
                _ => None,
            });
            let text = text.unwrap_or_default();
            match coords.as_slice() {
                [Coord::Node(target)] => {
                    let target = match &call.pos[0] {
                        Value::Str(s) => s.clone(),
                        _ => target.clone(),
                    };
                    return labels.push((target, text));
                }
                [Coord::P(a), Coord::P(b)] => {
                    let id = id.unwrap_or_else(|| d.fresh_id("t"));
                    return d.elements.push(Element {
                        id,
                        kind: shape(ShapeKind::Text, *a, *b),
                        style: Style {
                            stroke: None,
                            ..Style::default()
                        },
                        label: text,
                    });
                }
                [Coord::P(p)] => {
                    let (w2, h2) = ((text.chars().count() as f64 * 0.12).max(0.5), 0.35);
                    let id = id.unwrap_or_else(|| d.fresh_id("t"));
                    return d.elements.push(Element {
                        id,
                        kind: shape(
                            ShapeKind::Text,
                            Point::new(p.x - w2, p.y - h2),
                            Point::new(p.x + w2, p.y + h2),
                        ),
                        style: Style {
                            stroke: None,
                            ..Style::default()
                        },
                        label: text,
                    });
                }
                _ => return w.push("content with unsupported position skipped".into()),
            }
        }
        "line" | "bezier" => {
            if matches!(call.get("close"), Some(Value::Bool(true))) {
                return w.push("closed paths are not supported, skipped".into());
            }
            {
                if coords.len() < 2 {
                    return w.push(format!("{f} with too few coordinates skipped"));
                }
                let mut coords = coords;
                let bezier = f == "bezier";
                let (from, to, rest) = if bezier {
                    let rest = coords.split_off(2);
                    let to = coords.pop().unwrap();
                    (coords.pop().unwrap(), to, rest)
                } else {
                    let to = coords.pop().unwrap();
                    let rest = coords.split_off(1);
                    (coords.pop().unwrap(), to, rest)
                };
                let via = rest
                    .into_iter()
                    .filter_map(|c| match c {
                        Coord::P(p) => Some(p),
                        Coord::Node(_) => None,
                    })
                    .collect();
                style.fill = None;
                Kind::Line {
                    from: to_end(from),
                    to: to_end(to),
                    via,
                    curved: bezier,
                }
            }
        }
        _ => unreachable!(),
    };
    let prefix = if matches!(kind, Kind::Line { .. }) { "e" } else { "n" };
    let id = id.unwrap_or_else(|| d.fresh_id(prefix));
    d.elements.push(Element {
        id,
        kind,
        style,
        label: String::new(),
    });
}

fn shape(shape: ShapeKind, a: Point, b: Point) -> Kind {
    Kind::Shape {
        shape,
        x: a.x.min(b.x),
        y: a.y.min(b.y),
        w: (a.x - b.x).abs(),
        h: (a.y - b.y).abs(),
    }
}

fn read_style(call: &Call) -> Style {
    let mut s = Style::default();
    match call.get("stroke") {
        Some(Value::None) => s.stroke = None,
        Some(Value::Color(c)) => s.stroke = Some(c.clone()),
        Some(Value::Num(n)) => s.width = n * 28.3465,
        Some(v @ Value::Dict(_)) => {
            if let Some(Value::Color(c)) = v.get("paint") {
                s.stroke = Some(c.clone());
            }
            if let Some(Value::Num(n)) = v.get("thickness") {
                s.width = (n * 28.3465 * 1000.0).round() / 1000.0;
            }
            if let Some(Value::Str(dash)) = v.get("dash") {
                s.dash = if dash.contains("dot") {
                    Dash::Dotted
                } else {
                    Dash::Dashed
                };
            }
        }
        _ => {}
    }
    if let Some(Value::Color(c)) = call.get("fill") {
        s.fill = Some(c.clone());
    }
    if let Some(Value::Num(r)) = call.get("radius")
        && call.pos.len() == 2
        && call.pos.iter().all(|p| matches!(p, Value::Array(_)))
    {
        s.radius = *r;
    }
    if let Some(m) = call.get("mark") {
        s.start_arrow = m.get("start").is_some_and(|v| !matches!(v, Value::None));
        s.end_arrow = m.get("end").is_some_and(|v| !matches!(v, Value::None));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::tests::sample;

    #[test]
    fn roundtrip() {
        let d = sample();
        let src = write(&d);
        let mut w = vec![];
        let back = read(&src, &mut w).unwrap();
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(back, d, "\n{src}");
    }

    #[test]
    fn hand_written() {
        let src = r#"
#import "@preview/cetz:0.5.3"
#cetz.canvas({
  import cetz.draw: *
  circle((0, 0), name: "a", fill: blue)
  rect((2, -1), (4, 1), name: "b", stroke: 2pt + red)
  line("a", "b.west", mark: (end: ">"))
  content("b", [$x^2$])
  arc((0,0), start: 0deg, stop: 90deg)
})"#;
        let mut w = vec![];
        let d = read(src, &mut w).unwrap();
        assert_eq!(d.elements.len(), 3);
        assert_eq!(w.len(), 1);
        assert_eq!(d.get("b").unwrap().label, "$x^2$");
        assert_eq!(d.get("b").unwrap().style.stroke.as_deref(), Some("#ff4136"));
        assert!(d.elements[2].style.end_arrow);
    }
}
