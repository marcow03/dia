use super::Format;
use crate::ir::*;
use crate::{Result, Warnings};

pub const FORMAT: Format = Format {
    name: "drawio",
    extensions: &["drawio", "xml"],
    read,
    write,
};

pub fn write(_d: &Diagram) -> String {
    todo!()
}
pub fn read(_s: &str, _w: &mut Warnings) -> Result<Diagram> {
    todo!()
}
