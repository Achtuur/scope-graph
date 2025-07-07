#[cfg(feature = "mermaid")]
pub mod mermaid;
#[cfg(feature = "plantuml")]
pub mod plantuml;

mod color;
use std::io::Write;

pub use color::*;

mod error;
pub use error::*;

mod renderer;
pub use renderer::*;

pub(crate) trait CssProperty {
    fn write(&self, writer: &mut impl Write) -> RenderResult<()>;
}

impl<T> CssProperty for T
where
    T: std::fmt::Display,
{
    fn write(&self, writer: &mut impl Write) -> RenderResult<()> {
        write!(writer, "{}", self)?;
        Ok(())
    }
}
