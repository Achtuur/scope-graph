use crate::CssProperty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dotted,
    Dashed,
    LongDashed,
}

impl LineStyle {
    pub fn as_num(&self) -> usize {
        match self {
            LineStyle::Solid => 0,
            LineStyle::Dotted => 3,
            LineStyle::Dashed => 5,
            LineStyle::LongDashed => 10,
        }
    }
}

impl CssProperty for LineStyle {
    fn as_css(&self) -> String {
        self.as_num().to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationSettings {
    pub(crate) speed: AnimationSpeed,
    pub(crate) style: AnimationStyle,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            speed: AnimationSpeed::Medium,
            style: AnimationStyle::Linear,
        }
    }
}

impl CssProperty for AnimationSettings {
    fn as_css(&self) -> String {
        format!("dash {}s {} infinite", self.speed.as_num(), self.style)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub enum AnimationStyle {
    #[display("linear")]
    Linear,
    #[display("ease-in-out")]
    Pulse,
}

/// Animation speed for the animation property
///
/// This overrides the line style field due to how mmd diagrams work
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationSpeed {
    Slow,
    Medium,
    Fast,
}

impl AnimationSpeed {
    pub fn as_num(&self) -> usize {
        match self {
            AnimationSpeed::Slow => 3,
            AnimationSpeed::Medium => 2,
            AnimationSpeed::Fast => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, derive_more::Display)]
pub enum Size {
    #[display("{}em", _0)]
    Em(f32),
    #[display("{}px", _0)]
    Px(usize),
    #[display("{}pt", _0)]
    Pt(usize),
}
