pub mod formats;
pub mod ir;
pub mod text;

pub use formats::{FORMATS, Format, format, format_for_path};
pub use ir::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown format: {0}")]
    UnknownFormat(String),
    #[error("parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Non-fatal issues found while reading, e.g. unsupported constructs that were skipped.
pub type Warnings = Vec<String>;

pub fn read(input: &str, from: &str, warnings: &mut Warnings) -> Result<Diagram> {
    (format(from)?.read)(input, warnings)
}

pub fn write(diagram: &Diagram, to: &str) -> Result<String> {
    Ok((format(to)?.write)(diagram))
}

pub fn convert(input: &str, from: &str, to: &str, warnings: &mut Warnings) -> Result<String> {
    write(&read(input, from, warnings)?, to)
}
