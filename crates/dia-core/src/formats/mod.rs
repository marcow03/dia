use crate::{Diagram, Error, Result, Warnings};

pub mod cetz;
pub mod drawio;
pub mod tikz;

pub struct Format {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub read: fn(&str, &mut Warnings) -> Result<Diagram>,
    pub write: fn(&Diagram) -> String,
}

pub const FORMATS: &[Format] = &[cetz::FORMAT, tikz::FORMAT, drawio::FORMAT];

pub fn format(name: &str) -> Result<&'static Format> {
    FORMATS
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| Error::UnknownFormat(name.into()))
}

pub fn format_for_path(path: &str) -> Result<&'static Format> {
    let ext = path.rsplit('.').next().unwrap_or_default().to_lowercase();
    FORMATS
        .iter()
        .find(|f| f.extensions.contains(&ext.as_str()))
        .ok_or_else(|| Error::UnknownFormat(path.into()))
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::ir::*;

    pub fn sample() -> Diagram {
        let shape = |id: &str, shape, x, y, label: &str, style: Style| Element {
            id: id.into(),
            kind: Kind::Shape {
                shape,
                x,
                y,
                w: 3.0,
                h: 1.5,
                src: String::new(),
                rotate: 0.0,
            },
            style,
            label: label.into(),
            group: None,
            label_at: None,
        };
        let node = |n: &str| End::node(n);
        let mut d = Diagram {
            elements: vec![
                shape(
                    "n1",
                    ShapeKind::Rect,
                    0.0,
                    0.0,
                    "Start",
                    Style {
                        radius: 0.2,
                        fill: Some("#e3f2fd".into()),
                        ..Style::default()
                    },
                ),
                shape(
                    "n2",
                    ShapeKind::Ellipse,
                    5.0,
                    0.0,
                    "$x^2 + y^2$",
                    Style {
                        fill: Some("#34c75980".into()),
                        ..Style::default()
                    },
                ),
                shape(
                    "n3",
                    ShapeKind::Diamond,
                    5.0,
                    3.0,
                    "ok?",
                    Style {
                        stroke: Some("#1e88e5".into()),
                        width: 2.0,
                        ..Style::default()
                    },
                ),
                shape(
                    "t1",
                    ShapeKind::Text,
                    0.0,
                    3.0,
                    "note",
                    Style {
                        stroke: None,
                        size: 14.0,
                        bold: true,
                        italic: true,
                        color: "#ff3b30".into(),
                        align: Align::Right,
                        ..Style::default()
                    },
                ),
                Element {
                    id: "i1".into(),
                    kind: Kind::Shape {
                        shape: ShapeKind::Image,
                        x: 10.0,
                        y: 0.0,
                        w: 2.0,
                        h: 1.0,
                        src: "logo.png".into(),
                        rotate: 0.0,
                    },
                    style: Style {
                        stroke: None,
                        ..Style::default()
                    },
                    label: String::new(),
                    group: None,
                    label_at: None,
                },
                Element {
                    id: "e1".into(),
                    kind: Kind::Line {
                        from: node("n1"),
                        to: node("n2"),
                        via: vec![],
                        curved: false,
                    },
                    style: Style {
                        end: Arrow::Stealth,
                        ..Style::default()
                    },
                    label: "$alpha$".into(),
                    group: None,
                    label_at: Some(LabelAt::Along { t: 0.3 }),
                },
                Element {
                    id: "e2".into(),
                    kind: Kind::Line {
                        from: node("n2"),
                        to: node("n3"),
                        via: vec![Point::new(8.5, 0.75), Point::new(8.5, 3.75)],
                        curved: false,
                    },
                    style: Style {
                        dash: Dash::Dotted,
                        start: Arrow::Circle,
                        end: Arrow::Triangle,
                        ..Style::default()
                    },
                    label: String::new(),
                    group: None,
                    label_at: None,
                },
                Element {
                    id: "e3".into(),
                    kind: Kind::Line {
                        from: End::Point(Point::new(0.0, 6.0)),
                        to: node("n3"),
                        via: vec![Point::new(2.0, 5.0), Point::new(4.0, 7.0)],
                        curved: true,
                    },
                    style: Style {
                        end: Arrow::Open,
                        width: 0.5,
                        ..Style::default()
                    },
                    label: String::new(),
                    group: None,
                    label_at: None,
                },
            ],
        };
        d.elements[0].label_at = Some(LabelAt::Offset { dx: 0.5, dy: -0.25 });
        d.elements[0].group = Some("g1".into());
        for e in &mut d.elements {
            if let (Kind::Line { from, to, .. }, "e2") = (&mut e.kind, e.id.as_str()) {
                *from = End::Node {
                    node: "n2".into(),
                    anchor: Some(Anchor::East),
                };
                *to = End::Node {
                    node: "n3".into(),
                    anchor: Some(Anchor::NorthEast),
                };
            }
        }
        d.elements[1].group = Some("g1".into());
        if let Kind::Shape { rotate, .. } = &mut d.elements[2].kind {
            *rotate = 20.0;
        }
        d
    }

    #[test]
    fn cross_format_chains() {
        for a in super::FORMATS {
            for b in super::FORMATS {
                let mut w = vec![];
                let via_a = (a.read)(&(a.write)(&sample()), &mut w).unwrap();
                let via_b = (b.read)(&(b.write)(&via_a), &mut w).unwrap();
                assert!(w.is_empty(), "{} -> {}: {w:?}", a.name, b.name);
                assert_eq!(via_b, sample(), "{} -> {}", a.name, b.name);
            }
        }
    }

    #[test]
    #[ignore]
    fn dump_sample() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures");
        for f in super::FORMATS {
            std::fs::write(format!("{dir}/sample.{}", f.extensions[0]), (f.write)(&sample())).unwrap();
        }
    }
}
