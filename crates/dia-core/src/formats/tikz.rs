use super::Format;
use crate::ir::*;
use crate::text::{latex_to_typst, typst_to_latex};
use crate::{Result, Warnings};
use std::collections::HashMap;
use std::fmt::Write;

pub const FORMAT: Format = Format {
    name: "tikz",
    extensions: &["tex", "tikz"],
    read,
    write,
};

pub fn write(d: &Diagram) -> String {
    let mut out = String::from(
        "\\documentclass[tikz]{standalone}\n\\usetikzlibrary{shapes.geometric, arrows.meta}\n\\begin{document}\n\\begin{tikzpicture}[line width=1pt, font=\\fontsize{14}{17}\\selectfont]\n",
    );
    let mut group: Option<&str> = None;
    for e in &d.elements {
        if e.group.as_deref() != group {
            if group.is_some() {
                out.push_str("  \\end{scope}\n");
            }
            if let Some(g) = &e.group {
                writeln!(out, "  \\begin{{scope}}[local bounding box={g}]").unwrap();
            }
            group = e.group.as_deref();
        }
        let start = out.len();
        let label = typst_to_latex(&e.label);
        match &e.kind {
            Kind::Shape {
                shape,
                x,
                y,
                w,
                h,
                src,
                rotate,
            } => {
                let mut opts = Vec::new();
                let mut label = label;
                match shape {
                    ShapeKind::Rect => opts.push("rectangle".to_string()),
                    ShapeKind::Ellipse => opts.push("ellipse".into()),
                    ShapeKind::Diamond => opts.push(format!("diamond, aspect={}", num(w / h))),
                    ShapeKind::Text => {}
                    ShapeKind::Image => {
                        label = format!(
                            "\\includegraphics[width={}cm, height={}cm]{{{}}}",
                            num(*w),
                            num(*h),
                            image_path(&e.id, src)
                        )
                    }
                }
                if !matches!(shape, ShapeKind::Text | ShapeKind::Image) {
                    style_opts(&e.style, true, &mut opts);
                }
                if *rotate != 0.0 {
                    opts.push(format!("rotate={}", num(-rotate)));
                }
                if let (Some(LabelAt::Offset { dx, dy }), false) = (e.label_at, label.is_empty()) {
                    let mut label_opts = vec![format!("shift={{({}, {})}}", num(dx), num(-dy))];
                    text_opts(&e.style, &mut label_opts);
                    opts.push(format!("label={{[{}]center:{{{label}}}}}", label_opts.join(", ")));
                    label.clear();
                } else {
                    text_opts(&e.style, &mut opts);
                }
                opts.push(format!(
                    "minimum width={}cm, minimum height={}cm, inner sep=0pt, align={}",
                    num(*w),
                    num(*h),
                    if *shape == ShapeKind::Text { "left" } else { "center" }
                ));
                writeln!(
                    out,
                    "  \\node[{}] ({}) at ({}) {{{label}}};",
                    opts.join(", "),
                    e.id,
                    xy(x + w / 2.0, y + h / 2.0)
                )
                .unwrap();
            }
            Kind::Line { from, to, via, curved } => {
                let mut opts = Vec::new();
                style_opts(&e.style, false, &mut opts);
                let mut path = end(from);
                if *curved && (1..=2).contains(&via.len()) {
                    let ctrl: Vec<_> = via.iter().map(|p| format!("({})", xy(p.x, p.y))).collect();
                    write!(path, " .. controls {} .. {}", ctrl.join(" and "), end(to)).unwrap();
                } else {
                    for p in via {
                        write!(path, " -- ({})", xy(p.x, p.y)).unwrap();
                    }
                    write!(path, " -- {}", end(to)).unwrap();
                }
                if !label.is_empty() {
                    let pos = match e.label_at {
                        Some(LabelAt::Along { t }) => format!("pos={}", num(t)),
                        _ => "midway".into(),
                    };
                    let mut node_opts = vec![pos, "fill=white".into()];
                    text_opts(&e.style, &mut node_opts);
                    write!(path, " node[{}] {{{label}}}", node_opts.join(", ")).unwrap();
                }
                let opts = if opts.is_empty() {
                    String::new()
                } else {
                    format!("[{}]", opts.join(", "))
                };
                writeln!(out, "  \\draw{opts} {path};").unwrap();
            }
        }
        if group.is_some() {
            let chunk = out.split_off(start);
            out.extend(chunk.lines().map(|l| format!("  {l}\n")));
        }
    }
    if group.is_some() {
        out.push_str("  \\end{scope}\n");
    }
    out.push_str("\\end{tikzpicture}\n\\end{document}\n");
    out
}

fn xy(x: f64, y: f64) -> String {
    format!("{}, {}", num(x), num(-y))
}

fn end(e: &End) -> String {
    match e {
        End::Point(p) => format!("({})", xy(p.x, p.y)),
        End::Node { node } => format!("({node})"),
    }
}

