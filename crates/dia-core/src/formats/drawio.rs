use super::Format;
use crate::ir::*;
use crate::text::{drawio_to_typst, typst_to_drawio};
use crate::{Error, Result, Warnings};
use base64::Engine;
use quick_xml::escape::escape;
use quick_xml::events::{BytesStart, Event};
use std::collections::HashMap;
use std::fmt::Write;
use std::io::Read;

pub const FORMAT: Format = Format {
    name: "drawio",
    extensions: &["drawio", "xml"],
    read,
    write,
};

/// drawio pixels per centimeter (drawio's default 120x60 shape becomes 3x1.5cm).
const PX: f64 = 40.0;

pub fn write(d: &Diagram) -> String {
    let math = d.elements.iter().any(|e| e.label.contains('$'));
    let mut out = format!(
        "<mxfile host=\"dia\">\n  <diagram id=\"dia\" name=\"Page-1\">\n    <mxGraphModel grid=\"1\" gridSize=\"10\" math=\"{}\">\n      <root>\n        <mxCell id=\"0\"/>\n        <mxCell id=\"1\" parent=\"0\"/>\n",
        math as u8
    );
    for e in &d.elements {
        let value = escape(typst_to_drawio(&e.label)).to_string();
        let id = escape(e.id.as_str()).to_string();
        let s = &e.style;
        let mut style = Vec::new();
        match &e.kind {
            Kind::Shape { shape, x, y, w, h } => {
                match shape {
                    ShapeKind::Rect => style.push("rounded=0".to_string()),
                    ShapeKind::Ellipse => style.push("ellipse".into()),
                    ShapeKind::Diamond => style.push("rhombus".into()),
                    ShapeKind::Text => style.push("text".into()),
                }
                if s.radius > 0.0 {
                    style[0] = format!("rounded=1;absoluteArcSize=1;arcSize={}", num(s.radius * 2.0 * PX));
                }
                style.push("whiteSpace=wrap;html=1".into());
                style_attrs(s, &mut style);
                writeln!(
                    out,
                    "        <mxCell id=\"{id}\" value=\"{value}\" style=\"{};\" vertex=\"1\" parent=\"1\">\n          <mxGeometry x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" as=\"geometry\"/>\n        </mxCell>",
                    style.join(";"),
                    num(x * PX),
                    num(y * PX),
                    num(w * PX),
                    num(h * PX)
                )
                .unwrap();
            }
            Kind::Line { from, to, via, curved } => {
                style.push("html=1;rounded=0".into());
                style.push(format!("endArrow={}", if s.end_arrow { "classic" } else { "none" }));
                style.push(format!("startArrow={}", if s.start_arrow { "classic" } else { "none" }));
                if *curved {
                    style.push("curved=1".into());
                }
                if !e.label.is_empty() {
                    style.push("labelBackgroundColor=#ffffff".into());
                }
                style_attrs(s, &mut style);
                let mut attrs = String::new();
                let mut points = String::new();
                for (end, attr, as_) in [(from, "source", "sourcePoint"), (to, "target", "targetPoint")] {
                    match end {
                        End::Node { node } => write!(attrs, " {attr}=\"{}\"", escape(node.as_str())).unwrap(),
                        End::Point(p) => write!(
                            points,
                            "\n            <mxPoint x=\"{}\" y=\"{}\" as=\"{as_}\"/>",
                            num(p.x * PX),
                            num(p.y * PX)
                        )
                        .unwrap(),
                    }
                }
                if !via.is_empty() {
                    points.push_str("\n            <Array as=\"points\">");
                    for p in via {
                        write!(
                            points,
                            "\n              <mxPoint x=\"{}\" y=\"{}\"/>",
                            num(p.x * PX),
                            num(p.y * PX)
                        )
                        .unwrap();
                    }
                    points.push_str("\n            </Array>");
                }
                writeln!(
                    out,
                    "        <mxCell id=\"{id}\" value=\"{value}\" style=\"{};\" edge=\"1\" parent=\"1\"{attrs}>\n          <mxGeometry relative=\"1\" as=\"geometry\">{points}\n          </mxGeometry>\n        </mxCell>",
                    style.join(";")
                )
                .unwrap();
            }
        }
    }
    out.push_str("      </root>\n    </mxGraphModel>\n  </diagram>\n</mxfile>\n");
    out
}

