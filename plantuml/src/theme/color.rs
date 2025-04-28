use super::CssProperty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 200,
        b: 0,
        a: 255,
    };
    pub const PURPLE: Self = Self {
        r: 128,
        g: 0,
        b: 128,
        a: 255,
    };
    pub const ORANGE: Self = Self {
        r: 255,
        g: 165,
        b: 0,
        a: 255,
    };
    pub const YELLOW: Self = Self {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

impl CssProperty for Color {
    fn as_css(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
