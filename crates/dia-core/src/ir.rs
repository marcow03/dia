use serde::{Deserialize, Serialize};

/// Canonical diagram. Units are centimeters, y grows downwards.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Diagram {
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub id: String,
    #[serde(flatten)]
    pub kind: Kind,
    #[serde(default)]
    pub style: Style,
    /// Typst markup, `$...$` for math. Empty means no label.
    #[serde(default)]
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Kind {
    Shape {
        shape: ShapeKind,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
    Line {
        from: End,
        to: End,
        #[serde(default)]
        via: Vec<Point>,
        /// With 1 or 2 `via` points these are bezier control points.
        #[serde(default)]
        curved: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShapeKind {
    Rect,
    Ellipse,
    Diamond,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum End {
    Point(Point),
    Node { node: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dash {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Style {
    /// Hex colors like `#1e88e5`. `None` means no stroke / no fill.
    pub stroke: Option<String>,
    pub fill: Option<String>,
    /// Stroke width in pt.
    pub width: f64,
    pub dash: Dash,
    /// Corner radius in cm.
    pub radius: f64,
    pub start_arrow: bool,
    pub end_arrow: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            stroke: Some("#000000".into()),
            fill: None,
            width: 1.0,
            dash: Dash::Solid,
            radius: 0.0,
            start_arrow: false,
            end_arrow: false,
        }
    }
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

impl Diagram {
    pub fn get(&self, id: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.id == id)
    }

    pub fn center(&self, end: &End) -> Point {
        match end {
            End::Point(p) => *p,
            End::Node { node } => match self.get(node).map(|e| &e.kind) {
                Some(Kind::Shape { x, y, w, h, .. }) => Point::new(x + w / 2.0, y + h / 2.0),
                _ => Point::default(),
            },
        }
    }

    /// Generates an id not used yet, like `n3`.
    pub fn fresh_id(&self, prefix: &str) -> String {
        (1..)
            .map(|i| format!("{prefix}{i}"))
            .find(|id| self.get(id).is_none())
            .unwrap()
    }
}

/// Rounds to 3 decimals and drops a trailing `.0`.
pub fn num(v: f64) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r == 0.0 { "0".into() } else { format!("{r}") }
}