fn style_opts(s: &Style, node: bool, opts: &mut Vec<String>) {
    match (&s.stroke, node) {
        (None, false) => opts.push("draw=none".into()),
        (None, true) => {}
        (Some(c), _) if c == "#000000" => {
            if node {
                opts.push("draw".into())
            }
        }
        (Some(c), _) => opts.push(format!("draw={}", color(c))),
    }
    if let Some(a) = s.stroke.as_deref().map(|c| split_alpha(c).1).filter(|a| *a < 1.0) {
        opts.push(format!("draw opacity={}", num(a)));
    }
    if let Some(f) = &s.fill {
        opts.push(format!("fill={}", color(f)));
        let a = split_alpha(f).1;
        if a < 1.0 {
            opts.push(format!("fill opacity={}, text opacity=1", num(a)));
        }
    }
    if s.width != 1.0 {
        opts.push(format!("line width={}pt", num(s.width)));
    }
    match s.dash {
        Dash::Solid => {}
        Dash::Dashed => opts.push("dashed".into()),
        Dash::Dotted => opts.push("dotted".into()),
    }
    if s.radius > 0.0 {
        opts.push(format!("rounded corners={}cm", num(s.radius)));
    }
    if s.start != Arrow::None || s.end != Arrow::None {
        opts.push(format!("{}-{}", tip(s.start), tip(s.end)));
    }
}

fn text_opts(s: &Style, opts: &mut Vec<String>) {
    let mut font = String::new();
    if s.bold {
        font.push_str("\\bfseries");
    }
    if s.italic {
        font.push_str("\\itshape");
    }
    // A node's `font` replaces the picture-wide one, so the size must be repeated.
    if s.size != Style::default().size || !font.is_empty() {
        write!(
            font,
            "\\fontsize{{{}}}{{{}}}\\selectfont",
            num(s.size),
            num(s.size * 1.2)
        )
        .unwrap();
    }
    if !font.is_empty() {
        opts.push(format!("font={font}"));
    }
    if s.color != Style::default().color {
        opts.push(format!("text={}", color(&s.color)));
        let a = split_alpha(&s.color).1;
        if a < 1.0 {
            opts.push(format!("text opacity={}", num(a)));
        }
    }
}

fn tip(a: Arrow) -> &'static str {
    match a {
        Arrow::None => "",
        Arrow::Stealth => "{Stealth}",
        Arrow::Triangle => "{Triangle}",
        Arrow::Open => "{Straight Barb}",
        Arrow::Circle => "{Circle}",
        Arrow::Diamond => "{Diamond}",
        Arrow::Bar => "{Bar}",
    }
}

fn arrow_from_tip(t: &str) -> Arrow {
    let name = t.trim().trim_matches(|c| c == '{' || c == '}');
    let name = name.split('[').next().unwrap_or_default().trim();
    match name {
        "" => Arrow::None,
        ">"
        | "<"
        | "to"
        | "To"
        | "Straight Barb"
        | "straight barb"
        | "Classical TikZ Rightarrow"
        | "Computer Modern Rightarrow" => Arrow::Open,
        "stealth" | "Stealth" | "stealth'" => Arrow::Stealth,
        "Circle" | "o" | "*" => Arrow::Circle,
        "Diamond" | "diamond" => Arrow::Diamond,
        "Bar" | "|" => Arrow::Bar,
        _ => Arrow::Triangle,
    }
}

fn color(hex: &str) -> String {
    let hex = split_alpha(hex).0;
    if let Some((name, _)) = BASE_COLORS.iter().find(|(_, h)| *h == hex) {
        return name.to_string();
    }
    let (r, g, b) = rgb(&hex);
    format!("{{rgb,255:red,{r};green,{g};blue,{b}}}")
}

fn rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    let c = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    (c(0), c(2), c(4))
}

const BASE_COLORS: &[(&str, &str)] = &[
    ("black", "#000000"),
    ("white", "#ffffff"),
    ("red", "#ff0000"),
    ("green", "#00ff00"),
    ("blue", "#0000ff"),
    ("cyan", "#00ffff"),
    ("magenta", "#ff00ff"),
    ("yellow", "#ffff00"),
    ("gray", "#808080"),
    ("darkgray", "#404040"),
    ("lightgray", "#bfbfbf"),
    ("brown", "#bf8040"),
    ("lime", "#bfff00"),
    ("olive", "#808000"),
    ("orange", "#ff8000"),
    ("pink", "#ffbfbf"),
    ("purple", "#bf0040"),
    ("teal", "#008080"),
    ("violet", "#800080"),
];

// ---------- reader ----------

struct Reader<'a> {
    d: Diagram,
    w: &'a mut Warnings,
    colors: HashMap<String, String>,
    coords: HashMap<String, Point>,
    /// Group of the innermost open `scope`, and the groups of the enclosing ones.
    group: Option<String>,
    scopes: Vec<Option<String>>,
}

