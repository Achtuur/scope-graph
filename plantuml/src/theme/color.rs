use super::CssProperty;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub const BLACK: Self = Self::new_rgb_u32(0x000000);
    pub const WHITE: Self = Self::new_rgb_u32(0xFFFFFF);
    pub const RED: Self = Self::new_rgb_u32(0xFF0000);
    pub const GREEN: Self = Self::new_rgb_u32(0x12d812);
    pub const BLUE: Self = Self::new_rgb_u32(0x0000FF);
    pub const PURPLE: Self = Self::new_rgb_u32(0x800080);
    pub const ORANGE: Self = Self::new_rgb_u32(0xFFA500);
    pub const YELLOW: Self = Self::new_rgb_u32(0xc0bd22);
    pub const CYAN: Self = Self::new_rgb_u32(0x00FFFF);

    pub const LIGHT_GRAY: Self = Self::new_rgb_u32(0xE6E6E6);
    pub const LIGHT_RED: Self = Self::new_rgb_u32(0xFFF1F1);
    pub const LIGHT_GREEN: Self = Self::new_rgb_u32(0xF1FFF1);
    pub const LIGHT_BLUE: Self = Self::new_rgb_u32(0xF1F1FF);
    pub const LIGHT_PURPLE: Self = Self::new_rgb_u32(0xFFF1FB);
    pub const LIGHT_ORANGE: Self = Self::new_rgb_u32(0xFFFBF1);
    pub const LIGHT_YELLOW: Self = Self::new_rgb_u32(0xFEFFF1);
    pub const LIGHT_CYAN: Self = Self::new_rgb_u32(0xF1FFFF);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn new_rgba_u32(clr: u32) -> Self {
        let r = (clr >> 24) as u8;
        let g = (clr >> 16) as u8;
        let b = (clr >> 8) as u8;
        let a = (clr) as u8;
        Self::new(r, g, b, a)
    }

    pub const fn new_rgb_u32(clr: u32) -> Self {
        Self::new_rgba_u32(clr << 8)
    }
}

impl CssProperty for Color {
    fn as_css(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