fn style_attrs(s: &Style, style: &mut Vec<String>) {
    style.push(format!("strokeColor={}", s.stroke.as_deref().unwrap_or("none")));
    style.push(format!("fillColor={}", s.fill.as_deref().unwrap_or("none")));
    if s.width != 1.0 {
        style.push(format!("strokeWidth={}", num(s.width)));
    }
    match s.dash {
        Dash::Solid => {}
        Dash::Dashed => style.push("dashed=1".into()),
        Dash::Dotted => style.push("dashed=1;dashPattern=1 2".into()),
    }
}

// ---------- reader ----------

#[derive(Default)]
struct Cell {
    id: String,
    value: String,
    style: String,
    vertex: bool,
    edge: bool,
    parent: String,
    source: Option<String>,
    target: Option<String>,
    rect: [f64; 4],
    source_point: Option<Point>,
    target_point: Option<Point>,
    points: Vec<Point>,
}

pub fn read(src: &str, w: &mut Warnings) -> Result<Diagram> {
    let model = graph_model(src, w)?;
    let cells = parse_cells(&model)?;
    let by_id: HashMap<&str, &Cell> = cells.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut d = Diagram::default();
    for c in &cells {
        let st = parse_style(&c.style);
        let get = |k: &str| st.get(k).map(String::as_str);
        let html = get("html") == Some("1");
        let label = if html {
            drawio_to_typst(&c.value)
        } else {
            c.value.clone()
        };
        let o = offset(&by_id, &c.parent);
        let px = |p: Point| Point::new((p.x + o.x) / PX, (p.y + o.y) / PX);
        if get("edgeLabel").is_some() || (c.vertex && by_id.get(c.parent.as_str()).is_some_and(|p| p.edge)) {
            if let Some(e) = d.elements.iter_mut().find(|e| e.id == c.parent) {
                e.label = label;
            }
            continue;
        }
        let mut style = Style {
            stroke: color(get("strokeColor"), Some("#000000")),
            width: get("strokeWidth").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            dash: match (get("dashed"), get("dashPattern")) {
                (Some("1"), Some(p)) if p.starts_with("1 ") => Dash::Dotted,
                (Some("1"), _) => Dash::Dashed,
                _ => Dash::Solid,
            },
            ..Style::default()
        };
        if c.vertex {
            let bare = st.iter().find(|(_, v)| v.is_empty()).map(|(k, _)| k.as_str());
            let shape = match get("shape").or(bare) {
                Some("ellipse") => ShapeKind::Ellipse,
                Some("rhombus") => ShapeKind::Diamond,
                Some("text") => ShapeKind::Text,
                Some(other) => {
                    w.push(format!("unsupported drawio shape `{other}` imported as rectangle"));
                    ShapeKind::Rect
                }
                _ => ShapeKind::Rect,
            };
            let [x, y, cw, ch] = c.rect;
            let fill_default = if shape == ShapeKind::Text {
                None
            } else {
                Some("#ffffff")
            };
            style.fill = color(get("fillColor"), fill_default);
            if shape == ShapeKind::Text {
                style.stroke = color(get("strokeColor"), None);
            }
            if get("rounded") == Some("1") && shape == ShapeKind::Rect {
                let arc: Option<f64> = get("arcSize").and_then(|v| v.parse().ok());
                style.radius = if get("absoluteArcSize") == Some("1") {
                    arc.unwrap_or(30.0) / 2.0 / PX
                } else {
                    cw.min(ch) * arc.unwrap_or(15.0) / 100.0 / PX
                };
            }
            let p = px(Point::new(x, y));
            d.elements.push(Element {
                id: c.id.clone(),
                kind: Kind::Shape {
                    shape,
                    x: p.x,
                    y: p.y,
                    w: cw / PX,
                    h: ch / PX,
                },
                style,
                label,
            });
        } else if c.edge {
            style.end_arrow = get("endArrow") != Some("none");
            style.start_arrow = get("startArrow").is_some_and(|v| v != "none");
            let end = |node: &Option<String>, point: Option<Point>| match (node, point) {
                (Some(n), _) => Some(End::Node { node: n.clone() }),
                (None, Some(p)) => Some(End::Point(px(p))),
                _ => None,
            };
            let (Some(from), Some(to)) = (end(&c.source, c.source_point), end(&c.target, c.target_point)) else {
                w.push(format!("edge `{}` without endpoints skipped", c.id));
                continue;
            };
            d.elements.push(Element {
                id: c.id.clone(),
                kind: Kind::Line {
                    from,
                    to,
                    via: c.points.iter().map(|p| px(*p)).collect(),
                    curved: get("curved") == Some("1"),
                },
                style,
                label,
            });
        }
    }
    Ok(d)
}

