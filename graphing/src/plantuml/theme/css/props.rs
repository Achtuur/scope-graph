use crate::Color;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LineStyle {
    #[default]
    Solid,
    Dotted,
    Dashed,
    LongDashed,
}

impl CssProperty for LineStyle {
    fn as_css(&self) -> String {
        match self {
            LineStyle::Solid => "0",
            LineStyle::Dotted => "1",
            LineStyle::Dashed => "4",
            LineStyle::LongDashed => "8",
        }
        .to_string()
    }
}

#[derive(Clone, Copy, Debug, Default, derive_more::Display)]
pub enum FontStyle {
    #[default]
    #[display("normal")]
    Normal,
    #[display("bold")]
    Bold,
    #[display("italic")]
    Italic,
    #[display("bold italic")]
    Underline,
}

#[derive(Clone, Copy, Debug, Default, derive_more::Display)]
pub enum FontFamily {
    #[display("Ubuntu Mono")]
    UbuntuMono,
    #[default]
    #[display("SansSerif")]
    SansSerif,
}

#[derive(Clone, Copy, Debug, Default, derive_more::Display)]
pub enum HyperlinkUnderlineStyle {
    #[default]
    #[display("normal")]
    Normal,
}

#[derive(Clone, Copy, Debug, Default, derive_more::Display)]
pub enum HorizontalAlignment {
    #[display("left")]
    Left,
    #[default]
    #[display("center")]
    Center,
    #[display("right")]
    Right,
}