pub fn read(src: &str, w: &mut Warnings) -> Result<Diagram> {
    let src: String = src.lines().map(strip_comment).collect::<Vec<_>>().join("\n");
    let mut r = Reader {
        d: Diagram::default(),
        w,
        colors: HashMap::new(),
        coords: HashMap::new(),
        group: None,
        scopes: vec![],
    };
    r.define_colors(&src);
    let body = match src.find("\\begin{tikzpicture}") {
        Some(i) => {
            let rest = &src[i + "\\begin{tikzpicture}".len()..];
            let rest = rest.trim_start();
            let rest = if rest.starts_with('[') {
                &rest[group_end(rest, '[', ']')..]
            } else {
                rest
            };
            &rest[..rest.find("\\end{tikzpicture}").unwrap_or(rest.len())]
        }
        None => &src[..],
    };
    for stmt in statements(body) {
        r.statement(stmt.trim());
    }
    Ok(r.d)
}

fn strip_comment(line: &str) -> &str {
    let b = line.as_bytes();
    for i in 0..b.len() {
        if b[i] == b'%' && (i == 0 || b[i - 1] != b'\\') {
            return &line[..i];
        }
    }
    line
}

/// Byte index just after the group closing the one opened at `s[0]`.
fn group_end(s: &str, open: char, close: char) -> usize {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
    }
    s.len()
}

fn statements(s: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (vec![], 0i32, 0);
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            ';' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if !s[start..].trim().is_empty() {
        out.push(&s[start..]);
    }
    out
}

