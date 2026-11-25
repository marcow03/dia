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
        /// Image path or data URL, only for `ShapeKind::Image`.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        src: String,
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
    Image,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arrow {
    #[default]
    None,
    Stealth,
    Triangle,
    Open,
    Circle,
    Diamond,
    Bar,
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
    /// Hex colors like `#1e88e5` or `#1e88e580` with alpha. `None` means no stroke / no fill.
    pub stroke: Option<String>,
    pub fill: Option<String>,
    /// Stroke width in pt.
    pub width: f64,
    pub dash: Dash,
    /// Corner radius in cm.
    pub radius: f64,
    pub start: Arrow,
    pub end: Arrow,
    /// Label font family; `None` uses the target's default.
    pub font: Option<String>,
    /// Label font size in pt.
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    /// Label text color.
    pub color: String,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            stroke: Some("#000000".into()),
            fill: None,
            width: 1.0,
            dash: Dash::Solid,
            radius: 0.0,
            start: Arrow::None,
            end: Arrow::None,
            font: None,
            size: 11.0,
            bold: false,
            italic: false,
            color: "#000000".into(),
        }
    }
}

impl Style {
    pub fn has_text_style(&self) -> bool {
        let d = Style::default();
        self.font.is_some() || self.size != d.size || self.bold || self.italic || self.color != d.color
    }
}

/// Splits `#rrggbbaa` into `#rrggbb` and an alpha in 0..=1.
pub fn split_alpha(hex: &str) -> (String, f64) {
    match hex.get(7..9).and_then(|a| u8::from_str_radix(a, 16).ok()) {
        Some(a) => (hex[..7].to_string(), (a as f64 / 255.0 * 100.0).round() / 100.0),
        None => (hex.to_string(), 1.0),
    }
}

/// Inverse of [`split_alpha`]; drops the alpha when it is 1.
pub fn with_alpha(rgb: &str, alpha: f64) -> String {
    if alpha >= 1.0 {
        rgb.to_string()
    } else {
        format!("{rgb}{:02x}", (alpha.clamp(0.0, 1.0) * 255.0).round() as u8)
    }
}

/// File name for an image source; data URLs get a name derived from the element id.
pub fn image_path(id: &str, src: &str) -> String {
    match src.strip_prefix("data:image/") {
        Some(rest) => {
            let ext = rest.split([';', ',', '+']).next().unwrap_or("png");
            format!("{id}.{}", if ext == "jpeg" { "jpg" } else { ext })
        }
        None => src.to_string(),
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
