mod css;
mod color;

pub use color::*;
pub use css::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    Bold,
}


impl LineStyle {
    pub fn inline_uml_str(&self) -> &'static str {
        match self {
            LineStyle::Solid => "",
            LineStyle::Dashed => "line.dashed",
            LineStyle::Dotted => "line.dotted",
            LineStyle::Bold => "line.bold",
        }
    }

    fn css_str(&self) -> &'static str {
        match self {
            LineStyle::Solid => "solid",
            LineStyle::Dashed => "dashed",
            LineStyle::Dotted => "dotted",
            LineStyle::Bold => "bold",
        }
    }
}