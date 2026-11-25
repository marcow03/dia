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
            },
            style,
            label: label.into(),
        };
        let node = |n: &str| End::Node { node: n.into() };
        Diagram {
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
                shape("n2", ShapeKind::Ellipse, 5.0, 0.0, "$x^2 + y^2$", Style::default()),
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
                        ..Style::default()
                    },
                ),
                Element {
                    id: "e1".into(),
                    kind: Kind::Line {
                        from: node("n1"),
                        to: node("n2"),
                        via: vec![],
                        curved: false,
                    },
                    style: Style {
                        end_arrow: true,
                        ..Style::default()
                    },
                    label: "$alpha$".into(),
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
                        dash: Dash::Dashed,
                        end_arrow: true,
                        start_arrow: true,
                        ..Style::default()
                    },
                    label: String::new(),
                },
                Element {
                    id: "e3".into(),
                    kind: Kind::Line {
                        from: End::Point(Point::new(0.0, 6.0)),
                        to: node("n3"),
                        via: vec![Point::new(2.0, 5.0), Point::new(4.0, 7.0)],
                        curved: true,
                    },
                    style: Style::default(),
                    label: String::new(),
                },
            ],
        }
    }

    #[test]
    #[ignore]
    fn dump_sample() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures");
        for f in &super::FORMATS[..1] {
            std::fs::write(format!("{dir}/sample.{}", f.extensions[0]), (f.write)(&sample())).unwrap();
        }
    }
}
