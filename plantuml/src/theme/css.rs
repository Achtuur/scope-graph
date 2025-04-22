use derive_more::derive;

use super::{Color, LineStyle};

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


#[derive(Default, Clone, Debug)]
pub struct ElementCss {
    // typography
    /// Set the font family for text elements
    font_family: FontFamily,
    font_color: Color,
    font_size: usize,
    font_style: FontStyle,

    // color and background
    background_color: Color,
    hyper_link_color: Color,

    // borders and corners
    round_corner: usize,
    diagonal_corner: usize,
    line_style: LineStyle,
    line_color: Color,
    line_thickness: usize,

    // Spacing and sizing
    padding: usize,
    margin: usize,
    maximum_width: usize,

    // Additional visuals and effects
    shadowing: usize,
    hyperlink_underline_style: HyperlinkUnderlineStyle,
    hyperlink_underline_thickness: usize,
    horizontal_alignment: HorizontalAlignment,
}

impl ElementCss {
    pub const fn new() -> Self {
        Self {
            font_family: FontFamily::SansSerif,
            font_color: Color::BLACK,
            font_size: 12,
            font_style: FontStyle::Normal,
            background_color: Color::new_rgb(255, 255, 255),
            hyper_link_color: Color::BLUE,
            round_corner: 0,
            diagonal_corner: 0,
            line_style: LineStyle::Solid,
            line_color: Color::BLACK,
            line_thickness: 1,
            padding: 0,
            margin: 0,
            maximum_width: 0,
            shadowing: 0,
            hyperlink_underline_style: HyperlinkUnderlineStyle::Normal,
            hyperlink_underline_thickness: 0,
            horizontal_alignment: HorizontalAlignment::Center,
        }
    }

    pub fn as_class(self, class_name: impl ToString) -> CssClass {
        CssClass::new_class(class_name.to_string(), self)
    }

    pub fn as_selector(self, class_name: impl ToString) -> CssClass {
        CssClass::new_selector(class_name.to_string(), self)
    }

    pub const fn font_family(mut self, font_family: FontFamily) -> Self {
        self.font_family = font_family;
        self
    }

    pub const fn font_color(mut self, font_color: Color) -> Self {
        self.font_color = font_color;
        self
    }

    pub const fn font_size(mut self, font_size: usize) -> Self {
        self.font_size = font_size;
        self
    }

    pub const fn font_style(mut self, font_style: FontStyle) -> Self {
        self.font_style = font_style;
        self
    }

    pub const fn background_color(mut self, background_color: Color) -> Self {
        self.background_color = background_color;
        self
    }

    pub const fn hyperlink_color(mut self, hyperlink_color: Color) -> Self {
        self.hyper_link_color = hyperlink_color;
        self
    }

    pub const fn round_corner(mut self, round_corner: usize) -> Self {
        self.round_corner = round_corner;
        self
    }

    pub const fn diagonal_corner(mut self, diagonal_corner: usize) -> Self {
        self.diagonal_corner = diagonal_corner;
        self
    }

    pub const fn line_style(mut self, line_style: LineStyle) -> Self {
        self.line_style = line_style;
        self
    }

    pub const fn line_color(mut self, line_color: Color) -> Self {
        self.line_color = line_color;
        self
    }

    pub const fn line_thickness(mut self, line_thickness: usize) -> Self {
        self.line_thickness = line_thickness;
        self
    }

    pub const fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    pub const fn margin(mut self, margin: usize) -> Self {
        self.margin = margin;
        self
    }

    pub const fn maximum_width(mut self, maximum_width: usize) -> Self {
        self.maximum_width = maximum_width;
        self
    }

    pub const fn shadowing(mut self, shadowing: usize) -> Self {
        self.shadowing = shadowing;
        self
    }

    pub const fn hyperlink_underline_style(mut self, hyperlink_underline_style: HyperlinkUnderlineStyle) -> Self {
        self.hyperlink_underline_style = hyperlink_underline_style;
        self
    }

    pub const fn hyperlink_underline_thickness(mut self, hyper_link_underline_thickness: usize) -> Self {
        self.hyperlink_underline_thickness = hyper_link_underline_thickness;
        self
    }

    pub const fn horizontal_alignment(mut self, horizontal_alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = horizontal_alignment;
        self
    }

    pub fn as_css(&self) -> String {
        // https://plantuml.com/style
        [
            format!("FontFamily: {};", self.font_family),
            format!("FontColor: {};", self.font_color.hex()),
            format!("FontSize: {};", self.font_size),
            format!("FontStyle: {};", self.font_style),
            format!("BackGroundColor: {};", self.background_color.hex()),
            format!("RoundCorner: {};", self.round_corner),
            format!("DiagonalCorner: {};", self.diagonal_corner),
            format!("LineStyle: {};", self.line_style.css_str()),
            format!("LineColor: {};", self.line_color.hex()),
            format!("LineThickness: {};", self.line_thickness),
            format!("Padding: {};", self.padding),
            format!("Margin: {};", self.margin),
            format!("MaximumWidth: {};", self.maximum_width),
            format!("Shadowing: {};", self.shadowing),
            format!("HyperLinkColor: {};", self.hyper_link_color.hex()),
            format!("HyperLinkUnderlineStyle: {};", self.hyperlink_underline_style),
            format!("HyperLinkUnderlineThickness: {};", self.hyperlink_underline_thickness),
            format!("HorizontalAlignment: {};", self.horizontal_alignment),
        ].join("\n")
    }
}

#[derive(Clone, Debug)]
pub struct CssClass {
    name: String,
    element: ElementCss,
    // True if this is a selector (no '.' is needed then)
    is_selector: bool,
}

impl CssClass {
    /// Create a new CSS class
    pub fn new_class(name: String, element: ElementCss) -> Self {
        Self {
            name,
            element,
            is_selector: false,
        }
    }

    /// Creates a new styling for an element selector
    pub fn new_selector(name: String, element: ElementCss) -> Self {
        Self {
            name,
            element,
            is_selector: true,
        }
    }

    pub fn set_element(&mut self, element: ElementCss) {
        self.element = element;
    }

    pub fn as_css(&self) -> String {
        let selector = if self.is_selector { "" } else { "." };
        let class_name = format!("{}{}", selector, self.name);
        format!("{} {{\n{}\n}}", class_name, self.element.as_css())
    }
}


#[derive(Default)]
pub struct StyleSheet {
    classes: Vec<CssClass>,
}

impl FromIterator<CssClass> for StyleSheet {
    fn from_iter<T: IntoIterator<Item = CssClass>>(iter: T) -> Self {
        let mut style_sheet = StyleSheet::new();
        style_sheet.extend(iter);
        style_sheet
    }
}

impl From<Vec<CssClass>> for StyleSheet {
    fn from(value: Vec<CssClass>) -> Self {
        StyleSheet { classes: value }
    }
}

impl<const N: usize> From<[CssClass; N]> for StyleSheet {
    fn from(value: [CssClass; N]) -> Self {
        StyleSheet { classes: value.to_vec() }
    }
}

impl StyleSheet {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
        }
    }

    pub fn push(&mut self, class: CssClass) {
        self.classes.push(class);
    }

    pub fn extend(&mut self, classes: impl IntoIterator<Item = CssClass>) {
        self.classes.extend(classes);
    }

    pub fn as_css(&self) -> String {
        let classes = self.classes
        .iter()
        .map(|c| c.as_css())
        .collect::<Vec<_>>()
        .join("\n");
        format!("<style>\n{}\n</style>", classes)
    }
}