/// Absolute position of a container, since drawio child geometry is relative to its parent.
fn offset<'a>(by_id: &HashMap<&str, &'a Cell>, mut parent: &'a str) -> Point {
    let mut o = Point::default();
    while let Some(p) = by_id.get(parent).filter(|p| p.vertex) {
        o.x += p.rect[0];
        o.y += p.rect[1];
        parent = &p.parent;
    }
    o
}

fn color(v: Option<&str>, default: Option<&str>) -> Option<String> {
    match v {
        Some("none") => None,
        Some(c) if c.starts_with('#') => Some(c.to_lowercase()),
        _ => default.map(String::from),
    }
}

fn parse_style(s: &str) -> HashMap<String, String> {
    s.split(';')
        .filter(|p| !p.is_empty())
        .map(|p| match p.split_once('=') {
            Some((k, v)) => (k.to_string(), v.to_string()),
            None => (p.to_string(), String::new()),
        })
        .collect()
}

fn xml_err(e: impl std::fmt::Display) -> Error {
    Error::Parse(e.to_string())
}

fn attr(e: &BytesStart, name: &str) -> Option<String> {
    e.attributes().flatten().find(|a| a.key.as_ref() == name).and_then(|a| {
        a.normalized_value(quick_xml::XmlVersion::Implicit1_0)
            .ok()
            .map(|v| v.into_owned())
    })
}

