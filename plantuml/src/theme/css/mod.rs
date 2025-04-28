use super::Color;

mod props;
pub use props::*;

#[derive(Default, Clone, Debug)]
pub struct ElementCss {
    // typography
    /// Set the font family for text elements
    font_family: Option<FontFamily>,
    font_color: Option<Color>,
    font_size: Option<usize>,
    font_style: Option<FontStyle>,

    // color and background
    background_color: Option<Color>,
    hyper_link_color: Option<Color>,

    // borders and corners
    round_corner: Option<usize>,
    diagonal_corner: Option<usize>,
    line_style: Option<LineStyle>,
    line_color: Option<Color>,
    line_thickness: Option<f32>,

    // Spacing and sizing
    padding: Option<usize>,
    margin: Option<usize>,
    maximum_width: Option<usize>,

    // Additional visuals and effects
    shadowing: Option<usize>,
    hyperlink_underline_style: Option<HyperlinkUnderlineStyle>,
    hyperlink_underline_thickness: Option<usize>,
    horizontal_alignment: Option<HorizontalAlignment>,
}

impl ElementCss {
    pub const fn new() -> Self {
        Self {
            font_family: None,
            font_color: None,
            font_size: None,
            font_style: None,
            background_color: None,
            hyper_link_color: None,
            round_corner: None,
            diagonal_corner: None,
            line_style: None,
            line_color: None,
            line_thickness: None,
            padding: None,
            margin: None,
            maximum_width: None,
            shadowing: None,
            hyperlink_underline_style: None,
            hyperlink_underline_thickness: None,
            horizontal_alignment: None,
        }
    }

    pub fn as_class(self, class_name: impl ToString) -> CssClass {
        CssClass::new_class(class_name.to_string(), self)
    }

    pub fn as_selector(self, class_name: impl ToString) -> CssClass {
        CssClass::new_selector(class_name.to_string(), self)
    }

    pub const fn font_family(mut self, font_family: FontFamily) -> Self {
        self.font_family = Some(font_family);
        self
    }

    pub const fn font_color(mut self, font_color: Color) -> Self {
        self.font_color = Some(font_color);
        self
    }

    pub const fn font_size(mut self, font_size: usize) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub const fn font_style(mut self, font_style: FontStyle) -> Self {
        self.font_style = Some(font_style);
        self
    }

    pub const fn background_color(mut self, background_color: Color) -> Self {
        self.background_color = Some(background_color);
        self
    }

    pub const fn hyperlink_color(mut self, hyperlink_color: Color) -> Self {
        self.hyper_link_color = Some(hyperlink_color);
        self
    }

    pub const fn round_corner(mut self, round_corner: usize) -> Self {
        self.round_corner = Some(round_corner);
        self
    }

    pub const fn diagonal_corner(mut self, diagonal_corner: usize) -> Self {
        self.diagonal_corner = Some(diagonal_corner);
        self
    }

    pub const fn line_style(mut self, line_style: LineStyle) -> Self {
        self.line_style = Some(line_style);
        self
    }

    pub const fn line_color(mut self, line_color: Color) -> Self {
        self.line_color = Some(line_color);
        self
    }

    pub const fn line_thickness(mut self, line_thickness: f32) -> Self {
        self.line_thickness = Some(line_thickness);
        self
    }

    pub const fn padding(mut self, padding: usize) -> Self {
        self.padding = Some(padding);
        self
    }

    pub const fn margin(mut self, margin: usize) -> Self {
        self.margin = Some(margin);
        self
    }

    pub const fn maximum_width(mut self, maximum_width: usize) -> Self {
        self.maximum_width = Some(maximum_width);
        self
    }

    pub const fn shadowing(mut self, shadowing: usize) -> Self {
        self.shadowing = Some(shadowing);
        self
    }

    pub const fn hyperlink_underline_style(
        mut self,
        hyperlink_underline_style: HyperlinkUnderlineStyle,
    ) -> Self {
        self.hyperlink_underline_style = Some(hyperlink_underline_style);
        self
    }

    pub const fn hyperlink_underline_thickness(
        mut self,
        hyper_link_underline_thickness: usize,
    ) -> Self {
        self.hyperlink_underline_thickness = Some(hyper_link_underline_thickness);
        self
    }

    pub const fn horizontal_alignment(mut self, horizontal_alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = Some(horizontal_alignment);
        self
    }

    pub fn as_css(&self) -> String {
        // https://plantuml.com/style

        macro_rules! format_css {
            ($key:literal, $prop:expr) => {
                $prop.map(|x| format!("{}: {};", $key, x.as_css()))
            };
        }

        [
            format_css!("FontFamily", self.font_family),
            format_css!("FontColor", self.font_color),
            format_css!("FontSize", self.font_size),
            format_css!("FontStyle", self.font_style),
            format_css!("BackGroundColor", self.background_color),
            format_css!("RoundCorner", self.round_corner),
            format_css!("DiagonalCorner", self.diagonal_corner),
            format_css!("LineStyle", self.line_style),
            format_css!("LineColor", self.line_color),
            format_css!("LineThickness", self.line_thickness),
            format_css!("Padding", self.padding),
            format_css!("Margin", self.margin),
            format_css!("MaximumWidth", self.maximum_width),
            format_css!("Shadowing", self.shadowing),
            format_css!("HyperLinkColor", self.hyper_link_color),
            format_css!("HyperLinkUnderlineStyle", self.hyperlink_underline_style),
            format_css!(
                "HyperLinkUnderlineThickness",
                self.hyperlink_underline_thickness
            ),
            format_css!("HorizontalAlignment", self.horizontal_alignment),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n")
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

#[derive(Default, Clone, Debug)]
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
        StyleSheet {
            classes: value.to_vec(),
        }
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
        let classes = self
            .classes
            .iter()
            .map(|c| c.as_css())
            .collect::<Vec<_>>()
            .join("\n");
        format!("<style>\n{}\n</style>", classes)
    }
}