fn split_top(s: &str, sep: char) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (vec![], 0i32, 0);
    for (i, c) in s.char_indices() {
        match c {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth -= 1,
            c if c == sep && depth == 0 => {
                out.push(&s[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

fn parse_opts(s: &str) -> Vec<(String, String)> {
    split_top(s, ',')
        .into_iter()
        .map(str::trim)
        .filter(|o| !o.is_empty())
        .map(|o| match o.split_once('=') {
            Some((k, v)) => (k.trim().to_string(), v.trim().to_string()),
            None => (o.to_string(), String::new()),
        })
        .collect()
}

/// Length in cm. Bare numbers are cm.
fn length(s: &str) -> Option<f64> {
    let s = s.trim().trim_matches(|c| c == '{' || c == '}').trim();
    let split = s.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(s.len());
    let v: f64 = s[..split].trim().parse().ok()?;
    Some(match s[split..].trim() {
        "" | "cm" => v,
        "mm" => v / 10.0,
        "pt" => v / 28.4528,
        "bp" => v / 28.3465,
        "in" => v * 2.54,
        "em" => v * 0.35,
        "ex" => v * 0.15,
        _ => return None,
    })
}

fn pt_len(s: &str) -> Option<f64> {
    length(s).map(|cm| (cm * 28.4528 * 100.0).round() / 100.0)
}

#[derive(Debug, Clone)]
enum Tok {
    Opts(String),
    Paren(String),
    Rel(String, bool),
    Brace(String),
    Word(String),
}

fn tokenize(s: &str) -> Vec<Tok> {
    let mut out = vec![];
    let mut i = 0;
    let b = s.as_bytes();
    while i < s.len() {
        let c = b[i] as char;
        let rest = &s[i..];
        if c.is_whitespace() {
            i += 1;
        } else if c == '[' {
            let e = group_end(rest, '[', ']');
            out.push(Tok::Opts(rest[1..e - 1].to_string()));
            i += e;
        } else if c == '(' {
            let e = group_end(rest, '(', ')');
            out.push(Tok::Paren(rest[1..e - 1].to_string()));
            i += e;
        } else if c == '{' {
            let e = group_end(rest, '{', '}');
            out.push(Tok::Brace(rest[1..e.max(2) - 1].to_string()));
            i += e;
        } else if rest.starts_with("++(") || rest.starts_with("+(") {
            let skip = if rest.starts_with("++") { 2 } else { 1 };
            let e = group_end(&rest[skip..], '(', ')');
            out.push(Tok::Rel(rest[skip + 1..skip + e - 1].to_string(), skip == 2));
            i += skip + e;
        } else if let Some(op) = ["--", "-|", "|-", ".."].into_iter().find(|op| rest.starts_with(op)) {
            out.push(Tok::Word(op.into()));
            i += 2;
        } else {
            let len = rest
                .find(|c: char| c.is_whitespace() || "[({;".contains(c))
                .unwrap_or(rest.len())
                .max(c.len_utf8());
            out.push(Tok::Word(rest[..len].to_string()));
            i += len;
        }
    }
    out
}

enum Pos {
    P(Point),
    Node(String),
}

struct NodeSpec {
    opts: Vec<(String, String)>,
    name: Option<String>,
    at: Option<Point>,
    text: String,
}

impl Reader<'_> {
    fn define_colors(&mut self, src: &str) {
        let mut rest = src;
        while let Some(i) = rest.find("\\definecolor") {
            rest = &rest[i + "\\definecolor".len()..];
            let args: Vec<String> = tokenize(rest)
                .into_iter()
                .take(3)
                .filter_map(|t| match t {
                    Tok::Brace(s) => Some(s),
                    _ => None,
                })
                .collect();
            if let [name, model, spec] = args.as_slice() {
                let hex = match model.as_str() {
                    "HTML" => Some(format!("#{}", spec.to_lowercase())),
                    "RGB" | "rgb" => {
                        let v: Vec<f64> = spec.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                        let k = if model == "rgb" { 255.0 } else { 1.0 };
                        (v.len() == 3).then(|| hex3(v[0] * k, v[1] * k, v[2] * k))
                    }
                    _ => None,
                };
                if let Some(h) = hex {
                    self.colors.insert(name.clone(), h);
                }
            }
        }
    }

    fn color(&mut self, s: &str) -> Option<String> {
        let s = s.trim().trim_matches(|c| c == '{' || c == '}');
        if let Some(spec) = s.strip_prefix("rgb,255:") {
            let v: Vec<f64> = spec
                .split(';')
                .filter_map(|p| p.split(',').nth(1)?.trim().parse().ok())
                .collect();
            return (v.len() == 3).then(|| hex3(v[0], v[1], v[2]));
        }
        let parts: Vec<&str> = s.split('!').map(str::trim).collect();
        let mut cur = self.base_color(parts[0])?;
        let mut i = 1;
        while i < parts.len() {
            let pct: f64 = parts[i].parse().ok()?;
            let other = match parts.get(i + 1) {
                Some(n) => self.base_color(n)?,
                None => "#ffffff".into(),
            };
            cur = mix(&cur, &other, pct / 100.0);
            i += 2;
        }
        Some(cur)
    }

    fn base_color(&mut self, name: &str) -> Option<String> {
        if let Some(c) = self.colors.get(name) {
            return Some(c.clone());
        }
        let found = BASE_COLORS.iter().find(|(n, _)| *n == name).map(|(_, h)| h.to_string());
        if found.is_none() {
            self.w.push(format!("unknown color `{name}`"));
        }
        found
    }

    fn statement(&mut self, stmt: &str) {
        let mut stmt = stmt.trim();
        loop {
            if let Some(rest) = stmt.strip_prefix("\\begin{scope}") {
                let rest = rest.trim_start();
                let (opts, rest) = if rest.starts_with('[') {
                    let e = group_end(rest, '[', ']');
                    (parse_opts(&rest[1..e - 1]), &rest[e..])
                } else {
                    (vec![], rest)
                };
                let name = opts
                    .into_iter()
                    .find(|(k, _)| k == "local bounding box")
                    .map(|(_, v)| v);
                self.scopes.push(self.group.take());
                self.group = Some(name.unwrap_or_else(|| format!("g{}", self.scopes.len() + self.d.elements.len())));
                stmt = rest.trim();
            } else if let Some(rest) = stmt.strip_prefix("\\end{scope}") {
                self.group = self.scopes.pop().flatten();
                stmt = rest.trim();
            } else {
                break;
            }
        }
        if stmt.is_empty() {
            return;
        }
        let cmd_end = stmt[1..]
            .find(|c: char| !c.is_ascii_alphabetic())
            .map_or(stmt.len(), |i| i + 1);
        let (cmd, rest) = stmt.split_at(cmd_end);
        let toks = tokenize(rest);
        match cmd {
            "\\node" => {
                let spec = self.node_spec(&toks, Point::default());
                self.add_node(spec);
            }
            "\\coordinate" => {
                let spec = self.node_spec(&toks, Point::default());
                if let (Some(n), Some(p)) = (spec.name, spec.at) {
                    self.coords.insert(n, p);
                }
            }
            "\\draw" | "\\path" | "\\fill" | "\\filldraw" | "\\shade" | "\\shadedraw" => self.path(cmd, &toks),
            _ => self.w.push(format!("unsupported tikz command `{cmd}` skipped")),
        }
    }

    fn node_spec(&mut self, toks: &[Tok], current: Point) -> NodeSpec {
        let mut spec = NodeSpec {
            opts: vec![],
            name: None,
            at: None,
            text: String::new(),
        };
        let mut i = 0;
        while i < toks.len() {
            match &toks[i] {
                Tok::Opts(o) => spec.opts.extend(parse_opts(o)),
                Tok::Word(w) if w == "at" => {
                    if let Some(Tok::Paren(p)) = toks.get(i + 1) {
                        spec.at = self.pos(p).map(|p| self.resolve(&p));
                        i += 1;
                    }
                }
                Tok::Paren(p) => spec.name = Some(p.trim().to_string()),
                Tok::Brace(t) => {
                    spec.text = t.trim().to_string();
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        if spec.at.is_none() {
            spec.at = Some(self.relative_position(&spec.opts).unwrap_or(current));
        }
        spec
    }

    /// Supports `right=of a`, `right=2cm of a` and the older `right of=a`.
    fn relative_position(&mut self, opts: &[(String, String)]) -> Option<Point> {
        for (k, v) in opts {
            let (dir, dist, target) = if let Some(dir) = k.strip_suffix(" of") {
                (dir, 2.0, v.as_str())
            } else if let Some((d, t)) = v.split_once("of ") {
                (k.as_str(), length(d).map_or(2.0, |l| l + 1.0), t)
            } else {
                continue;
            };
            let (dx, dy) = match dir.trim() {
                "right" => (1.0, 0.0),
                "left" => (-1.0, 0.0),
                "above" => (0.0, -1.0),
                "below" => (0.0, 1.0),
                "above right" => (1.0, -1.0),
                "above left" => (-1.0, -1.0),
                "below right" => (1.0, 1.0),
                "below left" => (-1.0, 1.0),
                _ => continue,
            };
            let c = self.resolve(&Pos::Node(target.trim().to_string()));
            return Some(Point::new(c.x + dx * dist, c.y + dy * dist));
        }
        None
    }

    fn add_node(&mut self, spec: NodeSpec) -> String {
        let has = |k: &str| spec.opts.iter().any(|(o, _)| o == k);
        let get = |k: &str| spec.opts.iter().find(|(o, _)| o == k).map(|(_, v)| v.as_str());
        let shape_opt = get("shape").unwrap_or_default().to_string();
        let shape = if has("circle") || has("ellipse") || shape_opt == "circle" || shape_opt == "ellipse" {
            ShapeKind::Ellipse
        } else if has("diamond") || shape_opt == "diamond" {
            ShapeKind::Diamond
        } else {
            ShapeKind::Rect
        };
        let mut style = self.style(&spec.opts);
        let drawn = has("draw") || get("draw").is_some_and(|v| v != "none");
        if !drawn {
            style.stroke = None;
        }
        let image = spec.text.split_once("\\includegraphics").map(|(_, rest)| {
            let toks = tokenize(rest);
            let opts = toks.iter().find_map(|t| match t {
                Tok::Opts(o) => Some(parse_opts(o)),
                _ => None,
            });
            let path = toks.iter().find_map(|t| match t {
                Tok::Brace(b) => Some(b.trim().to_string()),
                _ => None,
            });
            (path.unwrap_or_default(), opts.unwrap_or_default())
        });
        let shape = if image.is_some() {
            ShapeKind::Image
        } else if !drawn && style.fill.is_none() {
            ShapeKind::Text
        } else {
            shape
        };
        let chars = spec.text.chars().filter(|c| !"\\${}".contains(*c)).count() as f64;
        let size = get("minimum size").and_then(length);
        let mut w = get("minimum width")
            .and_then(length)
            .or(size)
            .unwrap_or((chars * 0.2 + 0.4).max(0.6));
        let mut h = get("minimum height").and_then(length).or(size).unwrap_or(0.6);
        if has("circle") || shape_opt == "circle" {
            w = w.max(h);
            h = w;
        }
        let mut src = String::new();
        let mut text = spec.text.as_str();
        if let Some((path, opts)) = &image {
            let dim = |k: &str| opts.iter().find(|(o, _)| o == k).and_then(|(_, v)| length(v));
            w = dim("width").unwrap_or(w);
            h = dim("height").unwrap_or(h);
            src = path.clone();
            text = "";
        }
        let mut label = latex_to_typst(text);
        let mut label_at = None;
        if let Some(v) = get("label").filter(|_| text.is_empty()) {
            let (opts, place, body) = parse_label(v);
            let t = self.style(&opts);
            (style.font, style.size, style.bold, style.italic, style.color) =
                (t.font, t.size, t.bold, t.italic, t.color);
            let shift = opts.iter().find(|(k, _)| k == "shift").and_then(|(_, v)| {
                let inner = v.trim_matches(|c| "{}() ".contains(c));
                let (a, b) = inner.split_once(',')?;
                Some((length(a)?, length(b)?))
            });
            let (sx, sy) = shift.unwrap_or_default();
            let (px, py) = match place.as_str() {
                "above" => (0.0, -(h / 2.0 + 0.35)),
                "below" => (0.0, h / 2.0 + 0.35),
                "left" => (-(w / 2.0 + 0.5), 0.0),
                "right" => (w / 2.0 + 0.5, 0.0),
                _ => (0.0, 0.0),
            };
            label = latex_to_typst(&body);
            label_at = Some(LabelAt::Offset {
                dx: sx + px,
                dy: py - sy,
            });
        }
        let at = spec.at.unwrap_or_default();
        let id = match spec.name {
            Some(n) => n,
            None => self.d.fresh_id("n"),
        };
        self.d.elements.push(Element {
            id: id.clone(),
            kind: Kind::Shape {
                shape,
                x: at.x - w / 2.0,
                y: at.y - h / 2.0,
                w,
                h,
                src,
                rotate: get("rotate").and_then(|v| v.parse::<f64>().ok()).map_or(0.0, |r| -r),
            },
            style,
            label,
            label_at,
            group: self.group.clone(),
        });
        id
    }

    fn style(&mut self, opts: &[(String, String)]) -> Style {
        let mut s = Style::default();
        let mut opacity: (Option<f64>, Option<f64>, Option<f64>) = (None, None, None);
        for (k, v) in opts {
            match (k.as_str(), v.as_str()) {
                ("draw" | "color", c) if !c.is_empty() => s.stroke = if c == "none" { None } else { self.color(c) },
                ("fill", c) => s.fill = if c == "none" { None } else { self.color(c) },
                ("line width", l) => s.width = pt_len(l).unwrap_or(1.0),
                ("ultra thin", _) => s.width = 0.1,
                ("very thin", _) => s.width = 0.2,
                ("thin", _) => s.width = 0.4,
                ("semithick", _) => s.width = 0.6,
                ("thick", _) => s.width = 0.8,
                ("very thick", _) => s.width = 1.2,
                ("ultra thick", _) => s.width = 1.6,
                ("rounded corners", r) => s.radius = if r.is_empty() { 0.14 } else { length(r).unwrap_or(0.14) },
                (k, "") if k.ends_with("dashed") => s.dash = Dash::Dashed,
                (k, "") if k.ends_with("dotted") => s.dash = Dash::Dotted,
                (k, "") if is_arrow_spec(k) => {
                    let i = arrow_dash(k).unwrap();
                    s.start = arrow_from_tip(&k[..i]);
                    s.end = arrow_from_tip(&k[i + 1..]);
                }
                ("text", c) => s.color = self.color(c).unwrap_or(s.color),
                ("font", f) => {
                    s.bold = f.contains("\\bfseries") || f.contains("\\bf");
                    s.italic = f.contains("\\itshape") || f.contains("\\it") || f.contains("\\slshape");
                    if let Some(size) = font_size(f) {
                        s.size = size;
                    }
                }
                ("opacity", a) => opacity = (a.parse().ok(), a.parse().ok(), opacity.2),
                ("draw opacity", a) => opacity.0 = a.parse().ok(),
                ("fill opacity", a) => opacity.1 = a.parse().ok(),
                ("text opacity", a) => opacity.2 = a.parse().ok(),
                (k, "")
                    if BASE_COLORS.iter().any(|(n, _)| *n == k) || self.colors.contains_key(k) || k.contains('!') =>
                {
                    s.stroke = self.color(k)
                }
                _ => {}
            }
        }
        if let (Some(a), Some(c)) = (opacity.0, &s.stroke) {
            s.stroke = Some(with_alpha(&split_alpha(c).0, a));
        }
        if let (Some(a), Some(c)) = (opacity.1, &s.fill) {
            s.fill = Some(with_alpha(&split_alpha(c).0, a));
        }
        if let Some(a) = opacity.2 {
            s.color = with_alpha(&split_alpha(&s.color).0, a);
        }
        s
    }

    fn pos(&mut self, s: &str) -> Option<Pos> {
        let s = s.trim();
        if s.starts_with('$') {
            self.w.push(format!("calc coordinate `({s})` not supported"));
            return None;
        }
        if let Some((a, b)) = s.split_once(',') {
            return Some(Pos::P(Point::new(length(a)?, -length(b)?)));
        }
        if let Some((a, r)) = s.split_once(':') {
            let (a, r) = (a.trim().parse::<f64>().ok()?.to_radians(), length(r)?);
            return Some(Pos::P(Point::new(r * a.cos(), -r * a.sin())));
        }
        let name = s.split('.').next().unwrap_or(s).trim();
        match self.coords.get(name) {
            Some(p) => Some(Pos::P(*p)),
            None => Some(Pos::Node(name.to_string())),
        }
    }

    fn resolve(&self, p: &Pos) -> Point {
        match p {
            Pos::P(p) => *p,
            Pos::Node(n) => self.d.center(&End::Node { node: n.clone() }),
        }
    }

    fn path(&mut self, cmd: &str, toks: &[Tok]) {
        let mut opts = vec![];
        let mut i = 0;
        while let Some(Tok::Opts(o)) = toks.get(i) {
            opts.extend(parse_opts(o));
            i += 1;
        }
        let mut style = self.style(&opts);
        if cmd == "\\path" && !opts.iter().any(|(k, _)| k == "draw") {
            style.stroke = None;
        }
        if cmd == "\\fill" {
            style.stroke = None;
            style.fill = style.fill.clone().or(Some("#000000".into()));
        }

        let mut pts: Vec<Pos> = vec![];
        let mut controls: Vec<Point> = vec![];
        let mut curved = false;
        let mut label = String::new();
        let mut label_at: Option<LabelAt> = None;
        let mut connected = true;
        let mut corner: Option<&str> = None;
        let mut last = Point::default();
        while i < toks.len() {
            match &toks[i] {
                Tok::Paren(p) | Tok::Rel(p, _) => {
                    let Some(mut pos) = self.pos(p) else {
                        i += 1;
                        continue;
                    };
                    if let (Tok::Rel(..), Pos::P(d)) = (&toks[i], &pos) {
                        pos = Pos::P(Point::new(last.x + d.x, last.y + d.y));
                    }
                    if !connected && !pts.is_empty() {
                        self.flush(&mut pts, &mut controls, curved, &style, &mut label, &mut label_at);
                        curved = false;
                    }
                    let p = self.resolve(&pos);
                    if let Some(c) = corner.take() {
                        let k = if c == "-|" {
                            Point::new(p.x, last.y)
                        } else {
                            Point::new(last.x, p.y)
                        };
                        pts.push(Pos::P(k));
                    }
                    if !matches!(toks[i], Tok::Rel(_, false)) {
                        last = p;
                    }
                    pts.push(pos);
                    connected = false;
                }
                Tok::Word(w) => match w.as_str() {
                    "--" | "to" => connected = true,
                    "-|" | "|-" => {
                        connected = true;
                        corner = Some(if w == "-|" { "-|" } else { "|-" });
                    }
                    ".." => {
                        connected = true;
                        if let Some(Tok::Word(c)) = toks.get(i + 1)
                            && c == "controls"
                        {
                            curved = true;
                            i += 2;
                            while let Some(Tok::Paren(p) | Tok::Rel(p, _)) = toks.get(i) {
                                if let Some(pos) = self.pos(p) {
                                    controls.push(self.resolve(&pos));
                                }
                                i += 1;
                                if let Some(Tok::Word(a)) = toks.get(i)
                                    && a == "and"
                                {
                                    i += 1;
                                }
                            }
                            continue;
                        }
                    }
                    "rectangle" => {
                        if let (Some(a), Some(Tok::Paren(p))) = (pts.pop(), toks.get(i + 1)) {
                            let (a, b) = (self.resolve(&a), self.pos(p).map(|p| self.resolve(&p)).unwrap_or(last));
                            self.push_shape(ShapeKind::Rect, a, b, &style);
                            last = b;
                            i += 1;
                        }
                    }
                    "circle" | "ellipse" => {
                        let c = pts.pop().map(|p| self.resolve(&p)).unwrap_or(last);
                        let (mut rx, mut ry) = (1.0, 1.0);
                        match toks.get(i + 1) {
                            Some(Tok::Opts(o)) => {
                                for (k, v) in parse_opts(o) {
                                    match k.as_str() {
                                        "radius" => (rx, ry) = (length(&v).unwrap_or(1.0), length(&v).unwrap_or(1.0)),
                                        "x radius" => rx = length(&v).unwrap_or(1.0),
                                        "y radius" => ry = length(&v).unwrap_or(1.0),
                                        _ => {}
                                    }
                                }
                                i += 1;
                            }
                            Some(Tok::Paren(p)) => {
                                let r: Vec<f64> = p.split(" and ").filter_map(length).collect();
                                (rx, ry) = (
                                    r.first().copied().unwrap_or(1.0),
                                    r.get(1).or(r.first()).copied().unwrap_or(1.0),
                                );
                                i += 1;
                            }
                            _ => {}
                        }
                        self.push_shape(
                            ShapeKind::Ellipse,
                            Point::new(c.x - rx, c.y - ry),
                            Point::new(c.x + rx, c.y + ry),
                            &style,
                        );
                    }
                    "node" => {
                        let start = i + 1;
                        let mut j = start;
                        while j < toks.len() && !matches!(toks[j], Tok::Brace(_)) {
                            j += 1;
                        }
                        let spec = self.node_spec(&toks[start..(j + 1).min(toks.len())], last);
                        if pts.len() >= 2 || connected {
                            label = latex_to_typst(&spec.text);
                            label_at = node_pos(&spec.opts).map(|t| LabelAt::Along { t });
                            let t = self.style(&spec.opts);
                            (style.font, style.size, style.bold, style.italic, style.color) =
                                (t.font, t.size, t.bold, t.italic, t.color);
                        } else {
                            pts.pop();
                            self.add_node(spec);
                        }
                        i = j;
                    }
                    "cycle" => self
                        .w
                        .push("closed paths (`cycle`) are not supported, drawn open".into()),
                    "edge" | "plot" | "arc" | "grid" | "sin" | "cos" | "parabola" | "child" => {
                        self.w.push(format!("unsupported path operation `{w}` skipped"));
                    }
                    "and" | "controls" => {}
                    other => self.w.push(format!("unexpected token `{other}`")),
                },
                Tok::Opts(_) | Tok::Brace(_) => {}
            }
            i += 1;
        }
        self.flush(&mut pts, &mut controls, curved, &style, &mut label, &mut label_at);
    }

    fn push_shape(&mut self, shape: ShapeKind, a: Point, b: Point, style: &Style) {
        let id = self.d.fresh_id("n");
        self.d.elements.push(Element {
            id,
            kind: Kind::Shape {
                shape,
                x: a.x.min(b.x),
                y: a.y.min(b.y),
                w: (a.x - b.x).abs(),
                h: (a.y - b.y).abs(),
                src: String::new(),
                rotate: 0.0,
            },
            style: style.clone(),
            label: String::new(),
            group: self.group.clone(),
            label_at: None,
        });
    }

    fn flush(
        &mut self,
        pts: &mut Vec<Pos>,
        controls: &mut Vec<Point>,
        curved: bool,
        style: &Style,
        label: &mut String,
        label_at: &mut Option<LabelAt>,
    ) {
        if pts.len() < 2 {
            pts.clear();
            controls.clear();
            return;
        }
        let to_end = |p: Pos| match p {
            Pos::P(p) => End::Point(p),
            Pos::Node(node) => End::Node { node },
        };
        let to = to_end(pts.pop().unwrap());
        let mut it = pts.drain(..);
        let from = to_end(it.next().unwrap());
        let mut via: Vec<Point> = it.map(|p| self.resolve(&p)).collect();
        let curved = curved && via.is_empty() && (1..=2).contains(&controls.len());
        if curved {
            via = std::mem::take(controls);
        } else if !controls.is_empty() {
            self.w
                .push("multi-segment bezier paths are approximated by straight lines".into());
        }
        controls.clear();
        let mut style = style.clone();
        style.fill = None;
        let id = self.d.fresh_id("e");
        self.d.elements.push(Element {
            id,
            kind: Kind::Line { from, to, via, curved },
            style,
            label: std::mem::take(label),
            group: self.group.clone(),
            label_at: label_at.take(),
        });
    }
}

/// Position of an inline path node (`pos=0.3`, `near end`, ...); `None` for the middle.
fn node_pos(opts: &[(String, String)]) -> Option<f64> {
    let t = opts.iter().find_map(|(k, v)| match k.as_str() {
        "pos" => v.parse().ok(),
        "at start" => Some(0.0),
        "very near start" => Some(0.125),
        "near start" => Some(0.25),
        "near end" => Some(0.75),
        "very near end" => Some(0.875),
        "at end" => Some(1.0),
        _ => None,
    })?;
    (t != 0.5).then_some(t)
}

/// Splits a node `label` option like `{[shift={(1,0)}]center:{text}}` into options, position and text.
fn parse_label(v: &str) -> (Vec<(String, String)>, String, String) {
    let mut v = v.trim();
    if v.starts_with('{') && group_end(v, '{', '}') == v.len() {
        v = &v[1..v.len() - 1];
    }
    let mut opts = vec![];
    if v.starts_with('[') {
        let e = group_end(v, '[', ']');
        opts = parse_opts(&v[1..e - 1]);
        v = &v[e..];
    }
    let (place, text) = v.split_once(':').unwrap_or(("center", v));
    (
        opts,
        place.trim().to_string(),
        text.trim().trim_matches(|c| c == '{' || c == '}').to_string(),
    )
}

/// Index of the `-` separating start and end tips in an arrow spec like `{Bar}-{Stealth[length=3mm]}`.
fn arrow_dash(k: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in k.char_indices() {
        match c {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            '-' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

fn is_arrow_spec(k: &str) -> bool {
    let Some(i) = arrow_dash(k) else { return false };
    let outside = |s: &str| {
        let mut depth = 0;
        s.chars().all(|c| {
            match c {
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                ' ' if depth == 0 => return false,
                _ => {}
            }
            true
        })
    };
    outside(&k[..i]) && outside(&k[i + 1..]) && !k[i + 1..].contains('-')
}

fn font_size(f: &str) -> Option<f64> {
    if let Some(rest) = f.split("\\fontsize{").nth(1) {
        return rest.split('}').next()?.trim_end_matches("pt").parse().ok();
    }
    [
        ("\\tiny", 5.0),
        ("\\scriptsize", 7.0),
        ("\\footnotesize", 8.0),
        ("\\small", 9.0),
        ("\\normalsize", 10.0),
        ("\\large", 12.0),
        ("\\Large", 14.4),
        ("\\LARGE", 17.28),
        ("\\huge", 20.74),
        ("\\Huge", 24.88),
    ]
    .iter()
    .find(|(cmd, _)| f.split('\\').any(|part| format!("\\{part}").trim() == *cmd))
    .map(|(_, s)| *s)
}

fn hex3(r: f64, g: f64, b: f64) -> String {
    let c = |v: f64| v.round().clamp(0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b))
}

fn mix(a: &str, b: &str, t: f64) -> String {
    let (a, b) = (rgb(a), rgb(b));
    let m = |x: u8, y: u8| x as f64 * t + y as f64 * (1.0 - t);
    hex3(m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
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
        let src = r"
\definecolor{mine}{HTML}{1E88E5}
\begin{tikzpicture}[node distance=2cm]
  \node[draw, circle, fill=red!20] (a) at (0,0) {$\alpha$}; % comment
  \node[draw=mine, rounded corners, right=of a] (b) {B};
  \draw[->, thick] (a) -- (b);
  \draw[dashed] (0,-2) -- ++(1,0) -| (b.south);
  \draw (3,3) rectangle (4,4);
  \draw (0,0) circle (1);
  \foreach \x in {1,2} { \draw (\x,0) -- (\x,1); }
\end{tikzpicture}";
        let mut w = vec![];
        let d = read(src, &mut w).unwrap();
        let a = d.get("a").unwrap();
        assert_eq!(a.label, "$alpha$");
        assert_eq!(a.style.fill.as_deref(), Some("#ffcccc"));
        assert_eq!(d.get("b").unwrap().style.stroke.as_deref(), Some("#1e88e5"));
        let lines: Vec<_> = d
            .elements
            .iter()
            .filter(|e| matches!(e.kind, Kind::Line { .. }))
            .collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].style.end, Arrow::Open);
        assert_eq!(lines[0].style.width, 0.8);
        if let Kind::Line { via, to, .. } = &lines[1].kind {
            assert_eq!(via.len(), 2);
            assert_eq!(to, &End::Node { node: "b".into() });
        }
        assert_eq!(d.elements.len(), 6);
        assert_eq!(w.len(), 1, "{w:?}");
    }
}
