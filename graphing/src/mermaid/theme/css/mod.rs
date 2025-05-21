mod props;

use std::io::Write;

pub use props::*;

use crate::{Color, CssProperty, RenderResult};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StyleOptions {
    /// stroke
    line_color: Option<Color>,
    /// stroke-dasharray
    line_style: Option<LineStyle>,
    /// stroke-dashoffet, should be a multiple of line_style
    line_offset: Option<f32>,
    /// stroke-width
    line_thickness: Option<f32>,
    /// fill
    background_color: Option<Color>,
    /// font-size
    font_size: Option<Size>,
    animation: Option<AnimationSettings>,
    margin: Option<Size>,
    padding: Option<Size>,
}

impl StyleOptions {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::new()
    }

    pub(crate) fn write(&self, writer: &mut impl Write) -> RenderResult<()> {
        let mut is_first = true;
        macro_rules! write_prop {
            ($key:literal, $prop:expr) => {
                if let Some(x) = $prop {
                    if !is_first {
                        write!(writer, ", ")?;
                    }
                    write!(writer, "{}: ", $key)?;
                    x.write(writer)?;
                    is_first = false;
                }
            };
        }

        write_prop!("stroke", self.line_color);
        write_prop!("stroke-dasharray", self.line_style);
        write_prop!("stroke-dashoffset", self.line_offset);
        write_prop!("stroke-width", self.line_thickness);
        write_prop!("fill", self.background_color);
        write_prop!("font-size", self.font_size);
        write_prop!("padding", self.padding);
        write_prop!("margin", self.margin);
        write_prop!("animation", self.animation);
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ElementStyle {
    style: StyleOptions,
}

impl ElementStyle {
    pub fn new() -> Self {
        Self {
            style: StyleOptions::new(),
        }
    }

    pub fn padding(mut self, padding: Size) -> Self {
        self.style.padding = Some(padding);
        self
    }

    pub fn margin(mut self, margin: Size) -> Self {
        self.style.margin = Some(margin);
        self
    }

    pub fn line_color(mut self, color: Color) -> Self {
        self.style.line_color = Some(color);
        self
    }

    pub fn line_style(mut self, style: LineStyle) -> Self {
        self.style.line_style = Some(style);
        self.style.line_offset = Some(10.0 * style.as_num() as f32);
        self
    }

    pub fn line_thickness(mut self, thickness: f32) -> Self {
        self.style.line_thickness = Some(thickness);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.style.background_color = Some(color);
        self
    }

    pub fn font_size(mut self, size: Size) -> Self {
        self.style.font_size = Some(size);
        self
    }

    pub fn animation_style(mut self, style: AnimationStyle) -> Self {
        let animation = self.style.animation.get_or_insert_default();
        animation.style = style;
        self.line_style(LineStyle::Dashed)
    }

    pub fn animation_speed(mut self, speed: AnimationSpeed) -> Self {
        let animation = self.style.animation.get_or_insert_default();
        animation.speed = speed;
        self.line_style(LineStyle::Dashed)
    }

    pub(crate) fn write(&self, writer: &mut impl Write, class_name: &str) -> RenderResult<()> {
        if self.style.is_empty() {
            return Ok(())
        }
        write!(writer, "classDef {} ", class_name)?;
        self.style.write(writer)?;
        let _ = writer.write(b"\n")?;
        Ok(())
    }
}