/// Returns the `<mxGraphModel>` XML of the first page, decompressing it if needed.
fn graph_model(src: &str, w: &mut Warnings) -> Result<String> {
    if !src.contains("<diagram") {
        return Ok(src.to_string());
    }
    let mut reader = quick_xml::Reader::from_str(src);
    let (mut pages, mut in_diagram, mut text) = (0, false, String::new());
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Start(e) if e.name().as_ref() == "diagram" => {
                pages += 1;
                in_diagram = pages == 1;
            }
            Event::End(e) if e.name().as_ref() == "diagram" => in_diagram = false,
            Event::Start(e) if in_diagram && e.name().as_ref() == "mxGraphModel" => return Ok(src.to_string()),
            Event::Text(t) if in_diagram => text.push_str(&t.xml10_content()),
            Event::Eof => break,
            _ => {}
        }
    }
    if pages > 1 {
        w.push(format!("only the first of {pages} pages was imported"));
    }
    let raw = base64::engine::general_purpose::STANDARD
        .decode(text.trim())
        .map_err(xml_err)?;
    let mut inflated = String::new();
    flate2::read::DeflateDecoder::new(raw.as_slice())
        .read_to_string(&mut inflated)
        .map_err(xml_err)?;
    Ok(percent_decode(&inflated))
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && let Some(v) = s.get(i + 1..i + 3).and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn parse_cells(xml: &str) -> Result<Vec<Cell>> {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut cells: Vec<Cell> = vec![];
    let mut object: Option<(String, String)> = None;
    let mut in_points = false;
    let mut in_model = false;
    loop {
        let (e, empty) = match reader.read_event().map_err(xml_err)? {
            Event::Start(e) => (e, false),
            Event::Empty(e) => (e, true),
            Event::End(e) => {
                match e.name().as_ref() {
                    "Array" => in_points = false,
                    "UserObject" | "object" => object = None,
                    "mxGraphModel" => break,
                    _ => {}
                }
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };
        match e.name().as_ref() {
            "mxGraphModel" => in_model = true,
            _ if !in_model => {}
            "UserObject" | "object" => {
                object = Some((
                    attr(&e, "id").unwrap_or_default(),
                    attr(&e, "label").unwrap_or_default(),
                ));
            }
            "mxCell" => {
                let (id, value) = match &object {
                    Some((id, label)) => (id.clone(), label.clone()),
                    None => (
                        attr(&e, "id").unwrap_or_default(),
                        attr(&e, "value").unwrap_or_default(),
                    ),
                };
                cells.push(Cell {
                    id,
                    value,
                    style: attr(&e, "style").unwrap_or_default(),
                    vertex: attr(&e, "vertex").as_deref() == Some("1"),
                    edge: attr(&e, "edge").as_deref() == Some("1"),
                    parent: attr(&e, "parent").unwrap_or_default(),
                    source: attr(&e, "source"),
                    target: attr(&e, "target"),
                    ..Cell::default()
                });
            }
            "mxGeometry" => {
                if let Some(c) = cells.last_mut() {
                    let f = |k: &str| attr(&e, k).and_then(|v| v.parse().ok()).unwrap_or(0.0);
                    c.rect = [f("x"), f("y"), f("width"), f("height")];
                }
            }
            "Array" if attr(&e, "as").as_deref() == Some("points") => in_points = !empty,
            "mxPoint" => {
                let f = |k: &str| attr(&e, k).and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let p = Point::new(f("x"), f("y"));
                if let Some(c) = cells.last_mut() {
                    match attr(&e, "as").as_deref() {
                        Some("sourcePoint") => c.source_point = Some(p),
                        Some("targetPoint") => c.target_point = Some(p),
                        _ if in_points => c.points.push(p),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    Ok(cells)
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
    fn compressed() {
        // A drawio file with one compressed page containing a rounded rect "Hi" and an ellipse joined by an edge.
        let model = r#"<mxGraphModel><root><mxCell id="0"/><mxCell id="1" parent="0"/><mxCell id="a" value="Hi" style="rounded=1;whiteSpace=wrap;html=1;" vertex="1" parent="1"><mxGeometry x="40" y="40" width="120" height="60" as="geometry"/></mxCell><mxCell id="b" value="" style="ellipse;whiteSpace=wrap;html=1;fillColor=#dae8fc;" vertex="1" parent="1"><mxGeometry x="240" y="40" width="80" height="80" as="geometry"/></mxCell><mxCell id="c" edge="1" parent="1" source="a" target="b" style="edgeStyle=orthogonalEdgeStyle;"><mxGeometry relative="1" as="geometry"/></mxCell><mxCell id="l" value="\(x^2\)" style="edgeLabel;html=1;" vertex="1" connectable="0" parent="c"><mxGeometry relative="1" as="geometry"/></mxCell></root></mxGraphModel>"#;
        let encoded: String = model
            .bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() {
                    (b as char).to_string()
                } else {
                    format!("%{b:02X}")
                }
            })
            .collect();
        let mut z = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(&mut z, encoded.as_bytes()).unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode(z.finish().unwrap());
        let src = format!(r#"<mxfile><diagram id="x" name="P">{b64}</diagram></mxfile>"#);
        let mut w = vec![];
        let d = read(&src, &mut w).unwrap();
        assert_eq!(d.elements.len(), 3);
        assert_eq!(d.get("a").unwrap().style.radius, 0.225);
        assert_eq!(d.get("b").unwrap().style.fill.as_deref(), Some("#dae8fc"));
        let c = d.get("c").unwrap();
        assert!(c.style.end_arrow);
        assert_eq!(c.label, "$x^2$");
    }
}
