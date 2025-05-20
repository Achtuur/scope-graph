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

    fn to_string(&self) -> RenderResult<String> {
        let mut buf = Vec::new();
        self.write(&mut buf)?;
        String::from_utf8(buf).map_err(Into::into)
    }
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

// impl CssProperty for Color {
//     fn to_string(&self) -> String {
//         self.hex_string()
//     }
// }
