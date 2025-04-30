#[cfg(feature = "plantuml")]
pub mod plantuml;
#[cfg(feature = "mermaid")]
pub mod mermaid;

mod color;
pub use color::*;


pub(crate) trait CssProperty {
    fn as_css(&self) -> String;
}


impl<T> CssProperty for T
where
    T: std::fmt::Display,
{
    fn as_css(&self) -> String {
        format!("{}", self)
    }
}

impl CssProperty for Color {
    fn as_css(&self) -> String {
        self.hex_string()
    }
}